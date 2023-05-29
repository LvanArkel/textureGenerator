#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use gui::NodeGraphExample;

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

// fn main() -> Result<(), eframe::Error> {
//     let options = eframe::NativeOptions {
//         initial_window_size: Some(egui::vec2(800.0, 640.0)),
//         ..Default::default()
//     };

//     eframe::run_simple_native("Texture generator", options, move |ctx, _frame| {
//         egui::TopBottomPanel::top("Toolbar").show(ctx, |ui| {
//             ui.heading("Toolbar");
//         });
//         egui::SidePanel::right("Configuration settings").show(ctx, |ui| {
//             ui.heading("Right section")
//         });
//         egui::CentralPanel::default().show(ctx, |ui| {
//             ui.heading("Main window");
//         });
//     })
// }