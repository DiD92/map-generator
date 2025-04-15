use super::MapBuilderConfig;
use crate::types::{MapStyle, NeighbourTable, RoomTable};

mod castlevania;

pub(super) trait RoomDecorator {
    fn decorate(
        &self,
        rooms: &mut RoomTable,
        neighbour_table: &NeighbourTable,
        config: &MapBuilderConfig,
    );
}

pub(super) struct RoomDecoratorFactory;

impl RoomDecoratorFactory {
    pub(super) fn decorator_for(style: MapStyle) -> impl RoomDecorator {
        match style {
            MapStyle::CastlevaniaSOTN => castlevania::CastlevaniaRoomDectorator,
            MapStyle::CastlevaniaAOS => castlevania::CastlevaniaRoomDectorator,
            MapStyle::CastlevaniaCOTN => castlevania::CastlevaniaRoomDectorator,
            MapStyle::MetroidZM => todo!(),
            MapStyle::MetroidFS => todo!(),
            MapStyle::MetroidSP => todo!(),
        }
    }
}
