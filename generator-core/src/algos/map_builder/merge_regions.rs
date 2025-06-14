use std::collections::HashSet;

use super::MapBuilder;
use crate::types::{MapRegion, Rect};

impl MapBuilder {
    pub(super) fn merge_regions(origin_rect: Rect, mut map_regions: Vec<MapRegion>) -> MapRegion {
        let neighbour_regions = Self::compure_neighbouring_regions(&map_regions);

        for (from_id, to_id) in neighbour_regions.into_iter() {
            Self::connect_separate_regions(from_id, to_id, map_regions.as_mut_slice());
        }

        let mut map_region = map_regions
            .into_iter()
            .reduce(|mut acc, region| {
                acc.rooms.extend(region.rooms);
                acc.removed_rooms.extend(region.removed_rooms);
                acc.neighbours.extend(region.neighbours);

                acc
            })
            .unwrap();

        map_region.origin_rect = origin_rect;

        map_region
    }

    fn compure_neighbouring_regions(map_regions: &[MapRegion]) -> Vec<(usize, usize)> {
        let mut closer_groups = Vec::new();
        let mut visited_links = HashSet::new();

        // We compute the closest groups to each other
        for (from_id, map_region_from) in map_regions.iter().enumerate() {
            for (to_id, map_region_to) in map_regions.iter().enumerate() {
                if from_id == to_id || visited_links.contains(&(to_id, from_id)) {
                    continue;
                }

                if map_region_from
                    .origin_rect
                    .is_neighbour_of(&map_region_to.origin_rect)
                    .is_some()
                {
                    closer_groups.push((from_id, to_id));
                    visited_links.insert((from_id, to_id));
                    visited_links.insert((to_id, from_id));
                }
            }
        }

        closer_groups
    }

    fn connect_separate_regions(from_id: usize, to_id: usize, map_regions: &mut [MapRegion]) {
        let from_region = &map_regions[from_id];
        let from_room_ids = from_region
            .rooms
            .iter()
            .chain(from_region.removed_rooms.iter());
        let to_region = &map_regions[to_id];

        let mut match_distance = f32::MAX;

        let mut rooms_to_connect = Vec::new();

        for (from_id, from_room) in from_room_ids.into_iter() {
            let to_room_ids = to_region.rooms.iter().chain(to_region.removed_rooms.iter());

            for (to_id, to_room) in to_room_ids.into_iter() {
                let from_center = from_room.get_center();
                let to_center = to_room.get_center();

                let distance = from_center.distance(&to_center);
                if distance <= match_distance && from_room.is_neighbour_of(to_room).is_some() {
                    rooms_to_connect.push((*from_id, *to_id));

                    match_distance = distance;
                }
            }
        }

        let regions = map_regions
            .get_disjoint_mut([from_id, to_id])
            .expect("Failed to get disjoint mutable references for merging regions");

        for (from_room_id, to_room_id) in rooms_to_connect {
            regions[0]
                .neighbours
                .get_mut(&from_room_id)
                .unwrap()
                .insert(to_room_id);
            regions[1]
                .neighbours
                .get_mut(&to_room_id)
                .unwrap()
                .insert(from_room_id);
        }
    }
}
