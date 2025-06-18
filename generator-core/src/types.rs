use crate::constants::DIRECTIONS;

use std::{
    collections::{HashMap, HashSet},
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

    // Same as `Self::distance`, but return the distance multiplied by
    // 100 and truncated to its integer part.
    pub fn scalar_distance(&self, other: &Vector2) -> u32 {
        (self.distance(other) * 100.0).trunc() as u32
    }

    pub fn divide_by(&self, divisor: f32) -> Vector2 {
        if divisor == 0.0 {
            panic!("Cannot divide by zero");
        }
        Vector2 {
            x: self.x / divisor,
            y: self.y / divisor,
        }
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
enum RoomEntry {
    #[default]
    Empty,
    Active(Room),
    Removed(Room),
}

impl RoomEntry {
    pub fn is_active(&self) -> bool {
        matches!(self, RoomEntry::Active(_))
    }

    pub fn is_removed(&self) -> bool {
        matches!(self, RoomEntry::Removed(_))
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, RoomEntry::Empty)
    }

    pub fn take(self) -> Room {
        match self {
            RoomEntry::Active(room) | RoomEntry::Removed(room) => room,
            RoomEntry::Empty => panic!("Cannot take an empty room"),
        }
    }
}

pub(crate) type NeighbourSet = tinyset::SetUsize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MapRegion {
    pub origin_rect: Rect,
    room_buffer: Vec<RoomEntry>,
    neighbour_buffer: Vec<Option<NeighbourSet>>,
}

#[allow(dead_code)]
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
        neighbours.insert(0, NeighbourSet::from_iter([1, 6]));
        neighbours.insert(1, NeighbourSet::from_iter([0, 2, 7]));
        neighbours.insert(2, NeighbourSet::from_iter([1, 3, 4, 7, 9]));
        neighbours.insert(3, NeighbourSet::from_iter([2, 4]));
        neighbours.insert(4, NeighbourSet::from_iter([2, 3, 5, 9, 10, 11]));
        neighbours.insert(5, NeighbourSet::from_iter([4]));
        neighbours.insert(6, NeighbourSet::from_iter([0, 7, 8]));
        neighbours.insert(7, NeighbourSet::from_iter([1, 2, 6, 8]));
        neighbours.insert(8, NeighbourSet::from_iter([6, 7, 9, 12, 13]));
        neighbours.insert(9, NeighbourSet::from_iter([2, 4, 8, 10, 13, 14]));
        neighbours.insert(10, NeighbourSet::from_iter([4, 9, 11, 14, 15]));
        neighbours.insert(11, NeighbourSet::from_iter([4, 10, 15]));
        neighbours.insert(12, NeighbourSet::from_iter([8, 13]));
        neighbours.insert(13, NeighbourSet::from_iter([8, 9, 12, 14]));
        neighbours.insert(14, NeighbourSet::from_iter([9, 10, 13]));
        neighbours.insert(15, NeighbourSet::from_iter([10, 11]));

        MapRegion::new(origin_rect, rooms, removed_rooms, neighbours)
    }

    #[cfg(test)]
    pub fn new_test_small_region() -> Self {
        /*
           Creates the following map:

           +---+---+---+
           | 0 | 3 |   |
           +---+---+ 9 +
           |   5   |   |
           +---+---+---+
        */

        let origin_rect = Rect::new(0, 0, 3, 2);

        let mut rooms = HashMap::new();
        let room_0 = Room::new_from_rect(Rect::new(0, 0, 1, 1));
        rooms.insert(0, room_0.clone());
        let room_3 = Room::new_from_rect(Rect::new(1, 0, 1, 1));
        rooms.insert(3, room_3.clone());

        let mut removed_rooms = HashMap::new();
        let rect_5 = Room::new_from_rect(Rect::new(0, 1, 2, 1));
        removed_rooms.insert(5, rect_5.clone());
        let rect_9 = Room::new_from_rect(Rect::new(2, 0, 1, 2));
        removed_rooms.insert(9, rect_9.clone());

        let mut neighbours = HashMap::new();
        neighbours.insert(0, NeighbourSet::from_iter([3, 5]));
        neighbours.insert(3, NeighbourSet::from_iter([0, 5, 9]));
        neighbours.insert(5, NeighbourSet::from_iter([0, 3, 9]));
        neighbours.insert(9, NeighbourSet::from_iter([3, 5]));

        MapRegion::new(origin_rect, rooms, removed_rooms, neighbours)
    }

    pub fn new(
        origin_rect: Rect,
        mut rooms: RoomTable,
        mut removed: RoomTable,
        mut neighbours: NeighbourTable,
    ) -> Self {
        let min_idx = *rooms.keys().chain(removed.keys()).min().unwrap_or(&0);
        let max_idx = *rooms.keys().chain(removed.keys()).max().unwrap_or(&0);

        let mut room_buffer = vec![RoomEntry::Empty; (max_idx - min_idx) + 1];

        for (room_id, room) in rooms.drain() {
            room_buffer[room_id - min_idx] = RoomEntry::Active(room);
        }

        for (room_id, room) in removed.drain() {
            room_buffer[room_id - min_idx] = RoomEntry::Removed(room);
        }

        let mut neighbour_buffer = vec![None; (max_idx - min_idx) + 1];

        for (room_id, mut neighbour_set) in neighbours.drain() {
            let target_set = neighbour_set
                .drain()
                .map(|id| id - min_idx)
                .collect::<NeighbourSet>();
            let _ = neighbour_buffer[room_id - min_idx].replace(target_set);
        }

        MapRegion {
            origin_rect,
            room_buffer,
            neighbour_buffer,
        }
    }

    pub fn into_map(self, doors: Vec<Door>) -> Map {
        Map {
            origin_rect: self.origin_rect,
            rooms: self
                .room_buffer
                .into_iter()
                .filter_map(|entry| match entry {
                    RoomEntry::Active(room) => Some(room),
                    _ => None,
                })
                .collect(),
            doors,
        }
    }

    /// This function returns the number of room slots in the region.
    /// It does not count the number of active and removed rooms.
    pub fn room_slots(&self) -> usize {
        self.room_buffer.len()
    }

    pub fn is_active(&self, room_id: RoomId) -> bool {
        self.room_buffer[room_id].is_active()
    }

    pub fn is_removed(&self, room_id: RoomId) -> bool {
        self.room_buffer[room_id].is_removed()
    }

    pub fn is_empty(&self, room_id: RoomId) -> bool {
        self.room_buffer[room_id].is_empty()
    }

    // Gets the room with id `room_id`.
    // Panics if the id does not point to an active or removed room.
    pub fn get_room(&self, room_id: RoomId) -> &Room {
        match &self.room_buffer[room_id] {
            RoomEntry::Active(room) | RoomEntry::Removed(room) => room,
            RoomEntry::Empty => panic!("Room with ID {} is empty", room_id),
        }
    }

    pub fn get_mut_room(&mut self, room_id: RoomId) -> &mut Room {
        match &mut self.room_buffer[room_id] {
            RoomEntry::Active(room) | RoomEntry::Removed(room) => room,
            RoomEntry::Empty => panic!("Room with ID {} is empty", room_id),
        }
    }

    pub fn insert_room(&mut self, room: Room) -> RoomId {
        let room_id = self.room_buffer.len();
        self.room_buffer.push(RoomEntry::Active(room));
        self.neighbour_buffer.push(Some(NeighbourSet::new()));
        room_id
    }

    // Gets the active room with id `room_id`.
    // Panics if the id does not point to an active room.
    pub fn get_active(&self, room_id: RoomId) -> &Room {
        if let RoomEntry::Active(room) = &self.room_buffer[room_id] {
            room
        } else {
            panic!("Room with ID {} is not active", room_id);
        }
    }

    // Gets the removed room with id `room_id`.
    // Panics if the id does not point to a removed room.
    pub fn get_removed(&self, room_id: RoomId) -> &Room {
        if let RoomEntry::Removed(room) = &self.room_buffer[room_id] {
            room
        } else {
            panic!("Room with ID {} is not removed", room_id);
        }
    }

    pub fn mark_removed(&mut self, room_id: RoomId) {
        let room_entry = &mut self.room_buffer[room_id];

        if room_entry.is_active() {
            let room = std::mem::take(room_entry).take();
            *room_entry = RoomEntry::Removed(room);
        } else if room_entry.is_empty() {
            panic!("Room with ID {} is not valid!", room_id);
        }
    }

    pub fn mark_active(&mut self, room_id: RoomId) {
        let room_entry = &mut self.room_buffer[room_id];

        if room_entry.is_removed() {
            let room = std::mem::take(room_entry).take();
            *room_entry = RoomEntry::Active(room);
        } else if room_entry.is_empty() {
            panic!("Room with ID {} is not valid!", room_id);
        }
    }

    pub fn iter_rooms(&self) -> impl Iterator<Item = (usize, &Room)> {
        self.room_buffer
            .iter()
            .enumerate()
            .filter_map(|(idx, entry)| match entry {
                RoomEntry::Active(room) | RoomEntry::Removed(room) => Some((idx, room)),
                RoomEntry::Empty => None,
            })
    }

    pub fn iter_active(&self) -> impl Iterator<Item = (usize, &Room)> {
        self.room_buffer
            .iter()
            .enumerate()
            .filter_map(|(idx, entry)| {
                if let RoomEntry::Active(room) = entry {
                    Some((idx, room))
                } else {
                    None
                }
            })
    }

    pub fn iter_removed(&self) -> impl Iterator<Item = (usize, &Room)> {
        self.room_buffer
            .iter()
            .enumerate()
            .filter_map(|(idx, entry)| {
                if let RoomEntry::Removed(room) = entry {
                    Some((idx, room))
                } else {
                    None
                }
            })
    }

    pub fn take_room(&mut self, room_id: RoomId) -> Room {
        let room_entry = &mut self.room_buffer[room_id];

        if room_entry.is_empty() {
            panic!("Room with ID {} is not valid!", room_id);
        } else {
            std::mem::take(room_entry).take()
        }
    }

    pub fn take_active(&mut self, room_id: RoomId) -> Room {
        let room_entry = &mut self.room_buffer[room_id];

        if room_entry.is_active() {
            std::mem::take(room_entry).take()
        } else {
            panic!("Room with ID {} is not active!", room_id);
        }
    }

    pub fn take_removed(&mut self, room_id: RoomId) -> Room {
        let room_entry = &mut self.room_buffer[room_id];

        if room_entry.is_removed() {
            std::mem::take(room_entry).take()
        } else {
            panic!("Room with ID {} is not removed!", room_id);
        }
    }

    // Gets the neighbours of the room with id `room_id`.
    // Panics if the id does not point to a valid room.
    pub fn get_neighbours(&self, room_id: RoomId) -> &NeighbourSet {
        if self.neighbour_buffer[room_id].is_none() {
            panic!("Room with ID {} is not valid!", room_id);
        }

        self.neighbour_buffer[room_id].as_ref().unwrap()
    }

    pub fn get_mut_neighbours(&mut self, room_id: RoomId) -> &mut NeighbourSet {
        if self.neighbour_buffer[room_id].is_none() {
            panic!("Room with ID {} is not valid!", room_id);
        }

        self.neighbour_buffer[room_id].as_mut().unwrap()
    }

    pub fn iter_neighbours(&self, room_id: RoomId) -> impl Iterator<Item = RoomId> {
        self.neighbour_buffer[room_id].as_ref().unwrap().iter()
    }

    pub fn iter_active_neighbours(&self, room_id: RoomId) -> impl Iterator<Item = RoomId> {
        self.neighbour_buffer[room_id]
            .as_ref()
            .unwrap()
            .iter()
            .filter(|&neighbour_id| self.is_active(neighbour_id))
    }

    pub fn iter_removed_neighbours(&self, room_id: RoomId) -> impl Iterator<Item = RoomId> {
        self.neighbour_buffer[room_id]
            .as_ref()
            .unwrap()
            .iter()
            .filter(|&neighbour_id| self.is_removed(neighbour_id))
    }

    pub fn take_neighbours(&mut self, room_id: RoomId) -> NeighbourSet {
        if self.neighbour_buffer[room_id].is_none() {
            panic!("Room with ID {} is not valid!", room_id);
        }

        std::mem::take(&mut self.neighbour_buffer[room_id]).unwrap()
    }

    // Merges two active rooms into one.
    // Panics if either of the rooms does not exist.
    pub fn merge_active_rooms(&mut self, room_id_a: RoomId, room_id_b: RoomId) -> Result<()> {
        let from_room = self.take_active(room_id_a);
        let to_room = self.take_active(room_id_b);

        let merged_room = from_room.merged_with(to_room);
        let _ = std::mem::replace(
            &mut self.room_buffer[room_id_a],
            RoomEntry::Active(merged_room),
        );

        // We extract the neighbours of the `from` room
        // and remove the `to` room from its neighbours
        let mut from_neighbours = self.take_neighbours(room_id_a);
        from_neighbours.remove(room_id_b);

        // We extract the neighbours of the `to` room
        // and remove the `from` room from its neighbours
        let mut to_neighbours = self.take_neighbours(room_id_b);
        to_neighbours.remove(room_id_a);

        // We merge the neighbours of both rooms into the `from` room
        from_neighbours.extend(to_neighbours);

        // We update the neighbours of the `to` room to point to the `from` room
        for neighbour in from_neighbours.iter() {
            let neighbours = self.get_mut_neighbours(neighbour);
            neighbours.remove(room_id_b);
            neighbours.insert(room_id_a);
        }

        // We insert the merged room back into the neighbours map
        let _ = self.neighbour_buffer[room_id_a].replace(from_neighbours);

        Ok(())
    }

    // Compacts the buffers by removing empty entries
    // and shifting the indices of the remaining rooms.
    // WARNING: After this operation, the room IDs will change!
    pub fn compact_buffers(&mut self) {
        let mut empty_slots = self
            .room_buffer
            .iter()
            .enumerate()
            .filter_map(
                |(idx, entry)| {
                    if entry.is_empty() { Some(idx) } else { None }
                },
            )
            .rev()
            .collect::<Vec<_>>();

        let mut non_empty_slots = self
            .room_buffer
            .iter()
            .enumerate()
            .filter_map(
                |(idx, entry)| {
                    if !entry.is_empty() { Some(idx) } else { None }
                },
            )
            .collect::<Vec<_>>();

        let mut neigh_buff = Vec::with_capacity(3);

        while let Some(empty_idx) = empty_slots.pop() {
            if let Some(non_empty_idx) = non_empty_slots.pop() {
                if non_empty_idx <= empty_idx {
                    // If the non-empty index is less than or equal to the empty index,
                    // we already filled all the empty slots before this one.
                    // So we can stop here.
                    break;
                }

                // We swap the room indexes
                self.room_buffer.swap(empty_idx, non_empty_idx);
                // We also need to update the neighbours buffer
                self.neighbour_buffer.swap(empty_idx, non_empty_idx);

                neigh_buff.extend(self.iter_neighbours(empty_idx));

                for &neighbour_id in neigh_buff.iter() {
                    let neighbour_set = self.get_mut_neighbours(neighbour_id);
                    neighbour_set.remove(non_empty_idx);
                    neighbour_set.insert(empty_idx);
                }

                neigh_buff.clear();
            } else {
                break; // No more non-empty slots to fill
            }
        }

        let non_empty_count = self
            .room_buffer
            .iter()
            .filter(|entry| !entry.is_empty())
            .count();

        self.room_buffer.truncate(non_empty_count);
        self.room_buffer.shrink_to(non_empty_count);

        self.neighbour_buffer.truncate(non_empty_count);
        self.neighbour_buffer.shrink_to(non_empty_count);
    }

    /// Merges the current region with another region.
    /// This method will increase the indexes of the rooms in the other region
    /// by the number of rooms in the current region.
    /// So as to avoid index collisions.
    pub fn merge_with(&mut self, other: MapRegion) {
        let offset = self.room_buffer.len();

        // Merging the rooms is as simple as extending the room buffer
        // with the other region's room buffer
        self.room_buffer.extend(other.room_buffer);

        // First we apply the offset to the room IDs in the neighbour buffer
        let offset_neighbours = other
            .neighbour_buffer
            .into_iter()
            .map(|maybe_neighbour_set| {
                maybe_neighbour_set.map(|neighbour_set| {
                    neighbour_set
                        .into_iter()
                        .map(|id| id + offset)
                        .collect::<NeighbourSet>()
                })
            })
            .collect::<Vec<_>>();
        self.neighbour_buffer.extend(offset_neighbours);
    }

    pub fn shrink_buffers(&mut self) {
        self.room_buffer.shrink_to_fit();
        self.neighbour_buffer.shrink_to_fit();
    }
}

impl Display for MapRegion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {}",
            self.origin_rect.origin,
            self.room_buffer.len(),
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
    Navigation,
    Save,
    Item,
    RegionConnection(Direction),
}

pub(crate) type RoomTable = HashMap<RoomId, Room>;
pub(crate) type NeighbourTable = HashMap<RoomId, NeighbourSet>;

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

    pub fn is_neighbour_of(&self, other: &Room) -> bool {
        if self == other {
            return false;
        }

        for cell in self.cells.iter() {
            for other_cell in other.cells.iter() {
                if cell.is_neighbour_of(other_cell).is_some() {
                    return true;
                }
            }
        }

        false
    }

    pub fn get_neighbouring_cells_for(&self, other: &Room) -> Option<Vec<(Cell, Cell, Direction)>> {
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

        assert!(room_1.is_neighbour_of(&room_2));
    }

    #[test]
    fn test_map_region_compact_buffers() {
        let mut map_region = MapRegion::new_test_small_region();

        // Check the initial state of the buffers
        assert_eq!(map_region.room_buffer.len(), 10);
        assert_eq!(map_region.room_buffer.capacity(), 10);
        assert_eq!(map_region.neighbour_buffer.len(), 10);
        assert_eq!(map_region.neighbour_buffer.capacity(), 10);

        // Room instances
        let room_0 = Room::new_from_rect(Rect::new(0, 0, 1, 1));
        let room_3 = Room::new_from_rect(Rect::new(1, 0, 1, 1));
        let rect_5 = Room::new_from_rect(Rect::new(0, 1, 2, 1));
        let rect_9 = Room::new_from_rect(Rect::new(2, 0, 1, 2));

        // Check the inital contents of the buffers
        let expected_room_buffer = vec![
            RoomEntry::Active(room_0.clone()),
            RoomEntry::Empty,
            RoomEntry::Empty,
            RoomEntry::Active(room_3.clone()),
            RoomEntry::Empty,
            RoomEntry::Removed(rect_5.clone()),
            RoomEntry::Empty,
            RoomEntry::Empty,
            RoomEntry::Empty,
            RoomEntry::Removed(rect_9.clone()),
        ];
        assert_eq!(map_region.room_buffer, expected_room_buffer);

        let expected_neighbour_buffer = vec![
            Some(NeighbourSet::from_iter([3, 5])),
            None,
            None,
            Some(NeighbourSet::from_iter([0, 5, 9])),
            None,
            Some(NeighbourSet::from_iter([0, 3, 9])),
            None,
            None,
            None,
            Some(NeighbourSet::from_iter([3, 5])),
        ];
        assert_eq!(map_region.neighbour_buffer, expected_neighbour_buffer);

        // Compact the buffers
        map_region.compact_buffers();

        // Check that the buffers are compacted
        assert_eq!(map_region.room_buffer.len(), 4);
        assert_eq!(map_region.room_buffer.capacity(), 4);
        assert_eq!(map_region.neighbour_buffer.len(), 4);
        assert_eq!(map_region.neighbour_buffer.capacity(), 4);

        // Check the contents of the buffers after compaction
        let expected_room_buffer = vec![
            RoomEntry::Active(room_0),
            RoomEntry::Removed(rect_9),
            RoomEntry::Removed(rect_5),
            RoomEntry::Active(room_3),
        ];
        assert_eq!(map_region.room_buffer, expected_room_buffer);

        let expected_neighbour_buffer = vec![
            Some(NeighbourSet::from_iter([3, 2])),
            Some(NeighbourSet::from_iter([3, 2])),
            Some(NeighbourSet::from_iter([0, 3, 1])),
            Some(NeighbourSet::from_iter([0, 2, 1])),
        ];
        assert_eq!(map_region.neighbour_buffer, expected_neighbour_buffer);
    }

    #[test]
    fn test_map_region_merge_with() {
        let mut map_region_1 = MapRegion::new_test_small_region();
        let map_region_2 = MapRegion::new_test_small_region();

        // Checking so that the next checks are proven to
        // also apply to map_region_2
        assert_eq!(map_region_1, map_region_2);
        // Check the initial state of the buffers
        assert_eq!(map_region_1.room_buffer.len(), 10);
        assert_eq!(map_region_1.room_buffer.capacity(), 10);
        assert_eq!(map_region_1.neighbour_buffer.len(), 10);
        assert_eq!(map_region_1.neighbour_buffer.capacity(), 10);

        // Room instances
        let room_0 = Room::new_from_rect(Rect::new(0, 0, 1, 1));
        let room_3 = Room::new_from_rect(Rect::new(1, 0, 1, 1));
        let rect_5 = Room::new_from_rect(Rect::new(0, 1, 2, 1));
        let rect_9 = Room::new_from_rect(Rect::new(2, 0, 1, 2));

        let expected_room_buffer = vec![
            RoomEntry::Active(room_0.clone()),
            RoomEntry::Empty,
            RoomEntry::Empty,
            RoomEntry::Active(room_3.clone()),
            RoomEntry::Empty,
            RoomEntry::Removed(rect_5.clone()),
            RoomEntry::Empty,
            RoomEntry::Empty,
            RoomEntry::Empty,
            RoomEntry::Removed(rect_9.clone()),
        ];
        assert_eq!(map_region_1.room_buffer, expected_room_buffer);

        let expected_neighbour_buffer = vec![
            Some(NeighbourSet::from_iter([3, 5])),
            None,
            None,
            Some(NeighbourSet::from_iter([0, 5, 9])),
            None,
            Some(NeighbourSet::from_iter([0, 3, 9])),
            None,
            None,
            None,
            Some(NeighbourSet::from_iter([3, 5])),
        ];
        assert_eq!(map_region_1.neighbour_buffer, expected_neighbour_buffer);

        map_region_1.merge_with(map_region_2);

        // Check the state of the buffers after merging
        assert_eq!(map_region_1.room_buffer.len(), 20);
        assert_eq!(map_region_1.room_buffer.capacity(), 20);
        assert_eq!(map_region_1.neighbour_buffer.len(), 20);
        assert_eq!(map_region_1.neighbour_buffer.capacity(), 20);

        let expected_room_buffer = vec![
            RoomEntry::Active(room_0.clone()),
            RoomEntry::Empty,
            RoomEntry::Empty,
            RoomEntry::Active(room_3.clone()),
            RoomEntry::Empty,
            RoomEntry::Removed(rect_5.clone()),
            RoomEntry::Empty,
            RoomEntry::Empty,
            RoomEntry::Empty,
            RoomEntry::Removed(rect_9.clone()),
            RoomEntry::Active(room_0.clone()),
            RoomEntry::Empty,
            RoomEntry::Empty,
            RoomEntry::Active(room_3.clone()),
            RoomEntry::Empty,
            RoomEntry::Removed(rect_5.clone()),
            RoomEntry::Empty,
            RoomEntry::Empty,
            RoomEntry::Empty,
            RoomEntry::Removed(rect_9.clone()),
        ];
        assert_eq!(map_region_1.room_buffer, expected_room_buffer);

        let expected_neighbour_buffer = vec![
            Some(NeighbourSet::from_iter([3, 5])),
            None,
            None,
            Some(NeighbourSet::from_iter([0, 5, 9])),
            None,
            Some(NeighbourSet::from_iter([0, 3, 9])),
            None,
            None,
            None,
            Some(NeighbourSet::from_iter([3, 5])),
            Some(NeighbourSet::from_iter([13, 15])),
            None,
            None,
            Some(NeighbourSet::from_iter([10, 15, 19])),
            None,
            Some(NeighbourSet::from_iter([10, 13, 19])),
            None,
            None,
            None,
            Some(NeighbourSet::from_iter([13, 15])),
        ];
        assert_eq!(map_region_1.neighbour_buffer, expected_neighbour_buffer);

        println!("{:?}", map_region_1.iter_active().collect::<Vec<_>>());

        map_region_1.compact_buffers();

        // Check the state of the buffers after compaction
        assert_eq!(map_region_1.room_buffer.len(), 8);
        assert_eq!(map_region_1.room_buffer.capacity(), 8);
        assert_eq!(map_region_1.neighbour_buffer.len(), 8);
        assert_eq!(map_region_1.neighbour_buffer.capacity(), 8);

        // Check the inital contents of the buffers
        let expected_room_buffer = vec![
            RoomEntry::Active(room_0.clone()),
            RoomEntry::Removed(rect_9.clone()),
            RoomEntry::Removed(rect_5.clone()),
            RoomEntry::Active(room_3.clone()),
            RoomEntry::Active(room_3),
            RoomEntry::Removed(rect_5),
            RoomEntry::Active(room_0),
            RoomEntry::Removed(rect_9),
        ];
        assert_eq!(map_region_1.room_buffer, expected_room_buffer);

        let expected_neighbour_buffer = vec![
            Some(NeighbourSet::from_iter([3, 5])),
            Some(NeighbourSet::from_iter([4, 2])),
            Some(NeighbourSet::from_iter([6, 4, 1])),
            Some(NeighbourSet::from_iter([0, 5, 7])),
            Some(NeighbourSet::from_iter([6, 2, 1])),
            Some(NeighbourSet::from_iter([0, 3, 7])),
            Some(NeighbourSet::from_iter([4, 2])),
            Some(NeighbourSet::from_iter([3, 5])),
        ];
        assert_eq!(map_region_1.neighbour_buffer, expected_neighbour_buffer);
    }
}
