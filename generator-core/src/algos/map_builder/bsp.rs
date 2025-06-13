use super::BinarySpacePartitioningConfig;
use crate::{
    algos::RngHandler,
    constants::{MIN_RECT_HEIGHT, MIN_RECT_WIDTH},
    types::{Rect, RectModifier, RectRegion, SplitAxis},
};

use std::collections::{HashMap, HashSet, VecDeque};

use rand::Rng;
use rayon::prelude::*;
use tracing::event;

pub(crate) type RectTable = HashMap<usize, Rect>;
pub(crate) type RemovedRectTable = HashMap<usize, Rect>;
pub(crate) type NeighbourTable = HashMap<usize, HashSet<usize>>;

pub(crate) struct BinarySpacePartitioning;

impl BinarySpacePartitioning {
    pub fn generate_and_trim_partitions(
        width: u32,
        height: u32,
        config: BinarySpacePartitioningConfig,
    ) -> Vec<(Rect, RectTable, RemovedRectTable, NeighbourTable)> {
        if width <= MIN_RECT_WIDTH || height <= MIN_RECT_HEIGHT {
            event!(
                tracing::Level::WARN,
                "Skipping partition generation for too small dimensions: [{}x{}]",
                width,
                height
            );

            return vec![];
        }

        let initial_rect = Rect::new(0, 0, width, height);
        let region_count = u32::max(initial_rect.area() / config.region_split_factor, 2);

        event!(
            tracing::Level::DEBUG,
            "Splitting inital rect [{}] into [{}] regions",
            initial_rect,
            region_count
        );

        let regions = Self::generate_regions(initial_rect, region_count, &config);

        let avg_region_area =
            regions.iter().map(|r| r.rect.area()).sum::<u32>() / regions.len() as u32;
        let min_region_area = regions.iter().map(|r| r.rect.area()).min().unwrap_or(0);
        let max_region_area = regions.iter().map(|r| r.rect.area()).max().unwrap_or(0);

        event!(
            tracing::Level::DEBUG,
            "Region area sizes: [{}±{}]",
            avg_region_area,
            max_region_area - min_region_area
        );

        regions
            .into_par_iter()
            .map(|region| {
                let origin_rect = region.rect;

                let (mut region_rects, mut removed_rects, mut neighbours) =
                    Self::generate_partitions(region, &config);

                Self::trim_connected_rects(
                    &mut region_rects,
                    &mut removed_rects,
                    &mut neighbours,
                    &config,
                );

                Self::trim_orphaned_rects(&mut region_rects, &mut removed_rects, &mut neighbours);

                (origin_rect, region_rects, removed_rects, neighbours)
            })
            .filter(|(_, rects, _, _)| {
                // Filter out empty regions
                !rects.is_empty()
            })
            .collect()
    }

    fn generate_regions(
        initial_rect: Rect,
        region_count: u32,
        config: &BinarySpacePartitioningConfig,
    ) -> Vec<RectRegion> {
        let mut rect_queue = VecDeque::new();
        rect_queue.push_back(initial_rect);

        let width = initial_rect.width;
        let height = initial_rect.height;

        while rect_queue.len() < region_count as usize {
            let rect = rect_queue.pop_front().unwrap();

            if rect.width < width / region_count || rect.height < height / region_count {
                rect_queue.push_back(rect);
                continue;
            }

            let (rect_a, maybe_rect_b) = Self::split_rect(
                rect,
                config.height_factor_cutoff,
                config.width_factor_cutoff,
                config.horizontal_split_prob,
            );
            rect_queue.push_front(rect_a);

            if let Some(rect_b) = maybe_rect_b {
                rect_queue.push_front(rect_b);
            }
        }

        let mut rng = RngHandler::rng();

        rect_queue
            .into_iter()
            .map(|rect| {
                let roll = rng.random_range(1_u32..101);
                let horizontal_bound = (85.0 * config.horizontal_region_prob) as u32;

                let rect_modifier = if (0..10).contains(&roll) {
                    RectModifier::Standard
                } else if (10..horizontal_bound).contains(&roll) {
                    RectModifier::PreferHorizontal
                } else if (horizontal_bound..85).contains(&roll) {
                    RectModifier::PreferVertical
                } else {
                    RectModifier::Chaotic
                };

                RectRegion {
                    rect,
                    modifier: rect_modifier,
                }
            })
            .collect()
    }

    fn generate_partitions(
        region: RectRegion,
        config: &BinarySpacePartitioningConfig,
    ) -> (RectTable, RemovedRectTable, NeighbourTable) {
        let mut rng = RngHandler::rng();

        let min_area = config.rect_area_cutoff;
        let max_area = min_area * config.big_rect_area_cutoff;

        let height_factor_cutoff = match region.modifier {
            RectModifier::Standard => config.height_factor_cutoff,
            RectModifier::PreferHorizontal => config.height_factor_cutoff - 1.0,
            RectModifier::PreferVertical => config.height_factor_cutoff + 1.0,
            RectModifier::Chaotic => config.height_factor_cutoff + rng.random_range(-0.5..0.5),
        }
        .clamp(1.0, 5.0);

        let width_factor_cutoff = match region.modifier {
            RectModifier::Standard => config.width_factor_cutoff,
            RectModifier::PreferHorizontal => config.width_factor_cutoff + 1.0,
            RectModifier::PreferVertical => config.width_factor_cutoff - 1.0,
            RectModifier::Chaotic => config.width_factor_cutoff + rng.random_range(-0.5..0.5),
        }
        .clamp(1.0, 5.0);

        let horizontal_split_prob = match region.modifier {
            RectModifier::Standard => config.horizontal_split_prob,
            RectModifier::PreferHorizontal => config.horizontal_split_prob + 0.3,
            RectModifier::PreferVertical => config.horizontal_split_prob - 0.3,
            RectModifier::Chaotic => config.horizontal_split_prob + rng.random_range(-0.3..0.3),
        }
        .clamp(0.1, 0.9);

        event!(
            tracing::Level::DEBUG,
            "Splitting with params HF:[{:.2}] - WF:[{:.2}] - HS:[{:.2}] | Target region: {} ",
            height_factor_cutoff,
            width_factor_cutoff,
            horizontal_split_prob,
            region,
        );

        let mut rect_idx = 1_usize;

        let mut rect_table = HashMap::new();
        rect_table.insert(rect_idx, region.rect);

        let mut neighbour_table = HashMap::new();
        neighbour_table.insert(rect_idx, HashSet::with_capacity(0));

        let mut removed_rects = HashMap::new();

        let mut idx_stack = vec![rect_idx];

        while let Some(idx) = idx_stack.pop() {
            let rect = rect_table.remove(&idx).unwrap();
            let rect_area = rect.area();

            if rect_area > min_area {
                if rect_area <= max_area && rng.random_bool(config.big_rect_survival_prob) {
                    // The rectangle survived, so we put it back into the table
                    rect_table.insert(idx, rect);
                } else {
                    let (rect_a, maybe_rect_b) = Self::split_rect(
                        rect,
                        height_factor_cutoff,
                        width_factor_cutoff,
                        horizontal_split_prob,
                    );

                    rect_idx += 1;
                    let rect_a_idx = rect_idx;
                    let mut rect_a_neighbours = HashSet::new();

                    let mut maybe_rect_b = if let Some(rect_b) = maybe_rect_b {
                        rect_idx += 1;

                        rect_a_neighbours.insert(rect_idx);

                        let mut rect_b_neighbours = HashSet::new();
                        rect_b_neighbours.insert(rect_a_idx);

                        Some((rect_idx, rect_b, rect_b_neighbours))
                    } else {
                        None
                    };

                    let mut current_neighbours = neighbour_table.remove(&idx).unwrap();

                    for neighbour in current_neighbours.drain() {
                        let neighbour_rect =
                            if let Some(neighbour_rect) = rect_table.get(&neighbour) {
                                neighbour_rect
                            } else if let Some(removed_rect) = removed_rects.get(&neighbour) {
                                removed_rect
                            } else {
                                panic!("Negihbour not found in either rect tables! {}", neighbour)
                            };
                        let neighbour_neighbours = neighbour_table.get_mut(&neighbour).unwrap();
                        neighbour_neighbours.remove(&idx);

                        if neighbour_rect.is_neighbour_of(&rect_a).is_some() {
                            rect_a_neighbours.insert(neighbour);
                            neighbour_neighbours.insert(rect_a_idx);
                        }

                        if let Some((rect_b_idx, rect_b, ref mut rect_b_neighbours)) = maybe_rect_b
                        {
                            if neighbour_rect.is_neighbour_of(&rect_b).is_some() {
                                rect_b_neighbours.insert(neighbour);
                                neighbour_neighbours.insert(rect_b_idx);
                            }
                        }
                    }

                    rect_table.insert(rect_a_idx, rect_a);
                    neighbour_table.insert(rect_a_idx, rect_a_neighbours);
                    idx_stack.push(rect_a_idx);

                    if let Some((rect_b_idx, rect_b, rect_b_neighbours)) = maybe_rect_b {
                        rect_table.insert(rect_b_idx, rect_b);
                        neighbour_table.insert(rect_b_idx, rect_b_neighbours);
                        idx_stack.push(rect_b_idx);
                    }
                }
            } else if rng.random_bool(config.rect_survival_prob) {
                // The rectangle survived, so we put it back into the table
                rect_table.insert(idx, rect);
            } else {
                removed_rects.insert(idx, rect);
            }
        }

        (rect_table, removed_rects, neighbour_table)
    }

    fn split_rect(
        rect: Rect,
        height_cutoff: f32,
        width_cutoff: f32,
        horizontal_split_prob: f64,
    ) -> (Rect, Option<Rect>) {
        let height_factor = rect.height as f32 / rect.width as f32;
        let width_factor = rect.width as f32 / rect.height as f32;

        let mut rng = RngHandler::rng();

        let split_axis = {
            if height_factor > height_cutoff {
                SplitAxis::Horizontal
            } else if width_factor > width_cutoff {
                SplitAxis::Vertical
            } else if rng.random_bool(horizontal_split_prob) {
                SplitAxis::Horizontal
            } else {
                SplitAxis::Vertical
            }
        };

        match split_axis {
            SplitAxis::Horizontal => {
                if rect.height > 1 {
                    let split_col = rng.random_range(1..rect.height);

                    let (up, down) = rect.try_split_at(SplitAxis::Horizontal, split_col).unwrap();

                    (up, Some(down))
                } else {
                    (rect, None)
                }
            }
            SplitAxis::Vertical => {
                if rect.width > 1 {
                    let split_row = rng.random_range(1..rect.width);

                    let (left, right) = rect.try_split_at(SplitAxis::Vertical, split_row).unwrap();

                    (left, Some(right))
                } else {
                    (rect, None)
                }
            }
        }
    }

    fn trim_connected_rects(
        rects: &mut HashMap<usize, Rect>,
        removed: &mut HashMap<usize, Rect>,
        neighbour_map: &mut HashMap<usize, HashSet<usize>>,
        config: &BinarySpacePartitioningConfig,
    ) {
        let rects_to_remove = rects
            .par_iter()
            .filter_map(|(idx, _)| {
                let neighbour_count = neighbour_map.get(idx).map_or(0, |neighbour_set| {
                    neighbour_set
                        .iter()
                        .filter(|n_idx| rects.contains_key(n_idx))
                        .count()
                });

                let mut rng = RngHandler::rng();

                let should_remove = match neighbour_count {
                    8.. => rng.random_bool(config.trim_fully_connected_rect_prob),
                    5..8 => rng.random_bool(config.trim_highly_connected_rect_prob),
                    0 => true,
                    _ => false,
                };

                if should_remove { Some(*idx) } else { None }
            })
            .collect::<Vec<_>>();

        for rect_idx in rects_to_remove {
            if let Some(rect) = rects.remove(&rect_idx) {
                removed.insert(rect_idx, rect);
            }
        }
    }

    fn trim_orphaned_rects(
        rects: &mut HashMap<usize, Rect>,
        removed: &mut HashMap<usize, Rect>,
        neighbour_map: &mut HashMap<usize, HashSet<usize>>,
    ) {
        let rects_to_remove = rects
            .par_iter()
            .filter_map(|(idx, _)| {
                let neighbour_count = neighbour_map.get(idx).map_or(0, |neighbour_set| {
                    neighbour_set
                        .iter()
                        .filter(|n_idx| rects.contains_key(n_idx))
                        .count()
                });

                if neighbour_count == 0 {
                    Some(*idx)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for rect_idx in rects_to_remove {
            if let Some(rect) = rects.remove(&rect_idx) {
                removed.insert(rect_idx, rect);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_generate_and_trim_partitions() {
        let width = 20;
        let height = 20;

        let mut config = BinarySpacePartitioningConfig::default();
        // Force only two regions to be generated
        config.region_split_factor = width * height;

        let results = BinarySpacePartitioning::generate_and_trim_partitions(width, height, config);

        assert_eq!(results.len(), 2);

        for (origin_rect, rect_table, removed_rects, neighbours) in results {
            // Check origin rect dimensions
            assert!(origin_rect.width > 0 && origin_rect.width <= width);
            assert!(origin_rect.height > 0 && origin_rect.height <= height);

            // Check that neighbours table is not empty
            assert!(!neighbours.is_empty());

            // Check that the total number of rects in rect_table and removed_rects
            assert!(rect_table.len() + removed_rects.len() == neighbours.len());

            // Check that rects are not in the removed table
            // and that they are in the neighbours table
            for rect_idx in rect_table.keys() {
                assert!(!removed_rects.contains_key(rect_idx));
                assert!(neighbours.contains_key(rect_idx));
            }

            // Check that removed rects are not in the rect table
            // and that they are in the neighbours table
            for removed_rect_idx in removed_rects.keys() {
                assert!(!rect_table.contains_key(removed_rect_idx));
                assert!(neighbours.contains_key(removed_rect_idx));
            }

            // Verify neighbor relationships
            for (idx, neighbor_set) in neighbours.iter() {
                let rect = rect_table
                    .get(idx)
                    .or_else(|| removed_rects.get(idx))
                    .expect("Rect or removed rect should exist for index");
                // Check each rect has valid neighbors
                for neighbour_idx in neighbor_set.iter() {
                    assert!(neighbour_idx != idx, "Rect should not be its own neighbour");

                    if let Some(neighbor_rect) = rect_table.get(neighbour_idx) {
                        assert!(rect.is_neighbour_of(neighbor_rect).is_some());
                    } else if let Some(neighbor_rect) = removed_rects.get(neighbour_idx) {
                        assert!(rect.is_neighbour_of(neighbor_rect).is_some());
                    } else {
                        panic!("Rect or removed rect not found for index {}", idx);
                    }
                }
            }
        }
    }
}
