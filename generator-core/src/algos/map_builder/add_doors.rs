use super::{MapBuilder, MapBuilderConfig};
use crate::{
    algos::RngHandler,
    types::{Door, DoorModifier, MapRegion, RoomId},
};

use std::collections::{HashMap, HashSet};

use rand::Rng;

impl MapBuilder {
    pub(super) fn generate_doors_for(
        map_region: &MapRegion,
        config: &MapBuilderConfig,
    ) -> Vec<Door> {
        let room_count = map_region.iter_active().count();
        let mut doors = Vec::with_capacity(room_count * 2);

        let mut visited_rooms = HashSet::new();
        let mut connected_count = HashMap::<RoomId, u32>::new();

        for (room_id, _) in map_region.iter_active() {
            connected_count.insert(room_id, 0);
        }

        let mut rng = RngHandler::rng();

        let initial_room = {
            let idx = rng.random_range(0..room_count);
            map_region
                .iter_active()
                .nth(idx)
                .map(|(id, _)| id)
                .expect("There should be at least one active room")
        };

        let mut room_queue = Vec::new();
        room_queue.push(initial_room);

        while let Some(room_id) = room_queue.pop() {
            visited_rooms.insert(room_id);

            let room = map_region.get_active(room_id);

            for neighbour_id in map_region.iter_active_neighbours(room_id) {
                if visited_rooms.contains(&neighbour_id) {
                    continue;
                }

                if connected_count[&neighbour_id] >= 1
                    && !rng.random_bool(config.door_loop_connection_chance)
                {
                    continue;
                }

                let neighbour_room = map_region.get_active(neighbour_id);

                if let Some(neighbouring_cells) = room.get_neighbouring_cells_for(neighbour_room) {
                    let priority_neighbouring_cells = neighbouring_cells
                        .iter()
                        .copied()
                        .filter(|(_, _, direction)| direction.is_horizontal())
                        .collect::<Vec<_>>();

                    let neighbouring_cells_selection = if priority_neighbouring_cells.is_empty() {
                        neighbouring_cells
                    } else {
                        priority_neighbouring_cells
                    };

                    let selected_cell = rng.random_range(0..neighbouring_cells_selection.len());
                    let (from, to, _) = neighbouring_cells_selection[selected_cell];

                    let mut door = Door::new(from, to);

                    match rng.random_range(0..100_u32) {
                        0 => door.modifier = DoorModifier::Locked,
                        1..6 => door.modifier = DoorModifier::Secret,
                        6..10 => door.modifier = DoorModifier::None,
                        _ => door.modifier = DoorModifier::Open,
                    }

                    doors.push(door);

                    *connected_count.get_mut(&room_id).unwrap() += 1;
                    *connected_count.get_mut(&neighbour_id).unwrap() += 1;

                    room_queue.push(neighbour_id);
                }
            }
        }

        doors
    }
}
