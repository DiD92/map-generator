mod algo;
mod consants;
mod types;

fn main() {
    let columns: u32 = 48;
    let rows = 27;

    let builder = algo::MapBuilder::new(columns, rows).unwrap();
    let config = algo::MapBuilderConfig::default();
    let map = builder.build(&config);
    println!(
        "Built map with {} rooms and {} doors",
        map.rooms.len(),
        map.doors.len()
    );

    let svg = map.into_svg(columns, rows);
    let file_name = {
        use std::time::SystemTime;

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        format!("generated/{:?}-map.svg", now)
    };
    svg::save(file_name, &svg).unwrap();
}
