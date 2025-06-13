use std::collections::HashSet;

use super::MapBuilder;
use crate::{
    algos::RngHandler,
    constants::MIN_BISECT_SIZE,
    types::{MapRegion, Room, RoomId},
};

use rand::Rng;

impl MapBuilder {
    pub(super) fn bisect_long_horizontal_rooms(map_region: &mut MapRegion, bisect_chance: f64) {
        let mut target_rooms = Vec::new();
        let mut next_room_id = map_region.rooms.keys().max().unwrap() + 1;

        let mut rng = RngHandler::rng();

        for (idx, room) in map_region.rooms.iter() {
            if room.cells.len() < MIN_BISECT_SIZE {
                continue;
            }

            let is_not_fully_horizontal = room
                .cells
                .windows(2)
                .any(|cells| cells[0].row != cells[1].row);

            if is_not_fully_horizontal {
                continue;
            }

            target_rooms.push(*idx);
        }

        for room_id in target_rooms.into_iter() {
            let should_bisect = rng.random_bool(bisect_chance);

            if !should_bisect {
                continue;
            }

            let mut room = map_region.rooms.remove(&room_id).unwrap();
            let room_neighbours = map_region
                .neighbours
                .remove(&room_id)
                .expect("Room should have neighbours");

            room.cells.sort_by(|a, b| a.col.cmp(&b.col));

            let bisect_cell = rng.random_range(0..room.cells.len());

            if bisect_cell == 0 {
                let room_a_id = next_room_id;
                let room_a = Room {
                    cells: vec![room.cells[bisect_cell]],
                    modifier: room.modifier,
                };
                Self::recompute_neighours_for(
                    room_id,
                    &room_neighbours,
                    map_region,
                    room_a_id,
                    room_a,
                );

                next_room_id += 1;
                let room_b_id = next_room_id;
                let room_b = Room {
                    cells: room.cells[(bisect_cell + 1)..].to_vec(),
                    modifier: room.modifier,
                };
                Self::recompute_neighours_for(
                    room_id,
                    &room_neighbours,
                    map_region,
                    room_b_id,
                    room_b,
                );

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
                Self::recompute_neighours_for(
                    room_id,
                    &room_neighbours,
                    map_region,
                    room_a_id,
                    room_a,
                );

                next_room_id += 1;
                let room_b_id = next_room_id;
                let room_b = Room {
                    cells: vec![room.cells[bisect_cell]],
                    modifier: room.modifier,
                };
                Self::recompute_neighours_for(
                    room_id,
                    &room_neighbours,
                    map_region,
                    room_b_id,
                    room_b,
                );
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
                Self::recompute_neighours_for(
                    room_id,
                    &room_neighbours,
                    map_region,
                    room_a_id,
                    room_a,
                );

                next_room_id += 1;
                let room_b_id = next_room_id;
                let room_b = Room {
                    cells: vec![room.cells[bisect_cell]],
                    modifier: room.modifier,
                };
                Self::recompute_neighours_for(
                    room_id,
                    &room_neighbours,
                    map_region,
                    room_b_id,
                    room_b,
                );

                next_room_id += 1;
                let room_c_id = next_room_id;
                let room_c = Room {
                    cells: room.cells[(bisect_cell + 1)..].to_vec(),
                    modifier: room.modifier,
                };
                Self::recompute_neighours_for(
                    room_id,
                    &room_neighbours,
                    map_region,
                    room_c_id,
                    room_c,
                );

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
        }
    }

    fn recompute_neighours_for(
        room_id: RoomId,
        room_neighbours: &HashSet<RoomId>,
        map_region: &mut MapRegion,
        new_room_id: RoomId,
        new_room: Room,
    ) {
        let mut new_neighbours = HashSet::new();

        let neighours = room_neighbours
            .iter()
            .filter(|n| map_region.rooms.contains_key(n));

        for neighbour in neighours {
            let neighbours = map_region.neighbours.get_mut(neighbour).unwrap();

            neighbours.remove(&room_id);

            let neighbour_room = map_region.rooms.get(neighbour).unwrap();

            if new_room.is_neighbour_of(neighbour_room).is_some() {
                neighbours.insert(new_room_id);
                new_neighbours.insert(*neighbour);
            }
        }

        map_region.neighbours.insert(new_room_id, new_neighbours);
        map_region.rooms.insert(new_room_id, new_room);
    }
}
