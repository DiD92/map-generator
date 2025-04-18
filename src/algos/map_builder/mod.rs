use crate::types::*;

use anyhow::Result;
use rayon::prelude::*;

mod add_doors;
mod bsp;
mod gen_rooms;
mod merge_rooms;
mod reconnect_rooms;
mod room_decorator;

pub struct MapBuilderConfig {
    pub bsp_config: bsp::BinarySpacePartitioningConfig,
    // Should we merge the regions after generating their rooms?
    pub merge_regions: bool,
    // The probability of randomly merging two rooms into one.
    pub random_room_merge_prob: f64,
    // Probability of having a group reconnect to two groups instead of one
    pub group_loop_connection_chance: f64,
    // Probability of opening a connection between rooms that will
    // cause a navigation loop in the map.
    pub loop_connection_chance: f64,
    pub repeat_small_room_merge_prob: f64,
    pub bisect_room_prob: f64,
}

impl Default for MapBuilderConfig {
    fn default() -> Self {
        MapBuilderConfig {
            bsp_config: bsp::BinarySpacePartitioningConfig::default(),
            merge_regions: true,
            random_room_merge_prob: 0.05,
            group_loop_connection_chance: 0.17,
            loop_connection_chance: 0.2,
            repeat_small_room_merge_prob: 0.2,
            bisect_room_prob: 0.1,
        }
    }
}

impl MapBuilderConfig {
    pub fn from_style(style: MapStyle) -> Self {
        let mut base = Self::default();

        match style {
            MapStyle::CastlevaniaSOTN => {
                base.bsp_config.horizontal_region_prob = 0.75;
                base.bsp_config.big_rect_area_cutoff = 14;
                base.bsp_config.big_rect_survival_prob = 0.09;
                base.bsp_config.horizontal_split_prob = 0.85;
                base.bsp_config.height_factor_cutoff = 2.9;
                base.bsp_config.width_factor_cutoff = 2.6;
                base.bsp_config.rect_survival_prob = 0.33;
                base.bsp_config.trim_highly_connected_rect_prob = 0.8;
                base.bsp_config.trim_fully_connected_rect_prob = 0.9;

                base.random_room_merge_prob = 0.03;
                base.group_loop_connection_chance = 0.19;
                base.loop_connection_chance = 0.22;
                base.repeat_small_room_merge_prob = 0.51;
                base.bisect_room_prob = 0.17;
            }
            MapStyle::CastlevaniaAOS => {
                base.bsp_config.horizontal_region_prob = 0.0;
                base.bsp_config.big_rect_area_cutoff = 11;
                base.bsp_config.big_rect_survival_prob = 0.12;
                base.bsp_config.horizontal_split_prob = 0.82;
                base.bsp_config.height_factor_cutoff = 2.4;
                base.bsp_config.width_factor_cutoff = 2.6;
                base.bsp_config.rect_survival_prob = 0.51;
                base.bsp_config.trim_highly_connected_rect_prob = 0.77;
                base.bsp_config.trim_fully_connected_rect_prob = 0.85;

                base.random_room_merge_prob = 0.01;
                base.group_loop_connection_chance = 0.19;
                base.loop_connection_chance = 0.24;
                base.repeat_small_room_merge_prob = 0.45;
                base.bisect_room_prob = 0.15;
            }
            MapStyle::CastlevaniaCOTN => {
                base.bsp_config.horizontal_region_prob = 0.1;
                base.bsp_config.big_rect_area_cutoff = 12;
                base.bsp_config.big_rect_survival_prob = 0.15;
                base.bsp_config.horizontal_split_prob = 0.82;
                base.bsp_config.height_factor_cutoff = 1.4;
                base.bsp_config.width_factor_cutoff = 2.6;
                base.bsp_config.rect_survival_prob = 0.95;
                base.bsp_config.trim_highly_connected_rect_prob = 0.95;
                base.bsp_config.trim_fully_connected_rect_prob = 0.60;

                base.random_room_merge_prob = 0.15;
                base.group_loop_connection_chance = 0.10;
                base.loop_connection_chance = 0.14;
                base.repeat_small_room_merge_prob = 0.85;
                base.bisect_room_prob = 0.29;
            }
            MapStyle::CastlevaniaHOD => {
                base.bsp_config.horizontal_region_prob = 0.75;
                base.bsp_config.big_rect_area_cutoff = 8;
                base.bsp_config.big_rect_survival_prob = 0.09;
                base.bsp_config.horizontal_split_prob = 0.85;
                base.bsp_config.height_factor_cutoff = 1.9;
                base.bsp_config.width_factor_cutoff = 1.6;
                base.bsp_config.rect_survival_prob = 0.70;
                base.bsp_config.trim_highly_connected_rect_prob = 0.8;
                base.bsp_config.trim_fully_connected_rect_prob = 0.9;

                base.random_room_merge_prob = 0.03;
                base.group_loop_connection_chance = 0.19;
                base.loop_connection_chance = 0.22;
                base.repeat_small_room_merge_prob = 0.81;
                base.bisect_room_prob = 0.17;
            }
            MapStyle::MetroidZM => todo!(),
            MapStyle::MetroidFS => todo!(),
            MapStyle::MetroidSP => todo!(),
        }

        base
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

    pub fn build(&self, config: &MapBuilderConfig, style: MapStyle) -> Vec<Map> {
        let build_start = std::time::SystemTime::now();

        let rects = bsp::BinarySpacePartitioning::generate_and_trim_partitions(
            self.cols,
            self.rows,
            config.bsp_config,
        );

        let rooms = rects.into_par_iter().map(|rects| {
            let (rooms, neighbours) = Self::generate_initial_rooms(rects);

            let (mut rooms, mut neighbours) = Self::merge_random_rooms(rooms, neighbours, config);

            Self::reconnect_room_groups(&mut rooms, &mut neighbours, config);

            rooms.into_values().collect::<Vec<_>>()
        });

        let room_decorator = room_decorator::RoomDecoratorFactory::decorator_for(style);

        let generate_map = if config.merge_regions {
            let mut room_table = rooms
                .flatten()
                .collect::<Vec<_>>()
                .into_par_iter()
                .enumerate()
                .collect::<RoomTable>();
            let mut neighbour_table = Self::generate_neighbour_table(&room_table);

            Self::reconnect_room_groups(&mut room_table, &mut neighbour_table, config);

            let doors = Self::add_doors_to_rooms(&room_table, &neighbour_table, config);

            room_decorator::RoomDecorator::decorate(
                &room_decorator,
                &mut room_table,
                &neighbour_table,
                &doors,
                config,
            );

            let rooms = room_table.into_values().collect();

            vec![Map { rooms, doors }]
        } else {
            let room_tables = rooms.collect::<Vec<_>>();

            room_tables
                .into_par_iter()
                .map(|room_table| {
                    let mut room_table = room_table
                        .into_par_iter()
                        .enumerate()
                        .collect::<RoomTable>();
                    let neighbour_table = Self::generate_neighbour_table(&room_table);

                    let doors = Self::add_doors_to_rooms(&room_table, &neighbour_table, config);

                    room_decorator::RoomDecorator::decorate(
                        &room_decorator,
                        &mut room_table,
                        &neighbour_table,
                        &doors,
                        config,
                    );

                    Map {
                        rooms: room_table.into_values().collect(),
                        doors,
                    }
                })
                .collect::<Vec<_>>()

            // TODO: We need an additional step to connect the regions either with special doors or another means

            //Map { rooms, doors }
        };

        let built_rooms = generate_map
            .iter()
            .fold(0_usize, |acc, map| acc + map.rooms.len());

        let built_doors = generate_map
            .iter()
            .fold(0_usize, |acc, map| acc + map.doors.len());

        let build_end = std::time::SystemTime::now();
        println!(
            "Built map with {} rooms and {} doors in {:?}ms",
            built_rooms,
            built_doors,
            build_end.duration_since(build_start).unwrap().as_millis()
        );

        generate_map
    }
}
