use super::MapBuilder;
use crate::{
    algos::map_builder::bsp::{RectTable, RemovedRectTable},
    types::{MapRegion, NeighbourTable, Rect, Room, RoomTable},
};

impl MapBuilder {
    pub(super) fn generate_group_offsets(
        region_rects: &[(Rect, RectTable, RemovedRectTable, NeighbourTable)],
    ) -> Vec<usize> {
        let region_count = region_rects.len();

        region_rects
            .iter()
            .map(|(_, rects, removed, _)| {
                rects.keys().chain(removed.keys()).max().unwrap_or(&1_usize)
            })
            .take(region_count - 1)
            .fold(vec![0], |mut acc, max_idx| {
                let last = *acc.last().unwrap();
                acc.push(last + *max_idx);
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

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn tests_generate_group_offsets() {
        let rect = Rect::new(0, 0, 2, 2);

        let rects_1 = HashMap::from([(1, rect), (3, rect)]);
        let removed_1 = HashMap::from([(2, rect), (5, rect), (7, rect)]);

        let rects_2 = HashMap::from([(3, rect), (5, rect)]);
        let removed_2 = HashMap::from([(2, rect), (6, rect), (10, rect)]);

        let payload = vec![
            (Rect::new(0, 0, 10, 10), rects_1, removed_1, HashMap::new()),
            (Rect::new(0, 0, 10, 10), rects_2, removed_2, HashMap::new()),
        ];

        let offsets = MapBuilder::generate_group_offsets(&payload);

        assert_eq!(offsets, vec![0, 7]);
    }
}
