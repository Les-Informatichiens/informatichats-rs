use eframe::egui;
use log::{error, info};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::ffmpeg;

pub(crate) struct Informatichat {
    picked_path: Option<String>,
    recording_status: Arc<Mutex<String>>,
}

impl Informatichat {
    pub(crate) fn new() -> Self {
        Self {
            picked_path: None,
            recording_status: Arc::new(Mutex::new("Idle".to_string())),
        }
    }

    pub(crate) fn start_recording(&self) {
        let status = self.recording_status.clone();

        thread::spawn(move || {
            let mut status_guard = status.lock().unwrap();
            *status_guard = "Recording in progress...".to_string();
            drop(status_guard);

            // Simulate recording process
            let result = Self::record();

            let mut status_guard = status.lock().unwrap();
            match result {
                Ok(_) => {
                    *status_guard = "Recording completed successfully.".to_string();
                    info!("Recording completed successfully.");
                }
                Err(e) => {
                    *status_guard = format!("Recording failed: {:?}", e);
                    error!("Recording failed: {:?}", e);
                }
            }
        });
    }

    pub(crate) fn record() -> Result<(), Box<dyn std::error::Error>> {
        info!("Recording has started");

        // Replace this with actual FFmpeg recording logic
        std::thread::sleep(std::time::Duration::from_secs(2));

        Ok(())
    }
}

impl eframe::App for Informatichat {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Drag-and-drop files onto the window!");

            if ui.button("Open file…").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.picked_path = Some(path.display().to_string());
                }
            }

            if ui.button("Start Recording").clicked() {
                self.start_recording();
            }

            if let Some(picked_path) = &self.picked_path {
                ui.horizontal(|ui| {
                    ui.label("Picked file:");
                    ui.monospace(picked_path);
                });
            }

            let status = self.recording_status.lock().unwrap();
            ui.horizontal(|ui| {
                ui.label("Status:");
                ui.monospace(&*status);
            });
        });

        ctx.request_repaint();
    }
}

