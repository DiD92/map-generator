use crate::{consants::*, types::*};

use std::collections::{HashMap, HashSet};

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
    // The probability of removing a cell from a rect before creating a room.
    pub remove_cell_prob: f64,
    // The probability of merging two rooms into one.
    pub room_merge_prov: f64,
}

impl Default for MapBuilderConfig {
    fn default() -> Self {
        MapBuilderConfig {
            rect_area_cutoff: 2,
            height_factor_cutoff: 1.8,
            width_factor_cutoff: 2.5,
            horizontal_split_prob: 0.6,
            remove_highly_connected_rect_prob: 0.4,
            remove_fully_connected_rect_prob: 0.5,
            remove_cell_prob: 0.7,
            room_merge_prov: 0.4,
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

        // We remove the rects that have remained orphaned
        let mut orphaned_rects = Vec::new();
        for rect in rects_to_keep.iter() {
            let mut neighbour_count = 0;
            for other_rect in rects_to_keep.iter() {
                if rect == other_rect {
                    continue;
                }

                if rect.is_neighbour_of(other_rect) {
                    neighbour_count += 1;
                }
            }

            if neighbour_count == 0 {
                orphaned_rects.push(*rect);
            }
        }

        for rect in orphaned_rects.iter() {
            rects_to_keep.retain(|&r| r != *rect);
        }

        let mut intial_rooms = Vec::new();

        // We randomly remove some cells from the rects of a certain size
        for rect in rects_to_keep {
            let room =
                if rect.width > 1 && rect.height > 1 && rng.random_bool(config.remove_cell_prob) {
                    let points = rect.into_points();
                    let idx = rng.random_range(0..points.len());

                    let rects = rect.puncture(points[idx]);

                    Room::new(rects)
                } else {
                    Room::new(vec![rect])
                };

            intial_rooms.push(room);
        }

        let mut map = Map::new();

        let mut rooms_to_merge = intial_rooms.clone().into_iter().collect::<HashSet<_>>();

        let mut merged_rooms = HashSet::new();

        let mut initial_rooms_2 = Vec::new();

        // We randomly merge some rooms
        for room in intial_rooms.into_iter() {
            if merged_rooms.contains(&room) {
                continue;
            }

            if rng.random_bool(config.room_merge_prov) {
                let mut maybe_room_to_merge = None;

                for maybe_neighbour in rooms_to_merge.iter() {
                    if room.is_neighbour_of(maybe_neighbour) {
                        maybe_room_to_merge = Some(maybe_neighbour.clone());
                        break;
                    }
                }

                if let Some(room_to_merge) = maybe_room_to_merge {
                    rooms_to_merge.remove(&room_to_merge);
                    rooms_to_merge.remove(&room);

                    merged_rooms.insert(room.clone());
                    merged_rooms.insert(room_to_merge.clone());

                    let merged_room = room.merged_with(room_to_merge);
                    //map.add_room(merged_room.clone());
                    initial_rooms_2.push(merged_room);
                }
            } else {
                rooms_to_merge.remove(&room);
                initial_rooms_2.push(room);
                //map.add_room(room);
            }
        }

        let mut room_link_map = HashMap::<Room, HashSet<Room>>::new();
        let neighour_set = initial_rooms_2.clone().into_iter().collect::<HashSet<_>>();

        for room in initial_rooms_2.iter() {
            for maybe_neighbour in neighour_set.iter() {
                if room == maybe_neighbour {
                    continue;
                }

                if room.is_neighbour_of(maybe_neighbour) {
                    if let Some(neighbours) = room_link_map.get_mut(room) {
                        neighbours.insert(maybe_neighbour.clone());
                    } else {
                        let mut neighbours = HashSet::new();
                        neighbours.insert(maybe_neighbour.clone());
                        room_link_map.insert(room.clone(), neighbours);
                    }
                }
            }

            map.add_room(room.clone());
        }

        // We remove the rooms that are not connected to any other room
        map.rooms = map
            .rooms
            .into_iter()
            .filter(|room| room_link_map.contains_key(room))
            .collect();

        let mut room_groups = Vec::new();
        let mut map_rooms = map.rooms.clone().into_iter().collect::<HashSet<_>>();

        while !map_rooms.is_empty() {
            let initial_room = map_rooms.iter().next().unwrap().clone();
            let mut rooms_to_visit = vec![initial_room];
            let mut visited_rooms = HashSet::new();

            while let Some(room) = rooms_to_visit.pop() {
                visited_rooms.insert(room.clone());
                map_rooms.remove(&room);

                if let Some(neighbours) = room_link_map.get(&room) {
                    for neighbour in neighbours.iter() {
                        if !visited_rooms.contains(neighbour) {
                            rooms_to_visit.push(neighbour.clone());
                        }
                    }
                }
            }

            room_groups.push(visited_rooms);
        }

        println!("Room groups: {}", room_groups.len());

        for key in room_link_map.keys() {
            println!(
                "{:?}: {:?}",
                key,
                room_link_map.get(key).unwrap_or(&HashSet::default())
            );
        }

        map.rooms.clear();

        for group in room_groups.into_iter() {
            let color = match rng.random_range(1..6) {
                1 => RoomColor::Red,
                2 => RoomColor::Green,
                3 => RoomColor::Blue,
                4 => RoomColor::Yellow,
                5 => RoomColor::Purple,
                _ => RoomColor::Red,
            };

            for mut room in group.into_iter() {
                room.color = color;
                map.add_room(room);
            }
        }

        // We need to fill the empty spaces with rooms

        // We need to add doors to most rooms

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
