use crate::types::{Map, MapStyle};

use svg::Document;

mod castlevania;
mod metroid;

const LIGHT_BLUE: &str = "#0080ff";
const CYAN_BLUE: &str = "#00c8c8";
const DARK_BLUE: &str = "#004bff";
const DEEP_BLUE: &str = "#0000e0";
const LIME_GREEN: &str = "#00e000";

const LIGHT_WHITE: &str = "#f8f8f8";
const LIGHT_GRAY: &str = "#c0c0c0";

const RED: &str = "#f80000";
const YELLOW: &str = "#f8f800";

const STROKE_WIDTH: u32 = 12;

pub(crate) struct DrawConfig {
    pub(crate) canvas_width: u32,
    pub(crate) canvas_height: u32,
}

pub(crate) trait MapDrawer {
    fn draw(&self, maps: Vec<Map>, config: &DrawConfig) -> Document;
}

pub(crate) struct MapDrawerFactory;

impl MapDrawerFactory {
    pub(crate) fn create_drawer(map_style: MapStyle) -> Box<dyn MapDrawer> {
        match map_style {
            MapStyle::CastlevaniaSOTN => {
                Box::new(castlevania::CastlevaniaMapDrawer::CastlevaniaSOTN)
            }
            MapStyle::CastlevaniaAOS => Box::new(castlevania::CastlevaniaMapDrawer::CastlevaniaAOS),
            MapStyle::CastlevaniaCOTN => {
                Box::new(castlevania::CastlevaniaMapDrawer::CastlevaniaCOTN)
            }
            MapStyle::CastlevaniaHOD => Box::new(castlevania::CastlevaniaMapDrawer::CastlevaniaHOD),
            MapStyle::MetroidZM => Box::new(metroid::MetroidMapDrawer::ZeroMission),
            MapStyle::MetroidFS => Box::new(metroid::MetroidMapDrawer::Fusion),
            MapStyle::MetroidSP => Box::new(metroid::MetroidMapDrawer::Super),
        }
    }
}
