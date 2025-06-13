use super::RegionConnectorDrawer;
use crate::{
    algos::{RngHandler, map_drawer::STROKE_WIDTH},
    constants::RECT_SIZE_MULTIPLIER,
    types::{Cell, Direction, Room, RoomModifier},
};

use rand::Rng;
use svg::node::element::{Path, Polygon, path::Data};
use tracing::event;

pub(super) struct FusionRegionConnectorDrawer;

impl RegionConnectorDrawer for FusionRegionConnectorDrawer {
    fn draw_region_connector(
        &self,
        room: &Room,
        col_offset: u32,
        row_offset: u32,
        room_color: &str,
        wall_color: &str,
        door_color: &str,
    ) -> (Path, Path, Polygon) {
        let mut data = Data::new();
        let mut arrow_points = vec![];

        let mut rng = RngHandler::rng();

        let (selected_cell, origin_cell, direction) = match room.modifier {
            Some(RoomModifier::RegionConnection(Direction::North)) => {
                let min_row = room.cells.iter().map(|cell| cell.row).min().unwrap_or(0);

                let valid_cells = room
                    .cells
                    .iter()
                    .copied()
                    .filter(|cell| cell.row == min_row)
                    .collect::<Vec<_>>();

                let cell_idx = rng.random_range(0..valid_cells.len());
                let selected_cell = valid_cells[cell_idx];

                (
                    Cell {
                        col: selected_cell.col,
                        row: selected_cell.row - 1,
                    },
                    selected_cell,
                    Direction::North,
                )
            }
            Some(RoomModifier::RegionConnection(Direction::South)) => {
                let max_row = room.cells.iter().map(|cell| cell.row).max().unwrap_or(0);

                let valid_cells = room
                    .cells
                    .iter()
                    .copied()
                    .filter(|cell| cell.row == max_row)
                    .collect::<Vec<_>>();

                let cell_idx = rng.random_range(0..valid_cells.len());
                let selected_cell = valid_cells[cell_idx];

                (
                    Cell {
                        col: selected_cell.col,
                        row: selected_cell.row + 1,
                    },
                    selected_cell,
                    Direction::South,
                )
            }
            Some(RoomModifier::RegionConnection(Direction::East)) => {
                let max_col = room.cells.iter().map(|cell| cell.col).max().unwrap_or(0);

                let valid_cells = room
                    .cells
                    .iter()
                    .copied()
                    .filter(|cell| cell.col == max_col)
                    .collect::<Vec<_>>();

                let cell_idx = rng.random_range(0..valid_cells.len());
                let selected_cell = valid_cells[cell_idx];

                (
                    Cell {
                        col: selected_cell.col + 1,
                        row: selected_cell.row,
                    },
                    selected_cell,
                    Direction::East,
                )
            }
            Some(RoomModifier::RegionConnection(Direction::West)) => {
                let min_col = room.cells.iter().map(|cell| cell.col).min().unwrap_or(0);

                let valid_cells = room
                    .cells
                    .iter()
                    .copied()
                    .filter(|cell| cell.col == min_col)
                    .collect::<Vec<_>>();

                let cell_idx = rng.random_range(0..valid_cells.len());
                let selected_cell = valid_cells[cell_idx];

                (
                    Cell {
                        col: selected_cell.col - 1,
                        row: selected_cell.row,
                    },
                    selected_cell,
                    Direction::West,
                )
            }
            _ => panic!("Invalid room modifier"),
        };
        let (cell_col, cell_row) = selected_cell
            .stretched_by(RECT_SIZE_MULTIPLIER)
            .offset_by_two(col_offset, row_offset)
            .into();

        match direction {
            Direction::North => {
                data = data.move_to::<(u32, u32)>((cell_col + 12, cell_row));
                data = data.line_to::<(u32, u32)>((cell_col + 12, cell_row + 24));
                data = data.line_to::<(u32, u32)>((cell_col, cell_row + 24));
                data = data.line_to::<(u32, u32)>((cell_col, cell_row + 48));
                data = data.line_to::<(u32, u32)>((cell_col + 48, cell_row + 48));
                data = data.line_to::<(u32, u32)>((cell_col + 48, cell_row + 24));
                data = data.line_to::<(u32, u32)>((cell_col + 36, cell_row + 24));
                data = data.line_to::<(u32, u32)>((cell_col + 36, cell_row));

                arrow_points.push((cell_col + 8, cell_row - 8));
                arrow_points.push((cell_col + 40, cell_row - 8));
                arrow_points.push((cell_col + 24, cell_row - 24));
            }
            Direction::South => {
                data = data.move_to::<(u32, u32)>((cell_col + 12, cell_row + 48));
                data = data.line_to::<(u32, u32)>((cell_col + 12, cell_row + 24));
                data = data.line_to::<(u32, u32)>((cell_col, cell_row + 24));
                data = data.line_to::<(u32, u32)>((cell_col, cell_row));
                data = data.line_to::<(u32, u32)>((cell_col + 48, cell_row));
                data = data.line_to::<(u32, u32)>((cell_col + 48, cell_row + 24));
                data = data.line_to::<(u32, u32)>((cell_col + 36, cell_row + 24));
                data = data.line_to::<(u32, u32)>((cell_col + 36, cell_row + 48));

                arrow_points.push((cell_col + 8, cell_row + 56));
                arrow_points.push((cell_col + 40, cell_row + 56));
                arrow_points.push((cell_col + 24, cell_row + 72));
            }
            Direction::East => {
                data = data.move_to::<(u32, u32)>((cell_col + 48, cell_row + 12));
                data = data.line_to::<(u32, u32)>((cell_col + 24, cell_row + 12));
                data = data.line_to::<(u32, u32)>((cell_col + 24, cell_row));
                data = data.line_to::<(u32, u32)>((cell_col, cell_row));
                data = data.line_to::<(u32, u32)>((cell_col, cell_row + 48));
                data = data.line_to::<(u32, u32)>((cell_col + 24, cell_row + 48));
                data = data.line_to::<(u32, u32)>((cell_col + 24, cell_row + 36));
                data = data.line_to::<(u32, u32)>((cell_col + 48, cell_row + 36));

                arrow_points.push((cell_col + 56, cell_row + 8));
                arrow_points.push((cell_col + 56, cell_row + 40));
                arrow_points.push((cell_col + 72, cell_row + 24));
            }
            Direction::West => {
                data = data.move_to::<(u32, u32)>((cell_col, cell_row + 12));
                data = data.line_to::<(u32, u32)>((cell_col + 24, cell_row + 12));
                data = data.line_to::<(u32, u32)>((cell_col + 24, cell_row));
                data = data.line_to::<(u32, u32)>((cell_col + 48, cell_row));
                data = data.line_to::<(u32, u32)>((cell_col + 48, cell_row + 48));
                data = data.line_to::<(u32, u32)>((cell_col + 24, cell_row + 48));
                data = data.line_to::<(u32, u32)>((cell_col + 24, cell_row + 36));
                data = data.line_to::<(u32, u32)>((cell_col, cell_row + 36));

                arrow_points.push((cell_col - 8, cell_row + 8));
                arrow_points.push((cell_col - 8, cell_row + 40));
                arrow_points.push((cell_col - 24, cell_row + 24));
            }
        };

        let arrow = Polygon::new()
            .set("fill", wall_color)
            .set("points", arrow_points);

        let room = Path::new()
            .set("fill", room_color)
            .set("stroke", wall_color)
            .set("stroke-width", STROKE_WIDTH)
            .set("d", data);

        let door = Self::draw_connection_door(
            origin_cell,
            selected_cell,
            col_offset,
            row_offset,
            door_color,
        );

        (room, door, arrow)
    }
}

impl FusionRegionConnectorDrawer {
    fn draw_connection_door(
        from: Cell,
        to: Cell,
        col_offset: u32,
        row_offset: u32,
        door_color: &str,
    ) -> Path {
        let mut data = Data::new();

        let from = from
            .stretched_by(RECT_SIZE_MULTIPLIER)
            .offset_by_two(col_offset, row_offset);
        let to: Cell = to
            .stretched_by(RECT_SIZE_MULTIPLIER)
            .offset_by_two(col_offset, row_offset);

        let line_separation = 16;

        if from.col != to.col {
            // Veritical door

            if from.row == to.row {
                let x = if from.col > to.col { from.col } else { to.col };

                let from_y = from.row + line_separation;
                let to_y = from.row + RECT_SIZE_MULTIPLIER - line_separation;

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

                let from_x = from.col + line_separation;
                let to_x = from.col + RECT_SIZE_MULTIPLIER - line_separation;

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
                "Door axis is a point! from: {:?}, to: {:?}",
                from,
                to
            );
        }

        data = data.close();

        Path::new()
            .set("stroke", door_color)
            .set("stroke-width", STROKE_WIDTH + 8)
            .set("d", data)
    }
}
