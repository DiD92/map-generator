use super::{MapBuilder, MapBuilderConfig};
use crate::types::{NeighbourTable, RoomModifier, RoomTable};

use rand::Rng;

impl MapBuilder {
    pub(super) fn decorate_rooms(
        rooms: &mut RoomTable,
        _neighbour_table: &NeighbourTable,
        _config: &MapBuilderConfig,
    ) {
        let mut rng = rand::rng();

        for room in rooms.values_mut() {
            match rng.random_range(0..100_u32) {
                0 => room.modifier = RoomModifier::Save,
                1..=5 => room.modifier = RoomModifier::Item,
                6..=10 => room.modifier = RoomModifier::Secret,
                _ => room.modifier = RoomModifier::None,
            }
        }
    }
}
