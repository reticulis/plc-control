#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use app::PlcControlWindow;
use eframe::NativeOptions;

mod app;
mod device;
mod error;
mod utils;

fn main() {
    let native_options = NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(400., 200.)),
        ..Default::default()
    };
    eframe::run_native(
        "Plc Control",
        native_options,
        Box::new(|_| Box::new(PlcControlWindow::new())),
    )
    .unwrap();
}
