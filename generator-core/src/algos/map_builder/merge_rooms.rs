use super::{MapBuilder, MapBuilderConfig};
use crate::{algos::RngHandler, types::MapRegion};

use std::collections::{HashMap, HashSet};

use rand::Rng;

impl MapBuilder {
    pub(super) fn merge_random_rooms(map_region: &mut MapRegion, config: &MapBuilderConfig) {
        let mut rooms_to_merge = HashSet::with_capacity(map_region.rooms.len() / 4);
        let mut merge_groups = Vec::with_capacity(rooms_to_merge.capacity() / 2);

        let mut rng = RngHandler::rng();

        // We will use a buffer to collect filterd neighbours
        // to avoid excessive allocations
        let mut neighbour_buffer = Vec::new();

        // First, we build a set of rooms that are going to be merged
        for (room_id, room) in map_region.rooms.iter() {
            if rooms_to_merge.contains(room_id) {
                continue;
            }

            // We only want to merge room_id with a neighbour that is not already in the rooms_to_merge set
            // and that is a valid room in the map_region
            neighbour_buffer.extend(
                map_region
                    .neighbours
                    .get(room_id)
                    .expect("Room should have neighbours")
                    .iter()
                    .filter(|n| map_region.rooms.contains_key(n) && !rooms_to_merge.contains(*n))
                    .copied(),
            );

            let neighbour_count = neighbour_buffer.len();

            if room.cells.len() == 1 && neighbour_count == 1 {
                neighbour_buffer.clear();
                // For now we skip rooms that are single cells with only one neighbour
                continue;
            }

            // If the room has at least one neighbour and the random condition is met,
            // we will merge it with one of its neighbours randomly
            if neighbour_count > 0 && rng.random_bool(config.random_room_merge_prob) {
                let selected_neighbour = neighbour_buffer
                    .iter()
                    .nth(rng.random_range(0..neighbour_count))
                    .copied()
                    .unwrap();
                
                if room_id == &selected_neighbour {
                    println!("Warning: Room ID {} is merging with itself, skipping.", room_id);
                }

                rooms_to_merge.insert(*room_id);
                rooms_to_merge.insert(selected_neighbour);

                merge_groups.push((*room_id, selected_neighbour));
            }

            neighbour_buffer.clear();
        }

        // Now we merge the rooms in the merge_groups
        for (room_a_idx, room_b_idx) in merge_groups.into_iter() {
            map_region
                .try_merge_rooms(room_a_idx, room_b_idx)
                .expect("Should merge rooms");
        }
    }

    pub(super) fn merge_repeated_simple_rooms(
        map_region: &mut MapRegion,
        max_size: usize,
        merge_prob: f64,
    ) {
        let mut merge_candidates = HashSet::new();

        let mut non_merge_candidates = HashSet::new();

        for (i, room) in map_region.rooms.iter() {
            let room_cells = room.cells.len();

            if room_cells <= max_size {
                merge_candidates.insert(*i);
            } else {
                non_merge_candidates.insert(*i);
            }
        }

        let mut visited_rooms = HashSet::new();
        let mut merge_pairs = HashMap::new();

        let mut rng = RngHandler::rng();

        for room_id in merge_candidates.iter() {
            if visited_rooms.contains(room_id) {
                continue;
            }

            visited_rooms.insert(room_id);

            let mut room_merged = false;

            let neighours = map_region.neighbours[room_id]
                .iter()
                .filter(|n| map_region.rooms.contains_key(n));
            for neighbour_id in neighours {
                if visited_rooms.contains(neighbour_id) {
                    continue;
                }

                let room = map_region
                    .rooms
                    .get(room_id)
                    .expect(format!("Room {} should exist", room_id).as_str());
                let neighbour_room = map_region
                    .rooms
                    .get(neighbour_id)
                    .expect(format!("Room {} should exist", room_id).as_str());

                if neighbour_room.cells.len() > max_size {
                    continue;
                }

                // If either rooms are the only neighbour of the other and that room has a area of 1
                // we don't merge them
                if (room.cells.len() == 1 && map_region.neighbours[room_id].len() == 1)
                    || neighbour_room.cells.len() == 1
                        && map_region.neighbours[neighbour_id].len() == 1
                {
                    continue;
                }

                if rng.random_bool(merge_prob) {
                    visited_rooms.insert(neighbour_id);

                    merge_pairs.insert(*room_id, *neighbour_id);

                    room_merged = true;

                    break;
                }
            }

            if !room_merged {
                non_merge_candidates.insert(*room_id);
            }
        }

        for (from, to) in merge_pairs.into_iter() {
            map_region
                .try_merge_rooms(from, to)
                .expect("Should merge rooms");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::algos::map_builder::bsp::{BinarySpacePartitioning, BinarySpacePartitioningConfig};

    #[test]
    fn test_merge_random_rooms() {
        let width = 20;
        let height = 20;
        let mut config = BinarySpacePartitioningConfig::default();
        // Force only two regions to be generated
        config.region_split_factor = width * height;

        let results = BinarySpacePartitioning::generate_and_trim_partitions(width, height, config);

        assert_eq!(results.len(), 2);

        // We take only the first region for simplicity
        let (origin_rect, rect_table, removed_rects, neighbours) = results[0].clone();

        let mut map_region =
            MapBuilder::generate_map_region(origin_rect, rect_table, removed_rects, neighbours);

        let room_count = map_region.rooms.len();
        assert!(room_count > 1, "There should be more than one room");
        let removed_room_count = map_region.removed_rooms.len();
        assert!(removed_room_count > 0, "There should be some removed rooms");
        let removed_neighbour_count = map_region.neighbours.len();
        assert!(
            removed_neighbour_count > 0,
            "There should be some neighbours"
        );

        let config = MapBuilderConfig::default();

        // Verify neighbor relationships
        for (idx, neighbor_set) in map_region.neighbours.iter() {
            let room = map_region
                .rooms
                .get(idx)
                .or_else(|| map_region.removed_rooms.get(idx))
                .expect("Room or removed room should exist for index");
            // Check each room has valid neighbors
            for neighbour_idx in neighbor_set.iter() {
                assert!(neighbour_idx != idx, "Room should not be its own neighbour");

                if let Some(neighbor_rect) = map_region.rooms.get(neighbour_idx) {
                    assert!(room.is_neighbour_of(neighbor_rect).is_some());
                } else if let Some(neighbor_rect) = map_region.removed_rooms.get(neighbour_idx) {
                    assert!(room.is_neighbour_of(neighbor_rect).is_some());
                } else {
                    panic!("Room or removed rect not found for index {}", idx);
                }
            }
        }

        MapBuilder::merge_random_rooms(&mut map_region, &config);

        // Check that the number of rooms, removed rooms, and neighbours has not increased
        let new_room_count = map_region.rooms.len();
        assert!(
            new_room_count <= room_count,
            "The number of rooms may have decreased after merging"
        );
        let new_removed_room_count = map_region.removed_rooms.len();
        assert!(
            new_removed_room_count == removed_room_count,
            "The number of removed rooms should not change after merging"
        );
        let new_removed_neighbour_count = map_region.neighbours.len();
        assert!(
            new_removed_neighbour_count <= removed_neighbour_count,
            "The number of neighbours may have decreased after merging"
        );

        // Check that the total number of rooms and removed rooms matches the neighbours
        assert!(
            map_region.rooms.len() + map_region.removed_rooms.len() == map_region.neighbours.len()
        );

        // Check that rects are not in the removed table
        // and that they are in the neighbours table
        for rect_idx in map_region.rooms.keys() {
            assert!(!map_region.removed_rooms.contains_key(rect_idx));
            assert!(map_region.neighbours.contains_key(rect_idx));
        }

        // Check that removed rects are not in the rect table
        // and that they are in the neighbours table
        for removed_rect_idx in map_region.removed_rooms.keys() {
            assert!(!map_region.rooms.contains_key(removed_rect_idx));
            assert!(map_region.neighbours.contains_key(removed_rect_idx));
        }

        // Verify neighbor relationships
        for (idx, neighbor_set) in map_region.neighbours.iter() {
            let rect = map_region
                .rooms
                .get(idx)
                .or_else(|| map_region.removed_rooms.get(idx))
                .expect("Rect or removed rect should exist for index");
            // Check each rect has valid neighbors
            for neighbour_idx in neighbor_set.iter() {
                if let Some(neighbor_rect) = map_region.rooms.get(neighbour_idx) {
                    assert!(rect.is_neighbour_of(neighbor_rect).is_some());
                } else if let Some(neighbor_rect) = map_region.removed_rooms.get(neighbour_idx) {
                    assert!(rect.is_neighbour_of(neighbor_rect).is_some());
                } else {
                    panic!("Rect or removed rect not found for index {}", idx);
                }
            }
        }
    }
}
