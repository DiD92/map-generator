use super::*;
use crate::{
    algos::PolygonBuilder,
    constants::{MAP_SIZE_MARGIN, RECT_SIZE_MULTIPLIER},
    types::{Cell, Door, DoorModifier, Edge, Map, Rect, Room, RoomModifier},
};

use svg::{
    Document,
    node::element::{Path, Rectangle, path::Data},
};
use tracing::event;

#[derive(Debug, PartialEq)]
pub(super) enum CastlevaniaMapDrawer {
    CastlevaniaSOTN,
    CastlevaniaAOS,
    CastlevaniaCOTN,
    CastlevaniaHOD,
}

impl MapDrawer for CastlevaniaMapDrawer {
    fn draw(&self, maps: Vec<Map>, config: &DrawConfig) -> svg::Document {
        let document_width = (config.canvas_width * RECT_SIZE_MULTIPLIER) + MAP_SIZE_MARGIN;
        let document_height = (config.canvas_height * RECT_SIZE_MULTIPLIER) + MAP_SIZE_MARGIN;

        event!(
            tracing::Level::DEBUG,
            "Document size: [{}x{}]",
            document_width,
            document_height
        );
        let mut document = Document::new()
            .set("width", document_width)
            .set("height", document_height);

        let (room_color, door_color, wall_color) = match self {
            CastlevaniaMapDrawer::CastlevaniaSOTN => (LIGHT_BLUE, LIGHT_BLUE, LIGHT_GRAY),
            CastlevaniaMapDrawer::CastlevaniaAOS => (DEEP_BLUE, CYAN_BLUE, LIGHT_WHITE),
            CastlevaniaMapDrawer::CastlevaniaCOTN => (DARK_BLUE, DARK_BLUE, LIGHT_WHITE),
            CastlevaniaMapDrawer::CastlevaniaHOD => (LIME_GREEN, LIME_GREEN, LIGHT_WHITE),
        };

        let merged_map = {
            let mut map = Map {
                origin_rect: Rect {
                    origin: Cell::new(0, 0),
                    width: config.canvas_width,
                    height: config.canvas_height,
                },
                rooms: vec![],
                doors: vec![],
            };

            for map_fragment in maps.into_iter() {
                map.rooms.extend(map_fragment.rooms);
                map.doors.extend(map_fragment.doors);
            }

            map
        };

        let full_door = self == &CastlevaniaMapDrawer::CastlevaniaAOS;

        for room_path in merged_map
            .rooms
            .iter()
            .map(|room| Self::draw_room(room, room_color, wall_color))
        {
            document = document.add(room_path);
        }

        for door_path in merged_map
            .doors
            .iter()
            .map(|door| Self::draw_door(door, door_color, full_door))
        {
            document = document.add(door_path);
        }

        for room in merged_map.rooms.iter() {
            // We need to overlay a rect for the save and navigation rooms
            // to avoid clipping artifacts with the doors.
            if let Some(modifier) = room.modifier {
                let point = room.cells[0]
                    .stretched_by(RECT_SIZE_MULTIPLIER)
                    .offset_by(MAP_SIZE_MARGIN / 2 + STROKE_WIDTH / 2);

                let mut rect = Rectangle::new()
                    .set("x", point.col)
                    .set("y", point.row)
                    .set("width", RECT_SIZE_MULTIPLIER - STROKE_WIDTH)
                    .set("height", RECT_SIZE_MULTIPLIER - STROKE_WIDTH);

                match modifier {
                    RoomModifier::Navigation => {
                        rect = rect.set("fill", YELLOW);
                        document = document.add(rect);
                    }
                    RoomModifier::Save => {
                        rect = rect.set("fill", RED);
                        document = document.add(rect);
                    }
                    _ => {}
                }
            }
        }

        document
    }
}

impl CastlevaniaMapDrawer {
    fn draw_room(room: &Room, room_color: &str, wall_color: &str) -> Path {
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
            .set("fill", room_color)
            .set("stroke", wall_color)
            .set("stroke-width", STROKE_WIDTH)
            .set("d", data)
    }

    fn draw_door(door: &Door, door_color: &str, full_door: bool) -> Path {
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

                let (from_y, to_y) = if full_door {
                    (
                        from.row + STROKE_WIDTH / 2,
                        from.row + RECT_SIZE_MULTIPLIER - STROKE_WIDTH / 2,
                    )
                } else {
                    (
                        from.row + line_separation,
                        from.row + RECT_SIZE_MULTIPLIER - line_separation,
                    )
                };

                data = data.move_to::<(u32, u32)>((x, from_y));
                data = data.line_to::<(u32, u32)>((x, to_y));
            } else {
                event!(
                    tracing::Level::ERROR,
                    "Door axis is not a straight line! from: {:?}, to: {:?}",
                    from,
                    to
                );
            }
        } else if from.row != to.row {
            if from.col == to.col {
                // Horizontal door
                let y = if from.row > to.row { from.row } else { to.row };

                let (from_x, to_x) = if full_door {
                    (
                        from.col + STROKE_WIDTH / 2,
                        from.col + RECT_SIZE_MULTIPLIER - STROKE_WIDTH / 2,
                    )
                } else {
                    (
                        from.col + line_separation,
                        from.col + RECT_SIZE_MULTIPLIER - line_separation,
                    )
                };

                data = data.move_to::<(u32, u32)>((from_x, y));
                data = data.line_to::<(u32, u32)>((to_x, y));
            } else {
                event!(
                    tracing::Level::ERROR,
                    "Door axis is not a straight line! from: {:?}, to: {:?}",
                    from,
                    to
                );
            }
        } else {
            event!(
                tracing::Level::ERROR,
                "Door axis is not a straight line! from: {:?}, to: {:?}",
                from,
                to
            );
        }

        data = data.close();

        let extra_width = if full_door { 0 } else { 8 };

        Path::new()
            .set("stroke", door_color)
            .set("stroke-width", STROKE_WIDTH + extra_width)
            .set("d", data)
    }
}
