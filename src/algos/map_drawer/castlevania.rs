use svg::{
    Document,
    node::element::{Path, path::Data},
};

use super::{DrawConfig, MapDrawer};
use crate::{
    algos::PolygonBuilder,
    constants::{MAP_SIZE_MARGIN, RECT_SIZE_MULTIPLIER},
    types::{Cell, Door, DoorModifier, Edge, Map, Room, RoomModifier},
};

const LIGHT_BLUE: &str = "#0080ff";
const LIGHT_GRAY: &str = "#c0c0c0";
const RED: &str = "#ff0000";
const YELLOW: &str = "#ffff00";

const STROKE_WIDTH: u32 = 12;

pub(super) struct CastlevaniaMapDrawer;

impl MapDrawer for CastlevaniaMapDrawer {
    fn draw(&self, map: &Map, config: &DrawConfig) -> svg::Document {
        let mut document = Document::new()
            .set(
                "width",
                (config.canvas_width * RECT_SIZE_MULTIPLIER) + MAP_SIZE_MARGIN,
            )
            .set(
                "height",
                (config.canvas_height * RECT_SIZE_MULTIPLIER) + MAP_SIZE_MARGIN,
            );

        for room_path in map.rooms.iter().map(Self::draw_room) {
            document = document.add(room_path);
        }

        for door_path in map.doors.iter().map(Self::draw_door) {
            document = document.add(door_path);
        }

        document
    }
}

impl CastlevaniaMapDrawer {
    fn draw_room(room: &Room) -> Path {
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

        if vertex_path.len() % 2 != 0 {
            println!(
                "Vertex path is not even {} {}!",
                valid_vertices.len(),
                valid_edges.len()
            );
            for vertex in valid_vertices.iter() {
                print!("{}, ", vertex);
            }
            println!();
            for edge in valid_edges.iter() {
                print!("{}, ", edge);
            }
            println!();
            println!("{:?}", room.modifier);
            for cell in room.cells.iter() {
                print!("{}, ", cell);
            }
            println!();
        }

        vertex_path = vertex_path
            .into_iter()
            .map(|point| {
                point
                    .stretched_by(RECT_SIZE_MULTIPLIER)
                    .offset_by(MAP_SIZE_MARGIN / 2)
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

        Path::new()
            .set("fill", Self::room_color_for(&room.modifier))
            .set("stroke", LIGHT_GRAY)
            .set("stroke-width", STROKE_WIDTH)
            .set("d", data)
    }

    fn room_color_for(maybe_modifier: &Option<RoomModifier>) -> &'static str {
        if let Some(modifier) = maybe_modifier {
            match modifier {
                RoomModifier::None => LIGHT_BLUE,
                RoomModifier::Connector => LIGHT_BLUE,
                RoomModifier::Navigation => YELLOW,
                RoomModifier::Save => RED,
                RoomModifier::Item => LIGHT_BLUE,
            }
        } else {
            LIGHT_BLUE
        }
    }

    fn draw_door(door: &Door) -> Path {
        let mut data = Data::new();

        let from = door
            .from
            .stretched_by(RECT_SIZE_MULTIPLIER)
            .offset_by(MAP_SIZE_MARGIN / 2);
        let to: Cell = door
            .to
            .stretched_by(RECT_SIZE_MULTIPLIER)
            .offset_by(MAP_SIZE_MARGIN / 2);

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
            .set("stroke", Self::door_color_for(&door.modifier))
            .set("stroke-width", STROKE_WIDTH + 1)
            .set("d", data)
    }

    fn door_color_for(modifier: &DoorModifier) -> &'static str {
        match modifier {
            DoorModifier::Open => LIGHT_BLUE,
            DoorModifier::Secret => LIGHT_BLUE,
            DoorModifier::Locked => LIGHT_BLUE,
            DoorModifier::None => LIGHT_BLUE,
        }
    }
}
