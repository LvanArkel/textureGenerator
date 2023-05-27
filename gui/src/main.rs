#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    // env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 640.0)),
        ..Default::default()
    };

    eframe::run_simple_native("Texture generator", options, move |ctx, _frame| {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.heading("Toolbar");
        });
        egui::SidePanel::right("configuration settings").show(ctx, |ui| {
            ui.heading("Right section")
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Main window");
        });
    })
}