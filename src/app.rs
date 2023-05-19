use anyhow::Result;
use serialport::{DataBits, Parity, StopBits};

use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use eframe::{
    egui::{self, ComboBox, DragValue, Layout, Ui},
    epaint::Vec2,
};
use strum::Display;

use crate::{
    device::Device,
    error::{PResult, PlcError},
    utils::CValue,
};

#[derive(Default)]
pub struct PlcControlWindow {
    selected_device: Arc<Mutex<String>>,
    device: Option<Device>,
    command: String,
    text_buffer: CValue<Vec<String>>,
    mode: Arc<Mutex<Mode>>,
    send_mode: DataMode,
    preferences: Preferences,
    preferences_window: bool,
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

#[derive(Default)]
pub struct Preferences {
    pub baud_rate: u32,
    pub data_bits: DataBits,
    pub parity: Parity,
    pub stop_bits: StopBits,
}

impl PlcControlWindow {
    pub fn new() -> PlcControlWindow {
        let plc = PlcControlWindow {
            preferences: Preferences {
                baud_rate: 115_000,
                ..Default::default()
            },
            ..Default::default()
        };

        let selected = plc.selected_device.clone();
        let mode = plc.mode.clone();

        thread::spawn(move || update_device_list(selected, mode));

        plc
    }
}

impl eframe::App for PlcControlWindow {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if let Err(err) = build_ui(self, ctx) {
            self.text_buffer.push(err.to_string())
        }
    }
}

fn build_ui(app: &mut PlcControlWindow, ctx: &eframe::egui::Context) -> Result<(), PlcError> {
    egui::Window::new("Preferences")
        .collapsible(false)
        .open(&mut app.preferences_window)
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(10., 10.);
            ui.horizontal(|ui| {
                ui.label("Baud rate: ");
                ui.add(DragValue::new(&mut app.preferences.baud_rate).speed(0.))
            });

            ui.horizontal(|ui| {
                ui.label("Data bits: ");
                create_combobox(
                    ui,
                    &mut app.preferences.data_bits,
                    &[
                        DataBits::Five,
                        DataBits::Six,
                        DataBits::Seven,
                        DataBits::Eight,
                    ],
                );
            });

            ui.horizontal(|ui| {
                ui.label("Stop bits: ");
                create_combobox(
                    ui,
                    &mut app.preferences.stop_bits,
                    &[StopBits::One, StopBits::Two],
                );
            });

            ui.horizontal(|ui| {
                ui.label("Parity: ");
                create_combobox(
                    ui,
                    &mut app.preferences.parity,
                    &[Parity::None, Parity::Odd, Parity::Even],
                )
            });
        });

    egui::CentralPanel::default()
        .show(ctx, |ui| -> PResult<()> {
            ui.spacing_mut().item_spacing = Vec2::new(10., 10.);

            ui.horizontal(|ui| -> PResult<()> {
                ui.label("Select device:");
                let selected = &mut *app.selected_device.lock().unwrap();
                let mode = &mut *app.mode.lock().unwrap();

                ui.horizontal(|ui| -> PResult<()> {
                    if *mode != Mode::Connect {
                        ui.set_enabled(false)
                    }

                    egui::ComboBox::from_label("")
                        .selected_text(&**selected)
                        .show_ui(ui, |ui| -> PResult<()> {
                            for device in Device::get_devices_list()? {
                                ui.selectable_value(&mut *selected, device.clone(), device);
                            }

                            Ok(())
                        });

                    Ok(())
                })
                .inner?;

                let btn = ui.button(&*mode.to_string());
                if btn.clicked() {
                    if *mode == Mode::Connect {
                        if !selected.is_empty() {
                            let device = Device::new(&*selected, &app.preferences)?;

                            app.text_buffer
                                .push(format!("{} has been connected", *selected));

                            app.device = Some(device);
                            *mode = Mode::Disconnect
                        } else {
                            app.text_buffer.push("Device isn't selected".to_string());
                        }
                    } else {
                        app.text_buffer
                            .push(format!("{} has been disconnected", *selected));
                        app.device = None;
                        *mode = Mode::Connect
                    }
                }

                if ui.button("Preferences").clicked() {
                    app.preferences_window = true;
                }

                Ok(())
            })
            .inner?;

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
                                    for text in &*app.text_buffer {
                                        ui.label(text);
                                    }
                                });
                                if app.text_buffer.is_changed() {
                                    ui.scroll_to_cursor(None);
                                }
                            });
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Command");
                ui.radio_value(&mut app.send_mode, DataMode::Hex, "Hex");
                ui.radio_value(&mut app.send_mode, DataMode::Ascii, "Ascii");
            });

            ui.horizontal(|ui| -> PResult<()> {
                let mode = &mut *app.mode.lock().unwrap();

                ui.text_edit_singleline(&mut app.command);
                if ui.button("Send").clicked()
                    && !app.command.is_empty()
                    && *mode == Mode::Disconnect
                {
                    if let Some(device) = &mut app.device {
                        device.send(&app.command, app.send_mode)?;

                        let command =
                            format!("Command: {}", app.command.drain(..).collect::<String>());
                        let received_data = format!("Received data: {:?}", device.read()?);

                        app.text_buffer.push(command);
                        app.text_buffer.push(received_data)
                    }
                }

                Ok(())
            })
            .inner?;

            Ok(())
        })
        .inner?;
    Ok(())
}

fn update_device_list(selected: Arc<Mutex<String>>, mode: Arc<Mutex<Mode>>) -> Result<()> {
    loop {
        thread::sleep(Duration::from_millis(500));

        let devices = Device::get_devices_list()?;

        let selected = &mut *selected.lock().unwrap();
        let mode = &mut *mode.lock().unwrap();

        if !&selected.is_empty() && !devices.contains(selected) {
            *selected = String::new();
            *mode = Mode::Connect;
        } else if selected.is_empty() && !devices.is_empty() {
            *selected = devices[0].to_owned()
        }
    }
}

fn create_combobox<'a, T: ToString + PartialEq + Copy>(
    ui: &mut Ui,
    data: &'a mut T,
    array: &'a [T],
) {
    ComboBox::new(array.len(), "")
        .selected_text(data.to_string())
        .show_ui(ui, |ui| {
            for a in array {
                ui.selectable_value(data, *a, a.to_string());
            }
        });
}
