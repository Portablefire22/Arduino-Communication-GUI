use crate::arduino::Arduino;
use crate::arduino::PacketData;
use crate::arduino::ThreadMSG;
use crate::data_window;
use crate::data_window::DataWindow;
use crate::error_message;
use colored::Colorize;
use egui::vec2;
use std::borrow::BorrowMut;
use std::collections::HashMap;
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
    #[serde(skip)]
    windows: Vec<DataWindow>,
    #[serde(skip)]
    window_status: HashMap<String, bool>,
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
            windows: Vec::new(),
            window_status: HashMap::new(),
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
        ctx.request_repaint();
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
                show_port_menu(self, ui);
                show_data_menu(self, ctx, ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Arduino Serial Reader");
            ui.label("
                To select the serial device, navigate the 'Ports' menu on the top left of the 
                program. 
                To read data, navigate the data menu (located on the right of 'Ports') and 
                select the data you wish to read. The name of the data set may be renamed by 
                editing the text box in the newly created window.
                The number of shown data entries may also be modified by modifying the 'Limit output' box.
            ");
            ui.separator();
            ui.heading("Available Serial Devices:");
            show_available_ports(self, ui);
            ui.separator();
            ui.add(egui::github_link_file!(
                "https://github.com/Portablefire22/Arduino-Communication-GUI/blob/master/",
                "Source code (Private)"
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });

        show_windows(self, ctx);
        match self.rx.try_recv() {
            Err(TryRecvError::Disconnected) => {
                let mut err_win = error_message::ErrorInfo::new(
                    "Receiver Disconnected!".to_owned(),
                    "Receiver has disconnected, the Arduino thread has likely panicked!".to_owned(),
                    error_message::ErrorSeverity::Critical,
                );
                err_win.show(&ctx);
            } // TODO, error message as pop-up
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
                                        t.resize(id as usize + 1, Vec::new());
                                    }
                                    t[id as usize].push(data);
                                }
                                Some(_) => {
                                    t[id as usize].push(data);
                                }
                            },
                            Err(_) => {
                                eprintln!("Mutex error: Error unlocking whilst retrieving data")
                            }
                        }
                    }
                    _ => (),
                },
                _ => (),
            },
        }
    }
}

fn show_windows(app: &mut TemplateApp, ctx: &egui::Context) {
    match app.data_collection.lock() {
        Err(_e) => eprintln!("Error locking mutex!"),
        Ok(data) => {
            for window in &mut app.windows {
                let tmp_str = &window.selected_data.to_string();
                window.show(
                    ctx,
                    &data[window.selected_data],
                    app.window_status.get_mut(tmp_str).unwrap(),
                );
            }
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

fn show_data_menu(app: &mut TemplateApp, ctx: &egui::Context, ui: &mut egui::Ui) {
    match app.data_collection.lock() {
        Err(e) => {
            eprintln!("Attempted to access data whilst mutex was locked!");
            eprintln!("{}", e);
        }
        Ok(data) => {
            ui.menu_button("Data", |ui| {
                if data.len() == 0 {
                    ui.label("No data stored!");
                } else {
                    for (index_iter, dat) in data.iter().enumerate() {
                        if ui
                            .button(format!(
                                "{} | {}",
                                index_iter,
                                match dat.get(index_iter) {
                                    None => "Unknown!",
                                    Some(t) => t.display_variant(),
                                }
                            ))
                            .clicked()
                        {
                            // Prevents duplicate windows
                            let mut t: Vec<_> = app
                                .windows
                                .iter()
                                .filter(|w| w.selected_data == index_iter)
                                .collect();
                            match t.get_mut(0) {
                                Some(_) => {
                                    app.window_status
                                        .entry(index_iter.to_string())
                                        .and_modify(|x| *x = !*x);
                                }
                                None => {
                                    let window = data_window::DataWindow::new(
                                        index_iter.to_string(),
                                        index_iter,
                                    );
                                    app.windows.push(window);
                                    app.window_status.insert(index_iter.to_string(), true);
                                }
                            }
                        }
                    }
                }
            });
        }
    }
}

fn show_available_ports(app: &mut TemplateApp, ui: &mut egui::Ui) {
    let ports = tokio_serial::available_ports();
    match ports {
        Err(e) => {
            ui.label("Error finding serial ports!");
            eprintln!("{:?}", e);
        }
        Ok(ports) => 'port: {
            if ports.len() == 0 {
                ui.label("No serial devices found!");
                break 'port;
            }
            for port in ports.iter() {
                if port.port_name.eq_ignore_ascii_case(&app.selected_port) {
                    ui.label(format!("{} (Connected)", &port.port_name));
                } else {
                    ui.label(format!("{}", &port.port_name));
                }
            }
        }
    }
}

fn show_port_menu(app: &mut TemplateApp, ui: &mut egui::Ui) {
    let ports = tokio_serial::available_ports();
    ui.menu_button("Ports", |ui| match ports {
        Err(e) => {
            ui.label("No ports found!");
            println!("{:?}", e);
        }
        Ok(ports) => 'port: {
            if ports.len() == 0 {
                ui.label("No ports found!");
                break 'port;
            }
            for port in ports.iter() {
                if port.port_name.eq_ignore_ascii_case(&app.selected_port) {
                    if ui
                        .button(format!("{} | Disconnect", &port.port_name))
                        .clicked()
                    {
                        app.selected_port = "Disconnected".to_owned();
                        send_thread_msg(app.tx.clone(), ThreadMSG::Disconnect());
                    }
                } else {
                    if ui.button(port.port_name.clone()).clicked() {
                        app.selected_port = port.port_name.clone();
                        app.arduino
                            .lock()
                            .unwrap()
                            .connect(app.selected_port.clone(), 9600);
                        send_thread_msg(
                            app.tx.clone(),
                            ThreadMSG::Start((port.port_name.clone(), 9600)),
                        );
                    }
                }
            }
        }
    });
}
