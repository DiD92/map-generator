use super::{MapBuilder, MapBuilderConfig};
use crate::types::{Door, DoorModifier, NeighbourTable, Room, RoomId, RoomTable};

use std::collections::{HashMap, HashSet};

use rand::Rng;

impl MapBuilder {
    pub(super) fn add_doors_to_rooms(
        rooms: RoomTable,
        neighbour_table: NeighbourTable,
        config: &MapBuilderConfig,
    ) -> (Vec<Room>, Vec<Door>) {
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

                if let Some((from, to, _)) = room.is_neighbour_of(neighbour_room) {
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

        (rooms.into_values().collect(), doors)
    }
}
