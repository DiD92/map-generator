// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use generator_core::{MapStyle, create_map};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use tiny_skia::PixmapMut;

use std::error::Error;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    ui.on_request_new_map({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();

            let map = create_map(48, 32, MapStyle::CastlevaniaSOTN);

            let map_str = map.to_string();

            let opt = {
                let mut opt = usvg::Options::default();

                opt.fontdb_mut().load_system_fonts();

                opt
            };

            let tree = usvg::Tree::from_str(&map_str, &opt).unwrap();
            let pixmap_size = tree.size().to_int_size();
            let width = pixmap_size.width();
            let height = pixmap_size.height();

            let mut pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);

            let pixmap_opt: Option<PixmapMut> =
                PixmapMut::from_bytes(pixel_buffer.make_mut_bytes(), width, height);
            if pixmap_opt.is_none() {
                println!("Couldn't create pixmap image!");
            }
            let mut pixmap = pixmap_opt.unwrap();
            pixmap.fill(tiny_skia::Color::BLACK);

            resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap);

            let image = Image::from_rgba8_premultiplied(pixel_buffer);

            ui.set_map(image);
        }
    });

    ui.run()?;

    Ok(())
}
