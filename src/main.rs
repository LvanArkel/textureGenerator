#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use texture_generator::NodeGraphExample;

fn main() {
    use eframe::egui::Visuals;

    eframe::run_native(
        "Egui node graph example",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(Visuals::dark());
            #[cfg(feature = "persistence")]
            {
                Box::new(NodeGraphExample::new(cc))
            }
            #[cfg(not(feature = "persistence"))]
            Box::<NodeGraphExample>::default()
        }),
    )
    .expect("Failed to run native example");
}