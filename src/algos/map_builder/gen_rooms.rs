use super::MapBuilder;
use crate::types::{NeighbourTable, Rect, Room, RoomTable};

use std::{collections::HashSet, sync::Mutex};

use rayon::prelude::*;

impl MapBuilder {
    pub(super) fn generate_initial_rooms(rects: Vec<Rect>) -> (RoomTable, NeighbourTable) {
        let rooms = rects
            .into_par_iter()
            .enumerate()
            .map(|(i, rect)| (i, Room::new_from_rect(rect)))
            .collect::<RoomTable>();

        let neighbour_map = rooms
            .par_iter()
            .map(|(i, _)| (*i, HashSet::new()))
            .collect::<NeighbourTable>();

        let neighbour_map_mutex = Mutex::new(neighbour_map);
        rooms.par_iter().for_each(|(i, room)| {
            rooms.par_iter().for_each(|(j, other_room)| {
                if room.is_neighbour_of(other_room).is_some() {
                    let mut neighbour_map = neighbour_map_mutex.lock().unwrap();
                    neighbour_map.get_mut(i).unwrap().insert(*j);
                }
            });
        });

        (rooms, neighbour_map_mutex.into_inner().unwrap())
    }
}
