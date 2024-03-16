/*
 *  ECE Briefing
 *  Data Collected:
 *      Revolutions
 *      Speed
 *      Direction of rotation
 */

use std::{fmt::Display, ops::Add, slice::Iter, time::Instant};

use egui::ScrollArea;
use egui_plot::{Line, Plot, PlotPoints};

use crate::arduino::PacketData;

#[derive(Clone, Debug)]
pub struct DataWindow {
    window_name: String,
    pub selected_data: usize,
    data_cap: usize,
    display_type: DisplayType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DisplayType {
    Graph,
    Text,
    NoDisplay,
}

impl DisplayType {
    pub fn iterator() -> Iter<'static, DisplayType> {
        static DISPLAYS: [DisplayType; 3] = [
            DisplayType::Graph,
            DisplayType::Text,
            DisplayType::NoDisplay,
        ];
        DISPLAYS.iter()
    }
}

impl Default for DataWindow {
    fn default() -> Self {
        Self {
            window_name: "Name Not Set!".to_owned(),
            selected_data: 1337420,
            display_type: DisplayType::NoDisplay,
            data_cap: 100,
        }
    }
}

impl DataWindow {
    pub fn new(window_name: String, selected_data: usize) -> Self {
        Self {
            window_name,
            selected_data,
            display_type: DisplayType::NoDisplay,
            data_cap: 100,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, data: &Vec<PacketData>, open: &mut bool) {
        let Self {
            window_name,
            selected_data,
            display_type,
            data_cap,
        } = self.clone();

        let mut window = egui::Window::new(window_name)
            .id(egui::Id::new(format!("{}", &self.selected_data)))
            .resizable(true)
            .open(open)
            .constrain(true)
            .title_bar(true)
            .collapsible(true);
        window.show(ctx, |ui| self.ui(ui, data));
    }

    fn ui(&mut self, ui: &mut egui::Ui, data: &Vec<PacketData>) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Data Name:");
                ui.text_edit_singleline(&mut self.window_name);
            });
            // Allow limiting of shown data, maybe it prevents running out of memory?
            ui.horizontal(|ui| {
                ui.label("Limit output:");
                ui.add(
                    egui::DragValue::new(&mut self.data_cap)
                        .speed(0.1)
                        .clamp_range(0.0..=f32::MAX),
                );
            });
            match data.get(0) {
                Some(dat) => ui.label(format!("Data Type: {}", dat.display_variant())),
                None => ui.label(format!("UNKNOWN TYPE!")),
            };
            ui.horizontal(|ui| {
                egui::ComboBox::from_label("Display Type")
                    .selected_text(format!("{:?}", self.display_type))
                    .show_ui(ui, |ui| {
                        for variant in DisplayType::iterator() {
                            ui.selectable_value(
                                &mut self.display_type,
                                variant.clone(),
                                format!("{variant:?}"),
                            );
                        }
                    });
                ui.end_row();
            });

            ui.separator();
            let data_2 = data.iter().rev().collect::<Vec<&PacketData>>();
            match self.display_type {
                DisplayType::Graph => match data[0] {
                    PacketData::Integer(_, _, _) | PacketData::Float(_, _, _) => {
                        plot_data(ui, &self.window_name, &mut self.data_cap.clone(), &data_2)
                    }
                    _ => {
                        ui.label("Graph not supported for the following data type!");
                    }
                },
                DisplayType::Text => {
                    ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                        let tmp_string: String = get_text(data);
                        ui.label(&tmp_string);
                    });
                }
                _ => (),
            }
        });
    }
}

fn plot_data(ui: &mut egui::Ui, window_name: &String, cap: &mut usize, data: &Vec<&PacketData>) {
    let mut plot = Plot::new(format!("{}", window_name));
    if cap > &mut data.len() {
        *cap = data.len()
    }
    plot.show(ui, |plot_ui| {
        let points: PlotPoints = data[..*cap]
            .iter()
            .map(|d| match *d {
                PacketData::Integer(d1, _, t1) => [t1.elapsed().as_secs_f64(), *d1 as f64],
                PacketData::Float(d1, _, t1) => [t1.elapsed().as_secs_f64(), *d1 as f64],
                _ => [0.0, 0.0],
            })
            .collect();
        let line = Line::new(points);
        plot_ui.line(line);
    });
}

fn get_text(data: &Vec<PacketData>) -> String {
    let mut tmp = String::new();
    for d in data {
        tmp = tmp
            + &match d {
                PacketData::Float(d, _, t) => format_text(d, t),
                PacketData::String(d, _, t) => format_text(d, t),
                PacketData::Integer(d, _, t) => format_text(d, t),
                _ => "Unknown Data Type\n".to_string(),
            };
    }
    tmp
}

fn format_text<D: Display>(data: D, time: &Instant) -> String {
    format!("[{:>4.2}] {}\n", time.elapsed().as_secs_f32(), &data)
}
