use crate::{constants::REGION_SPLIT_FACTOR, types::*};

use anyhow::Result;
use rayon::prelude::*;
use tracing::event;

mod add_doors;
mod bsp;
mod connect_regions;
mod gen_rooms;
mod merge_rooms;
mod reconnect_rooms;
mod room_decorator;

pub(crate) struct MapBuilderConfig {
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
            MapStyle::CastlevaniaCOTM => {
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
            MapStyle::MetroidZM => {
                base.bsp_config.region_split_factor =
                    (REGION_SPLIT_FACTOR / 4) + (REGION_SPLIT_FACTOR / 2);
                base.bsp_config.horizontal_region_prob = 0.75;
                base.bsp_config.big_rect_area_cutoff = 14;
                base.bsp_config.big_rect_survival_prob = 0.09;
                base.bsp_config.horizontal_split_prob = 0.85;
                base.bsp_config.height_factor_cutoff = 2.9;
                base.bsp_config.width_factor_cutoff = 2.6;
                base.bsp_config.rect_survival_prob = 0.33;
                base.bsp_config.trim_highly_connected_rect_prob = 0.6;
                base.bsp_config.trim_fully_connected_rect_prob = 0.7;

                base.merge_regions = false;

                base.random_room_merge_prob = 0.03;
                base.group_loop_connection_chance = 0.19;
                base.loop_connection_chance = 0.22;
                base.repeat_small_room_merge_prob = 0.51;
                base.bisect_room_prob = 0.17;
            }
            MapStyle::MetroidFS => {
                base.bsp_config.region_split_factor =
                    (REGION_SPLIT_FACTOR / 4) + (REGION_SPLIT_FACTOR / 2);
                base.bsp_config.horizontal_region_prob = 0.75;
                base.bsp_config.big_rect_area_cutoff = 14;
                base.bsp_config.big_rect_survival_prob = 0.09;
                base.bsp_config.horizontal_split_prob = 0.85;
                base.bsp_config.height_factor_cutoff = 2.9;
                base.bsp_config.width_factor_cutoff = 2.6;
                base.bsp_config.rect_survival_prob = 0.33;
                base.bsp_config.trim_highly_connected_rect_prob = 0.8;
                base.bsp_config.trim_fully_connected_rect_prob = 0.9;

                base.merge_regions = false;

                base.random_room_merge_prob = 0.03;
                base.group_loop_connection_chance = 0.19;
                base.loop_connection_chance = 0.22;
                base.repeat_small_room_merge_prob = 0.51;
                base.bisect_room_prob = 0.17;
            }
            MapStyle::MetroidSP => {
                base.bsp_config.region_split_factor =
                    (REGION_SPLIT_FACTOR / 4) + (REGION_SPLIT_FACTOR / 2);
                base.bsp_config.horizontal_region_prob = 0.75;
                base.bsp_config.big_rect_area_cutoff = 14;
                base.bsp_config.big_rect_survival_prob = 0.09;
                base.bsp_config.horizontal_split_prob = 0.85;
                base.bsp_config.height_factor_cutoff = 2.9;
                base.bsp_config.width_factor_cutoff = 2.6;
                base.bsp_config.rect_survival_prob = 0.33;
                base.bsp_config.trim_highly_connected_rect_prob = 0.8;
                base.bsp_config.trim_fully_connected_rect_prob = 0.9;

                base.merge_regions = false;

                base.random_room_merge_prob = 0.03;
                base.group_loop_connection_chance = 0.19;
                base.loop_connection_chance = 0.22;
                base.repeat_small_room_merge_prob = 0.51;
                base.bisect_room_prob = 0.17;
            }
        }

        base
    }
}

pub(crate) struct MapBuilder {
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

        let rect_regions = bsp::BinarySpacePartitioning::generate_and_trim_partitions(
            self.cols,
            self.rows,
            config.bsp_config,
        );

        let region_idx_offsets = Self::generate_region_offsets(
            rect_regions.iter().map(|(_, rects, _, _)| rects),
            rect_regions.len(),
        );

        let bsp_time = std::time::SystemTime::now();
        event!(
            tracing::Level::DEBUG,
            "Generated BSP regions in {:?}ms",
            bsp_time.duration_since(build_start).unwrap().as_millis()
        );

        let map_regions = rect_regions.into_par_iter().zip(region_idx_offsets).map(
            |((origin_rect, region_rects, removed_rects, neighbours), idx_offset)| {
                let mut map_region =
                    Self::generate_map_region(origin_rect, region_rects, removed_rects, neighbours);

                map_region.offset_room_indexes(idx_offset);

                Self::merge_random_rooms(&mut map_region, config);

                Self::reconnect_room_groups(&mut map_region, config);

                // We randomly merge some groups of 1 sized-rooms first
                Self::merge_repeated_simple_rooms(
                    &mut map_region,
                    1,
                    config.repeat_small_room_merge_prob,
                );
                // Then we merge rooms of size 2 or less
                Self::merge_repeated_simple_rooms(
                    &mut map_region,
                    2,
                    config.repeat_small_room_merge_prob / 2.0,
                );

                map_region
            },
        );

        let map_regions_time = std::time::SystemTime::now();
        event!(
            tracing::Level::DEBUG,
            "Generated map regions in {:?}ms",
            map_regions_time
                .duration_since(bsp_time)
                .unwrap()
                .as_millis()
        );

        let generated_maps = if config.merge_regions {
            let origin_rect = Rect {
                origin: Cell::new(0, 0),
                width: self.cols,
                height: self.rows,
            };

            let mut map_region = map_regions.reduce(
                || MapRegion::new(origin_rect),
                |mut acc, region| {
                    acc.rooms.extend(region.rooms);
                    acc.removed_rooms.extend(region.removed_rooms);
                    acc.neighbours.extend(region.neighbours);

                    acc
                },
            );

            Self::reconnect_room_groups(&mut map_region, config);

            //let doors = Self::add_doors_to_rooms(&map_region, config);
            let doors = vec![];

            let room_decorator = room_decorator::RoomDecoratorFactory::decorator_for(style);
            room_decorator::RoomDecorator::decorate(
                room_decorator.as_ref(),
                &mut map_region,
                &doors,
                config,
            );

            let rooms = map_region.rooms.into_values().collect();

            vec![Map {
                origin_rect: map_region.origin_rect,
                rooms,
                doors,
            }]
        } else {
            let mut maps = map_regions
                .map(|mut map_region| {
                    let doors = Self::add_doors_to_rooms(&map_region, config);

                    let room_decorator = room_decorator::RoomDecoratorFactory::decorator_for(style);
                    room_decorator::RoomDecorator::decorate(
                        room_decorator.as_ref(),
                        &mut map_region,
                        &doors,
                        config,
                    );

                    Map {
                        origin_rect: map_region.origin_rect,
                        rooms: map_region.rooms.into_values().collect(),
                        doors,
                    }
                })
                .collect::<Vec<_>>();

            Self::connect_regions(&mut maps);

            maps
        };

        let generated_maps_time = std::time::SystemTime::now();
        event!(
            tracing::Level::DEBUG,
            "Generated maps in {:?}ms",
            generated_maps_time
                .duration_since(map_regions_time)
                .unwrap()
                .as_millis()
        );

        let built_rooms = generated_maps
            .iter()
            .fold(0_usize, |acc, map| acc + map.rooms.len());

        let built_doors = generated_maps
            .iter()
            .fold(0_usize, |acc, map| acc + map.doors.len());

        let build_end = std::time::SystemTime::now();
        event!(
            tracing::Level::DEBUG,
            "Built map with {} rooms and {} doors in {:?}ms",
            built_rooms,
            built_doors,
            build_end.duration_since(build_start).unwrap().as_millis()
        );

        generated_maps
    }
}
