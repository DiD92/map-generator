use std::collections::{HashMap, HashSet};

use svg::{
    Document,
    node::element::{Path, path::Data},
};

use super::*;
use crate::{
    algos::PolygonBuilder,
    constants::{MAP_SIZE_MARGIN, RECT_SIZE_MULTIPLIER},
    types::{Cell, Direction, Door, DoorModifier, Edge, Map, Room, RoomModifier},
};

const REGION_SEPRATION: u32 = RECT_SIZE_MULTIPLIER * 8;

#[derive(Debug, PartialEq)]
pub(super) enum MetroidMapDrawer {
    ZeroMission,
    Fusion,
    Super,
}

impl MapDrawer for MetroidMapDrawer {
    fn draw(&self, maps: Vec<Map>, config: &DrawConfig) -> svg::Document {
        for map in maps.iter() {
            println!("Map: {:?}", map.rooms.len());
        }

        let (region_matrix, offset_map) = Self::get_regions_matrix(&maps);
        println!("Region matrix: [{}x{}]", region_matrix.0, region_matrix.1);

        let document_width = (config.canvas_width * RECT_SIZE_MULTIPLIER)
            + REGION_SEPRATION
            + (REGION_SEPRATION * (region_matrix.0 - 1));

        let document_height = (config.canvas_height * RECT_SIZE_MULTIPLIER)
            + REGION_SEPRATION
            + (REGION_SEPRATION * (region_matrix.1 - 1));

        let (room_color, door_color, wall_color) = match self {
            MetroidMapDrawer::ZeroMission => (LIGHT_BLUE, LIGHT_BLUE, LIGHT_GRAY),
            MetroidMapDrawer::Fusion => (LIGHT_BLUE, LIGHT_BLUE, LIGHT_GRAY),
            MetroidMapDrawer::Super => (LIGHT_BLUE, LIGHT_BLUE, LIGHT_GRAY),
        };

        println!("Document size: [{}x{}]", document_width, document_height);
        let mut document = Document::new()
            .set("width", document_width)
            .set("height", document_height);

        for paths in maps.iter().map(|map| {
            let region_origin = map.origin_rect.origin;
            let (region_col_offset, region_row_offset) = offset_map[&region_origin];

            let col_offset = (region_col_offset * REGION_SEPRATION) + (REGION_SEPRATION / 2);
            let row_offset = (region_row_offset * REGION_SEPRATION) + (REGION_SEPRATION / 2);

            Self::draw_region(
                map, col_offset, row_offset, room_color, door_color, wall_color,
            )
        }) {
            for path in paths {
                document = document.add(path);
            }
        }

        document
    }
}

impl MetroidMapDrawer {
    fn get_regions_matrix(maps: &[Map]) -> ((u32, u32), HashMap<Cell, (u32, u32)>) {
        let mut col_set = HashSet::new();
        let mut row_set = HashSet::new();

        for map in maps.iter() {
            let origin = map.origin_rect.origin;

            col_set.insert(origin.col);
            row_set.insert(origin.row);
        }

        let cols = col_set.len() as u32;
        let rows = row_set.len() as u32;

        let col_map = {
            let mut col_vec = col_set.into_iter().collect::<Vec<_>>();
            col_vec.sort();
            col_vec
                .drain(..)
                .enumerate()
                .map(|(i, col)| (col, i as u32))
                .collect::<HashMap<_, _>>()
        };

        let row_map = {
            let mut row_vec = row_set.into_iter().collect::<Vec<_>>();
            row_vec.sort();
            row_vec
                .drain(..)
                .enumerate()
                .map(|(i, row)| (row, i as u32))
                .collect::<HashMap<_, _>>()
        };

        let mut region_matrix = HashMap::new();

        for map in maps.iter() {
            let origin = map.origin_rect.origin;

            let col = col_map[&origin.col];
            let row = row_map[&origin.row];

            region_matrix.insert(origin, (col, row));
        }

        ((cols, rows), region_matrix)
    }

    fn draw_region(
        map: &Map,
        col_offset: u32,
        row_offset: u32,
        room_color: &str,
        door_color: &str,
        wall_color: &str,
    ) -> Vec<Path> {
        let mut path_vec = map
            .rooms
            .iter()
            .map(|room| Self::draw_room(room, col_offset, row_offset, room_color, wall_color))
            .collect::<Vec<_>>();

        for door_path in map
            .doors
            .iter()
            .map(|door| Self::draw_door(door, col_offset, row_offset, door_color))
        {
            path_vec.push(door_path);
        }

        path_vec
    }

    fn draw_room(
        room: &Room,
        col_offset: u32,
        row_offset: u32,
        room_color: &str,
        wall_color: &str,
    ) -> Path {
        let (valid_vertices, valid_edges) = PolygonBuilder::build_for(room);

        let mut vertices_to_visit = valid_vertices.clone();
        let mut vertex_path = Vec::with_capacity(valid_edges.len());

        let mut vertex_stack = vec![*vertices_to_visit.iter().next().unwrap()];

        while let Some(vertex) = vertex_stack.pop() {
            vertices_to_visit.remove(&vertex);

            vertex_path.push(vertex);

            for other_vertex in vertices_to_visit.iter() {
                if vertex.distance(other_vertex) == 1 {
                    let edge = Edge::new(vertex, *other_vertex);
                    if valid_edges.contains(&edge) {
                        vertex_stack.push(*other_vertex);
                        break;
                    }
                }
            }
        }

        vertex_path = vertex_path
            .into_iter()
            .map(|point| {
                point
                    .stretched_by(RECT_SIZE_MULTIPLIER)
                    .offset_by_two(col_offset, row_offset)
            })
            .collect();

        let first_point = vertex_path.pop().unwrap();
        let mut data = Data::new();
        data = data.move_to::<(u32, u32)>(first_point.into());

        for point in vertex_path.into_iter() {
            data = data.line_to::<(u32, u32)>(point.into());
        }
        data = data.line_to::<(u32, u32)>(first_point.into());

        data = data.close();

        let room_color = match room.modifier {
            Some(RoomModifier::RegionConnection(Direction::North)) => RED,
            Some(RoomModifier::RegionConnection(Direction::South)) => LIME_GREEN,
            Some(RoomModifier::RegionConnection(Direction::East)) => YELLOW,
            Some(RoomModifier::RegionConnection(Direction::West)) => DARK_BLUE,
            _ => room_color,
        };

        Path::new()
            .set("fill", room_color)
            .set("stroke", wall_color)
            .set("stroke-width", STROKE_WIDTH)
            .set("d", data)
    }

    fn draw_door(door: &Door, col_offset: u32, row_offset: u32, door_color: &str) -> Path {
        let mut data = Data::new();

        let from = door
            .from
            .stretched_by(RECT_SIZE_MULTIPLIER)
            .offset_by_two(col_offset, row_offset);
        let to: Cell = door
            .to
            .stretched_by(RECT_SIZE_MULTIPLIER)
            .offset_by_two(col_offset, row_offset);

        let line_separation = match door.modifier {
            DoorModifier::Open => 16,
            DoorModifier::Secret => 16,
            DoorModifier::Locked => 16,
            DoorModifier::None => 16,
        };

        if from.col != to.col {
            // Veritical door

            if from.row == to.row {
                let x = if from.col > to.col { from.col } else { to.col };

                let from_y = from.row + line_separation;
                let to_y = from.row + RECT_SIZE_MULTIPLIER - line_separation;

                data = data.move_to::<(u32, u32)>((x, from_y));
                data = data.line_to::<(u32, u32)>((x, to_y));
            } else {
                println!("Door axis is not a straigt line!");
            }
        } else if from.row != to.row {
            if from.col == to.col {
                // Horizontal door
                let y = if from.row > to.row { from.row } else { to.row };

                let from_x = from.col + line_separation;
                let to_x = from.col + RECT_SIZE_MULTIPLIER - line_separation;

                data = data.move_to::<(u32, u32)>((from_x, y));
                data = data.line_to::<(u32, u32)>((to_x, y));
            } else {
                println!("Door axis is not a straigt line!");
            }
        } else {
            println!("Door axis is a point!");
        }

        data = data.close();

        Path::new()
            .set("stroke", door_color)
            .set("stroke-width", STROKE_WIDTH + 8)
            .set("d", data)
    }
}
