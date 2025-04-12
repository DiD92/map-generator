use crate::{
    constants::{MIN_RECT_HEIGHT, MIN_RECT_WIDTH},
    types::{Cell, Rect, SplitAxis},
};

use std::{collections::HashSet, sync::Mutex};

use rand::Rng;
use rayon::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct BinarySpacePartitioningConfig {
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
            rect_area_cutoff: 2,
            big_rect_area_cutoff: 10,
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

pub struct BinarySpacePartitioning;

impl BinarySpacePartitioning {
    pub fn generate_and_trim_partitions(
        width: u32,
        height: u32,
        config: BinarySpacePartitioningConfig,
    ) -> Vec<Rect> {
        if width <= MIN_RECT_WIDTH || height <= MIN_RECT_HEIGHT {
            return vec![];
        }

        let initial_rect = Rect {
            origin: Cell::ZERO,
            width,
            height,
        };

        let split_rects = Self::generate_partitions(initial_rect, &config);

        let trimmed_rects = Self::trim_split_rects(split_rects, &config);

        Self::trim_orphaned_rects(trimmed_rects)
    }

    fn generate_partitions(
        initial_rect: Rect,
        config: &BinarySpacePartitioningConfig,
    ) -> Vec<Rect> {
        let mut rng = rand::rng();

        let mut rect_stack = vec![initial_rect];

        let mut split_rects = vec![];

        let min_area = config.rect_area_cutoff;
        let max_area = min_area * config.big_rect_area_cutoff;

        while let Some(rect) = rect_stack.pop() {
            let rect_area = rect.area();
            if rect_area > min_area {
                if rect_area <= max_area && rng.random_bool(config.big_rect_survival_prob) {
                    split_rects.push(rect);
                } else {
                    let height_factor = rect.height as f32 / rect.width as f32;
                    let width_factor = rect.width as f32 / rect.height as f32;

                    let split_axis = {
                        if height_factor > config.height_factor_cutoff {
                            SplitAxis::Horizontal
                        } else if width_factor > config.width_factor_cutoff {
                            SplitAxis::Vertical
                        } else if rng.random_bool(config.horizontal_split_prob) {
                            SplitAxis::Horizontal
                        } else {
                            SplitAxis::Vertical
                        }
                    };

                    match split_axis {
                        SplitAxis::Horizontal => {
                            let split_col = rng.random_range(1..rect.height);

                            let (up, down) =
                                rect.try_split_at(SplitAxis::Horizontal, split_col).unwrap();

                            rect_stack.push(up);
                            rect_stack.push(down);
                        }
                        SplitAxis::Vertical => {
                            let split_row = rng.random_range(1..rect.width);

                            let (left, right) =
                                rect.try_split_at(SplitAxis::Vertical, split_row).unwrap();

                            rect_stack.push(left);
                            rect_stack.push(right);
                        }
                    }
                }
            } else if rng.random_bool(config.rect_survival_prob) {
                split_rects.push(rect);
            }
        }

        split_rects
    }

    fn trim_split_rects(rects: Vec<Rect>, config: &BinarySpacePartitioningConfig) -> Vec<Rect> {
        let neighbours = rects.clone();

        rects
            .into_par_iter()
            .filter(|rect| {
                let neighbour_count = neighbours
                    .par_iter()
                    .filter(|other_rect| rect.is_neighbour_of(other_rect))
                    .count();

                let mut rng = rand::rng();

                match neighbour_count {
                    4 => !rng.random_bool(config.trim_fully_connected_rect_prob),
                    3 => !rng.random_bool(config.trim_highly_connected_rect_prob),
                    0 => false,
                    _ => true,
                }
            })
            .collect()
    }

    fn trim_orphaned_rects(rects: Vec<Rect>) -> Vec<Rect> {
        let neighbours = rects.clone();

        rects
            .into_par_iter()
            .filter(|rect| {
                neighbours
                    .par_iter()
                    .filter(|other_rect| rect.is_neighbour_of(other_rect))
                    .count()
                    != 0
            })
            .collect()
    }
}
