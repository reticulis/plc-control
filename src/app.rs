use std::{
    sync::{Arc, RwLock},
    thread, time::Duration,
};

use eframe::{
    egui::{self, Layout},
    epaint::Vec2,
};
use strum::Display;

use crate::device::{self, Device};

#[derive(Default)]
pub struct PlcControlWindow {
    selected_device: Arc<RwLock<String>>,
    device: Option<Device>,
    command: String,
    text_buffor: Vec<String>,
    mode: Mode,
}

#[derive(Default, Display, PartialEq)]
enum Mode {
    #[default]
    Connect,
    Disconnect,
}

impl PlcControlWindow {
    pub fn new() -> PlcControlWindow {
        let plc = PlcControlWindow::default();
        

        let device = plc.selected_device.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(500));
            let devices = device::get_devices_list().unwrap();
            let selected = &mut *device.write().unwrap();
            if !devices.contains(selected) && !selected.is_empty() {
                *selected = String::new();
            } else if selected.is_empty() {
                *selected = devices.get(0).map(String::to_owned).unwrap_or_default();
            }
        });

        plc
    }
}

impl eframe::App for PlcControlWindow {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(10., 10.);

            ui.horizontal(|ui| {
                ui.label("Select device:");
                let selected = &mut *self.selected_device.write().unwrap();
                egui::ComboBox::from_label("")
                    .selected_text(&*selected)
                    .show_ui(ui, |ui| {
                        for device in device::get_devices_list().unwrap() {
                            ui.selectable_value(selected, device.clone(), device);
                        }
                    });
                    
                let btn = ui.button(&self.mode.to_string());
                if btn.clicked() {
                    if self.mode == Mode::Connect {
                        if !selected.is_empty() {
                            let device = Device::new(selected).unwrap();
                            
                            self.device = Some(device);
                            self.mode = Mode::Disconnect
                        } else {
                            self.text_buffor.push("Device isn't selected".to_string());
                        }
                    } else {
                        self.device = None;
                        self.mode = Mode::Connect
                    }
                }
            });

            ui.with_layout(Layout::top_down(eframe::emath::Align::Center), |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(0., 0.);
                egui::ScrollArea::vertical()
                    .max_height(ui.available_height() - 50.)
                    .show(ui, |ui| {
                        egui::Frame::none()
                            .fill(egui::Color32::from_black_alpha(28))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.set_min_height(ui.available_height());
                                ui.with_layout(Layout::top_down(eframe::emath::Align::Min), |ui| {
                                    for text in &self.text_buffor {
                                        ui.label(text);
                                    }
                                });
                            });
                    });
            });
            ui.label("Command");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.command);
                if ui.button("Send").clicked() {
                    self.text_buffor.push(self.command.drain(..).collect());
                }
            });
        });
    }
}
