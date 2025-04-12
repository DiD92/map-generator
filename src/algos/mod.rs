use crate::types::*;

use std::{
    cell,
    collections::{HashMap, HashSet, hash_map::Entry},
    sync::{Arc, Mutex, atomic::AtomicUsize},
};

use anyhow::Result;
use rand::Rng;
use rayon::{prelude::*, vec};

mod bsp;

pub struct MapBuilderConfig {
    // The probability of merging two rooms into one.
    pub room_merge_prob: f64,
    // Probability of opening a connection between rooms that will
    // cause a navigation loop in the map.
    pub loop_connection_chance: f64,
}

impl Default for MapBuilderConfig {
    fn default() -> Self {
        MapBuilderConfig {
            room_merge_prob: 0.05,
            loop_connection_chance: 0.1,
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
        let now = std::time::SystemTime::now();

        let rects = bsp::BinarySpacePartitioning::generate_and_trim_partitions(
            self.cols,
            self.rows,
            bsp::BinarySpacePartitioningConfig::default(),
        );

        let partition_time = std::time::SystemTime::now();
        println!(
            "Partitions generated and trimmed in {:?}ms",
            partition_time.duration_since(now).unwrap().as_millis()
        );

        let (rooms, neighbours) = Self::generate_initial_rooms(rects);

        let initial_rooms_time = std::time::SystemTime::now();
        println!(
            "Initial rooms generated in {:?}ms",
            initial_rooms_time
                .duration_since(partition_time)
                .unwrap()
                .as_millis()
        );

        let (mut rooms, mut neighbours) = Self::merge_random_rooms(rooms, neighbours, config);

        let merge_time = std::time::SystemTime::now();
        println!(
            "Rooms merged in {:?}ms",
            merge_time
                .duration_since(initial_rooms_time)
                .unwrap()
                .as_millis()
        );

        Self::reconnect_room_groups(&mut rooms, &mut neighbours, config);

        let reconnect_time = std::time::SystemTime::now();
        println!(
            "Rooms reconnected in {:?}ms",
            reconnect_time
                .duration_since(merge_time)
                .unwrap()
                .as_millis()
        );

        let (rooms, doors) = Self::add_doors_to_rooms(rooms, neighbours, config);

        let add_doors_time = std::time::SystemTime::now();
        println!(
            "Doors added in {:?}ms",
            add_doors_time
                .duration_since(reconnect_time)
                .unwrap()
                .as_millis()
        );

        Map { rooms, doors }
    }

    fn generate_initial_rooms(rects: Vec<Rect>) -> (RoomTable, NeighbourTable) {
        let rooms = rects
            .into_par_iter()
            .enumerate()
            .map(|(i, rect)| (i, Room::new_from_rect(rect)))
            .collect::<RoomTable>();

        let neighbour_map = rooms
            .par_iter()
            .map(|(i, _)| (*i, HashSet::new()))
            .collect::<NeighbourTable>();

        let neighbour_map_mutex = Mutex::new(neighbour_map);
        rooms.par_iter().for_each(|(i, room)| {
            rooms.par_iter().for_each(|(j, other_room)| {
                if room.is_neighbour_of(other_room).is_some() {
                    let mut neighbour_map = neighbour_map_mutex.lock().unwrap();
                    neighbour_map.get_mut(i).unwrap().insert(*j);
                }
            });
        });

        (rooms, neighbour_map_mutex.into_inner().unwrap())
    }

    fn merge_random_rooms(
        rooms: RoomTable,
        neighbour_table: NeighbourTable,
        config: &MapBuilderConfig,
    ) -> (RoomTable, NeighbourTable) {
        let rooms_to_merge_mutex = Mutex::new(HashSet::new());

        let merge_groups = neighbour_table
            .par_iter()
            .filter_map(|(i, neighbours)| {
                let mut rng = rand::rng();

                let neighbour_count = neighbours.len();

                if neighbour_count > 0 && rng.random_bool(config.room_merge_prob) {
                    let selected_neighbour = *neighbours
                        .iter()
                        .nth(rng.random_range(0..neighbour_count))
                        .unwrap();

                    if let Ok(ref mut guard) = rooms_to_merge_mutex.lock() {
                        if guard.contains(&selected_neighbour) || guard.contains(i) {
                            return None;
                        }

                        guard.insert(*i);
                        guard.insert(selected_neighbour);
                    } else {
                        println!("Failed to lock rooms_to_merge mutex");
                        return None;
                    }

                    Some((*i, selected_neighbour))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let rooms_mutex = Mutex::new(rooms);
        let neighbour_table_mutex = Mutex::new(neighbour_table);

        merge_groups.into_par_iter().for_each(|(from, to)| {
            if let Ok(ref mut rooms) = rooms_mutex.lock() {
                let from_room = rooms.remove(&from).unwrap();
                let to_room = rooms.remove(&to).unwrap();

                let merged_room = from_room.merged_with(to_room);
                rooms.insert(from, merged_room);
            } else {
                println!("Failed to lock rooms mutex");
                return;
            }

            if let Ok(ref mut neighbour_table) = neighbour_table_mutex.lock() {
                let mut from_neighbours = neighbour_table.remove(&from).unwrap();
                from_neighbours.remove(&to);

                let mut to_neighbours = neighbour_table.remove(&to).unwrap();
                to_neighbours.remove(&from);

                for neighbour in to_neighbours.iter() {
                    if let Some(neighbours) = neighbour_table.get_mut(neighbour) {
                        neighbours.remove(&to);
                        neighbours.insert(from);
                    }
                }

                from_neighbours.extend(to_neighbours);
                neighbour_table.insert(from, from_neighbours);
            } else {
                println!("Failed to lock neighbour table mutex");
                return;
            }
        });

        (
            rooms_mutex.into_inner().unwrap(),
            neighbour_table_mutex.into_inner().unwrap(),
        )
    }

    fn reconnect_room_groups(
        rooms: &mut RoomTable,
        neighbour_table: &mut NeighbourTable,
        config: &MapBuilderConfig,
    ) {
        let mut room_groups = Self::generate_room_groups(&rooms, &neighbour_table);

        loop {
            // We remove groups with just 1 room
            room_groups.retain(|_, group| {
                if group.len() > 1 {
                    true
                } else {
                    for room_id in group.iter() {
                        rooms.remove(room_id);
                        neighbour_table.remove(room_id);
                    }

                    false
                }
            });

            if room_groups.len() > 1 {
                // If there is more than one group, we need to connect them together
                Self::connect_room_groups(room_groups, rooms, neighbour_table);

                room_groups = Self::generate_room_groups(&rooms, &neighbour_table);
            } else {
                break;
            }
        }

        // We randomly merge some groups of 1 sized-rooms or 2x1-sized rooms
        Self::merge_repeated_simple_rooms(rooms, neighbour_table, config.room_merge_prob)
    }

    fn generate_room_groups(
        rooms: &RoomTable,
        neighbour_table: &NeighbourTable,
    ) -> HashMap<usize, HashSet<RoomId>> {
        let mut room_groups = HashMap::new();
        let mut group_id = 0;
        let mut map_rooms = rooms.keys().cloned().collect::<HashSet<_>>();

        while !map_rooms.is_empty() {
            let initial_room = map_rooms.iter().next().unwrap().clone();

            let mut rooms_to_visit = vec![initial_room];
            let mut visited_rooms = HashSet::new();

            while let Some(room_id) = rooms_to_visit.pop() {
                visited_rooms.insert(room_id);
                map_rooms.remove(&room_id);

                for neighbour_id in neighbour_table[&room_id].iter() {
                    if !visited_rooms.contains(neighbour_id) {
                        rooms_to_visit.push(*neighbour_id);
                    }
                }
            }

            room_groups.insert(group_id, visited_rooms);

            group_id += 1;
        }

        room_groups
    }

    fn connect_room_groups(
        room_groups: HashMap<usize, HashSet<RoomId>>,
        rooms: &mut RoomTable,
        neighbour_table: &mut NeighbourTable,
    ) {
        let group_count = room_groups.len() as f32;
        let group_size_cutoff = (room_groups
            .par_iter()
            .map(|(_, group_rooms)| group_rooms.len() as f32)
            .sum::<f32>()
            / group_count)
            * 0.3;

        let next_room_id = AtomicUsize::new(rooms.keys().max().unwrap().clone() + 1);

        let rooms_to_remove = Mutex::new(Vec::new());

        let group_centers =
            room_groups
                .par_iter()
                .filter_map(|(group_id, group)| {
                    if group.len() as f32 > group_size_cutoff {
                        Some((group_id, group))
                    } else {
                        rooms_to_remove
                            .lock()
                            .unwrap()
                            .extend(group.iter().copied());

                        None
                    }
                })
                .map(|(group_id, group)| {
                    let (mut center, count) =
                        group
                            .par_iter()
                            .flat_map(|room_id| {
                                rooms.get(room_id).unwrap().cells.par_iter().map(|cell| {
                                    (Vector2::new(cell.col as f32, cell.row as f32), 1_u32)
                                })
                            })
                            .reduce(
                                || (Vector2::ZERO, 0_u32),
                                |(mut center, mut count), (other_vector, other_count)| {
                                    center.x += other_vector.x;
                                    center.y += other_vector.y;
                                    count += other_count;

                                    (center, count)
                                },
                            );

                    center.x /= count as f32;
                    center.y /= count as f32;

                    (group_id, center)
                })
                .collect::<HashMap<_, _>>();

        let rooms_to_remove = rooms_to_remove.into_inner().unwrap();

        for room_id in rooms_to_remove.into_iter() {
            rooms.remove(&room_id);
            neighbour_table.remove(&room_id);
        }

        let mut closer_groups = Vec::new();
        let mut visited_links = HashSet::new();

        for (group_id, center) in group_centers.iter() {
            let mut min_distance = f32::MAX;
            let mut maybe_closest_group_id = None;

            for (other_group_id, other_center) in group_centers.iter() {
                if group_id == other_group_id
                    || visited_links.contains(&(*other_group_id, *group_id))
                {
                    continue;
                }

                let center_distance = center.distance(other_center);

                if center_distance < min_distance {
                    min_distance = center_distance;
                    maybe_closest_group_id = Some(*other_group_id);
                }
            }

            if let Some(closest_group_id) = maybe_closest_group_id {
                closer_groups.push((*group_id, closest_group_id, min_distance));

                visited_links.insert((*group_id, closest_group_id));
                visited_links.insert((closest_group_id, *group_id));
            }
        }

        let cell_map_mutex = Mutex::new(
            rooms
                .par_iter()
                .flat_map(|(room_id, room)| {
                    room.cells.par_iter().copied().map(|cell| (cell, *room_id))
                })
                .collect::<HashMap<_, _>>(),
        );

        let new_rooms = Mutex::new(Vec::new());

        closer_groups
            .into_par_iter()
            .for_each(|(group_a, group_b, _distance)| {
                let cells_a = {
                    let rooms_a = room_groups.get(group_a).unwrap();

                    rooms_a.iter().fold(Vec::new(), |mut acc, room_id| {
                        let room = rooms.get(room_id).unwrap();
                        acc.extend(room.cells.clone());
                        acc
                    })
                };

                let cells_b = {
                    let rooms_b = room_groups.get(group_b).unwrap();
                    rooms_b.iter().fold(Vec::new(), |mut acc, room_id| {
                        let room = rooms.get(room_id).unwrap();
                        acc.extend(room.cells.clone());
                        acc
                    })
                };

                let mut min_distance = u32::MAX;
                let mut selected_cell_a = Cell::ZERO;
                let mut selected_cell_b = Cell::ZERO;

                'outer: for cell_a in cells_a.iter() {
                    for cell_b in cells_b.iter() {
                        let cell_distance = cell_a.distance(cell_b);
                        if cell_distance < min_distance {
                            min_distance = cell_distance;
                            selected_cell_a = *cell_a;
                            selected_cell_b = *cell_b;
                        }

                        if min_distance == 0 {
                            break 'outer;
                        }
                    }
                }

                let mut cell_map = cell_map_mutex.lock().unwrap();

                let room = Self::connect_cells(selected_cell_a, selected_cell_b, &cell_map);
                if room.cells.len() > 0 {
                    // If we managed to connect the two points we store the new room
                    let new_room_id =
                        next_room_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    cell_map.extend(room.cells.iter().copied().map(|cell| (cell, new_room_id)));
                    new_rooms.lock().unwrap().push((new_room_id, room));
                }
            });

        let cell_map = cell_map_mutex.into_inner().unwrap();

        let new_rooms = new_rooms.into_inner().unwrap();

        for (room_id, _) in new_rooms.iter() {
            neighbour_table.insert(*room_id, HashSet::new());
        }

        for (room_id, room) in new_rooms.into_iter() {
            let mut room_neighours = HashSet::new();
            for cell in room.cells.iter() {
                for maybe_neighbour in cell.neighbours() {
                    if let Some(neighbour_id) = cell_map.get(&maybe_neighbour) {
                        if neighbour_id != &room_id {
                            room_neighours.insert(*neighbour_id);
                            neighbour_table
                                .get_mut(neighbour_id)
                                .unwrap()
                                .insert(room_id);
                        }
                    }
                }
            }

            neighbour_table
                .get_mut(&room_id)
                .unwrap()
                .extend(room_neighours);
            rooms.insert(room_id, room);
        }
    }

    fn connect_cells(
        point_a: Cell,
        point_b: Cell,
        occupied_points: &HashMap<Cell, RoomId>,
    ) -> Room {
        let mut visited_cells = HashSet::new();
        let mut cell_stack = vec![point_a];

        while let Some(cell) = cell_stack.pop() {
            if !occupied_points.contains_key(&cell) {
                visited_cells.insert(cell);
            }

            if cell.is_neighbour_of(&point_b).is_some() {
                break;
            }

            let current_distance = cell.distance(&point_b);

            for neighbour in cell.neighbours() {
                let distance = neighbour.distance(&point_b);
                if !visited_cells.contains(&neighbour)
                    && !occupied_points.contains_key(&neighbour)
                    && distance < current_distance
                {
                    cell_stack.push(neighbour);
                }
            }
        }

        Room {
            cells: visited_cells.drain().collect(),
            modifier: RoomModifier::Connector,
            color: RoomColor::Purple,
        }
    }

    fn merge_repeated_simple_rooms(
        rooms: &mut RoomTable,
        neighbour_table: &mut NeighbourTable,
        merge_prob: f64,
    ) {
        let mut merge_candidates = HashSet::new();
        let mut non_merge_candidates = HashSet::new();

        for (i, room) in rooms.iter() {
            let room_cells = room.cells.len();

            if room_cells == 1 || room_cells == 2 {
                merge_candidates.insert(*i);
            } else {
                non_merge_candidates.insert(*i);
            }
        }

        let mut visited_rooms = HashSet::new();
        let mut merge_pairs = HashMap::new();

        let mut rng = rand::rng();

        for room_id in merge_candidates.iter() {
            if visited_rooms.contains(room_id) {
                continue;
            }

            visited_rooms.insert(room_id);

            let mut room_merged = false;

            let room = &rooms[room_id];

            for neighbour_id in neighbour_table[room_id].iter() {
                if visited_rooms.contains(neighbour_id) {
                    continue;
                }

                let neighbour_room = &rooms[neighbour_id];

                if room.cells.len() == neighbour_room.cells.len() && rng.random_bool(merge_prob) {
                    visited_rooms.insert(neighbour_id);

                    room_merged = true;
                }

                if room_merged {
                    merge_pairs.insert(*room_id, *neighbour_id);
                    break;
                }
            }

            if !room_merged {
                non_merge_candidates.insert(room_id.clone());
            }
        }
        for (from, to) in merge_pairs.into_iter() {
            let from_room = rooms.remove(&from).unwrap();
            let to_room = rooms.remove(&to).unwrap();

            let merged_room = from_room.merged_with(to_room);
            rooms.insert(from, merged_room);

            let mut to_neighbours = neighbour_table.remove(&to).unwrap();
            to_neighbours.remove(&from);

            for neighbour in to_neighbours.iter() {
                if let Some(neighbours) = neighbour_table.get_mut(neighbour) {
                    neighbours.remove(&to);
                    neighbours.insert(from);
                }
            }

            let from_neighbours = neighbour_table.get_mut(&from).unwrap();
            from_neighbours.remove(&to);

            from_neighbours.extend(to_neighbours);
        }
    }

    fn add_doors_to_rooms(
        rooms: RoomTable,
        neighbour_table: NeighbourTable,
        config: &MapBuilderConfig,
    ) -> (Vec<Room>, Vec<Door>) {
        let mut doors = Vec::with_capacity(rooms.len());

        let mut visited_rooms = HashSet::new();
        //let mut connected_count = HashMap::<RoomId, RoomId>::new();

        /*for room in rooms.keys() {
            connected_count.insert(*room, 0);
        }*/

        let mut rng = rand::rng();

        let initial_room = {
            let idx = rng.random_range(0..rooms.len());
            rooms.keys().nth(idx).unwrap().clone()
        };

        let mut room_queue = Vec::new();
        room_queue.push(initial_room);

        //let mut neighbour_candidates = Vec::new();

        while let Some(room_id) = room_queue.pop() {
            visited_rooms.insert(room_id);

            let room = &rooms[&room_id];
            for neighbour_id in neighbour_table[&room_id].iter() {
                if visited_rooms.contains(&neighbour_id) {
                    continue;
                }

                let neighbour_room = &rooms[neighbour_id];

                if let Some((from, to, _)) = room.is_neighbour_of(neighbour_room) {
                    let door = Door::new(from, to);
                    doors.push(door);

                    room_queue.push(*neighbour_id);
                }
            }

            /*neighbour_candidates.sort_by(|(_, _, dir_a, _), (_, _, dir_b, _)| {
                if dir_a.is_horizontal() && !dir_b.is_horizontal() {
                    return std::cmp::Ordering::Less;
                } else if !dir_a.is_horizontal() && dir_b.is_horizontal() {
                    return std::cmp::Ordering::Greater;
                }
                std::cmp::Ordering::Equal
            });

            for (from, to, _, neighbour_id) in neighbour_candidates.drain(..) {
                let already_connected = connected_rooms.contains(neighbour_id);

                if !already_connected /*|| rng.random_bool(config.loop_connection_chance)*/ {
                    let door = Door::new(from, to);
                    doors.push(door);

                    connected_rooms.insert(*neighbour_id);
                }

                if !visited_rooms.contains(neighbour_id) {
                    room_queue.push(*neighbour_id);
                }
            }*/
        }

        println!("Visited rooms: {}", visited_rooms.len());
        println!("Total rooms: {}", rooms.len());

        (rooms.into_iter().map(|(_, room)| room).collect(), doors)
    }
}

pub struct PolygonBuilder;

impl PolygonBuilder {
    pub fn build_for(room: &Room) -> (HashSet<Cell>, HashSet<Edge>) {
        let mut valid_vertices = HashSet::new();
        let mut valid_edges = HashSet::new();

        let mut edges_to_remove = HashSet::new();

        for cell in &room.cells {
            for vertex in cell.get_vertices() {
                valid_vertices.insert(vertex);
            }

            for edge in cell.get_edges() {
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
        let mut vertices_to_remove = Vec::new();

        for vertex in valid_vertices.iter() {
            let mut neighbour_count = 0;
            for neighbour in vertex.neighbours() {
                let neighbour_edge = Edge::new(*vertex, neighbour);

                if valid_edges.contains(&neighbour_edge) {
                    neighbour_count += 1;
                }
            }

            if neighbour_count != 2 {
                vertices_to_remove.push(*vertex);
            }
        }

        for vertex in vertices_to_remove.iter() {
            valid_vertices.remove(vertex);
        }

        (valid_vertices, valid_edges)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn polygon_builder_works_for_simple_room() {
        let rect = Rect {
            origin: Cell { col: 0, row: 0 },
            width: 2,
            height: 2,
        };

        let room = Room::new_from_rect(rect);

        let (vertices, edges) = PolygonBuilder::build_for(&room);

        assert_eq!(vertices.len(), 8);
        assert_eq!(edges.len(), 8);

        let expected_vertices: HashSet<Cell> = vec![
            Cell { col: 0, row: 0 },
            Cell { col: 1, row: 0 },
            Cell { col: 2, row: 0 },
            Cell { col: 0, row: 1 },
            Cell { col: 0, row: 2 },
            Cell { col: 1, row: 2 },
            Cell { col: 2, row: 2 },
            Cell { col: 2, row: 1 },
        ]
        .into_iter()
        .collect();

        assert_eq!(vertices, expected_vertices);

        let expected_edges: HashSet<Edge> = vec![
            Edge::new(Cell { col: 0, row: 0 }, Cell { col: 1, row: 0 }),
            Edge::new(Cell { col: 1, row: 0 }, Cell { col: 2, row: 0 }),
            Edge::new(Cell { col: 0, row: 0 }, Cell { col: 0, row: 1 }),
            Edge::new(Cell { col: 0, row: 1 }, Cell { col: 0, row: 2 }),
            Edge::new(Cell { col: 0, row: 2 }, Cell { col: 1, row: 2 }),
            Edge::new(Cell { col: 1, row: 2 }, Cell { col: 2, row: 2 }),
            Edge::new(Cell { col: 2, row: 0 }, Cell { col: 2, row: 1 }),
            Edge::new(Cell { col: 2, row: 1 }, Cell { col: 2, row: 2 }),
        ]
        .into_iter()
        .collect();

        assert_eq!(edges, expected_edges);
    }

    #[test]
    fn polygon_builder_works_for_complex_room() {
        let rect_1 = Rect {
            origin: Cell { col: 0, row: 0 },
            width: 2,
            height: 1,
        };
        let rect_2 = Rect {
            origin: Cell { col: 2, row: 0 },
            width: 2,
            height: 2,
        };

        let room_1 = Room::new_from_rect(rect_1);
        let room_2 = Room::new_from_rect(rect_2);
        let room = room_1.merged_with(room_2);

        assert_eq!(room.cells.len(), 6);

        let (vertices, edges) = PolygonBuilder::build_for(&room);

        assert_eq!(vertices.len(), 12);
        assert_eq!(edges.len(), 12);

        let expected_vertices: HashSet<Cell> = vec![
            Cell { col: 0, row: 0 },
            Cell { col: 1, row: 0 },
            Cell { col: 2, row: 0 },
            Cell { col: 3, row: 0 },
            Cell { col: 4, row: 0 },
            Cell { col: 4, row: 1 },
            Cell { col: 4, row: 2 },
            Cell { col: 3, row: 2 },
            Cell { col: 2, row: 2 },
            Cell { col: 2, row: 1 },
            Cell { col: 1, row: 1 },
            Cell { col: 0, row: 1 },
        ]
        .into_iter()
        .collect();

        assert_eq!(vertices, expected_vertices);

        let expected_edges: HashSet<Edge> = vec![
            Edge::new(Cell { col: 0, row: 0 }, Cell { col: 1, row: 0 }),
            Edge::new(Cell { col: 1, row: 0 }, Cell { col: 2, row: 0 }),
            Edge::new(Cell { col: 2, row: 0 }, Cell { col: 3, row: 0 }),
            Edge::new(Cell { col: 3, row: 0 }, Cell { col: 4, row: 0 }),
            Edge::new(Cell { col: 4, row: 0 }, Cell { col: 4, row: 1 }),
            Edge::new(Cell { col: 4, row: 1 }, Cell { col: 4, row: 2 }),
            Edge::new(Cell { col: 4, row: 2 }, Cell { col: 3, row: 2 }),
            Edge::new(Cell { col: 3, row: 2 }, Cell { col: 2, row: 2 }),
            Edge::new(Cell { col: 2, row: 2 }, Cell { col: 2, row: 1 }),
            Edge::new(Cell { col: 2, row: 1 }, Cell { col: 1, row: 1 }),
            Edge::new(Cell { col: 1, row: 1 }, Cell { col: 0, row: 1 }),
            Edge::new(Cell { col: 0, row: 1 }, Cell { col: 0, row: 0 }),
        ]
        .into_iter()
        .collect();

        assert_eq!(edges, expected_edges);
    }
}
