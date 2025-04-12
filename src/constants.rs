use crate::types::Direction;

pub const MIN_RECT_WIDTH: u32 = 4;
pub const MIN_RECT_HEIGHT: u32 = 4;

pub const RECT_SIZE_MULTIPLIER: u32 = 48;

pub const MAP_SIZE_MARGIN: u32 = 96;
pub const MAP_STROKE_WIDTH: u32 = 4;

pub const DIRECTIONS: [Direction; 4] = [
    Direction::North,
    Direction::South,
    Direction::East,
    Direction::West,
];
