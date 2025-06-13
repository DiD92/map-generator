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

        // We generate index offsets so that when we merge the groups later,
        // we don't have to worry about colliding room IDs.
        let group_idx_offsets = Self::generate_group_offsets(&rect_groups);

        let rect_groups_time = std::time::Instant::now();
        event!(
            tracing::Level::DEBUG,
            "Generated rectangle groups in {:.2}ms",
            rect_groups_time.duration_since(build_start).as_millis()
        );

        let map_regions = rect_groups.into_par_iter().zip(group_idx_offsets).map(
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

                // Finally we bisect long horizontal rooms randomly
                //MapBuilder::bisect_long_horizontal_rooms(&mut map_region, config.bisect_room_prob);

                map_region
            },
        );

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

            let mut map_region = Self::merge_regions(origin_rect, map_regions, &config);

            let merge_time = std::time::Instant::now();
            event!(
                tracing::Level::DEBUG,
                "Merged map regions in {:.2}ms",
                merge_time.duration_since(map_regions_time).as_millis()
            );

            // We connect the rooms of the newly merged region together
            Self::reconnect_room_groups(&mut map_region, config);

            let merge_reconnect_time = std::time::Instant::now();
            event!(
                tracing::Level::DEBUG,
                "Reconnected rooms in {:.2}ms",
                merge_reconnect_time.duration_since(merge_time).as_millis()
            );

            // At this point we have no more use for the removed rooms
            // so we can clear them.
            map_region.clear_removed_rooms();

            let clear_removed_time = std::time::Instant::now();
            event!(
                tracing::Level::DEBUG,
                "Cleared removed rooms in {:.2}ms",
                clear_removed_time
                    .duration_since(merge_reconnect_time)
                    .as_millis()
            );

            let doors = Self::add_doors_to_rooms(&map_region, config);

            room_decorator::RoomDecoratorFactory::decorator_for(style).decorate(
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
                    // At this point we have no more used for the removed rooms
                    // so we can clear them.
                    map_region.clear_removed_rooms();

                    let doors = Self::add_doors_to_rooms(&map_region, config);

                    room_decorator::RoomDecoratorFactory::decorator_for(style).decorate(
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
            "Built {} map/s with {} rooms and {} doors",
            generated_maps.len(),
            built_rooms,
            built_doors
        );

        generated_maps
    }
}
