use super::MapBuilder;
use crate::{
    algos::RngHandler,
    constants::MIN_BISECT_SIZE,
    types::{MapRegion, NeighbourSet, Room, RoomId},
};

use rand::Rng;

impl MapBuilder {
    pub(super) fn bisect_long_horizontal_rooms(map_region: &mut MapRegion, bisect_chance: f64) {
        let mut target_rooms = Vec::new();

        let mut rng = RngHandler::rng();

        for (idx, room) in map_region.iter_active() {
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

            target_rooms.push(idx);
        }

        for room_id in target_rooms.into_iter() {
            let should_bisect = rng.random_bool(bisect_chance);

            if !should_bisect {
                continue;
            }

            let mut room = map_region.take_active(room_id);
            room.cells.sort_by(|a, b| a.col.cmp(&b.col));

            let bisect_cell = rng.random_range(1..room.cells.len() - 1);

            let room_a = Room {
                cells: room.cells[0..bisect_cell].to_vec(),
                modifier: room.modifier,
            };
            let room_a_id = map_region.insert_room(room_a);
            Self::recompute_neighbours_for(room_id, map_region, room_a_id);

            let room_b = Room {
                cells: vec![room.cells[bisect_cell]],
                modifier: room.modifier,
            };
            let room_b_id = map_region.insert_room(room_b);
            Self::recompute_neighbours_for(room_id, map_region, room_b_id);

            let room_c = Room {
                cells: room.cells[(bisect_cell + 1)..].to_vec(),
                modifier: room.modifier,
            };
            let room_c_id = map_region.insert_room(room_c);
            Self::recompute_neighbours_for(room_id, map_region, room_c_id);

            map_region.get_mut_neighbours(room_a_id).insert(room_b_id);
            map_region.get_mut_neighbours(room_b_id).insert(room_a_id);
            map_region.get_mut_neighbours(room_b_id).insert(room_c_id);
            map_region.get_mut_neighbours(room_c_id).insert(room_b_id);

            let _ = map_region.take_neighbours(room_id);
        }

        map_region.shrink_buffers();
    }

    fn recompute_neighbours_for(room_id: RoomId, map_region: &mut MapRegion, new_room_id: RoomId) {
        let mut new_neighbours = NeighbourSet::new();

        let neighbours = map_region.iter_neighbours(room_id).collect::<Vec<_>>();

        for neighbour in neighbours {
            let new_room = map_region.get_room(new_room_id);
            let neighbour_room = map_region.get_room(neighbour);

            if new_room.is_neighbour_of(neighbour_room) {
                let neighbour_neighbours = map_region.get_mut_neighbours(neighbour);

                neighbour_neighbours.remove(room_id);

                neighbour_neighbours.insert(new_room_id);
                new_neighbours.insert(neighbour);
            }
        }

        map_region
            .get_mut_neighbours(new_room_id)
            .extend(new_neighbours);
    }
}
