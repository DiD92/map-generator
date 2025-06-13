use generator_core::{MapStyle, create_map};

use std::{fs::create_dir as create_generated_dir, path::Path};

use clap::Parser;
use svg::save as save_as_svg;
use tracing::event;

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
    style: MapStyle,

    #[clap(short, long, default_value_t = false)]
    /// If true, the map will not be saved to a file
    dry_run: bool,
}

fn main() {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let map_data = create_map(args.columns, args.rows, args.style);

    if args.dry_run {
        event!(
            tracing::Level::INFO,
            "Dry run mode enabled. Map data not saved."
        );
        return;
    }

    let map_filename = {
        use std::time::SystemTime;

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        format!("generated/{:?}-map.svg", now)
    };

    match Path::new("generated").try_exists() {
        Ok(false) => {
            create_generated_dir("generated").expect("Failed to create 'generated' directory!");
            event!(tracing::Level::INFO, "Directory 'generated' created.");
        }
        Err(e) => {
            event!(
                tracing::Level::ERROR,
                "Error creating 'generated' directory: {}",
                e
            );
            return;
        }
        _ => {}
    }

    event!(
        tracing::Level::INFO,
        "Saving map as SVG to: {}",
        map_filename
    );

    save_as_svg(map_filename, &map_data).expect("Failed to save SVG file!");
}
