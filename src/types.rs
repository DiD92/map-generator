use crate::algo::PolygonBuilder;
use crate::consants::{DIRECTIONS, MAP_SIZE_MARGIN, MAP_STROKE_WIDTH, RECT_SIZE_MULTIPLIER};

use std::hash::Hash;
use std::{
    collections::HashSet,
    fmt::{Display, Formatter},
};

use anyhow::Result;
use svg::{Document, node::element::Path, node::element::path::Data};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
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
pub struct Point {
    pub col: u32,
    pub row: u32,
}

impl Point {
    pub const ZERO: Point = Point { col: 0, row: 0 };

    pub fn new(col: u32, row: u32) -> Self {
        Point { col, row }
    }

    pub fn offset_by(&self, offset: u32) -> Point {
        Point {
            col: self.col + offset,
            row: self.row + offset,
        }
    }

    pub fn stretched_by(&self, factor: u32) -> Point {
        Point {
            col: self.col * factor,
            row: self.row * factor,
        }
    }

    pub fn distance(&self, other: &Point) -> u32 {
        ((self.col as i32 - other.col as i32).abs() + (self.row as i32 - other.row as i32).abs())
            as u32
    }

    pub fn is_neighbour_of(&self, other: &Point) -> Option<Direction> {
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

    pub fn neighbours(&self) -> Vec<Point> {
        let mut neighbours = Vec::with_capacity(4);

        neighbours.push(Point::new(self.col + 1, self.row));
        neighbours.push(Point::new(self.col, self.row + 1));

        if self.col > 0 {
            neighbours.push(Point::new(self.col - 1, self.row));
        }

        if self.row > 0 {
            neighbours.push(Point::new(self.col, self.row - 1));
        }

        neighbours
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.col, self.row)
    }
}

impl Into<(u32, u32)> for Point {
    fn into(self) -> (u32, u32) {
        (self.col, self.row)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Edge {
    pub from: Point,
    pub to: Point,
}

impl Edge {
    pub fn new(from: Point, to: Point) -> Self {
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

            from_range.filter(|row| other_range.contains(&row)).count() > 1
        } else if self.from.row == self.to.row
            && other.from.row == self.from.row
            && other.to.row == self.to.row
        {
            // Check for horizontal intersection
            let from_range = self.from.col..=self.to.col;
            let other_range = other.from.col..=other.to.col;

            from_range.filter(|col| other_range.contains(&col)).count() > 1
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
pub struct Rect {
    pub origin: Point,
    pub width: u32,
    pub height: u32,
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
                    origin: Point {
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
                    origin: Point {
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

    pub fn puncture(&self, point: Point) -> Vec<Rect> {
        if point.col >= self.origin.col
            && point.col < self.origin.col + self.width
            && point.row >= self.origin.row
            && point.row < self.origin.row + self.height
        {
            let mut rects = Vec::new();

            for i in self.origin.col..(self.origin.col + self.width) {
                if point.col == i {
                    if point.row == self.origin.row {
                        let new_rect = Rect {
                            origin: Point {
                                col: i,
                                row: self.origin.row + 1,
                            },
                            width: 1,
                            height: self.height - 1,
                        };

                        if new_rect.height > 0 {
                            rects.push(new_rect);
                        }
                    } else if point.row == self.origin.row + self.height {
                        rects.push(Rect {
                            origin: Point {
                                col: i,
                                row: self.origin.row,
                            },
                            width: 1,
                            height: self.height - 1,
                        });
                    } else {
                        rects.push(Rect {
                            origin: Point {
                                col: i,
                                row: self.origin.row,
                            },
                            width: 1,
                            height: point.row - self.origin.row,
                        });

                        rects.push(Rect {
                            origin: Point {
                                col: i,
                                row: point.row + 1,
                            },
                            width: 1,
                            height: self.origin.row + self.height - point.row - 1,
                        });
                    }
                } else {
                    rects.push(Rect {
                        origin: Point {
                            col: i,
                            row: self.origin.row,
                        },
                        width: 1,
                        height: self.height,
                    });
                }
            }

            rects
        } else {
            return vec![*self];
        }
    }

    pub fn into_cells(&self) -> Vec<Point> {
        let mut cells = Vec::new();

        for row in self.origin.row..(self.origin.row + self.height) {
            for col in self.origin.col..(self.origin.col + self.width) {
                cells.push(Point { col, row });
            }
        }

        cells
    }

    pub fn into_points(&self) -> Vec<Point> {
        let mut points = Vec::new();

        for row in self.origin.row..=(self.origin.row + self.height) {
            for col in self.origin.col..=(self.origin.col + self.width) {
                points.push(Point { col, row });
            }
        }

        points
    }

    pub fn get_edge(&self, direction: Direction) -> Edge {
        match direction {
            Direction::North => Edge {
                from: Point::new(self.origin.col, self.origin.row),
                to: Point::new(self.origin.col + self.width, self.origin.row),
            },
            Direction::South => Edge {
                from: Point::new(self.origin.col, self.origin.row + self.height),
                to: Point::new(self.origin.col + self.width, self.origin.row + self.height),
            },
            Direction::West => Edge {
                from: Point::new(self.origin.col, self.origin.row),
                to: Point::new(self.origin.col, self.origin.row + self.height),
            },
            Direction::East => Edge {
                from: Point::new(self.origin.col + self.width, self.origin.row),
                to: Point::new(self.origin.col + self.width, self.origin.row + self.height),
            },
        }
    }

    pub fn into_edges(&self) -> Vec<Edge> {
        let mut edges = Vec::new();

        // North edges
        for col in self.origin.col..(self.origin.col + self.width) {
            edges.push(Edge {
                from: Point::new(col, self.origin.row),
                to: Point::new(col + 1, self.origin.row),
            });
        }

        // South edges
        for col in self.origin.col..(self.origin.col + self.width) {
            edges.push(Edge {
                from: Point::new(col, self.origin.row + self.height),
                to: Point::new(col + 1, self.origin.row + self.height),
            });
        }

        // West edges
        for row in self.origin.row..(self.origin.row + self.height) {
            edges.push(Edge {
                from: Point::new(self.origin.col, row),
                to: Point::new(self.origin.col, row + 1),
            });
        }

        // East edges
        for row in self.origin.row..(self.origin.row + self.height) {
            edges.push(Edge {
                from: Point::new(self.origin.col + self.width, row),
                to: Point::new(self.origin.col + self.width, row + 1),
            });
        }

        edges
    }

    pub fn area(&self) -> u32 {
        self.width * self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Door {
    pub at: Point,
    pub direction: Direction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum RoomColor {
    #[default]
    Purple,
    Red,
    Green,
    Blue,
    Yellow,
}

impl RoomColor {
    pub fn to_string(&self) -> &'static str {
        match self {
            RoomColor::Purple => "purple",
            RoomColor::Red => "red",
            RoomColor::Green => "green",
            RoomColor::Blue => "blue",
            RoomColor::Yellow => "yellow",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum RoomModifier {
    #[default]
    None,
    Secret,
    Save,
    Item,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Room {
    pub rects: Vec<Rect>,
    pub modifier: RoomModifier,
    pub color: RoomColor,
}

impl Room {
    pub fn new(rects: Vec<Rect>) -> Self {
        Self {
            rects,
            modifier: RoomModifier::default(),
            color: RoomColor::default(),
        }
    }

    pub fn new_with_modifier(rects: Vec<Rect>, modifier: RoomModifier) -> Self {
        Self {
            rects,
            modifier,
            color: RoomColor::default(),
        }
    }

    pub fn into_cells(&self) -> Vec<Point> {
        self.rects.iter().fold(Vec::new(), |mut cells, rect| {
            cells.extend(rect.into_cells());
            cells
        })
    }

    pub fn into_points(&self) -> Vec<Point> {
        self.rects.iter().fold(Vec::new(), |mut points, rect| {
            points.extend(rect.into_points());
            points
        })
    }

    pub fn into_svg(&self) -> Path {
        let (valid_points, valid_edges) = PolygonBuilder::build_for(self);

        let mut points_to_visit = valid_points.clone().into_iter().collect::<Vec<_>>();
        let mut point_path = Vec::with_capacity(valid_edges.len());
        let mut visited_set = HashSet::new();

        while let Some(point) = points_to_visit.pop() {
            if visited_set.contains(&point) {
                continue;
            }

            visited_set.insert(point);

            point_path.push(point);

            for neighbour in point.neighbours() {
                let edge = Edge::new(point, neighbour);
                if valid_edges.contains(&edge) && !visited_set.contains(&neighbour) {
                    points_to_visit.push(neighbour);
                    break;
                }
            }
        }

        point_path = point_path
            .into_iter()
            .map(|point| {
                point
                    .stretched_by(RECT_SIZE_MULTIPLIER)
                    .offset_by(MAP_SIZE_MARGIN / 2)
            })
            .collect();

        let first_point = point_path.pop().unwrap();

        let mut data = Data::new();
        data = data.move_to::<(u32, u32)>(first_point.into());
        for point in point_path.into_iter() {
            data = data.line_to::<(u32, u32)>(point.into());
        }
        //data = data.line_to::<(u32, u32)>(first_point.into());
        data = data.close();

        let path = Path::new()
            .set("fill", self.color.to_string())
            .set("stroke", "white")
            .set("stroke-width", MAP_STROKE_WIDTH)
            .set("d", data);

        path
    }

    pub fn is_neighbour_of(&self, other: &Room) -> bool {
        if self == other {
            return false;
        }

        for rect in self.rects.iter() {
            for other_rect in other.rects.iter() {
                if rect.is_neighbour_of(other_rect) {
                    return true;
                }
            }
        }

        false
    }

    pub fn merged_with(self, other: Room) -> Self {
        let mut merged_rects = self.rects.clone();
        merged_rects.extend(other.rects);

        Room::new(merged_rects)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Map {
    pub rooms: Vec<Room>,
    pub doors: Vec<Door>,
}

impl Map {
    pub fn new() -> Self {
        Map {
            rooms: Vec::new(),
            doors: Vec::new(),
        }
    }

    pub fn add_room(&mut self, room: Room) {
        self.rooms.push(room);
    }

    pub fn add_door(&mut self, door: Door) {
        self.doors.push(door);
    }

    pub fn into_svg(&self, width: u32, height: u32) -> Document {
        let mut document = Document::new()
            .set("width", (width * RECT_SIZE_MULTIPLIER) + MAP_SIZE_MARGIN)
            .set("height", (height * RECT_SIZE_MULTIPLIER) + MAP_SIZE_MARGIN)
            .set("stroke", "white")
            .set("stroke-width", "1");

        for room in self.rooms.iter() {
            let rect = room.into_svg();
            document = document.add(rect);
        }

        document
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_try_split_horizontal() {
        let rect1 = Rect {
            origin: Point { col: 0, row: 3 },
            width: 4,
            height: 5,
        };

        let split_result = rect1.try_split_at(SplitAxis::Horizontal, 3);
        assert!(split_result.is_ok());

        let (up, down) = split_result.unwrap();

        assert_eq!(up.origin, Point { col: 0, row: 3 });
        assert_eq!(up.width, 4);
        assert_eq!(up.height, 3);

        assert_eq!(down.origin, Point { col: 0, row: 6 });
        assert_eq!(down.width, 4);
        assert_eq!(down.height, 2);
    }

    #[test]
    fn test_try_split_vetical() {
        let rect2 = Rect {
            origin: Point { col: 4, row: 13 },
            width: 17,
            height: 9,
        };

        let split_result = rect2.try_split_at(SplitAxis::Vertical, 5);
        assert!(split_result.is_ok());

        let (left, right) = split_result.unwrap();

        assert_eq!(left.origin, Point { col: 4, row: 13 });
        assert_eq!(left.width, 5);
        assert_eq!(left.height, 9);

        assert_eq!(right.origin, Point { col: 9, row: 13 });
        assert_eq!(right.width, 12);
        assert_eq!(right.height, 9);
    }

    #[test]
    fn test_try_split_fails_with_small_rect() {
        let rect1 = Rect {
            origin: Point { col: 1, row: 3 },
            width: 2,
            height: 1,
        };

        let split_result = rect1.try_split_at(SplitAxis::Horizontal, 1);
        assert!(split_result.is_err());

        let rect2 = Rect {
            origin: Point { col: 1, row: 3 },
            width: 1,
            height: 2,
        };

        let split_result = rect2.try_split_at(SplitAxis::Vertical, 1);
        assert!(split_result.is_err());
    }

    #[test]
    fn test_try_split_fails_with_invalid_index() {
        let rect1 = Rect {
            origin: Point { col: 1, row: 3 },
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
            from: Point { col: 3, row: 2 },
            to: Point { col: 4, row: 2 },
        };

        let edge_2 = Edge {
            from: Point { col: 2, row: 2 },
            to: Point { col: 3, row: 2 },
        };

        assert!(edge_1.intersects_with(&edge_2));
    }

    #[test]
    fn test_rect_is_neighour_of() {
        let rect_1 = Rect {
            origin: Point { col: 3, row: 1 },
            width: 2,
            height: 1,
        };

        let rect_2 = Rect {
            origin: Point { col: 2, row: 2 },
            width: 2,
            height: 1,
        };

        assert!(rect_1.is_neighbour_of(&rect_2));
    }

    #[test]
    fn test_room_is_neighbour_of() {
        let room_1 = Room {
            rects: vec![
                Rect {
                    origin: Point { col: 3, row: 1 },
                    width: 2,
                    height: 1,
                },
                Rect {
                    origin: Point { col: 4, row: 0 },
                    width: 1,
                    height: 1,
                },
            ],
            modifier: RoomModifier::None,
            color: RoomColor::Purple,
        };

        let room_2 = Room {
            rects: vec![Rect {
                origin: Point { col: 2, row: 2 },
                width: 2,
                height: 1,
            }],
            modifier: RoomModifier::None,
            color: RoomColor::Purple,
        };

        assert!(room_1.is_neighbour_of(&room_2));
    }
}
