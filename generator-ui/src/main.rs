// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::worker::{Worker, WorkerMessage};

use generator_core::MapStyle;

use std::error::Error;

mod worker;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let ui = AppWindow::new()?;
    let ui_handle = ui.as_weak();

    let (worker_handle, tx) = Worker::init(ui_handle);

    ui.on_request_new_map({
        let tx = tx.clone();

        move |cols, rows, style_code| {
            let cols = cols as u32;
            let rows = rows as u32;

            let style = match MapStyle::try_from_str(&style_code) {
                Ok(style) => style,
                Err(err) => {
                    println!("Error parsing map style from UI: {}", err);
                    MapStyle::CastlevaniaSOTN
                }
            };

            tx.send(WorkerMessage::RequestNewMap(cols, rows, style))
                .unwrap_or_else(|err| {
                    println!("Error sending message to thread: {}", err);
                });
        }
    });

    ui.on_request_save_map(move || {
        tx.send(WorkerMessage::SaveCurrentMap)
            .unwrap_or_else(|err| {
                println!("Error sending save request to thread: {}", err);
            });
    });

    println!("Starting UI...");

    ui.run()?;

    println!("UI has exited, waiting for worker thread to finish...");

    // We manually drop the UI to ensure the worker thread can finish cleanly.
    drop(ui);

    worker_handle.join().unwrap_or_else(|err| {
        println!("Error joining worker thread: {:?}", err);
    });

    Ok(())
}
