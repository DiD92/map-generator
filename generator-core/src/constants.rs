use crate::types::Direction;

pub(crate) const MIN_RECT_WIDTH: u32 = 4;
pub(crate) const MIN_RECT_HEIGHT: u32 = 4;

pub(crate) const RECT_SIZE_MULTIPLIER: u32 = 48;
pub(crate) const REGION_SPLIT_FACTOR: u32 = 684;

pub(crate) const MAP_SIZE_MARGIN: u32 = 96;

pub(crate) const DIRECTIONS: [Direction; 4] = [
    Direction::North,
    Direction::South,
    Direction::East,
    Direction::West,
];
