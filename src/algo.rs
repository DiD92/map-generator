use crate::{consants::*, types::*};

use std::collections::HashSet;

use anyhow::Result;
use rand::Rng;

pub struct MapBuilderConfig {
    // The minimum area of a rectangle to be considered for splitting.
    pub rect_area_cutoff: u32,
    // The minimum height to width ratio at which we will always perform a
    // horizontal split.
    pub height_factor_cutoff: f32,
    // The minimum width to height ratio at which we will always perform a
    // vertical split.
    pub width_factor_cutoff: f32,
    // The random probability of performing a horizontal split.
    pub horizontal_split_prob: f64,
    // The probability of removing a highly connected rectangle.
    pub remove_highly_connected_rect_prob: f64,
    // The probability of removing a fully connected rectangle.
    pub remove_fully_connected_rect_prob: f64,
}

impl Default for MapBuilderConfig {
    fn default() -> Self {
        MapBuilderConfig {
            rect_area_cutoff: 6,
            height_factor_cutoff: 1.8,
            width_factor_cutoff: 2.3,
            horizontal_split_prob: 0.5,
            remove_highly_connected_rect_prob: 0.4,
            remove_fully_connected_rect_prob: 0.5,
        }
    }
}

pub struct MapBuilder {
    pub cols: u32,
    pub rows: u32,
}

impl MapBuilder {
    pub fn new(cols: u32, rows: u32) -> Result<Self> {
        if cols == 0 || rows == 0 {
            return Err(anyhow::anyhow!(
                "Columns and rows must be greater than zero"
            ));
        }

        Ok(MapBuilder { cols, rows })
    }

    pub fn build(&self, config: &MapBuilderConfig) -> Map {
        let mut map = Map::new();

        let initial_rect = Rect {
            origin: Point::ZERO,
            width: self.cols,
            height: self.rows,
        };

        let mut rng = rand::rng();
        let mut rect_stack = vec![initial_rect];

        let mut split_rects = vec![];

        while let Some(rect) = rect_stack.pop() {
            let rect_area = rect.area();
            if rect_area > config.rect_area_cutoff {
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
            } else if !rng.random_bool(REMOVE_RECT_PROB) {
                split_rects.push(rect);
            }
        }

        let mut rects_to_keep = Vec::new();

        // We randomly trim the rects with 3 or more neighbours
        for rect in split_rects.iter() {
            let mut neighbour_count = 0;
            for other_rect in split_rects.iter() {
                if rect == other_rect {
                    continue;
                }

                if rect.is_neighbour_of(other_rect) {
                    neighbour_count += 1;
                }
            }

            if neighbour_count > 0 {
                if neighbour_count == 4 {
                    if !rng.random_bool(config.remove_fully_connected_rect_prob) {
                        rects_to_keep.push(*rect);
                    }
                } else if neighbour_count >= 3 {
                    if !rng.random_bool(config.remove_highly_connected_rect_prob) {
                        rects_to_keep.push(*rect);
                    }
                } else {
                    rects_to_keep.push(*rect);
                }
            }
        }

        // Now we remove all orphaned rects

        for rect in rects_to_keep {
            let room = Room::new(vec![rect]);
            map.add_room(room);
        }

        map
    }
}

pub struct PolygonBuilder;

impl PolygonBuilder {
    pub fn build_for(room: &Room) -> (HashSet<Point>, HashSet<Edge>) {
        let mut valid_points = HashSet::new();
        let mut valid_edges = HashSet::new();

        let mut edges_to_remove = HashSet::new();

        for rect in &room.rects {
            for point in rect.into_points() {
                valid_points.insert(point);
            }

            for edge in rect.into_edges() {
                if valid_edges.contains(&edge) {
                    edges_to_remove.insert(edge);
                }
                valid_edges.insert(edge);
            }
        }

        for edge in edges_to_remove.iter() {
            valid_edges.remove(edge);
        }

        // To know if a point should be kept, we need to check if it has 2 edges connecting to it
        let mut points_to_remove = Vec::new();

        for point in valid_points.iter() {
            let mut neighbour_count = 0;
            for neighbour in point.neighbours() {
                let neighbour_edge = Edge::new(*point, neighbour);

                if valid_edges.contains(&neighbour_edge) {
                    neighbour_count += 1;
                }
            }

            if neighbour_count != 2 {
                points_to_remove.push(*point);
            }
        }

        for point in points_to_remove.iter() {
            valid_points.remove(point);
        }

        (valid_points, valid_edges)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn polygon_builder_works_for_simple_room() {
        let rect = Rect {
            origin: Point { col: 0, row: 0 },
            width: 2,
            height: 2,
        };

        let room = Room::new(vec![rect]);

        let (valid_points, valid_edges) = PolygonBuilder::build_for(&room);

        assert_eq!(valid_points.len(), 8);
        assert_eq!(valid_edges.len(), 8);

        let expected_points: HashSet<Point> = vec![
            Point { col: 0, row: 0 },
            Point { col: 1, row: 0 },
            Point { col: 2, row: 0 },
            Point { col: 0, row: 1 },
            Point { col: 0, row: 2 },
            Point { col: 1, row: 2 },
            Point { col: 2, row: 2 },
            Point { col: 2, row: 1 },
        ]
        .into_iter()
        .collect();

        assert_eq!(valid_points, expected_points);

        let expected_edges: HashSet<Edge> = vec![
            Edge::new(Point { col: 0, row: 0 }, Point { col: 1, row: 0 }),
            Edge::new(Point { col: 1, row: 0 }, Point { col: 2, row: 0 }),
            Edge::new(Point { col: 0, row: 0 }, Point { col: 0, row: 1 }),
            Edge::new(Point { col: 0, row: 1 }, Point { col: 0, row: 2 }),
            Edge::new(Point { col: 0, row: 2 }, Point { col: 1, row: 2 }),
            Edge::new(Point { col: 1, row: 2 }, Point { col: 2, row: 2 }),
            Edge::new(Point { col: 2, row: 0 }, Point { col: 2, row: 1 }),
            Edge::new(Point { col: 2, row: 1 }, Point { col: 2, row: 2 }),
        ]
        .into_iter()
        .collect();

        assert_eq!(valid_edges, expected_edges);
    }

    #[test]
    fn polygon_builder_works_for_complex_room() {
        let rect_1 = Rect {
            origin: Point { col: 0, row: 0 },
            width: 2,
            height: 1,
        };
        let rect_2 = Rect {
            origin: Point { col: 2, row: 0 },
            width: 2,
            height: 2,
        };

        let room = Room::new(vec![rect_1, rect_2]);

        let (valid_points, valid_edges) = PolygonBuilder::build_for(&room);

        assert_eq!(valid_points.len(), 12);
        assert_eq!(valid_edges.len(), 12);

        let expected_points: HashSet<Point> = vec![
            Point { col: 0, row: 0 },
            Point { col: 1, row: 0 },
            Point { col: 2, row: 0 },
            Point { col: 3, row: 0 },
            Point { col: 4, row: 0 },
            Point { col: 4, row: 1 },
            Point { col: 4, row: 2 },
            Point { col: 3, row: 2 },
            Point { col: 2, row: 2 },
            Point { col: 2, row: 1 },
            Point { col: 1, row: 1 },
            Point { col: 0, row: 1 },
        ]
        .into_iter()
        .collect();

        assert_eq!(valid_points, expected_points);

        let expected_edges: HashSet<Edge> = vec![
            Edge::new(Point { col: 0, row: 0 }, Point { col: 1, row: 0 }),
            Edge::new(Point { col: 1, row: 0 }, Point { col: 2, row: 0 }),
            Edge::new(Point { col: 2, row: 0 }, Point { col: 3, row: 0 }),
            Edge::new(Point { col: 3, row: 0 }, Point { col: 4, row: 0 }),
            Edge::new(Point { col: 4, row: 0 }, Point { col: 4, row: 1 }),
            Edge::new(Point { col: 4, row: 1 }, Point { col: 4, row: 2 }),
            Edge::new(Point { col: 4, row: 2 }, Point { col: 3, row: 2 }),
            Edge::new(Point { col: 3, row: 2 }, Point { col: 2, row: 2 }),
            Edge::new(Point { col: 2, row: 2 }, Point { col: 2, row: 1 }),
            Edge::new(Point { col: 2, row: 1 }, Point { col: 1, row: 1 }),
            Edge::new(Point { col: 1, row: 1 }, Point { col: 0, row: 1 }),
            Edge::new(Point { col: 0, row: 1 }, Point { col: 0, row: 0 }),
        ]
        .into_iter()
        .collect();

        assert_eq!(valid_edges, expected_edges);
    }
}
