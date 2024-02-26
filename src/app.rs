use crate::arduino::Arduino;
use crate::arduino::PacketData;
use crate::arduino::ThreadMSG;
use colored::Colorize;
use std::sync::Arc;
use std::sync::Mutex;
use std::usize;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TryRecvError;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,
    #[serde(skip)]
    pub data_collection: Arc<Mutex<Vec<Vec<PacketData>>>>,
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
            data_collection: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

// https://aryalinux.org/blog/how-to-use-the-serial-port-in-multiple-threads-in

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        rx: mpsc::Receiver<ThreadMSG>,
        tx: mpsc::Sender<ThreadMSG>,
        arduino: Arc<Mutex<Arduino>>,
        _data_collection: Arc<Mutex<Vec<Vec<usize>>>>,
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
        let _t = self.tx.clone();

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
                                    send_thread_msg(self.tx.clone(), ThreadMSG::Disconnect());
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
            for v in self.data_collection.lock().unwrap().iter() {
                for v_exp in v.iter() {
                    ui.add(egui::Label::new(format!("{:?}", v_exp)));
                }
            }
            ui.separator();
            ui.add(egui::github_link_file!(
                "https://github.com/Portablefire22/Arduino-Communication-GUI/blob/master/",
                "Source code (Private)"
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
        match self.rx.try_recv() {
            Err(TryRecvError::Disconnected) => (), // TODO, error message as pop-up
            Err(_) => (),
            Ok(t) => match t {
                ThreadMSG::Data(data) => match data {
                    PacketData::String(_, id, _time) | PacketData::Integer(_, id, _time) => {
                        match self.data_collection.lock() {
                            Ok(mut t) => match t.get(id as usize) {
                                None => {
                                    if id == 0 {
                                        t.resize(1, Vec::new());
                                    } else {
                                        t.resize(id as usize, Vec::new());
                                    }
                                    t[id as usize].push(data);
                                }
                                Some(_) => {
                                    t[id as usize].push(data);
                                }
                            },
                            Err(_) => {
                                eprintln!("Mutex error: Error unlocking whilst retrieving data")
                            } //self.data_collection.lock().unwrap().resize(id as usize, );
                              //self.data_collection.lock().unwrap()[id as usize].push(data);
                        }
                    }
                    _ => (),
                },
                _ => (),
            },
        }
    }
}

pub fn send_thread_msg(tx: mpsc::Sender<ThreadMSG>, msg: ThreadMSG) {
    tokio::spawn(async move {
        match tx.send(msg.clone()).await {
            Err(t) => eprintln!(
                "{} '{:?}' {}\n{}!",
                "Could not send".red(),
                &msg,
                "to Arduino thread!".red(),
                t,
            ),
            _ => (),
        };
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
