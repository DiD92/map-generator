use super::MapBuilder;
use crate::types::{MapRegion, NeighbourTable, Rect, Room, RoomTable};

use std::{collections::HashSet, sync::Mutex};

use rayon::prelude::*;

impl MapBuilder {
    pub(super) fn generate_map_region(origin_rect: Rect, rects: Vec<Rect>) -> MapRegion {
        let rooms = rects.into_par_iter().map(Room::new_from_rect).collect();

        let room_table = Self::generate_room_table(rooms);

        let neighbour_map = Self::generate_neighbour_table(&room_table);

        MapRegion {
            origin_rect,
            rooms: room_table,
            neighbours: neighbour_map,
        }
    }

    pub(super) fn generate_room_table(rooms: Vec<Room>) -> RoomTable {
        rooms
            .into_par_iter()
            .enumerate()
            .map(|(i, room)| (i, room))
            .collect::<RoomTable>()
    }

    pub(super) fn generate_neighbour_table(room_table: &RoomTable) -> NeighbourTable {
        let neighbour_map = room_table
            .par_iter()
            .map(|(i, _)| (*i, HashSet::new()))
            .collect::<NeighbourTable>();

        let neighbour_map_mutex = Mutex::new(neighbour_map);
        room_table.par_iter().for_each(|(i, room)| {
            room_table.par_iter().for_each(|(j, other_room)| {
                if room.is_neighbour_of(other_room).is_some() {
                    let mut neighbour_map = neighbour_map_mutex.lock().unwrap();
                    neighbour_map.get_mut(i).unwrap().insert(*j);
                }
            });
        });

        neighbour_map_mutex.into_inner().unwrap()
    }
}
