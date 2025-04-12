use super::{MapBuilder, MapBuilderConfig};
use crate::types::{Cell, NeighbourTable, Room, RoomId, RoomModifier, RoomTable, Vector2};

use std::{
    collections::{HashMap, HashSet},
    sync::{Mutex, atomic::AtomicUsize},
};

use rand::Rng;
use rayon::prelude::*;

impl MapBuilder {
    pub(super) fn reconnect_room_groups(
        rooms: &mut RoomTable,
        neighbour_table: &mut NeighbourTable,
        config: &MapBuilderConfig,
    ) {
        let mut room_groups = Self::generate_room_groups(rooms, neighbour_table);

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

                room_groups = Self::generate_room_groups(rooms, neighbour_table);
            } else {
                break;
            }
        }

        // We randomly merge some groups of 1 sized-rooms first
        Self::merge_repeated_simple_rooms(
            rooms,
            neighbour_table,
            1,
            config.repeat_small_room_merge_prob,
        );
        // Then we merge rooms of size 2 or less
        Self::merge_repeated_simple_rooms(
            rooms,
            neighbour_table,
            2,
            config.repeat_small_room_merge_prob / 2.0,
        );
    }

    fn generate_room_groups(
        rooms: &RoomTable,
        neighbour_table: &NeighbourTable,
    ) -> HashMap<usize, HashSet<RoomId>> {
        let mut room_groups = HashMap::new();
        let mut group_id = 0;
        let mut map_rooms = rooms.keys().cloned().collect::<HashSet<_>>();

        while !map_rooms.is_empty() {
            let initial_room = *map_rooms.iter().next().unwrap();

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

        let next_room_id = AtomicUsize::new(*rooms.keys().max().unwrap() + 1);

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

                let rooms = Self::connect_cells(selected_cell_a, selected_cell_b, &cell_map);
                for room in rooms {
                    if !room.cells.is_empty() {
                        // If we managed to connect the two points we store the new room
                        let new_room_id =
                            next_room_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                        cell_map.extend(room.cells.iter().copied().map(|cell| (cell, new_room_id)));
                        new_rooms.lock().unwrap().push((new_room_id, room));
                    }
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
    ) -> Vec<Room> {
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

        let mut cell_groups = Vec::new();

        let mut cells_to_visit = visited_cells.clone();

        while !cells_to_visit.is_empty() {
            let mut cell_stack = vec![*cells_to_visit.iter().next().unwrap()];

            let mut cell_group = Vec::new();

            while let Some(cell) = cell_stack.pop() {
                if cells_to_visit.remove(&cell) {
                    cell_group.push(cell);
                }

                for neighbour in cells_to_visit.iter() {
                    if cell.is_neighbour_of(neighbour).is_some()
                        && cells_to_visit.contains(neighbour)
                    {
                        cell_stack.push(*neighbour);
                    }
                }
            }

            cell_groups.push(cell_group);
        }

        cell_groups
            .into_iter()
            .map(|cells| Room {
                cells,
                modifier: RoomModifier::Connector,
            })
            .collect::<Vec<_>>()
    }

    fn merge_repeated_simple_rooms(
        rooms: &mut RoomTable,
        neighbour_table: &mut NeighbourTable,
        max_size: usize,
        merge_prob: f64,
    ) {
        let mut merge_candidates = HashSet::new();

        let mut non_merge_candidates = HashSet::new();

        for (i, room) in rooms.iter() {
            let room_cells = room.cells.len();

            if room_cells <= max_size {
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

            for neighbour_id in neighbour_table[room_id].iter() {
                if visited_rooms.contains(neighbour_id) {
                    continue;
                }

                let neighbour_room = rooms.get(neighbour_id).unwrap();

                if neighbour_room.cells.len() > max_size {
                    continue;
                }

                if rng.random_bool(merge_prob) {
                    visited_rooms.insert(neighbour_id);

                    merge_pairs.insert(*room_id, *neighbour_id);

                    room_merged = true;

                    break;
                }
            }

            if !room_merged {
                non_merge_candidates.insert(*room_id);
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
}
