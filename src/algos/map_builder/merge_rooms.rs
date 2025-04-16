use super::{MapBuilder, MapBuilderConfig};
use crate::types::{NeighbourTable, RoomTable};

use std::{collections::HashSet, sync::Mutex};

use rand::Rng;
use rayon::prelude::*;

impl MapBuilder {
    pub(super) fn merge_random_rooms(
        rooms: RoomTable,
        neighbour_table: NeighbourTable,
        config: &MapBuilderConfig,
    ) -> (RoomTable, NeighbourTable) {
        let rooms_to_merge_mutex = Mutex::new(HashSet::new());

        let merge_groups = neighbour_table
            .par_iter()
            .filter_map(|(i, neighbours)| {
                let mut rng = rand::rng();

                let neighbour_count = neighbours.len();

                let room = rooms.get(i).unwrap();

                if room.cells.len() == 1 && neighbour_count == 1 {
                    return None;
                }

                if neighbour_count > 0 && rng.random_bool(config.random_room_merge_prob) {
                    let selected_neighbour = *neighbours
                        .iter()
                        .nth(rng.random_range(0..neighbour_count))
                        .unwrap();

                    if let Ok(ref mut guard) = rooms_to_merge_mutex.lock() {
                        if guard.contains(&selected_neighbour) || guard.contains(i) {
                            return None;
                        }

                        guard.insert(*i);
                        guard.insert(selected_neighbour);
                    } else {
                        println!("Failed to lock rooms_to_merge mutex");
                        return None;
                    }

                    Some((*i, selected_neighbour))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let rooms_mutex = Mutex::new(rooms);
        let neighbour_table_mutex = Mutex::new(neighbour_table);

        merge_groups.into_par_iter().for_each(|(from, to)| {
            if let Ok(ref mut rooms) = rooms_mutex.lock() {
                let from_room = rooms.remove(&from).unwrap();
                let to_room = rooms.remove(&to).unwrap();

                let merged_room = from_room.merged_with(to_room);
                rooms.insert(from, merged_room);
            } else {
                println!("Failed to lock rooms mutex");
                return;
            }

            if let Ok(ref mut neighbour_table) = neighbour_table_mutex.lock() {
                let mut from_neighbours = neighbour_table.remove(&from).unwrap();
                from_neighbours.remove(&to);

                let mut to_neighbours = neighbour_table.remove(&to).unwrap();
                to_neighbours.remove(&from);

                for neighbour in to_neighbours.iter() {
                    if let Some(neighbours) = neighbour_table.get_mut(neighbour) {
                        neighbours.remove(&to);
                        neighbours.insert(from);
                    }
                }

                from_neighbours.extend(to_neighbours);
                neighbour_table.insert(from, from_neighbours);
            } else {
                println!("Failed to lock neighbour table mutex");
            }
        });

        (
            rooms_mutex.into_inner().unwrap(),
            neighbour_table_mutex.into_inner().unwrap(),
        )
    }
}
