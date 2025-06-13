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
pub(crate) struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub const ZERO: Vector2 = Vector2::new(0.0, 0.0);

    pub const fn new(x: f32, y: f32) -> Self {
        Vector2 { x, y }
    }

    // Computes the euclidean distance between two vectors
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
pub(crate) struct Cell {
    pub col: u32,
    pub row: u32,
}

impl Cell {
    pub const fn new(col: u32, row: u32) -> Self {
        Cell { col, row }
    }

    pub fn offset_by(&self, offset: u32) -> Cell {
        Cell {
            col: self.col + offset,
            row: self.row + offset,
        }
    }

    pub fn offset_by_two(&self, offset_col: u32, offset_row: u32) -> Cell {
        Cell {
            col: self.col + offset_col,
            row: self.row + offset_row,
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
pub(crate) struct Edge {
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
pub(crate) enum SplitAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum RectModifier {
    Standard,
    PreferHorizontal,
    PreferVertical,
    Chaotic,
}

impl Display for RectModifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RectModifier::Standard => write!(f, "standard"),
            RectModifier::PreferHorizontal => write!(f, "horizontal"),
            RectModifier::PreferVertical => write!(f, "vertical"),
            RectModifier::Chaotic => write!(f, "chaotic"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct RectRegion {
    pub rect: Rect,
    pub modifier: RectModifier,
}

impl Display for RectRegion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{} - {}]", self.rect, self.modifier)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MapRegion {
    pub origin_rect: Rect,
    pub rooms: RoomTable,
    pub removed_rooms: RoomTable,
    pub neighbours: NeighbourTable,
}

impl MapRegion {
    #[cfg(test)]
    pub fn new_test_region() -> MapRegion {
        /*
           Creates the following map:

           +---+---+---+---+---+---+
           | 0 | 1 | 2 | 3 | 4 | 5 |
           +---+---+   +---+   +---+
           | 6 | 7 |   |           |
           +---+---+---+---+---+---+
           |   8   |   9   | A | B |
           +---+---+---+---+   +---+
           | C |   D   | E |   | F |
           +---+---+---+---+---+---+

           * Active rooms groups:
              - [0, 1, 6]
              - [3, 4]
              - [C, D]
              - [F]

           * Removed rooms are:
              - [2, 5, 7, 8, 9, A, B, E]
        */

        let origin_rect = Rect::new(0, 0, 6, 4);

        let mut rooms = HashMap::new();
        rooms.insert(0, Room::new_from_rect(Rect::new(0, 0, 1, 1)));
        rooms.insert(1, Room::new_from_rect(Rect::new(1, 0, 1, 1)));
        rooms.insert(6, Room::new_from_rect(Rect::new(0, 1, 1, 1)));

        rooms.insert(3, Room::new_from_rect(Rect::new(3, 0, 1, 1)));
        let room_4_a = Room::new_from_rect(Rect::new(4, 0, 1, 1));
        let room_4_b = Room::new_from_rect(Rect::new(3, 1, 3, 1));
        rooms.insert(4, room_4_a.merged_with(room_4_b));

        rooms.insert(12, Room::new_from_rect(Rect::new(0, 3, 1, 1)));
        rooms.insert(13, Room::new_from_rect(Rect::new(1, 3, 2, 1)));

        rooms.insert(15, Room::new_from_rect(Rect::new(5, 3, 1, 1)));

        let mut removed_rooms = HashMap::new();
        removed_rooms.insert(2, Room::new_from_rect(Rect::new(2, 0, 1, 2)));
        removed_rooms.insert(5, Room::new_from_rect(Rect::new(5, 0, 1, 1)));
        removed_rooms.insert(7, Room::new_from_rect(Rect::new(1, 1, 1, 1)));
        removed_rooms.insert(8, Room::new_from_rect(Rect::new(0, 2, 2, 1)));
        removed_rooms.insert(9, Room::new_from_rect(Rect::new(2, 2, 2, 1)));
        removed_rooms.insert(10, Room::new_from_rect(Rect::new(4, 2, 1, 2)));
        removed_rooms.insert(11, Room::new_from_rect(Rect::new(5, 2, 1, 1)));
        removed_rooms.insert(14, Room::new_from_rect(Rect::new(3, 3, 1, 1)));

        let mut neighbours = HashMap::new();
        neighbours.insert(0, HashSet::from([1, 6]));
        neighbours.insert(1, HashSet::from([0, 2, 7]));
        neighbours.insert(2, HashSet::from([1, 3, 4, 7, 9]));
        neighbours.insert(3, HashSet::from([2, 4]));
        neighbours.insert(4, HashSet::from([2, 3, 5, 9, 10, 11]));
        neighbours.insert(5, HashSet::from([4]));
        neighbours.insert(6, HashSet::from([0, 7, 8]));
        neighbours.insert(7, HashSet::from([1, 2, 6, 8]));
        neighbours.insert(8, HashSet::from([6, 7, 9, 12, 13]));
        neighbours.insert(9, HashSet::from([2, 4, 8, 10, 13, 14]));
        neighbours.insert(10, HashSet::from([4, 9, 11, 14, 15]));
        neighbours.insert(11, HashSet::from([4, 10, 15]));
        neighbours.insert(12, HashSet::from([8, 13]));
        neighbours.insert(13, HashSet::from([8, 9, 12, 14]));
        neighbours.insert(14, HashSet::from([9, 10, 13]));
        neighbours.insert(15, HashSet::from([10, 11]));

        MapRegion {
            origin_rect,
            rooms,
            removed_rooms,
            neighbours,
        }
    }

    pub fn new(origin_rect: Rect) -> Self {
        MapRegion {
            origin_rect,
            rooms: RoomTable::new(),
            removed_rooms: RoomTable::new(),
            neighbours: NeighbourTable::new(),
        }
    }

    pub fn offset_room_indexes(&mut self, offset: usize) {
        let mut new_rooms = RoomTable::new();
        for (id, room) in self.rooms.drain() {
            new_rooms.insert(id + offset, room);
        }
        self.rooms = new_rooms;

        let mut new_removed_rooms = RoomTable::new();
        for (id, room) in self.removed_rooms.drain() {
            new_removed_rooms.insert(id + offset, room);
        }
        self.removed_rooms = new_removed_rooms;

        let mut new_neighbours = NeighbourTable::new();
        for (id, mut neighbour_set) in self.neighbours.drain() {
            let new_neighbour_set: HashSet<RoomId> = neighbour_set
                .drain()
                .map(|neighbour_id| neighbour_id + offset)
                .collect();
            new_neighbours.insert(id + offset, new_neighbour_set);
        }
        self.neighbours = new_neighbours;
    }

    pub fn try_merge_rooms(&mut self, room_id_a: RoomId, room_id_b: RoomId) -> Result<()> {
        let from_room = self.rooms.remove(&room_id_a).unwrap();
        let to_room = self.rooms.remove(&room_id_b).unwrap();

        let merged_room = from_room.merged_with(to_room);
        self.rooms.insert(room_id_a, merged_room);

        // We extract the neighbours of the `from` room
        // and remove the `to` room from its neighbours
        let mut from_neighbours = self.neighbours.remove(&room_id_a).unwrap();
        from_neighbours.remove(&room_id_b);

        // We extract the neighbours of the `to` room
        // and remove the `from` room from its neighbours
        let mut to_neighbours = self.neighbours.remove(&room_id_b).unwrap();
        to_neighbours.remove(&room_id_a);

        // We merge the neighbours of both rooms into the `from` room
        from_neighbours.extend(to_neighbours);

        // We update the neighbours of the `to` room to point to the `from` room
        for neighbour in from_neighbours.iter() {
            if let Some(neighbours) = self.neighbours.get_mut(neighbour) {
                neighbours.remove(&room_id_b);
                neighbours.insert(room_id_a);
            }
        }

        // We insert the merged room back into the neighbours map
        self.neighbours.insert(room_id_a, from_neighbours);

        Ok(())
    }

    // Removes all rooms that have been marked as removed
    // and removes their references from the neighbours table.
    pub fn clear_removed_rooms(&mut self) {
        for (room_id, _) in self.removed_rooms.drain() {
            for neighbour_id in self
                .neighbours
                .remove(&room_id)
                .expect("Should have neighbours")
            {
                if let Some(neighbours) = self.neighbours.get_mut(&neighbour_id) {
                    neighbours.remove(&room_id);
                }
            }
        }
    }
}

impl Display for MapRegion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {} - {}",
            self.origin_rect.origin,
            self.rooms.len(),
            self.removed_rooms.len()
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Rect {
    pub origin: Cell,
    pub width: u32,
    pub height: u32,
}

impl Display for Rect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({},{}):[{}x{}]",
            self.origin.col, self.origin.row, self.width, self.height
        )
    }
}

impl PartialOrd for Rect {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rect {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.origin.cmp(&other.origin)
    }
}

impl Rect {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Rect {
            origin: Cell { col: x, row: y },
            width,
            height,
        }
    }

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

    pub fn is_neighbour_of(&self, other: &Rect) -> Option<Direction> {
        if self == other {
            return None;
        }

        for direction in DIRECTIONS.iter() {
            let (self_edge, other_edge) = match direction {
                Direction::North => (self.get_edge(*direction), other.get_edge(Direction::South)),
                Direction::South => (self.get_edge(*direction), other.get_edge(Direction::North)),
                Direction::East => (self.get_edge(*direction), other.get_edge(Direction::West)),
                Direction::West => (self.get_edge(*direction), other.get_edge(Direction::East)),
            };

            if self_edge.intersects_with(&other_edge) {
                return Some(*direction);
            }
        }

        None
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
pub(crate) enum DoorModifier {
    Open,
    Secret,
    Locked,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Door {
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
pub(crate) enum Direction {
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

    pub fn reverse(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub(crate) enum RoomModifier {
    #[default]
    None,
    Connector,
    Navigation,
    Save,
    Item,
    RegionConnection(Direction),
}

pub(crate) type RoomTable = HashMap<RoomId, Room>;
pub(crate) type NeighbourTable = HashMap<RoomId, HashSet<RoomId>>;

pub(crate) type RoomId = usize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Room {
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

    pub fn get_center(&self) -> Vector2 {
        let mut center = Vector2::ZERO;

        let cell_count = self.cells.len() as f32;

        for cell in self.cells.iter() {
            center.x += cell.col as f32;
            center.y += cell.row as f32;
        }

        center.x /= cell_count;
        center.y /= cell_count;

        center
    }
}

#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
#[cfg_attr(feature = "style-ord-hash", derive(PartialOrd, Ord, Hash))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Serialize)]
pub enum MapStyle {
    #[default]
    CastlevaniaSOTN,
    CastlevaniaAOS,
    CastlevaniaCOTM,
    CastlevaniaHOD,
    MetroidZM,
    MetroidFS,
    MetroidSP,
}

#[cfg(feature = "style-try-from-str")]
impl MapStyle {
    pub fn try_from_str(style: &str) -> anyhow::Result<Self> {
        Ok(match style {
            "castlevania-sotn" => MapStyle::CastlevaniaSOTN,
            "castlevania-aos" => MapStyle::CastlevaniaAOS,
            "castlevania-cotm" => MapStyle::CastlevaniaCOTM,
            "castlevania-hod" => MapStyle::CastlevaniaHOD,
            "metroid-zm" => MapStyle::MetroidZM,
            "metroid-fs" => MapStyle::MetroidFS,
            "metroid-sp" => MapStyle::MetroidSP,
            _ => return Err(anyhow::anyhow!(r#"Unknown map style: "{}""#, style)),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Map {
    pub origin_rect: Rect,
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

        assert!(rect_1.is_neighbour_of(&rect_2).is_some());
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
