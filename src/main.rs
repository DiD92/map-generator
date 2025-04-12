mod algos;
mod constants;
mod types;

fn main() {
    let columns: u32 = 48;
    let rows = 27;

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
