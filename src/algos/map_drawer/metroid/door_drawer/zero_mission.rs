use super::DoorDrawer;
use crate::{
    algos::map_drawer::STROKE_WIDTH,
    constants::RECT_SIZE_MULTIPLIER,
    types::{Cell, Door, DoorModifier},
};

use svg::node::element::{Path, path::Data};

pub(super) struct ZeroMissionDoorDrawer;

impl DoorDrawer for ZeroMissionDoorDrawer {
    fn draw_door(&self, door: &Door, col_offset: u32, row_offset: u32, door_color: &str) -> Path {
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
