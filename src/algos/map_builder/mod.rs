use crate::types::*;

use anyhow::Result;
use rayon::prelude::*;

mod add_doors;
mod bsp;
mod decorate_rooms;
mod gen_rooms;
mod merge_rooms;
mod reconnect_rooms;

pub struct MapBuilderConfig {
    // The probability of randomly merging two rooms into one.
    pub random_room_merge_prob: f64,
    // Probability of having a group reconnect to two groups instead of one
    pub group_loop_connection_chance: f64,
    // Probability of opening a connection between rooms that will
    // cause a navigation loop in the map.
    pub loop_connection_chance: f64,
    pub repeat_small_room_merge_prob: f64,
}

impl Default for MapBuilderConfig {
    fn default() -> Self {
        MapBuilderConfig {
            random_room_merge_prob: 0.05,
            group_loop_connection_chance: 0.17,
            loop_connection_chance: 0.2,
            repeat_small_room_merge_prob: 0.2,
        }
    }
}

pub struct MapBuilder {
    pub cols: u32,
    pub rows: u32,
}

impl MapBuilder {
    pub fn new(cols: u32, rows: u32) -> Result<Self> {
        if cols == 0 || rows == 0 {
            return Err(anyhow::anyhow!(
                "Columns and rows must be greater than zero"
            ));
        }

        Ok(MapBuilder { cols, rows })
    }

    pub fn build(&self, config: &MapBuilderConfig) -> Map {
        let build_start = std::time::SystemTime::now();

        let rects = bsp::BinarySpacePartitioning::generate_and_trim_partitions(
            self.cols,
            self.rows,
            bsp::BinarySpacePartitioningConfig::default(),
        );

        let rooms = rects
            .into_par_iter()
            .map(|rects| {
                let (rooms, neighbours) = Self::generate_initial_rooms(rects);

                let (mut rooms, mut neighbours) =
                    Self::merge_random_rooms(rooms, neighbours, config);

                Self::reconnect_room_groups(&mut rooms, &mut neighbours, config);

                rooms.into_values().collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>();

        let mut room_table = rooms.into_par_iter().enumerate().collect::<RoomTable>();
        let mut neighbour_table = Self::generate_neighbour_table(&room_table);

        Self::reconnect_room_groups(&mut room_table, &mut neighbour_table, config);

        Self::decorate_rooms(&mut room_table, &neighbour_table, config);

        let (rooms, doors) = Self::add_doors_to_rooms(room_table, neighbour_table, config);

        let build_end = std::time::SystemTime::now();
        println!(
            "Built map with {} rooms and {} doors in {:?}ms",
            rooms.len(),
            doors.len(),
            build_end.duration_since(build_start).unwrap().as_millis()
        );

        Map { rooms, doors }
    }
}
