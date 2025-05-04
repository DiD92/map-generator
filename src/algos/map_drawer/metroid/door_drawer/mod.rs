use super::MetroidMapDrawer;
use crate::types::Door;

use svg::node::element::Path;

mod fusion;
mod super_metroid;
mod zero_mission;

pub(super) trait DoorDrawer {
    fn draw_door(&self, door: &Door, col_offset: u32, row_offset: u32, door_color: &str) -> Path;
}

pub(super) struct DoorDrawerFactory;

impl DoorDrawerFactory {
    pub(super) fn drawer_for(style: &MetroidMapDrawer) -> Box<dyn DoorDrawer> {
        match style {
            MetroidMapDrawer::ZeroMission => Box::new(zero_mission::ZeroMissionDoorDrawer),
            MetroidMapDrawer::Fusion => Box::new(fusion::FusionDoorDrawer),
            MetroidMapDrawer::Super => Box::new(super_metroid::SuperMetroidDoorDrawer),
        }
    }
}
