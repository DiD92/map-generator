use super::{MapBuilder, MapBuilderConfig};
use crate::types::{Cell, MapRegion, Room, RoomId, RoomModifier, Vector2};

use std::{
    collections::{HashMap, HashSet},
    sync::{Mutex, atomic::AtomicUsize},
};

use rand::Rng;
use rayon::prelude::*;

impl MapBuilder {
    pub(super) fn reconnect_room_groups(map_region: &mut MapRegion, config: &MapBuilderConfig) {
        let mut room_groups = Self::generate_room_groups(map_region);

        loop {
            // We remove groups with just 1 room
            room_groups.retain(|_, group| {
                if group.len() > 1 {
                    true
                } else {
                    for room_id in group.iter() {
                        map_region.rooms.remove(room_id);
                        map_region.neighbours.remove(room_id);
                    }

                    false
                }
            });

            if room_groups.len() > 1 {
                // If there is more than one group, we need to connect them together
                Self::connect_room_groups(room_groups, map_region, config);

                room_groups = Self::generate_room_groups(map_region);
            } else {
                break;
            }
        }

        // We randomly merge some groups of 1 sized-rooms first
        Self::merge_repeated_simple_rooms(map_region, 1, config.repeat_small_room_merge_prob);
        // Then we merge rooms of size 2 or less
        Self::merge_repeated_simple_rooms(map_region, 2, config.repeat_small_room_merge_prob / 2.0);

        Self::bisect_long_horizontal_rooms(map_region, config.bisect_room_prob);
    }

    fn generate_room_groups(map_region: &mut MapRegion) -> HashMap<usize, HashSet<RoomId>> {
        let mut room_groups = HashMap::new();
        let mut group_id = 0;
        let mut map_rooms = map_region.rooms.keys().cloned().collect::<HashSet<_>>();

        while !map_rooms.is_empty() {
            let initial_room = *map_rooms.iter().next().unwrap();

            let mut rooms_to_visit = vec![initial_room];
            let mut visited_rooms = HashSet::new();

            while let Some(room_id) = rooms_to_visit.pop() {
                visited_rooms.insert(room_id);
                map_rooms.remove(&room_id);

                for neighbour_id in map_region.neighbours[&room_id].iter() {
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
        map_region: &mut MapRegion,
        config: &MapBuilderConfig,
    ) {
        let group_count = room_groups.len() as f32;
        let group_size_cutoff = (room_groups
            .par_iter()
            .map(|(_, group_rooms)| group_rooms.len() as f32)
            .sum::<f32>()
            / group_count)
            * 0.3;

        let next_room_id = AtomicUsize::new(map_region.rooms.keys().max().unwrap() + 1);

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
                                map_region.rooms.get(room_id).unwrap().cells.par_iter().map(
                                    |cell| (Vector2::new(cell.col as f32, cell.row as f32), 1_u32),
                                )
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
            map_region.rooms.remove(&room_id);
            map_region.neighbours.remove(&room_id);
        }

        let mut closer_groups = Vec::new();
        let mut visited_links = HashSet::new();

        let mut rng = rand::rng();

        for (group_id, center) in group_centers.iter() {
            let mut min_distance = f32::MAX;
            let mut maybe_closest_group_id = None;
            let mut maybe_second_closest_group_id = None;

            let should_multi_connect = rng.random_bool(config.group_loop_connection_chance);

            for (other_group_id, other_center) in group_centers.iter() {
                if group_id == other_group_id
                    || visited_links.contains(&(*other_group_id, *group_id))
                {
                    continue;
                }

                let center_distance = center.distance(other_center);

                if center_distance < min_distance {
                    if should_multi_connect {
                        maybe_second_closest_group_id = maybe_closest_group_id;
                    }

                    min_distance = center_distance;
                    maybe_closest_group_id = Some(*other_group_id);
                }
            }

            if let Some(closest_group_id) = maybe_closest_group_id {
                closer_groups.push((*group_id, closest_group_id));

                visited_links.insert((*group_id, closest_group_id));
                visited_links.insert((closest_group_id, *group_id));
            }

            if let Some(second_closest_group_id) = maybe_second_closest_group_id {
                closer_groups.push((*group_id, second_closest_group_id));

                visited_links.insert((*group_id, second_closest_group_id));
                visited_links.insert((second_closest_group_id, *group_id));
            }
        }

        let cell_map_mutex = Mutex::new(
            map_region
                .rooms
                .par_iter()
                .flat_map(|(room_id, room)| {
                    room.cells.par_iter().copied().map(|cell| (cell, *room_id))
                })
                .collect::<HashMap<_, _>>(),
        );

        let new_rooms = Mutex::new(Vec::new());

        closer_groups
            .into_par_iter()
            .for_each(|(group_a, group_b)| {
                let cells_a = {
                    let rooms_a = room_groups.get(group_a).unwrap();

                    rooms_a.iter().fold(Vec::new(), |mut acc, room_id| {
                        let room = map_region.rooms.get(room_id).unwrap();
                        acc.extend(room.cells.clone());
                        acc
                    })
                };

                let cells_b = {
                    let rooms_b = room_groups.get(group_b).unwrap();
                    rooms_b.iter().fold(Vec::new(), |mut acc, room_id| {
                        let room = map_region.rooms.get(room_id).unwrap();
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
            map_region.neighbours.insert(*room_id, HashSet::new());
        }

        for (room_id, room) in new_rooms.into_iter() {
            let mut room_neighours = HashSet::new();
            for cell in room.cells.iter() {
                for maybe_neighbour in cell.neighbours() {
                    if let Some(neighbour_id) = cell_map.get(&maybe_neighbour) {
                        if neighbour_id != &room_id {
                            room_neighours.insert(*neighbour_id);
                            map_region
                                .neighbours
                                .get_mut(neighbour_id)
                                .unwrap()
                                .insert(room_id);
                        }
                    }
                }
            }

            map_region
                .neighbours
                .get_mut(&room_id)
                .unwrap()
                .extend(room_neighours);
            map_region.rooms.insert(room_id, room);
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
                modifier: Some(RoomModifier::Connector),
            })
            .collect::<Vec<_>>()
    }

    fn merge_repeated_simple_rooms(map_region: &mut MapRegion, max_size: usize, merge_prob: f64) {
        let mut merge_candidates = HashSet::new();

        let mut non_merge_candidates = HashSet::new();

        for (i, room) in map_region.rooms.iter() {
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

            for neighbour_id in map_region.neighbours[room_id].iter() {
                if visited_rooms.contains(neighbour_id) {
                    continue;
                }

                let room = map_region.rooms.get(room_id).unwrap();
                let neighbour_room = map_region.rooms.get(neighbour_id).unwrap();

                if neighbour_room.cells.len() > max_size {
                    continue;
                }

                // If either rooms are the only neighbour of the other and that room has a area of 1
                // we don't merge them
                if (room.cells.len() == 1 && map_region.neighbours[room_id].len() == 1)
                    || neighbour_room.cells.len() == 1
                        && map_region.neighbours[neighbour_id].len() == 1
                {
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
            let from_room = map_region.rooms.remove(&from).unwrap();
            let to_room = map_region.rooms.remove(&to).unwrap();

            let merged_room = from_room.merged_with(to_room);
            map_region.rooms.insert(from, merged_room);

            let mut to_neighbours = map_region.neighbours.remove(&to).unwrap();
            to_neighbours.remove(&from);

            for neighbour in to_neighbours.iter() {
                if let Some(neighbours) = map_region.neighbours.get_mut(neighbour) {
                    neighbours.remove(&to);
                    neighbours.insert(from);
                }
            }

            let from_neighbours = map_region.neighbours.get_mut(&from).unwrap();
            from_neighbours.remove(&to);

            from_neighbours.extend(to_neighbours);
        }
    }

    fn bisect_long_horizontal_rooms(map_region: &mut MapRegion, bisect_chance: f64) {
        let mut target_rooms = HashSet::new();
        let mut next_room_id = map_region.rooms.keys().max().unwrap() + 1;

        let mut rng = rand::rng();

        for (idx, room) in map_region.rooms.iter() {
            if room.cells.len() < 2 {
                continue;
            }

            let is_not_fully_horizontal = room
                .cells
                .windows(2)
                .any(|cells| cells[0].row != cells[1].row);

            if is_not_fully_horizontal {
                continue;
            }

            target_rooms.insert(*idx);
        }

        for room_id in target_rooms.into_iter() {
            let should_bisect = rng.random_bool(bisect_chance);

            if !should_bisect {
                continue;
            }

            let mut room = map_region.rooms.remove(&room_id).unwrap();

            room.cells.sort_by(|a, b| a.col.cmp(&b.col));

            let bisect_cell = rng.random_range(0..room.cells.len());

            if bisect_cell == 0 {
                let room_a_id = next_room_id;
                let room_a = Room {
                    cells: vec![room.cells[bisect_cell]],
                    modifier: room.modifier,
                };
                Self::recompute_neighours_for(room_id, map_region, room_a_id, room_a);

                next_room_id += 1;
                let room_b_id = next_room_id;
                let room_b = Room {
                    cells: room.cells[(bisect_cell + 1)..].to_vec(),
                    modifier: room.modifier,
                };
                Self::recompute_neighours_for(room_id, map_region, room_b_id, room_b);

                map_region
                    .neighbours
                    .get_mut(&room_a_id)
                    .unwrap()
                    .insert(room_b_id);
                map_region
                    .neighbours
                    .get_mut(&room_b_id)
                    .unwrap()
                    .insert(room_a_id);
            } else if bisect_cell == room.cells.len() - 1 {
                let room_a_id = next_room_id;
                let room_a = Room {
                    cells: room.cells[0..bisect_cell].to_vec(),
                    modifier: room.modifier,
                };
                Self::recompute_neighours_for(room_id, map_region, room_a_id, room_a);

                next_room_id += 1;
                let room_b_id = next_room_id;
                let room_b = Room {
                    cells: vec![room.cells[bisect_cell]],
                    modifier: room.modifier,
                };
                Self::recompute_neighours_for(room_id, map_region, room_b_id, room_b);
                map_region
                    .neighbours
                    .get_mut(&room_a_id)
                    .unwrap()
                    .insert(room_b_id);
                map_region
                    .neighbours
                    .get_mut(&room_b_id)
                    .unwrap()
                    .insert(room_a_id);
            } else {
                let room_a_id = next_room_id;
                let room_a = Room {
                    cells: room.cells[0..bisect_cell].to_vec(),
                    modifier: room.modifier,
                };
                Self::recompute_neighours_for(room_id, map_region, room_a_id, room_a);

                next_room_id += 1;
                let room_b_id = next_room_id;
                let room_b = Room {
                    cells: vec![room.cells[bisect_cell]],
                    modifier: room.modifier,
                };
                Self::recompute_neighours_for(room_id, map_region, room_b_id, room_b);

                next_room_id += 1;
                let room_c_id = next_room_id;
                let room_c = Room {
                    cells: room.cells[(bisect_cell + 1)..].to_vec(),
                    modifier: room.modifier,
                };
                Self::recompute_neighours_for(room_id, map_region, room_c_id, room_c);

                map_region
                    .neighbours
                    .get_mut(&room_a_id)
                    .unwrap()
                    .insert(room_b_id);
                map_region
                    .neighbours
                    .get_mut(&room_b_id)
                    .unwrap()
                    .insert(room_a_id);
                map_region
                    .neighbours
                    .get_mut(&room_b_id)
                    .unwrap()
                    .insert(room_c_id);
                map_region
                    .neighbours
                    .get_mut(&room_c_id)
                    .unwrap()
                    .insert(room_b_id);
            }

            next_room_id += 1;
            map_region.neighbours.remove(&room_id).unwrap();
        }
    }

    fn recompute_neighours_for(
        room_id: RoomId,
        map_region: &mut MapRegion,
        new_room_id: RoomId,
        new_room: Room,
    ) {
        let mut new_neighbours = HashSet::new();

        for neighbour in map_region.neighbours[&room_id].clone() {
            let neighbours = map_region.neighbours.get_mut(&neighbour).unwrap();

            neighbours.remove(&room_id);

            let neighbour_room = map_region.rooms.get(&neighbour).unwrap();

            if new_room.is_neighbour_of(neighbour_room).is_some() {
                neighbours.insert(new_room_id);
                new_neighbours.insert(neighbour);
            }
        }

        map_region.neighbours.insert(new_room_id, new_neighbours);
        map_region.rooms.insert(new_room_id, new_room);
    }
}
