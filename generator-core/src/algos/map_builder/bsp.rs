use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    constants::{MIN_RECT_HEIGHT, MIN_RECT_WIDTH, REGION_SPLIT_FACTOR},
    types::{Cell, Rect, RectModifier, RectRegion, SplitAxis},
};

use rand::Rng;
use rayon::prelude::*;
use tracing::event;

#[derive(Debug, Clone, Copy)]
pub(crate) struct BinarySpacePartitioningConfig {
    pub region_split_factor: u32,
    // The proportion of regions that are going to be PreferHorizontal
    // over PreferVertical. The Standard and Chaotic modifiers are
    // excluded from this calculation. Since their proportions are fixed.
    // The value is between 0.0 and 1.0.
    pub horizontal_region_prob: f64,
    // The minimum area of a rectangle to be considered for splitting.
    pub rect_area_cutoff: u32,
    // The maximum area of a rectangle proportional to rect_area_cutoff
    // to be considered for skipping its splitting.
    pub big_rect_area_cutoff: u32,
    // The probability of leaving a big rectangle without further splitting.
    pub big_rect_survival_prob: f64,
    // The random probability of performing a horizontal split.
    pub horizontal_split_prob: f64,
    // The minimum height to width ratio at which we will always perform a
    // horizontal split.
    pub height_factor_cutoff: f32,
    // The minimum width to height ratio at which we will always perform a
    // vertical split.
    pub width_factor_cutoff: f32,
    // The probability of keeping the finaly splitted rectangle.
    pub rect_survival_prob: f64,
    // The probability of removing a highly connected rectangle.
    pub trim_highly_connected_rect_prob: f64,
    // The probability of removing a fully connected rectangle.
    pub trim_fully_connected_rect_prob: f64,
}

impl Default for BinarySpacePartitioningConfig {
    fn default() -> Self {
        BinarySpacePartitioningConfig {
            region_split_factor: REGION_SPLIT_FACTOR,
            horizontal_region_prob: 0.5,
            rect_area_cutoff: 2,
            big_rect_area_cutoff: 9,
            big_rect_survival_prob: 0.03,
            horizontal_split_prob: 0.6,
            height_factor_cutoff: 1.8,
            width_factor_cutoff: 2.7,
            rect_survival_prob: 0.43,
            trim_highly_connected_rect_prob: 0.4,
            trim_fully_connected_rect_prob: 0.5,
        }
    }
}

pub(crate) struct BinarySpacePartitioning;

impl BinarySpacePartitioning {
    pub fn generate_and_trim_partitions(
        width: u32,
        height: u32,
        config: BinarySpacePartitioningConfig,
    ) -> Vec<(Rect, Vec<Rect>)> {
        if width <= MIN_RECT_WIDTH || height <= MIN_RECT_HEIGHT {
            return vec![];
        }

        let initial_rect = Rect {
            origin: Cell::ZERO,
            width,
            height,
        };

        let region_count = u32::max(initial_rect.area() / config.region_split_factor, 2);
        let region_count = 1;

        event!(
            tracing::Level::DEBUG,
            "Splitting inital rect of {}x{} into {} regions",
            width,
            height,
            region_count
        );

        let regions = Self::generate_regions(initial_rect, region_count, &config);

        let avg_region_area =
            regions.iter().map(|r| r.rect.area()).sum::<u32>() / regions.len() as u32;
        let min_region_area = regions.iter().map(|r| r.rect.area()).min().unwrap_or(0);
        let max_region_area = regions.iter().map(|r| r.rect.area()).max().unwrap_or(0);

        event!(
            tracing::Level::DEBUG,
            "Region area sizes: {}±{}",
            avg_region_area,
            max_region_area - min_region_area
        );

        regions
            .into_par_iter()
            .map(|region| {
                let origin_rect = region.rect;

                let base_time = std::time::Instant::now();

                let (mut region_rects, mut removed_rects, mut neighbours) =
                    Self::generate_partitions(region, &config);

                let generate_time = std::time::Instant::now();

                event!(
                    tracing::Level::DEBUG,
                    "Generated {} partitions for region {} in {:?}",
                    region_rects.len(),
                    origin_rect,
                    generate_time.duration_since(base_time)
                );

                Self::trim_connected_rects(
                    &mut region_rects,
                    &mut removed_rects,
                    &mut neighbours,
                    &config,
                );

                let trim_time = std::time::Instant::now();

                event!(
                    tracing::Level::DEBUG,
                    "Trimmed connected rects for region {} in {:?}",
                    origin_rect,
                    trim_time.duration_since(generate_time)
                );

                Self::trim_orphaned_rects(&mut region_rects, &mut removed_rects, &mut neighbours);

                let trim_orphaned_time = std::time::Instant::now();

                event!(
                    tracing::Level::DEBUG,
                    "Trimmed orphaned rects for region {} in {:?}",
                    origin_rect,
                    trim_orphaned_time.duration_since(trim_time)
                );

                (origin_rect, region_rects.into_values().collect::<Vec<_>>())
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

        let mut rng = rand::rng();

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
    ) -> (
        HashMap<usize, Rect>,
        HashMap<usize, Rect>,
        HashMap<usize, HashSet<usize>>,
    ) {
        let mut rng = rand::rng();

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
            "Splitting region: {} | HF:[{}] - WF:[{}] - HS:[{}]",
            region,
            height_factor_cutoff,
            width_factor_cutoff,
            horizontal_split_prob
        );

        let mut rect_idx = 0_usize;

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

        let mut rng = rand::rng();

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

                let mut rng = rand::rng();

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
