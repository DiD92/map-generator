use crate::types::{Map, MapStyle};

use svg::Document;

mod castlevania;

pub(crate) struct DrawConfig {
    pub(crate) canvas_width: u32,
    pub(crate) canvas_height: u32,
}

pub(crate) trait MapDrawer {
    fn draw(&self, map: &Map, config: &DrawConfig) -> Document;
}

pub(crate) struct MapDrawerFactory;

impl MapDrawerFactory {
    pub(crate) fn create_drawer(map_style: MapStyle) -> impl MapDrawer {
        match map_style {
            MapStyle::CastlevaniaSOTN => castlevania::CastlevaniaMapDrawer::CastlevaniaSOTN,
            MapStyle::CastlevaniaAOS => castlevania::CastlevaniaMapDrawer::CastlevaniaAOS,
            MapStyle::CastlevaniaCOTN => castlevania::CastlevaniaMapDrawer::CastlevaniaCOTN,
            MapStyle::CastlevaniaHOD => castlevania::CastlevaniaMapDrawer::CastlevaniaHOD,
            MapStyle::MetroidZM => todo!(),
            MapStyle::MetroidFS => todo!(),
            MapStyle::MetroidSP => todo!(),
        }
    }
}
