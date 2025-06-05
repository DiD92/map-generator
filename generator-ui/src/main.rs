// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use generator_core::{MapStyle, create_map};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use tiny_skia::PixmapMut;

use std::error::Error;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    let ui_handle = ui.as_weak();

    let (tx, rx) = std::sync::mpsc::channel();

    let worker_handle = std::thread::spawn(move || {
        while let Ok((cols, rows, style)) = rx.recv() {
            let map = create_map(cols, rows, style);

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

            let _ = ui_handle.upgrade_in_event_loop(move |handle| {
                let image = Image::from_rgba8_premultiplied(pixel_buffer);
                handle.set_map(image);
                handle.invoke_enable_generate_button();
            });
        }
    });

    let ui_handle = ui.as_weak();
    ui.on_request_new_map({
        move || {
            let ui = ui_handle.unwrap();

            let cols = ui.get_cols() as u32;
            let rows = ui.get_rows() as u32;

            let style = match MapStyle::try_from_str(&ui.get_style_code()) {
                Ok(style) => style,
                Err(err) => {
                    println!("Error parsing map style from UI: {}", err);
                    MapStyle::CastlevaniaSOTN
                }
            };

            tx.send((cols, rows, style)).unwrap_or_else(|err| {
                println!("Error sending message to thread: {}", err);
            });
        }
    });

    ui.on_request_save_map(|| {});

    ui.run()?;

    // We manually drop the UI to ensure the worker thread can finish cleanly.
    drop(ui);

    worker_handle.join().unwrap_or_else(|err| {
        println!("Error joining worker thread: {:?}", err);
    });

    Ok(())
}
