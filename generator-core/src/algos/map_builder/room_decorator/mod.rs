use super::MapBuilderConfig;
use crate::types::{Door, MapRegion, MapStyle};

mod castlevania;
mod metroid;

pub(super) trait RoomDecorator {
    fn decorate(&self, map_region: &mut MapRegion, doors: &[Door], config: &MapBuilderConfig);
}

pub(super) struct RoomDecoratorFactory;

impl RoomDecoratorFactory {
    pub(super) fn decorator_for(style: MapStyle) -> Box<dyn RoomDecorator> {
        match style {
            MapStyle::CastlevaniaSOTN => Box::new(castlevania::CastlevaniaRoomDectorator),
            MapStyle::CastlevaniaAOS => Box::new(castlevania::CastlevaniaRoomDectorator),
            MapStyle::CastlevaniaCOTM => Box::new(castlevania::CastlevaniaRoomDectorator),
            MapStyle::CastlevaniaHOD => Box::new(castlevania::CastlevaniaRoomDectorator),
            MapStyle::MetroidZM => Box::new(metroid::MetroidRoomDecorator::ZeroMission),
            MapStyle::MetroidFS => Box::new(metroid::MetroidRoomDecorator::Fusion),
            MapStyle::MetroidSP => Box::new(metroid::MetroidRoomDecorator::SuperMetroid),
        }
    }
}
