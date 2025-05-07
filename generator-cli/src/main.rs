use generator_core::{MapStyle, create_map};

use std::{fs::create_dir as create_generated_dir, path::Path};

use clap::Parser;
use svg::save as save_as_svg;

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
}

fn main() {
    let args = Args::parse();

    let map_data = create_map(args.columns, args.rows, args.style);

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
            println!("Directory 'generated' created.");
        }
        Err(e) => {
            eprintln!("Error checking for 'generated' directory: {}", e);
            return;
        }
        _ => {}
    }

    println!("Saving map as SVG to: {}", map_filename);

    save_as_svg(map_filename, &map_data).expect("Failed to save SVG file!");
}
