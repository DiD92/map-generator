use crate::types::Room;

use svg::node::element::{Path, Polygon};

use super::MetroidMapDrawer;

mod fusion;
mod super_metroid;
mod zero_mission;

pub(super) trait RegionConnectorDrawer {
    fn draw_region_connector(
        &self,
        room: &Room,
        col_offset: u32,
        row_offset: u32,
        room_color: &str,
        wall_color: &str,
        door_color: &str,
    ) -> (Path, Path, Polygon);
}

pub(super) struct RegionConnectorDrawerFactory;

impl RegionConnectorDrawerFactory {
    pub(super) fn drawer_for(style: &MetroidMapDrawer) -> Box<dyn RegionConnectorDrawer> {
        match style {
            MetroidMapDrawer::ZeroMission => {
                Box::new(zero_mission::ZeroMissionRegionConnectorDrawer)
            }
            MetroidMapDrawer::Fusion => Box::new(fusion::FusionRegionConnectorDrawer),
            MetroidMapDrawer::Super => Box::new(super_metroid::SuperMetroidRegionConnectorDrawer),
        }
    }
}
