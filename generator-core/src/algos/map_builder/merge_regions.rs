use std::collections::{HashMap, HashSet};

use rayon::iter::{IntoParallelIterator, ParallelIterator};

use super::MapBuilder;
use crate::types::{MapRegion, Rect, RoomId, Vector2};

impl MapBuilder {
    pub(super) fn merge_regions(origin_rect: Rect, map_regions: Vec<MapRegion>) -> MapRegion {
        let neighbour_regions = Self::compute_neighbouring_regions(&map_regions);
        let group_offsets = Self::generate_group_offsets(&map_regions);
        let region_centers = map_regions
            .iter()
            .map(Self::generate_region_centers)
            .collect::<Vec<_>>();

        let rooms_to_connect = neighbour_regions
            .into_par_iter()
            .map(|(from_id, to_id)| {
                let room_connections =
                    Self::generate_rooms_to_connect(from_id, to_id, &map_regions, &region_centers);

                room_connections
                    .into_iter()
                    .map(|(from_room_id, to_room_id)| {
                        let from_offset = group_offsets[from_id];
                        let to_offset = group_offsets[to_id];

                        (from_room_id + from_offset, to_room_id + to_offset)
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>();

        let mut map_region = map_regions
            .into_iter()
            .reduce(|mut acc, region| {
                acc.merge_with(region);

                acc
            })
            .unwrap();

        for (from_room_id, to_room_id) in rooms_to_connect {
            let from_neighbours = map_region.get_mut_neighbours(from_room_id);
            from_neighbours.insert(to_room_id);

            let to_neighbours = map_region.get_mut_neighbours(to_room_id);
            to_neighbours.insert(from_room_id);
        }

        map_region.shrink_buffers();

        map_region.origin_rect = origin_rect;

        map_region
    }

    fn compute_neighbouring_regions(map_regions: &[MapRegion]) -> Vec<(usize, usize)> {
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

    fn generate_group_offsets(map_regions: &[MapRegion]) -> Vec<usize> {
        let region_count = map_regions.len();

        map_regions
            .iter()
            .map(|map_region| map_region.room_slots())
            .take(region_count - 1)
            .fold(vec![0_usize], |mut acc, region_offset| {
                let last_offset = *acc.last().unwrap();
                acc.push(last_offset + region_offset);
                acc
            })
    }

    fn generate_region_centers(map_region: &MapRegion) -> (Vector2, HashMap<RoomId, Vector2>) {
        let room_centers = map_region
            .iter_rooms()
            .map(|(idx, room)| (idx, room.get_center()))
            .collect::<HashMap<_, _>>();

        let center_vec = room_centers
            .values()
            .fold(Vector2::ZERO, |mut center, other_vector| {
                center.x += other_vector.x;
                center.y += other_vector.y;

                center
            });

        (
            center_vec.divide_by(room_centers.len() as f32),
            room_centers,
        )
    }

    fn generate_rooms_to_connect(
        from_id: usize,
        to_id: usize,
        map_regions: &[MapRegion],
        region_centers: &[(Vector2, HashMap<RoomId, Vector2>)],
    ) -> Vec<(usize, usize)> {
        let mut rooms_to_connect = Vec::new();

        let region_distance = region_centers[from_id].0.distance(&region_centers[to_id].0);

        let from_candidates = map_regions[from_id].iter_rooms().filter(|(room_id, _)| {
            let to_region_center = region_centers[to_id].0;
            let room_center = region_centers[from_id].1[room_id];

            room_center.distance(&to_region_center) <= region_distance
        });

        let to_candidates = map_regions[to_id]
            .iter_rooms()
            .filter(|(room_id, _)| {
                let from_region_center = region_centers[from_id].0;
                let room_center = region_centers[to_id].1[room_id];

                room_center.distance(&from_region_center) <= region_distance
            })
            .collect::<Vec<_>>();

        for (from_id, from_room) in from_candidates {
            for (to_id, to_room) in to_candidates.iter() {
                if from_room.is_neighbour_of(to_room) {
                    rooms_to_connect.push((from_id, *to_id));
                }
            }
        }

        rooms_to_connect
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tests_generate_group_offsets() {
        let map_region = MapRegion::new_test_region();
        let map_regions = vec![map_region.clone(), map_region];

        let offsets = MapBuilder::generate_group_offsets(&map_regions);
        let expected_offsets = vec![0, 16];

        assert_eq!(offsets, expected_offsets);
    }
}
