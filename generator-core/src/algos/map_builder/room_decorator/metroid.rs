use super::RoomDecorator;
use crate::{
    algos::{MapBuilderConfig, RngHandler},
    types::{Cell, Door, MapRegion, RoomModifier},
};

use rand::Rng;

use std::collections::HashSet;

const MIN_ROOM_DISTANCE: u32 = 8;

pub(super) enum MetroidRoomDecorator {
    ZeroMission,
    Fusion,
    SuperMetroid,
}

impl RoomDecorator for MetroidRoomDecorator {
    fn decorate(&self, map_region: &mut MapRegion, doors: &[Door], _: &MapBuilderConfig) {
        let mut target_rooms = HashSet::new();

        let door_map = doors
            .iter()
            .map(|door| (&door.from, &door.to))
            .collect::<HashSet<_>>();

        for (idx, room) in map_region.iter_active() {
            if room.cells.len() > 1 {
                continue;
            }

            let any_neighbour_is_vertical =
                map_region.iter_active_neighbours(idx).any(|neighbour_id| {
                    let neighour = map_region.get_active(neighbour_id);
                    room.get_neighbouring_cells_for(neighour)
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

            target_rooms.insert(idx);
        }

        let mut rng = RngHandler::rng();

        let mut save_rooms = HashSet::<Cell>::new();
        let mut navigation_rooms = HashSet::<Cell>::new();

        for room_id in target_rooms.iter() {
            let room = map_region.get_active(*room_id);

            if room.modifier.is_some() {
                continue;
            }

            let room_cell = room.cells[0];

            let mut min_save_distance = u32::MAX;
            let mut min_nav_distance = u32::MAX;

            let modifier = match rng.random_range(0_u32..100) {
                0..30 => {
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
                30..45 => {
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
                45..60 => Some(RoomModifier::Item),
                _ => None,
            };

            map_region.get_mut_room(*room_id).modifier = modifier;
        }
    }
}
