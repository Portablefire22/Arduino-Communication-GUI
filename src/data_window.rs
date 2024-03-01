/*
 *  ECE Briefing
 *  Data Collected:
 *      Revolutions
 *      Speed
 *      Direction of rotation
 */

use std::slice::Iter;

use egui::ScrollArea;
use egui_plot::{Line, Plot, PlotPoints};

use crate::arduino::PacketData;

#[derive(Clone, Debug)]
pub struct DataWindow {
    window_name: String,
    pub selected_data: usize,
    display_type: DisplayType,
    pub open: bool,
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
            open: true,
        }
    }
}

impl DataWindow {
    pub fn new(window_name: String, selected_data: usize) -> Self {
        Self {
            window_name,
            selected_data,
            display_type: DisplayType::NoDisplay,
            open: true,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, data: &Vec<PacketData>) {
        let Self {
            window_name,
            selected_data,
            display_type,
            open,
        } = self.clone();

        let mut window = egui::Window::new(window_name)
            .id(egui::Id::new(format!("{}", &self.selected_data)))
            .resizable(true)
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
            ui.label(format!("Data Type: {}", data[0].display_variant()));
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
            match self.display_type {
                DisplayType::Graph => match data[0] {
                    PacketData::Integer(packet_data, _, packet_time) => {
                        let mut plot = Plot::new(format!("{}", self.window_name));
                        plot.show(ui, |plot_ui| {
                            let points: PlotPoints = data
                                .iter()
                                .map(|d| match *d {
                                    PacketData::Integer(d1, _, t1) => {
                                        [t1.elapsed().as_secs_f64(), d1 as f64]
                                    }
                                    _ => [0.0, 0.0],
                                })
                                .collect();
                            let line = Line::new(points);
                            plot_ui.line(line);
                        });
                    }
                    _ => {
                        ui.label("Graph not supported for the following data type!");
                    }
                },
                DisplayType::Text => {
                    ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                        for dat in data {
                            match dat {
                                PacketData::String(packet_data, _, packet_time) => {
                                    ui.label(format!(
                                        "[{:>4.2}] {}",
                                        packet_time.elapsed().as_secs_f32(),
                                        packet_data
                                    ));
                                }
                                PacketData::Integer(packet_data, _, packet_time) => {
                                    ui.label(format!(
                                        "[{:>4.2}] {}",
                                        packet_time.elapsed().as_secs_f32(),
                                        packet_data
                                    ));
                                }
                                _ => (),
                            }
                        }
                    });
                }
                _ => (),
            }
        });
    }
}
