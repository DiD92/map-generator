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
        }
    }
}

impl MapBuilderConfig {
    pub fn from_style(style: MapStyle) -> Self {
        let mut base = Self::default();

        match style {
            MapStyle::CastlevaniaSOTN => {
                base.bsp_config.horizontal_region_prob = 0.73;
                base.bsp_config.big_rect_survival_prob = 0.05;
                base.bsp_config.horizontal_split_prob = 0.75;
                base.bsp_config.width_factor_cutoff = 2.3;
                base.bsp_config.rect_survival_prob = 0.39;

                base.random_room_merge_prob = 0.03;
                base.group_loop_connection_chance = 0.19;
                base.loop_connection_chance = 0.22;
                base.repeat_small_room_merge_prob = 0.21;
            }
            MapStyle::CastlevaniaAOS => todo!(),
            MapStyle::CastlevaniaCOTN => todo!(),
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

    pub fn build(&self, config: &MapBuilderConfig, style: MapStyle) -> Map {
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

            room_decorator::RoomDecorator::decorate(
                &room_decorator,
                &mut room_table,
                &neighbour_table,
                config,
            );

            let (rooms, doors) = Self::add_doors_to_rooms(room_table, neighbour_table, config);

            Map { rooms, doors }
        } else {
            let room_tables = rooms.collect::<Vec<_>>();

            let (rooms, doors) = room_tables
                .into_par_iter()
                .map(|room_table| {
                    let mut room_table = room_table
                        .into_par_iter()
                        .enumerate()
                        .collect::<RoomTable>();
                    let neighbour_table = Self::generate_neighbour_table(&room_table);

                    room_decorator::RoomDecorator::decorate(
                        &room_decorator,
                        &mut room_table,
                        &neighbour_table,
                        config,
                    );

                    Self::add_doors_to_rooms(room_table, neighbour_table, config)
                })
                .flatten()
                .collect::<(Vec<_>, Vec<_>)>();

            // TODO: We need an additional step to connect the regions either with special doors or another means

            Map { rooms, doors }
        };

        let build_end = std::time::SystemTime::now();
        println!(
            "Built map with {} rooms and {} doors in {:?}ms",
            generate_map.rooms.len(),
            generate_map.doors.len(),
            build_end.duration_since(build_start).unwrap().as_millis()
        );

        generate_map
    }
}
