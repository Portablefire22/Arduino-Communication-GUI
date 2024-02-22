use crate::arduino;
use crate::arduino::Arduino;
use crate::arduino::ThreadMSG;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::{io, ops::Deref, time::Duration};
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio_serial::SerialPortInfo;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
    #[serde(skip)]
    selected_port: String,
    #[serde(skip)]
    tx: mpsc::Sender<ThreadMSG>,
    #[serde(skip)]
    rx: mpsc::Receiver<ThreadMSG>,
    #[serde(skip)]
    arduino: Arc<Mutex<Arduino>>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel(10);

        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            selected_port: "Disconnected".to_owned(),
            tx,
            rx,
            arduino: Arc::new(Mutex::new(Arduino::new())),
        }
    }
}

// https://aryalinux.org/blog/how-to-use-the-serial-port-in-multiple-threads-in

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        rx: mpsc::Receiver<ThreadMSG>,
        tx: mpsc::Sender<ThreadMSG>,
        arduino: Arc<Mutex<Arduino>>,
    ) -> Self {
        Self {
            arduino,
            rx,
            tx,
            ..Default::default()
        }
    }
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        let t = self.tx.clone();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                }
                let ports = tokio_serial::available_ports();
                ui.menu_button("Ports", |ui| match ports {
                    Err(e) => {
                        ui.label("No ports found!");
                        println!("{:?}", e);
                    }
                    _ => {
                        for port in ports.unwrap().iter() {
                            if port.port_name.eq_ignore_ascii_case(&self.selected_port) {
                                if ui
                                    .button(format!("{} | Disconnect", &port.port_name))
                                    .clicked()
                                {
                                    self.selected_port = "Disconnected".to_owned();
                                    self.arduino.lock().unwrap().disconnect();
                                }
                            } else {
                                if ui.button(port.port_name.clone()).clicked() {
                                    self.selected_port = port.port_name.clone();
                                    self.arduino
                                        .lock()
                                        .unwrap()
                                        .connect(self.selected_port.clone(), 9600);
                                    send_thread_msg(
                                        self.tx.clone(),
                                        ThreadMSG::Start((port.port_name.clone(), 9600)),
                                    );
                                }
                            }
                        }
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("eframe template");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.label);
            });

            ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                self.value += 1.0;
            }

            ui.separator();
            ui.add(egui::Label::new(&self.selected_port));
            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/Portablefire22/Arduino-Communication-GUI/blob/master/",
                "Source code"
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn send_thread_msg(tx: mpsc::Sender<ThreadMSG>, msg: ThreadMSG) {
    tokio::spawn(async move {
        tx.send(msg).await;
    });
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
