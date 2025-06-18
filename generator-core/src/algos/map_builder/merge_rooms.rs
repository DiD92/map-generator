use super::{MapBuilder, MapBuilderConfig};
use crate::{algos::RngHandler, types::MapRegion};

use std::collections::{HashMap, HashSet};

use rand::Rng;

impl MapBuilder {
    pub(super) fn merge_random_rooms(map_region: &mut MapRegion, config: &MapBuilderConfig) {
        let mut rooms_to_merge = HashSet::with_capacity(map_region.room_slots() / 4);
        let mut merge_groups = Vec::with_capacity(rooms_to_merge.capacity() / 2);

        let mut rng = RngHandler::rng();

        // We will use a buffer to collect filterd neighbours
        // to avoid excessive allocations
        let mut neighbour_buffer = Vec::new();

        // First, we build a set of rooms that are going to be merged
        for (room_id, room) in map_region.iter_active() {
            if rooms_to_merge.contains(&room_id) {
                continue;
            }

            // We only want to merge room_id with a neighbour that is not already in the rooms_to_merge set
            // and that is a valid room in the map_region
            neighbour_buffer.extend(
                map_region
                    .get_neighbours(room_id)
                    .iter()
                    .filter(|n| map_region.is_active(*n) && !rooms_to_merge.contains(n)),
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
                    .get(rng.random_range(0..neighbour_count))
                    .copied()
                    .expect("Should have a neighbour to merge with!");

                if room_id == selected_neighbour {
                    println!(
                        "Warning: Room ID {} is merging with itself, skipping.",
                        room_id
                    );
                }

                rooms_to_merge.insert(room_id);
                rooms_to_merge.insert(selected_neighbour);

                merge_groups.push((room_id, selected_neighbour));
            }

            neighbour_buffer.clear();
        }

        // Now we merge the rooms in the merge_groups
        for (room_a_idx, room_b_idx) in merge_groups.into_iter() {
            map_region
                .merge_active_rooms(room_a_idx, room_b_idx)
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

        for (i, room) in map_region.iter_active() {
            let room_cells = room.cells.len();

            if room_cells <= max_size {
                merge_candidates.insert(i);
            } else {
                non_merge_candidates.insert(i);
            }
        }

        let mut visited_rooms = HashSet::new();
        let mut merge_pairs = HashMap::new();

        let mut rng = RngHandler::rng();

        for &room_id in merge_candidates.iter() {
            if visited_rooms.contains(&room_id) {
                continue;
            }

            visited_rooms.insert(room_id);

            let mut room_merged = false;

            for neighbour_id in map_region.iter_active_neighbours(room_id) {
                if visited_rooms.contains(&neighbour_id) {
                    continue;
                }

                let room = map_region.get_active(room_id);
                let neighbour_room = map_region.get_active(neighbour_id);

                if neighbour_room.cells.len() > max_size {
                    continue;
                }

                let room_cells_count = room.cells.len();
                let room_neighbours_count = map_region.get_neighbours(room_id).len();
                let neighour_cells_count = neighbour_room.cells.len();
                let neighour_neighbours_count = map_region.get_neighbours(neighbour_id).len();

                // If either rooms are the only neighbour of the other and that room has a area of 1
                // we don't merge them
                if (room_cells_count == 1 && room_neighbours_count == 1)
                    || neighour_cells_count == 1 && neighour_neighbours_count == 1
                {
                    continue;
                }

                if rng.random_bool(merge_prob) {
                    visited_rooms.insert(neighbour_id);

                    merge_pairs.insert(room_id, neighbour_id);

                    room_merged = true;

                    break;
                }
            }

            if !room_merged {
                non_merge_candidates.insert(room_id);
            }
        }

        for (from, to) in merge_pairs.into_iter() {
            map_region
                .merge_active_rooms(from, to)
                .expect("Should merge rooms");
        }
    }
}

#[cfg(test)]
mod test {
    use crate::algos::map_builder::{BinarySpacePartitioningConfig, bsp::BinarySpacePartitioning};

    use super::*;

    #[test]
    fn test_merge_random_rooms() {
        let width = 20;
        let height = 20;
        let mut config = BinarySpacePartitioningConfig::default();
        // Force only two regions to be generated
        config.region_split_factor = width * height;

        let results = BinarySpacePartitioning::generate_and_trim_partitions(width, height, config);

        // We take only the first region for simplicity
        let (origin_rect, rect_table, removed_rects, neighbours) = results[0].clone();

        let mut map_region =
            MapBuilder::generate_map_region(origin_rect, rect_table, removed_rects, neighbours);

        let room_count = map_region.iter_active().count();
        assert!(room_count > 1, "There should be more than one room");
        let removed_room_count = map_region.iter_removed().count();
        assert!(removed_room_count > 0, "There should be some removed rooms");

        let config = MapBuilderConfig::default();

        MapBuilder::merge_random_rooms(&mut map_region, &config);

        // Check that the number of rooms, removed rooms, and neighbours has not increased
        let new_room_count = map_region.iter_active().count();
        assert!(
            new_room_count <= room_count,
            "The number of rooms may have decreased after merging"
        );
        let new_removed_room_count = map_region.iter_removed().count();
        assert!(
            new_removed_room_count == removed_room_count,
            "The number of removed rooms should not change after merging"
        );
    }
}
