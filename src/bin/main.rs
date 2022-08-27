//! Demo app for egui

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// When compiling natively:
fn main() {
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "rgeometry playground",
        options,
        Box::new(|_cc| Box::new(rgeometry_playground::MyApp::default())),
    );
}
