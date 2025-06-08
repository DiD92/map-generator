use crate::AppWindow;

use std::{
    fs::create_dir as create_generated_dir, path::Path, sync::mpsc::Sender, thread::JoinHandle,
};

use generator_core::{MapStyle, create_map};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer, Weak};
use svg::{Document, save as save_as_svg};
use tiny_skia::PixmapMut;

pub(crate) enum WorkerMessage {
    RequestNewMap(u32, u32, MapStyle),
    SaveCurrentMap,
}

pub(crate) struct Worker {
    last_map: Option<Document>,
    ui_handle: Weak<AppWindow>,
}

impl Worker {
    pub(crate) fn init(ui_handle: Weak<AppWindow>) -> (JoinHandle<()>, Sender<WorkerMessage>) {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut worker = Worker {
            last_map: None,
            ui_handle,
        };

        let worker_handle = std::thread::spawn(move || {
            while let Ok(message) = rx.recv() {
                match message {
                    WorkerMessage::RequestNewMap(cols, rows, style) => {
                        worker.generate_and_draw_new_map(cols, rows, style);
                    }
                    WorkerMessage::SaveCurrentMap => {
                        // We add a small delay to ensure the UI has time to update
                        std::thread::sleep(std::time::Duration::from_millis(500));

                        worker.save_current_map();
                    }
                }
            }
        });

        (worker_handle, tx)
    }

    fn generate_and_draw_new_map(&mut self, cols: u32, rows: u32, style: MapStyle) {
        let map = create_map(cols, rows, style);
        let map_str = map.to_string();

        self.last_map = Some(map);

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

        let _ = self.ui_handle.upgrade_in_event_loop(move |handle| {
            let image = Image::from_rgba8_premultiplied(pixel_buffer);
            handle.set_map(image);
            handle.invoke_enable_generate_button();
            handle.invoke_enable_save_button();
        });
    }

    fn save_current_map(&self) {
        let map_filename = {
            use std::time::SystemTime;

            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap();
            format!("generated/{:?}-map.svg", now)
        };

        if let Some(ref map) = self.last_map {
            match Path::new("generated").try_exists() {
                Ok(false) => {
                    create_generated_dir("generated")
                        .expect("Failed to create 'generated' directory!");
                    println!("Directory 'generated' created.");
                }
                Err(e) => {
                    println!("Error creating 'generated' directory: {}", e);
                    return;
                }
                _ => {}
            }
            println!("Saving current map: {:?}", map_filename);

            save_as_svg(map_filename, map).expect("Failed to save SVG file!");

            let _ = self.ui_handle.upgrade_in_event_loop(|handle| {
                handle.invoke_enable_save_button();
            });
        } else {
            println!("No map to save.");
        }
    }
}
