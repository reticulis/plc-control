use std::{
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use eframe::{
    egui::{self, Layout},
    epaint::Vec2,
};
use strum::Display;

use crate::{device::Device, utils::CValue};

#[derive(Default)]
pub struct PlcControlWindow {
    selected_device: Arc<RwLock<CValue<String>>>,
    device: Arc<RwLock<Option<Device>>>,
    command: String,
    text_buffer: CValue<Vec<String>>,
    mode: Arc<RwLock<Mode>>,
    send_mode: DataMode,
}

#[derive(Default, Display, PartialEq)]
enum Mode {
    #[default]
    Connect,
    Disconnect,
}

#[derive(Copy, Clone, Default, PartialEq)]
pub enum DataMode {
    #[default]
    Hex,
    Ascii,
}

impl PlcControlWindow {
    pub fn new() -> PlcControlWindow {
        let plc = PlcControlWindow::default();

        let selected = plc.selected_device.clone();
        let mode = plc.mode.clone();
        let device = plc.device.clone();
         
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(500));

            let devices = Device::get_devices_list().unwrap();

            let selected = &mut *selected.write().unwrap();
            let mode = &mut *mode.write().unwrap();
            let device = &mut *device.write().unwrap();

            if !&selected.is_empty() && !devices.contains(selected) {
                **selected = String::new();
                *mode = Mode::Connect;
            } else if selected.is_empty() && !devices.is_empty() {
                **selected = devices[0].to_owned()
            } else if selected.is_changed() {
                *mode = Mode::Connect;
                
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
                let mode = &mut *self.mode.write().unwrap();
                egui::ComboBox::from_label("")
                    .selected_text(&**selected)
                    .show_ui(ui, |ui| {
                        for device in Device::get_devices_list().unwrap() {
                            ui.selectable_value(&mut **selected, device.clone(), device);
                        }
                    });

                let btn = ui.button(&*mode.to_string());
                if btn.clicked() {
                    if *mode == Mode::Connect {
                        if !selected.is_empty() {
                            let device = Device::new(&*selected).unwrap();

                            self.text_buffer
                                .push(format!("{} has been connected", **selected));

                            self.device = Some(device);
                            *mode = Mode::Disconnect
                        } else {
                            self.text_buffer
                                .push("Device isn't selected".to_string());
                        }
                    } else {
                        self.text_buffer
                            .push(format!("{} has been disconnected", **selected));
                        self.device = None;
                        *mode = Mode::Connect
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
                                    for text in &*self.text_buffer {
                                        ui.label(text);
                                    }
                                });
                                if self.text_buffer.is_changed() {
                                    ui.scroll_to_cursor(Some(eframe::emath::Align::BOTTOM));
                                }
                                //
                            });
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Command");
                ui.radio_value(&mut self.send_mode, DataMode::Hex, "Hex");
                ui.radio_value(&mut self.send_mode, DataMode::Ascii, "Ascii");
            });

            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.command);
                if ui.button("Send").clicked() && !self.command.is_empty() {
                    if let Some(device) = &mut self.device {
                        println!("{:?}", device.serial.name());
                        device.send(&self.command, self.send_mode).unwrap();

                        self.text_buffer.push(self.command.drain(..).collect());
                        self.text_buffer
                            .push(format!("{:?}", device.read().unwrap()))
                    }
                }
            });
        });
    }
}
