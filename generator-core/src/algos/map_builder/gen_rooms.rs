use super::MapBuilder;
use crate::{
    algos::map_builder::bsp::RectTable,
    types::{MapRegion, NeighbourTable, Rect, Room, RoomTable},
};

impl MapBuilder {
    pub(super) fn generate_map_region(
        origin_rect: Rect,
        rects: RectTable,
        removed: RectTable,
        neighbours: NeighbourTable,
    ) -> MapRegion {
        let rooms = rects
            .into_iter()
            .map(|(idx, rect)| (idx, Room::new_from_rect(rect)))
            .collect();

        let removed_rooms = removed
            .into_iter()
            .map(|(idx, rect)| (idx, Room::new_from_rect(rect)))
            .collect::<RoomTable>();

        MapRegion::new(origin_rect, rooms, removed_rooms, neighbours)
    }
}
