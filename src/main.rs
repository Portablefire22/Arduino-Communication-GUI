#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod app;
mod arduino;
use std::sync::{Arc, Mutex};
// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> eframe::Result<()> {
    use std::thread;

    use app::TemplateApp;
    use tokio::sync::mpsc;

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let (tx_gui, mut rx_arduino) = mpsc::channel::<arduino::ThreadMSG>(100);
    let (tx_arduino, mut rx_gui) = mpsc::channel::<arduino::ThreadMSG>(100);

    let arduino_handler = Arc::new(Mutex::new(arduino::Arduino::new()));
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .unwrap(),
            ),
        ..Default::default()
    };

    eframe::run_native(
        "Arduino Communication",
        native_options,
        Box::new(|cc| {
            let frame = cc.egui_ctx.clone();
            let arduino_thread_handler = arduino_handler.clone();
            tokio::spawn(async move {
                loop {
                    match rx_arduino.recv().await {
                        Some(t) => {}
                        None => {
                            panic!("Transmitter has been dropped!");
                        }
                    }
                }
            });
            Box::new(TemplateApp::new(cc, rx_gui, tx_gui, arduino_handler))
        }),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(arduino_communication_gui::TemplateApp::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}
