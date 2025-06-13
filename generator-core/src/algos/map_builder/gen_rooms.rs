use super::MapBuilder;
use crate::{
    algos::map_builder::bsp::RectTable,
    types::{MapRegion, NeighbourTable, Rect, Room, RoomTable},
};

use std::collections::HashMap;

impl MapBuilder {
    pub(super) fn generate_region_offsets<'a>(
        region_rects: impl Iterator<Item = &'a HashMap<usize, Rect>>,
        region_count: usize,
    ) -> Vec<usize> {
        region_rects
            .map(|rects| rects.len())
            .take(region_count - 1)
            .fold(vec![0], |mut acc, len| {
                let last = *acc.last().unwrap();
                acc.push(last + len);
                acc
            })
    }

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

        MapRegion {
            origin_rect,
            rooms,
            removed_rooms,
            neighbours,
        }
    }
}
