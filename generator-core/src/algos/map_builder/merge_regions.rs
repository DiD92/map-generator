use super::MapBuilder;
use crate::algos::MapBuilderConfig;
use crate::types::{MapRegion, Rect};

use rayon::iter::ParallelIterator;
use rayon::prelude::*;

impl MapBuilder {
    pub(super) fn merge_regions(
        origin_rect: Rect,
        map_regions: impl ParallelIterator<Item = MapRegion>,
        config: &MapBuilderConfig,
    ) -> MapRegion {
        let mut map_region = map_regions.reduce(
            || MapRegion::new(origin_rect),
            |mut acc, region| {
                acc.rooms.extend(region.rooms);
                acc.removed_rooms.extend(region.removed_rooms);
                acc.neighbours.extend(region.neighbours);

                acc
            },
        );

        let now = std::time::Instant::now();

        Self::reconnect_room_groups_for_merged_region(&mut map_region, config);

        let elapsed = now.elapsed();
        tracing::event!(
            tracing::Level::DEBUG,
            "Reconnected room groups in {:.2}ms",
            elapsed.as_millis()
        );

        let map_room_ids = map_region
            .rooms
            .iter()
            .chain(map_region.removed_rooms.iter())
            .collect::<Vec<_>>();

        let (tx, rc) = std::sync::mpsc::channel();

        map_room_ids
            .par_iter()
            .by_uniform_blocks(200)
            .for_each_with(tx, |tx, (room_id, room)| {
                for (other_id, other_room) in map_room_ids.iter() {
                    if room_id == other_id {
                        continue;
                    }

                    if room.is_neighbour_of(other_room).is_some() {
                        tx.send((**room_id, **other_id)).unwrap();

                        /*map_region
                            .neighbours
                            .get_mut(room_id)
                            .unwrap()
                            .insert(**other_id);
                        map_region
                            .neighbours
                            .get_mut(other_id)
                            .unwrap()
                            .insert(**room_id);*/
                    }
                }
            });

        while let Ok((room_id, other_id)) = rc.recv() {
            map_region
                .neighbours
                .get_mut(&room_id)
                .unwrap()
                .insert(other_id);
            map_region
                .neighbours
                .get_mut(&other_id)
                .unwrap()
                .insert(room_id);
        }

        /*for (room_id, room) in map_room_ids.iter() {
            for (other_id, other_room) in map_room_ids.iter() {
                if room_id == other_id {
                    continue;
                }

                if room.is_neighbour_of(other_room).is_some() {
                    map_region
                        .neighbours
                        .get_mut(room_id)
                        .unwrap()
                        .insert(**other_id);
                    map_region
                        .neighbours
                        .get_mut(other_id)
                        .unwrap()
                        .insert(**room_id);
                }
            }
        }*/

        map_region
    }
}
