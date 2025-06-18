use crate::{
    MapStyle,
    types::{Map, Rect},
};

use anyhow::Result;
use rayon::prelude::*;
use tracing::event;

mod add_doors;
mod bisect_rooms;
mod bsp;
mod builder_config;
mod connect_regions;
mod gen_rooms;
mod merge_regions;
mod merge_rooms;
mod reconnect_rooms;
mod room_decorator;

use builder_config::BinarySpacePartitioningConfig;
pub(crate) use builder_config::MapBuilderConfig;

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
        let build_start = std::time::Instant::now();

        let rect_groups = bsp::BinarySpacePartitioning::generate_and_trim_partitions(
            self.cols,
            self.rows,
            config.bsp_config,
        );

        let rect_groups_time = std::time::Instant::now();
        event!(
            tracing::Level::DEBUG,
            "Generated rectangle groups in {:.2}ms",
            rect_groups_time.duration_since(build_start).as_millis()
        );

        let map_regions = rect_groups
            .into_par_iter()
            .by_uniform_blocks(30)
            .map(|(origin_rect, region_rects, removed_rects, neighbours)| {
                let mut map_region =
                    Self::generate_map_region(origin_rect, region_rects, removed_rects, neighbours);

                map_region.compact_buffers();

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

                // Finally we bisect long horizontal rooms randomly
                Self::bisect_long_horizontal_rooms(&mut map_region, config.bisect_room_prob);

                map_region
            })
            .collect::<Vec<_>>();

        let map_regions_time = std::time::Instant::now();
        event!(
            tracing::Level::DEBUG,
            "Generated map regions in {:.2}ms",
            map_regions_time
                .duration_since(rect_groups_time)
                .as_millis()
        );

        let generated_maps = if config.merge_regions {
            let origin_rect = Rect::new(0, 0, self.cols, self.rows);

            let mut map_region = Self::merge_regions(origin_rect, map_regions);

            // We connect the rooms of the newly merged region together
            Self::reconnect_room_groups(&mut map_region, config);

            let doors: Vec<crate::types::Door> = Self::generate_doors_for(&map_region, config);

            room_decorator::RoomDecoratorFactory::decorator_for(style).decorate(
                &mut map_region,
                &doors,
                config,
            );

            vec![map_region.into_map(doors)]
        } else {
            let mut maps = map_regions
                .into_iter()
                .map(|mut map_region| {
                    let doors = Self::generate_doors_for(&map_region, config);

                    room_decorator::RoomDecoratorFactory::decorator_for(style).decorate(
                        &mut map_region,
                        &doors,
                        config,
                    );

                    map_region.into_map(doors)
                })
                .collect::<Vec<_>>();

            Self::connect_regions(&mut maps);

            maps
        };

        let generated_maps_time = std::time::Instant::now();
        event!(
            tracing::Level::DEBUG,
            "Added doors and modifiers in {:.2}ms",
            generated_maps_time
                .duration_since(map_regions_time)
                .as_millis()
        );

        let built_rooms = generated_maps
            .iter()
            .fold(0_usize, |acc, map| acc + map.rooms.len());

        let built_doors = generated_maps
            .iter()
            .fold(0_usize, |acc, map| acc + map.doors.len());
        event!(
            tracing::Level::DEBUG,
            "Built {} map/s with {} rooms and {} doors in {:.2}ms total",
            generated_maps.len(),
            built_rooms,
            built_doors,
            generated_maps_time.duration_since(build_start).as_millis()
        );

        generated_maps
    }
}
