mod map_builder;
mod map_drawer;
mod polygon_builder;

pub(crate) use map_builder::{MapBuilder, MapBuilderConfig};
pub(crate) use map_drawer::{DrawConfig, MapDrawer, MapDrawerFactory};
pub(crate) use polygon_builder::PolygonBuilder;

pub(crate) struct RngHandler;

impl RngHandler {
    #[cfg(not(test))]
    pub fn rng() -> impl rand::Rng {
        rand::rng()
    }

    #[cfg(test)]
    pub fn rng() -> impl rand::Rng {
        use crate::constants::TEST_RANDOM_INCREMENT;
        use crate::constants::TEST_RANDOM_INITIAL;
        use rand::rngs::mock::StepRng;

        StepRng::new(TEST_RANDOM_INITIAL, TEST_RANDOM_INCREMENT)
    }
}
