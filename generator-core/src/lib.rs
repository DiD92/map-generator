mod algos;
mod constants;
mod types;

use tracing::{Level, span};

pub use types::MapStyle;

pub fn create_map(columns: u32, rows: u32, style: types::MapStyle) -> svg::Document {
    let span = span!(Level::DEBUG, "create_map");
    let _guard = span.enter();

    let build_config = algos::MapBuilderConfig::from_style(style);
    let builder = algos::MapBuilder::new(columns, rows).unwrap();

    let maps = builder.build(&build_config, style);

    let draw_config = algos::DrawConfig {
        canvas_width: columns,
        canvas_height: rows,
    };
    let drawer = algos::MapDrawerFactory::create_drawer(style);

    algos::MapDrawer::draw(drawer.as_ref(), maps, &draw_config)
}
