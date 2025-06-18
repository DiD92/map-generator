use super::MapBuilder;
use crate::types::{Map, RoomModifier};

use std::collections::{HashMap, HashSet};

impl MapBuilder {
    pub(super) fn connect_regions(regions: &mut [Map]) {
        let mut region_map = regions.iter_mut().enumerate().collect::<HashMap<_, _>>();

        let mut regions_to_visit = vec![region_map.keys().cloned().next().unwrap()];

        let mut expanded_regions = HashSet::new();
        expanded_regions.insert(regions_to_visit[0]);

        let mut region_links = Vec::new();

        while let Some(region_idx) = regions_to_visit.pop() {
            let region = region_map.get(&region_idx).unwrap();

            for neighbour_idx in region_map.keys() {
                if region_idx == *neighbour_idx || expanded_regions.contains(neighbour_idx) {
                    continue;
                }

                let neighbour_region = region_map.get(neighbour_idx).unwrap();

                if region
                    .origin_rect
                    .is_neighbour_of(&neighbour_region.origin_rect)
                    .is_some()
                {
                    regions_to_visit.push(*neighbour_idx);
                    region_links.push((region_idx, *neighbour_idx));
                    expanded_regions.insert(*neighbour_idx);
                }
            }
        }

        for (from_region, to_region) in region_links.drain(..) {
            Self::link_closest_rooms(from_region, to_region, &mut region_map);
        }
    }

    fn link_closest_rooms(
        from_region_id: usize,
        to_region_id: usize,
        region_map: &mut HashMap<usize, &mut Map>,
    ) {
        let from_room_map = region_map
            .get(&from_region_id)
            .unwrap()
            .rooms
            .iter()
            .enumerate()
            .collect::<HashMap<_, _>>();

        let to_room_map = region_map
            .get(&to_region_id)
            .unwrap()
            .rooms
            .iter()
            .enumerate()
            .collect::<HashMap<_, _>>();

        let mut closest_distance = f32::MAX;
        let mut closest_rooms = (0_usize, 0_usize);

        let from_axis = {
            let from_region = &region_map[&from_region_id];
            let to_region = &region_map[&to_region_id];
            from_region
                .origin_rect
                .is_neighbour_of(&to_region.origin_rect)
                .unwrap()
        };
        let to_axis = from_axis.reverse();

        for (from_room_id, from_room) in from_room_map.iter() {
            if from_room_map
                .values()
                .any(|room| match from_room.get_neighbouring_cells_for(room) {
                    Some(ref neighours) => neighours
                        .iter()
                        .any(|(_, _, direction)| direction == &from_axis),
                    None => false,
                })
            {
                continue;
            }

            let from_room_center = from_room.get_center();
            for (to_room_id, to_room) in to_room_map.iter() {
                if to_room_map
                    .values()
                    .any(|room| match to_room.get_neighbouring_cells_for(room) {
                        Some(ref neighours) => neighours
                            .iter()
                            .any(|(_, _, direction)| direction == &to_axis),
                        None => false,
                    })
                {
                    continue;
                }

                let to_room_center = to_room.get_center();
                let distance = from_room_center.distance(&to_room_center);

                if distance < closest_distance {
                    closest_distance = distance;
                    closest_rooms = (*from_room_id, *to_room_id);
                }
            }
        }

        region_map
            .get_mut(&from_region_id)
            .unwrap()
            .rooms
            .iter_mut()
            .enumerate()
            .for_each(|(room_id, room)| {
                if room_id == closest_rooms.0 {
                    room.modifier = Some(RoomModifier::RegionConnection(from_axis));
                }
            });

        region_map
            .get_mut(&to_region_id)
            .unwrap()
            .rooms
            .iter_mut()
            .enumerate()
            .for_each(|(room_id, room)| {
                if room_id == closest_rooms.1 {
                    room.modifier = Some(RoomModifier::RegionConnection(to_axis));
                }
            });
    }
}
