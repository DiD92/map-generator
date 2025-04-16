use crate::constants::DIRECTIONS;

use std::{
    collections::HashMap,
    collections::HashSet,
    fmt::{Display, Formatter},
    hash::Hash,
};

use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub const ZERO: Vector2 = Vector2 { x: 0.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Vector2 { x, y }
    }

    pub fn distance(&self, other: &Vector2) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

impl Display for Vector2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Cell {
    pub col: u32,
    pub row: u32,
}

impl Cell {
    pub const ZERO: Cell = Cell { col: 0, row: 0 };

    pub fn new(col: u32, row: u32) -> Self {
        Cell { col, row }
    }

    pub fn offset_by(&self, offset: u32) -> Cell {
        Cell {
            col: self.col + offset,
            row: self.row + offset,
        }
    }

    pub fn stretched_by(&self, factor: u32) -> Cell {
        Cell {
            col: self.col * factor,
            row: self.row * factor,
        }
    }

    pub fn get_vertices(&self) -> Vec<Cell> {
        vec![
            Cell::new(self.col, self.row),
            Cell::new(self.col, self.row + 1),
            Cell::new(self.col + 1, self.row + 1),
            Cell::new(self.col + 1, self.row),
        ]
    }

    pub fn get_edges(&self) -> Vec<Edge> {
        vec![
            // North
            Edge {
                from: *self,
                to: Cell::new(self.col + 1, self.row),
            },
            // East
            Edge {
                from: Cell::new(self.col + 1, self.row),
                to: Cell::new(self.col + 1, self.row + 1),
            },
            // South
            Edge {
                from: Cell::new(self.col + 1, self.row + 1),
                to: Cell::new(self.col, self.row + 1),
            },
            // West
            Edge {
                from: Cell::new(self.col, self.row + 1),
                to: *self,
            },
        ]
    }

    pub fn distance(&self, other: &Cell) -> u32 {
        ((self.col as i32 - other.col as i32).abs() + (self.row as i32 - other.row as i32).abs())
            as u32
    }

    pub fn is_neighbour_of(&self, other: &Cell) -> Option<Direction> {
        if self.row == other.row {
            if self.col == other.col + 1 {
                return Some(Direction::West);
            } else if other.col == 0 {
                return None;
            } else if self.col == other.col - 1 {
                return Some(Direction::East);
            }
        } else if self.col == other.col {
            if self.row == other.row + 1 {
                return Some(Direction::North);
            } else if other.row == 0 {
                return None;
            } else if self.row == other.row - 1 {
                return Some(Direction::South);
            }
        }

        None
    }

    pub fn neighbours(&self) -> Vec<Cell> {
        let mut neighbours = Vec::with_capacity(4);

        neighbours.push(Cell::new(self.col + 1, self.row));
        neighbours.push(Cell::new(self.col, self.row + 1));

        if self.col > 0 {
            neighbours.push(Cell::new(self.col - 1, self.row));
        }

        if self.row > 0 {
            neighbours.push(Cell::new(self.col, self.row - 1));
        }

        neighbours
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.col, self.row)
    }
}

impl From<Cell> for (u32, u32) {
    fn from(cell: Cell) -> Self {
        (cell.col, cell.row)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Edge {
    pub from: Cell,
    pub to: Cell,
}

impl Edge {
    pub fn new(from: Cell, to: Cell) -> Self {
        Edge { from, to }
    }

    pub fn intersects_with(&self, other: &Edge) -> bool {
        if self == other {
            return true;
        }

        if self.from.col == self.to.col
            && other.from.col == self.from.col
            && other.to.col == self.to.col
        {
            // Check for vertical intersection
            let from_range = self.from.row..=self.to.row;
            let other_range = other.from.row..=other.to.row;

            from_range.filter(|row| other_range.contains(row)).count() > 1
        } else if self.from.row == self.to.row
            && other.from.row == self.from.row
            && other.to.row == self.to.row
        {
            // Check for horizontal intersection
            let from_range = self.from.col..=self.to.col;
            let other_range = other.from.col..=other.to.col;

            from_range.filter(|col| other_range.contains(col)).count() > 1
        } else {
            false
        }
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        (self.from == other.from && self.to == other.to)
            || (self.from == other.to && self.to == other.from)
    }
}

impl Eq for Edge {}

impl Display for Edge {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -- {}", self.from, self.to)
    }
}

// Since we consider edges to be undirected, we need to implement a custom hash function
// to ensure that the hash is the same regardless of the order of the points
impl Hash for Edge {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if self.from <= self.to {
            self.from.hash(state);
            self.to.hash(state);
        } else {
            self.to.hash(state);
            self.from.hash(state);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RectModifier {
    Standard,
    PreferHorizontal,
    PreferVertical,
    Chaotic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RectRegion {
    pub rect: Rect,
    pub modifier: RectModifier,
}

impl Display for RectRegion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {:?}", self.rect, self.modifier)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rect {
    pub origin: Cell,
    pub width: u32,
    pub height: u32,
}

impl Display for Rect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({},{}) - [{}x{}]",
            self.origin.col, self.origin.row, self.width, self.height
        )
    }
}

impl Rect {
    pub fn try_split_at(self, axis: SplitAxis, at: u32) -> Result<(Rect, Rect)> {
        match axis {
            SplitAxis::Horizontal => {
                if self.height < 2 || at >= self.height || at == 0 {
                    return Err(anyhow::anyhow!("Invalid split {}-{}", self.height, at));
                }

                let up = Rect {
                    origin: self.origin,
                    width: self.width,
                    height: at,
                };

                let down = Rect {
                    origin: Cell {
                        col: self.origin.col,
                        row: self.origin.row + at,
                    },
                    width: self.width,
                    height: self.height - at,
                };

                Ok((up, down))
            }
            SplitAxis::Vertical => {
                if self.width < 2 || at >= self.width || at == 0 {
                    return Err(anyhow::anyhow!("Invalid split {}-{}", self.width, at));
                }

                let left = Rect {
                    origin: self.origin,
                    width: at,
                    height: self.height,
                };

                let right = Rect {
                    origin: Cell {
                        col: self.origin.col + at,
                        row: self.origin.row,
                    },
                    width: self.width - at,
                    height: self.height,
                };

                Ok((left, right))
            }
        }
    }

    pub fn is_neighbour_of(&self, other: &Rect) -> bool {
        if self == other {
            return false;
        }

        for direction in DIRECTIONS.iter() {
            let (self_edge, other_edge) = match direction {
                Direction::North => (self.get_edge(*direction), other.get_edge(Direction::South)),
                Direction::South => (self.get_edge(*direction), other.get_edge(Direction::North)),
                Direction::East => (self.get_edge(*direction), other.get_edge(Direction::West)),
                Direction::West => (self.get_edge(*direction), other.get_edge(Direction::East)),
            };

            if self_edge.intersects_with(&other_edge) {
                return true;
            }
        }

        false
    }

    pub fn get_cells(&self) -> Vec<Cell> {
        let mut cells = Vec::new();

        for row in self.origin.row..(self.origin.row + self.height) {
            for col in self.origin.col..(self.origin.col + self.width) {
                cells.push(Cell { col, row });
            }
        }

        cells
    }

    pub fn get_edge(&self, direction: Direction) -> Edge {
        match direction {
            Direction::North => Edge {
                from: Cell::new(self.origin.col, self.origin.row),
                to: Cell::new(self.origin.col + self.width, self.origin.row),
            },
            Direction::South => Edge {
                from: Cell::new(self.origin.col, self.origin.row + self.height),
                to: Cell::new(self.origin.col + self.width, self.origin.row + self.height),
            },
            Direction::West => Edge {
                from: Cell::new(self.origin.col, self.origin.row),
                to: Cell::new(self.origin.col, self.origin.row + self.height),
            },
            Direction::East => Edge {
                from: Cell::new(self.origin.col + self.width, self.origin.row),
                to: Cell::new(self.origin.col + self.width, self.origin.row + self.height),
            },
        }
    }

    pub fn area(&self) -> u32 {
        self.width * self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DoorModifier {
    Open,
    Secret,
    Locked,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Door {
    pub from: Cell,
    pub to: Cell,
    pub modifier: DoorModifier,
}

impl Door {
    pub fn new(from: Cell, to: Cell) -> Self {
        Door {
            from,
            to,
            modifier: DoorModifier::Open,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    pub fn is_horizontal(&self) -> bool {
        match self {
            Direction::North | Direction::South => false,
            Direction::East | Direction::West => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum RoomModifier {
    #[default]
    None,
    Connector,
    Navigation,
    Save,
    Item,
}

pub type RoomTable = HashMap<RoomId, Room>;
pub type NeighbourTable = HashMap<RoomId, HashSet<RoomId>>;

pub type RoomId = usize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Room {
    pub cells: Vec<Cell>,
    pub modifier: Option<RoomModifier>,
}

impl Room {
    pub fn new_from_rect(rect: Rect) -> Self {
        Self {
            cells: rect.get_cells(),
            modifier: None,
        }
    }

    pub fn is_neighbour_of(&self, other: &Room) -> Option<Vec<(Cell, Cell, Direction)>> {
        if self == other {
            return None;
        }

        let mut neighour_cells = Vec::new();

        for cell in self.cells.iter() {
            for other_cell in other.cells.iter() {
                if let Some(direction) = cell.is_neighbour_of(other_cell) {
                    neighour_cells.push((*cell, *other_cell, direction));
                }
            }
        }

        if neighour_cells.is_empty() {
            None
        } else {
            Some(neighour_cells)
        }
    }

    pub fn merged_with(self, other: Room) -> Self {
        let mut merged_cells = self.cells.clone().into_iter().collect::<HashSet<_>>();

        for cell in other.cells.iter() {
            merged_cells.insert(*cell);
        }

        Room {
            cells: merged_cells.into_iter().collect(),
            modifier: self.modifier,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Serialize, clap::ValueEnum)]
pub enum MapStyle {
    #[default]
    CastlevaniaSOTN,
    CastlevaniaAOS,
    CastlevaniaCOTN,
    CastlevaniaHOD,
    MetroidZM,
    MetroidFS,
    MetroidSP,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Map {
    pub rooms: Vec<Room>,
    pub doors: Vec<Door>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_try_split_horizontal() {
        let rect1 = Rect {
            origin: Cell { col: 0, row: 3 },
            width: 4,
            height: 5,
        };

        let split_result = rect1.try_split_at(SplitAxis::Horizontal, 3);
        assert!(split_result.is_ok());

        let (up, down) = split_result.unwrap();

        assert_eq!(up.origin, Cell { col: 0, row: 3 });
        assert_eq!(up.width, 4);
        assert_eq!(up.height, 3);

        assert_eq!(down.origin, Cell { col: 0, row: 6 });
        assert_eq!(down.width, 4);
        assert_eq!(down.height, 2);
    }

    #[test]
    fn test_try_split_vetical() {
        let rect2 = Rect {
            origin: Cell { col: 4, row: 13 },
            width: 17,
            height: 9,
        };

        let split_result = rect2.try_split_at(SplitAxis::Vertical, 5);
        assert!(split_result.is_ok());

        let (left, right) = split_result.unwrap();

        assert_eq!(left.origin, Cell { col: 4, row: 13 });
        assert_eq!(left.width, 5);
        assert_eq!(left.height, 9);

        assert_eq!(right.origin, Cell { col: 9, row: 13 });
        assert_eq!(right.width, 12);
        assert_eq!(right.height, 9);
    }

    #[test]
    fn test_try_split_fails_with_small_rect() {
        let rect1 = Rect {
            origin: Cell { col: 1, row: 3 },
            width: 2,
            height: 1,
        };

        let split_result = rect1.try_split_at(SplitAxis::Horizontal, 1);
        assert!(split_result.is_err());

        let rect2 = Rect {
            origin: Cell { col: 1, row: 3 },
            width: 1,
            height: 2,
        };

        let split_result = rect2.try_split_at(SplitAxis::Vertical, 1);
        assert!(split_result.is_err());
    }

    #[test]
    fn test_try_split_fails_with_invalid_index() {
        let rect1 = Rect {
            origin: Cell { col: 1, row: 3 },
            width: 5,
            height: 5,
        };

        let split_result = rect1.try_split_at(SplitAxis::Horizontal, 6);
        assert!(split_result.is_err());

        let split_result = rect1.try_split_at(SplitAxis::Vertical, 6);
        assert!(split_result.is_err());

        let split_result = rect1.try_split_at(SplitAxis::Horizontal, 0);
        assert!(split_result.is_err());

        let split_result = rect1.try_split_at(SplitAxis::Vertical, 0);
        assert!(split_result.is_err());
    }

    #[test]
    fn test_edge_intersects_with() {
        let edge_1 = Edge {
            from: Cell { col: 2, row: 2 },
            to: Cell { col: 4, row: 2 },
        };

        let edge_2 = Edge {
            from: Cell { col: 2, row: 2 },
            to: Cell { col: 3, row: 2 },
        };

        assert!(edge_1.intersects_with(&edge_2));
    }

    #[test]
    fn test_rect_is_neighour_of() {
        let rect_1 = Rect {
            origin: Cell { col: 3, row: 1 },
            width: 2,
            height: 1,
        };

        let rect_2 = Rect {
            origin: Cell { col: 2, row: 2 },
            width: 2,
            height: 1,
        };

        assert!(rect_1.is_neighbour_of(&rect_2));
    }

    #[test]
    fn test_room_is_neighbour_of() {
        let rect_1_1 = Rect {
            origin: Cell { col: 3, row: 1 },
            width: 2,
            height: 1,
        };
        let rect_1_2 = Rect {
            origin: Cell { col: 4, row: 0 },
            width: 1,
            height: 1,
        };

        let room_1 = Room {
            cells: rect_1_1
                .get_cells()
                .into_iter()
                .chain(rect_1_2.get_cells())
                .collect(),
            modifier: None,
        };

        let rect_2 = Rect {
            origin: Cell { col: 2, row: 2 },
            width: 2,
            height: 1,
        };

        let room_2 = Room {
            cells: rect_2.get_cells(),
            modifier: None,
        };

        assert!(room_1.is_neighbour_of(&room_2).is_some());
    }
}
