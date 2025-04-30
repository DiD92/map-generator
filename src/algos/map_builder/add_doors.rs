use super::{MapBuilder, MapBuilderConfig};
use crate::types::{Door, DoorModifier, MapRegion, RoomId};

use std::collections::{HashMap, HashSet};

use rand::Rng;

impl MapBuilder {
    pub(super) fn add_doors_to_rooms(
        map_region: &MapRegion,
        config: &MapBuilderConfig,
    ) -> Vec<Door> {
        let rooms = &map_region.rooms;
        let neighbour_table = &map_region.neighbours;

        let mut doors = Vec::with_capacity(rooms.len());

        let mut visited_rooms = HashSet::new();
        let mut connected_count = HashMap::<RoomId, u32>::new();

        for room in rooms.keys() {
            connected_count.insert(*room, 0);
        }

        let mut rng = rand::rng();

        let initial_room = {
            let idx = rng.random_range(0..rooms.len());
            *rooms.keys().nth(idx).unwrap()
        };

        let mut room_queue = Vec::new();
        room_queue.push(initial_room);

        while let Some(room_id) = room_queue.pop() {
            visited_rooms.insert(room_id);

            let room = &rooms[&room_id];
            for neighbour_id in neighbour_table[&room_id].iter() {
                if visited_rooms.contains(neighbour_id) {
                    continue;
                }

                if connected_count[neighbour_id] >= 1
                    && !rng.random_bool(config.loop_connection_chance)
                {
                    continue;
                }

                let neighbour_room = &rooms[neighbour_id];

                if let Some(neighbouring_cells) = room.is_neighbour_of(neighbour_room) {
                    let priority_neighbouring_cells = neighbouring_cells
                        .iter()
                        .copied()
                        .filter(|(_, _, direction)| direction.is_horizontal())
                        .collect::<Vec<_>>();

                    let neighouring_cells_selection = if priority_neighbouring_cells.is_empty() {
                        neighbouring_cells
                    } else {
                        priority_neighbouring_cells
                    };

                    let selected_cell = rng.random_range(0..neighouring_cells_selection.len());
                    let (from, to, _) = neighouring_cells_selection[selected_cell];

                    let mut door = Door::new(from, to);

                    match rng.random_range(0..100_u32) {
                        0 => door.modifier = DoorModifier::Locked,
                        1..6 => door.modifier = DoorModifier::Secret,
                        6..10 => door.modifier = DoorModifier::None,
                        _ => door.modifier = DoorModifier::Open,
                    }

                    doors.push(door);

                    *connected_count.get_mut(&room_id).unwrap() += 1;
                    *connected_count.get_mut(neighbour_id).unwrap() += 1;

                    room_queue.push(*neighbour_id);
                }
            }
        }

        doors
    }
}
