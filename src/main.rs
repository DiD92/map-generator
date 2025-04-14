use clap::Parser;

mod algos;
mod constants;
mod types;

#[derive(Parser, Debug)]
#[command(version, long_about = None)]
struct Args {
    /// Number of columns in the map
    #[arg(short, long, default_value_t = 64)]
    columns: u32,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 45)]
    rows: u32,
}

fn main() {
    let args = Args::parse();

    let columns = args.columns;
    let rows = args.rows;

    let builder = algos::MapBuilder::new(columns, rows).unwrap();
    let config = algos::MapBuilderConfig::default();
    let map = builder.build(&config);

    let svg = map.draw(columns, rows);
    let file_name = {
        use std::time::SystemTime;

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        format!("generated/{:?}-map.svg", now)
    };
    svg::save(file_name, &svg).unwrap();
}
