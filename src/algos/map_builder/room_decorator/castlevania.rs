use super::RoomDecorator;
use crate::{
    algos::MapBuilderConfig,
    types::{NeighbourTable, RoomModifier, RoomTable},
};

use rand::Rng;

use std::collections::HashSet;

pub(super) struct CastlevaniaRoomDectorator;

impl RoomDecorator for CastlevaniaRoomDectorator {
    fn decorate(
        &self,
        rooms: &mut RoomTable,
        neighbour_table: &NeighbourTable,
        _: &MapBuilderConfig,
    ) {
        let mut target_rooms = HashSet::new();

        for (idx, room) in rooms.iter() {
            if room.cells.len() == 1 && neighbour_table[idx].len() < 3 {
                target_rooms.insert(*idx);
            }
        }

        let mut rng = rand::rng();

        for room_id in target_rooms.iter() {
            let modifier = match rng.random_range(0_u32..100) {
                0..10 => Some(RoomModifier::Save),
                10..17 => Some(RoomModifier::Navigation),
                _ => None,
            };

            rooms.get_mut(room_id).unwrap().modifier = modifier;
        }
    }
}
