use crate::{MapStyle, constants::REGION_SPLIT_FACTOR};

#[derive(Debug, Clone, Copy)]
pub(crate) struct BinarySpacePartitioningConfig {
    pub region_split_factor: u32,
    // The proportion of regions that are going to be PreferHorizontal
    // over PreferVertical. The Standard and Chaotic modifiers are
    // excluded from this calculation. Since their proportions are fixed.
    // The value is between 0.0 and 1.0.
    pub horizontal_region_prob: f64,
    // The minimum area of a rectangle to be considered for splitting.
    pub rect_area_cutoff: u32,
    // The maximum area of a rectangle proportional to rect_area_cutoff
    // to be considered for skipping its splitting.
    pub big_rect_area_cutoff: u32,
    // The probability of leaving a big rectangle without further splitting.
    pub big_rect_survival_prob: f64,
    // The random probability of performing a horizontal split.
    pub horizontal_split_prob: f64,
    // The minimum height to width ratio at which we will always perform a
    // horizontal split.
    pub height_factor_cutoff: f32,
    // The minimum width to height ratio at which we will always perform a
    // vertical split.
    pub width_factor_cutoff: f32,
    // The probability of keeping the finaly splitted rectangle.
    pub rect_survival_prob: f64,
    // The probability of removing a highly connected rectangle.
    pub trim_highly_connected_rect_prob: f64,
    // The probability of removing a fully connected rectangle.
    pub trim_fully_connected_rect_prob: f64,
}

impl Default for BinarySpacePartitioningConfig {
    fn default() -> Self {
        BinarySpacePartitioningConfig {
            region_split_factor: REGION_SPLIT_FACTOR,
            horizontal_region_prob: 0.5,
            rect_area_cutoff: 2,
            big_rect_area_cutoff: 9,
            big_rect_survival_prob: 0.03,
            horizontal_split_prob: 0.6,
            height_factor_cutoff: 1.8,
            width_factor_cutoff: 2.7,
            rect_survival_prob: 0.43,
            trim_highly_connected_rect_prob: 0.4,
            trim_fully_connected_rect_prob: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MapBuilderConfig {
    pub bsp_config: BinarySpacePartitioningConfig,
    // Should we merge the regions after generating their rooms?
    pub merge_regions: bool,
    // The probability of randomly merging two rooms into one.
    pub random_room_merge_prob: f64,
    // Probability of having a group reconnect to two groups instead of one
    pub group_loop_connection_chance: f64,
    // Probability of opening a connection between rooms that will
    // cause a navigation loop in the map.
    pub door_loop_connection_chance: f64,
    pub repeat_small_room_merge_prob: f64,
    pub bisect_room_prob: f64,
}

impl Default for MapBuilderConfig {
    fn default() -> Self {
        MapBuilderConfig {
            bsp_config: BinarySpacePartitioningConfig::default(),
            merge_regions: true,
            random_room_merge_prob: 0.05,
            group_loop_connection_chance: 0.17,
            door_loop_connection_chance: 0.2,
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
                base.bsp_config.big_rect_survival_prob = 0.05;
                base.bsp_config.horizontal_split_prob = 0.85;
                base.bsp_config.height_factor_cutoff = 2.9;
                base.bsp_config.width_factor_cutoff = 2.6;
                base.bsp_config.rect_survival_prob = 0.13;
                base.bsp_config.trim_highly_connected_rect_prob = 0.8;
                base.bsp_config.trim_fully_connected_rect_prob = 0.9;

                base.random_room_merge_prob = 0.03;
                base.group_loop_connection_chance = 0.64;
                base.door_loop_connection_chance = 0.22;
                base.repeat_small_room_merge_prob = 0.51;
                base.bisect_room_prob = 0.17;
            }
            MapStyle::CastlevaniaAOS => {
                base.bsp_config.horizontal_region_prob = 0.0;
                base.bsp_config.big_rect_area_cutoff = 11;
                base.bsp_config.big_rect_survival_prob = 0.11;
                base.bsp_config.horizontal_split_prob = 0.82;
                base.bsp_config.height_factor_cutoff = 2.4;
                base.bsp_config.width_factor_cutoff = 2.6;
                base.bsp_config.rect_survival_prob = 0.49;
                base.bsp_config.trim_highly_connected_rect_prob = 0.77;
                base.bsp_config.trim_fully_connected_rect_prob = 0.85;

                base.random_room_merge_prob = 0.01;
                base.group_loop_connection_chance = 0.79;
                base.door_loop_connection_chance = 0.24;
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
                base.group_loop_connection_chance = 0.90;
                base.door_loop_connection_chance = 0.14;
                base.repeat_small_room_merge_prob = 0.85;
                base.bisect_room_prob = 0.29;
            }
            MapStyle::CastlevaniaHOD => {
                base.bsp_config.horizontal_region_prob = 0.75;
                base.bsp_config.big_rect_area_cutoff = 8;
                base.bsp_config.big_rect_survival_prob = 0.03;
                base.bsp_config.horizontal_split_prob = 0.85;
                base.bsp_config.height_factor_cutoff = 1.9;
                base.bsp_config.width_factor_cutoff = 1.6;
                base.bsp_config.rect_survival_prob = 0.40;
                base.bsp_config.trim_highly_connected_rect_prob = 0.8;
                base.bsp_config.trim_fully_connected_rect_prob = 0.9;

                base.random_room_merge_prob = 0.03;
                base.group_loop_connection_chance = 0.86;
                base.door_loop_connection_chance = 0.22;
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
                base.door_loop_connection_chance = 0.22;
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
                base.door_loop_connection_chance = 0.22;
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
                base.door_loop_connection_chance = 0.22;
                base.repeat_small_room_merge_prob = 0.51;
                base.bisect_room_prob = 0.17;
            }
        }

        base
    }
}
