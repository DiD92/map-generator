use super::{MapBuilder, MapBuilderConfig};
use crate::{
    algos::RngHandler,
    types::{MapRegion, RoomId, Vector2},
};

use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
};

use priority_queue::PriorityQueue;
use rand::Rng;

impl MapBuilder {
    pub(super) fn reconnect_room_groups(map_region: &mut MapRegion, config: &MapBuilderConfig) {
        let mut room_groups = Self::generate_room_groups(map_region, false);

        if room_groups.len() <= 1 {
            return;
        }

        // We remove groups with just 1 room
        room_groups.retain(|_, group| {
            if group.len() > 1 {
                true
            } else {
                let room_id = group.drain().next().unwrap();
                let room = map_region.rooms.remove(&room_id).unwrap();
                map_region.removed_rooms.insert(room_id, room);

                false
            }
        });

        loop {
            if room_groups.len() > 1 {
                // If there is more than one group, we need to connect them together
                Self::connect_room_groups(room_groups, map_region, config);

                room_groups = Self::generate_room_groups(map_region, false);
            } else {
                break;
            }
        }
    }

    fn generate_room_groups(
        map_region: &MapRegion,
        include_removed: bool,
    ) -> HashMap<usize, HashSet<RoomId>> {
        let mut room_groups = HashMap::new();
        let mut group_id = 0;

        // We need this for the case where we don't include removed rooms
        let empty_removed_rooms = HashMap::with_capacity(0);
        let mut map_rooms = map_region
            .rooms
            .keys()
            .chain(if include_removed {
                map_region.removed_rooms.keys()
            } else {
                empty_removed_rooms.keys()
            })
            .copied()
            .collect::<Vec<_>>();
        let mut visited_rooms = HashSet::new();

        while let Some(room_id) = map_rooms.pop() {
            if visited_rooms.contains(&room_id) {
                continue;
            }

            let mut rooms_to_visit = vec![room_id];
            let mut group_visited_rooms = HashSet::new();

            while let Some(room_id) = rooms_to_visit.pop() {
                group_visited_rooms.insert(room_id);
                visited_rooms.insert(room_id);

                for neighbour_id in map_region.neighbours[&room_id].iter() {
                    if map_region.rooms.contains_key(neighbour_id)
                        && !group_visited_rooms.contains(neighbour_id)
                    {
                        rooms_to_visit.push(*neighbour_id);
                    }

                    if include_removed
                        && map_region.removed_rooms.contains_key(neighbour_id)
                        && !group_visited_rooms.contains(neighbour_id)
                    {
                        rooms_to_visit.push(*neighbour_id);
                    }
                }
            }

            room_groups.insert(group_id, group_visited_rooms);

            group_id += 1;
        }

        room_groups
    }

    fn connect_room_groups(
        mut room_groups: HashMap<usize, HashSet<RoomId>>,
        map_region: &mut MapRegion,
        config: &MapBuilderConfig,
    ) {
        // First we remove the lowest percentile sized groups
        Self::remove_small_groups(map_region, &mut room_groups);

        // Then we compute the center of each group and each room
        let (group_centers, room_centers) = Self::generate_group_centers(map_region, &room_groups);

        // We use the group centers to find the closest groups to each other
        let closest_groups =
            Self::generate_closest_groups(&group_centers, config.group_loop_connection_chance);

        for (group_a_idx, group_b_idx) in closest_groups.into_iter() {
            if group_a_idx == group_b_idx {
                continue;
            }

            let mut closest_a = usize::MAX;
            let mut closest_b = usize::MAX;
            let mut closest_distance = f32::MAX;

            // We find the closest rooms in each group pair
            for room_a_id in room_groups[&group_a_idx].iter() {
                for room_b_id in room_groups[&group_b_idx].iter() {
                    let distance_a = room_centers[room_a_id];
                    let distance_b = room_centers[room_b_id];

                    let distance = distance_a.distance(&distance_b);
                    if distance < closest_distance {
                        closest_distance = distance;
                        closest_a = *room_a_id;
                        closest_b = *room_b_id;
                    }
                }
            }

            // We find the rooms that connect the two closest rooms
            let room_path = Self::get_path_between_rooms(closest_a, closest_b, map_region);

            // For every room in the path that was removed, we move it
            // back to the map region
            for room_id in room_path.into_iter() {
                if let Some(room) = map_region.removed_rooms.remove(&room_id) {
                    map_region.rooms.insert(room_id, room);
                }
            }
        }
    }

    fn remove_small_groups(
        map_region: &mut MapRegion,
        room_groups: &mut HashMap<usize, HashSet<usize>>,
    ) {
        let group_count = room_groups.len() as f32;

        let total_rooms = room_groups
            .iter()
            .map(|(_, group_rooms)| group_rooms.len() as f32)
            .sum::<f32>();

        let group_size_cutoff = {
            let room_avg_lowest_perc = (total_rooms / group_count) * 0.3;

            if room_avg_lowest_perc < 1.0 {
                1.0
            } else {
                room_avg_lowest_perc
            }
        } as usize;

        room_groups.retain(|_, rooms| {
            if rooms.len() > group_size_cutoff {
                true
            } else {
                for room_id in rooms.drain() {
                    let room = map_region.rooms.remove(&room_id).unwrap();
                    map_region.removed_rooms.insert(room_id, room);
                }
                false
            }
        });
    }

    fn generate_group_centers(
        map_region: &MapRegion,
        room_groups: &HashMap<usize, HashSet<usize>>,
    ) -> (HashMap<usize, Vector2>, HashMap<RoomId, Vector2>) {
        let room_centers = map_region
            .rooms
            .iter()
            .chain(map_region.removed_rooms.iter())
            .map(|(idx, room)| (*idx, room.get_center()))
            .collect::<HashMap<_, _>>();

        let group_centers = room_groups
            .iter()
            .map(|(group_id, group)| {
                let (mut center, count) = group.iter().map(|room_id| room_centers[room_id]).fold(
                    (Vector2::ZERO, 0_u32),
                    |(mut center, mut count), other_vector| {
                        center.x += other_vector.x;
                        center.y += other_vector.y;
                        count += 1;

                        (center, count)
                    },
                );

                center.x /= count as f32;
                center.y /= count as f32;

                (*group_id, center)
            })
            .collect::<HashMap<_, _>>();

        (group_centers, room_centers)
    }

    fn generate_closest_groups(
        group_centers: &HashMap<usize, Vector2>,
        loop_connection_chance: f64,
    ) -> Vec<(usize, usize)> {
        let mut closer_groups = Vec::new();
        let mut visited_links = HashSet::new();

        let mut rng = RngHandler::rng();

        // We compute the closest groups to each other
        for (group_id, center) in group_centers.iter() {
            let mut min_distance = f32::MAX;
            let mut maybe_closest_group_id = None;
            let mut maybe_second_closest_group_id = None;

            let should_multi_connect = rng.random_bool(loop_connection_chance);

            for (other_group_id, other_center) in group_centers.iter() {
                if group_id == other_group_id
                    || visited_links.contains(&(*other_group_id, *group_id))
                {
                    continue;
                }

                let center_distance = center.distance(other_center);

                if center_distance < min_distance {
                    if should_multi_connect {
                        maybe_second_closest_group_id = maybe_closest_group_id;
                    }

                    min_distance = center_distance;
                    maybe_closest_group_id = Some(*other_group_id);
                }
            }

            if let Some(closest_group_id) = maybe_closest_group_id {
                closer_groups.push((*group_id, closest_group_id));

                visited_links.insert((*group_id, closest_group_id));
                visited_links.insert((closest_group_id, *group_id));
            }

            if let Some(second_closest_group_id) = maybe_second_closest_group_id {
                closer_groups.push((*group_id, second_closest_group_id));

                visited_links.insert((*group_id, second_closest_group_id));
                visited_links.insert((second_closest_group_id, *group_id));
            }
        }

        closer_groups
    }

    fn get_path_between_rooms(
        origin_idx: usize,
        target_idx: usize,
        map_region: &mut MapRegion,
    ) -> Vec<usize> {
        let mut move_queue = PriorityQueue::new();
        let mut move_visited = HashMap::new();

        let mut smallest_cost = usize::MAX;
        let mut shortest_path = Vec::new();

        move_queue.push((origin_idx, vec![origin_idx]), Reverse(1_usize));

        while let Some(((node, path), Reverse(path_len))) = move_queue.pop() {
            let should_visit = if let Some(cost) = move_visited.get(&node) {
                cost > &path_len
            } else {
                true
            };

            if !should_visit {
                continue;
            }

            move_visited.insert(node, path_len);

            if node == target_idx && path_len < smallest_cost {
                smallest_cost = path_len + 1;
                shortest_path.clear();
                shortest_path.extend(path.into_iter());
            } else {
                for neighbour_idx in map_region.neighbours[&node].iter() {
                    let mut new_path = path.clone();
                    new_path.push(*neighbour_idx);
                    let new_path_cost = path_len + 1;

                    move_queue.push_increase((*neighbour_idx, new_path), Reverse(new_path_cost));
                }
            }
        }

        shortest_path
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn test_generate_room_groups() {
        let map_region = MapRegion::new_test_region();

        let room_groups = MapBuilder::generate_room_groups(&map_region, false);
        assert!(room_groups.len() == 4, "There should be 4 room groups");

        let mut groups_vec = room_groups
            .values()
            .map(|set| BTreeSet::from_iter(set))
            .collect::<Vec<_>>();
        groups_vec.sort();

        assert_eq!(groups_vec[0], BTreeSet::from_iter(&[0, 1, 6]));
        assert_eq!(groups_vec[1], BTreeSet::from_iter(&[3, 4]));
        assert_eq!(groups_vec[2], BTreeSet::from_iter(&[12, 13]));
        assert_eq!(groups_vec[3], BTreeSet::from_iter(&[15]));

        assert!(!room_groups.is_empty(), "Room groups should not be empty");
        assert!(
            room_groups.values().all(|group| !group.is_empty()),
            "All room groups should be non-empty"
        );

        let total_rooms: usize = room_groups.values().map(|group| group.len()).sum();
        assert_eq!(
            total_rooms,
            map_region.rooms.len(),
            "All rooms should be assigned to a group"
        );

        let mut visited_rooms = HashSet::new();
        for group in room_groups.values() {
            for room_id in group {
                assert!(
                    !visited_rooms.contains(room_id),
                    "Room {} should not be visited more than once",
                    room_id
                );
                visited_rooms.insert(*room_id);
            }
        }
    }

    #[test]
    fn test_remove_small_groups() {
        let mut map_region = MapRegion::new_test_region();

        let mut room_groups = MapBuilder::generate_room_groups(&map_region, false);
        assert!(room_groups.len() == 4, "There should be 4 room groups");

        let mut groups_vec = room_groups
            .values()
            .map(|set| BTreeSet::from_iter(set))
            .collect::<Vec<_>>();
        groups_vec.sort();

        assert_eq!(groups_vec[0], BTreeSet::from_iter(&[0, 1, 6]));
        assert_eq!(groups_vec[1], BTreeSet::from_iter(&[3, 4]));
        assert_eq!(groups_vec[2], BTreeSet::from_iter(&[12, 13]));
        assert_eq!(groups_vec[3], BTreeSet::from_iter(&[15]));

        MapBuilder::remove_small_groups(&mut map_region, &mut room_groups);

        room_groups = MapBuilder::generate_room_groups(&map_region, false);
        assert!(room_groups.len() == 3, "There should be 4 room groups");

        groups_vec = room_groups
            .values()
            .map(|set| BTreeSet::from_iter(set))
            .collect::<Vec<_>>();
        groups_vec.sort();

        assert_eq!(groups_vec[0], BTreeSet::from_iter(&[0, 1, 6]));
        assert_eq!(groups_vec[1], BTreeSet::from_iter(&[3, 4]));
        assert_eq!(groups_vec[2], BTreeSet::from_iter(&[12, 13]));
    }

    #[test]
    fn test_get_path_between_rooms() {
        let mut map_region = MapRegion::new_test_region();

        let origin_idx = 13; // Room E
        let target_idx = 15; // Room F

        let path = MapBuilder::get_path_between_rooms(origin_idx, target_idx, &mut map_region);

        assert!(!path.is_empty(), "Path should not be empty");
        assert!(
            path.contains(&origin_idx),
            "Path should contain the origin room"
        );
        assert!(
            path.contains(&target_idx),
            "Path should contain the target room"
        );
        // The path may go either through room A or room 9
        assert!(path == vec![13, 14, 10, 15] || path == vec![13, 9, 10, 15]);
    }

    #[test]
    fn test_reconnect_room_groups() {
        let mut map_region = MapRegion::new_test_region();

        let room_count = map_region.rooms.len();
        assert!(room_count == 8, "There should be 8 rooms");
        let removed_room_count = map_region.removed_rooms.len();
        assert!(removed_room_count == 8, "There should be 8 removed rooms");
        let neighbour_count = map_region.neighbours.len();
        assert!(
            neighbour_count == 16,
            "There should be 16 neighbour entries"
        );

        let room_groups = MapBuilder::generate_room_groups(&map_region, false);

        assert!(room_groups.len() == 4, "There should be 4 room groups");

        MapBuilder::reconnect_room_groups(&mut map_region, &MapBuilderConfig::default());

        let room_groups = MapBuilder::generate_room_groups(&map_region, false);

        assert_eq!(room_groups.len(), 1, "There should be only one room group");

        let new_room_count = map_region.rooms.len();
        assert!(
            new_room_count > room_count,
            "There should be more rooms after reconnecting groups"
        );

        let new_removed_room_count = map_region.removed_rooms.len();
        assert!(
            new_removed_room_count < removed_room_count,
            "There should be fewer removed rooms after reconnecting groups"
        );

        let new_neighbour_count = map_region.neighbours.len();
        assert_eq!(
            new_neighbour_count, neighbour_count,
            "There should be the same number of neighbours after reconnecting groups"
        );

        assert_eq!(
            new_room_count + new_removed_room_count,
            new_neighbour_count,
            "Total new rooms and new removed rooms should still equal total neighbours"
        );
    }
}
