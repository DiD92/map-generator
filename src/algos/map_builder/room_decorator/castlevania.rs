use super::RoomDecorator;
use crate::{
    algos::MapBuilderConfig,
    types::{Cell, Door, NeighbourTable, RoomModifier, RoomTable},
};

use rand::Rng;

use std::collections::HashSet;

const MIN_ROOM_DISTANCE: u32 = 8;

pub(super) struct CastlevaniaRoomDectorator;

impl RoomDecorator for CastlevaniaRoomDectorator {
    fn decorate(
        &self,
        rooms: &mut RoomTable,
        neighbour_table: &NeighbourTable,
        doors: &[Door],
        _: &MapBuilderConfig,
    ) {
        let mut target_rooms = HashSet::new();

        let door_map = doors
            .iter()
            .map(|door| (&door.from, &door.to))
            .collect::<HashSet<_>>();

        for (idx, room) in rooms.iter() {
            if room.cells.len() > 1 {
                continue;
            }

            let any_neighbour_is_vertical = neighbour_table[idx].iter().any(|neighbour_id| {
                let neighour = rooms.get(neighbour_id).unwrap();
                room.is_neighbour_of(neighour)
                    .unwrap()
                    .iter()
                    .any(|(from, to, direction)| {
                        (door_map.contains(&(from, to)) || door_map.contains(&(to, from)))
                            && !direction.is_horizontal()
                    })
            });

            if any_neighbour_is_vertical {
                continue;
            }

            target_rooms.insert(*idx);
        }

        let mut rng = rand::rng();

        let mut save_rooms = HashSet::<Cell>::new();
        let mut navigation_rooms = HashSet::<Cell>::new();

        for room_id in target_rooms.iter() {
            let room_cell = rooms.get(room_id).unwrap().cells[0];

            let mut min_save_distance = u32::MAX;
            let mut min_nav_distance = u32::MAX;

            let modifier = match rng.random_range(0_u32..100) {
                0..50 => {
                    for save_cell in save_rooms.iter() {
                        let distance = save_cell.distance(&room_cell);
                        if distance < min_save_distance {
                            min_save_distance = distance;
                        }
                    }

                    if min_save_distance != u32::MAX && min_save_distance < MIN_ROOM_DISTANCE {
                        None
                    } else {
                        save_rooms.insert(room_cell);

                        Some(RoomModifier::Save)
                    }
                }
                50..71 => {
                    for nav_cell in navigation_rooms.iter() {
                        let distance = nav_cell.distance(&room_cell);
                        if distance < min_nav_distance {
                            min_nav_distance = distance;
                        }
                    }

                    if min_nav_distance != u32::MAX && min_nav_distance < MIN_ROOM_DISTANCE {
                        None
                    } else {
                        navigation_rooms.insert(room_cell);

                        Some(RoomModifier::Navigation)
                    }
                }
                _ => None,
            };

            rooms.get_mut(room_id).unwrap().modifier = modifier;
        }
    }
}
