mod algos;
mod constants;
mod types;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, long_about = None)]
struct Args {
    /// Number of columns in the map
    #[arg(short, long, default_value_t = 64)]
    columns: u32,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 45)]
    rows: u32,

    #[clap(short, long, default_value_t, value_enum)]
    style: types::MapStyle,
}

fn main() {
    let args = Args::parse();

    let columns = args.columns;
    let rows = args.rows;

    let builder = algos::MapBuilder::new(columns, rows).unwrap();
    let build_config = algos::MapBuilderConfig::from_style(args.style);
    let map = builder.build(&build_config, args.style);

    let drawer = algos::MapDrawerFactory::create_drawer(args.style);
    let draw_config = algos::DrawConfig {
        canvas_width: columns,
        canvas_height: rows,
    };
    let svg = algos::MapDrawer::draw(&drawer, &map, &draw_config);

    let svg_name = {
        use std::time::SystemTime;

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        format!("generated/{:?}-map.svg", now)
    };
    svg::save(svg_name, &svg).expect("Failed to save SVG file!");
}
