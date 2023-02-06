use camino::Utf8PathBuf;
use eframe::egui;
use eframe::egui::Context;
use rfd::FileDialog;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
// Auto fill properties with their defaults if they are missing.
// Allows properties to be added to future versions without breaking the configs.
#[serde(default)]
pub struct Config {
    pub library_directory: Utf8PathBuf,
}

pub struct ConfigView {
    show_window: bool,
}

impl ConfigView {
    pub fn new() -> Self {
        ConfigView { show_window: false }
    }

    pub fn show(&mut self, ctx: &Context, config: &mut Config) {
        egui::Window::new("Config")
            .open(&mut self.show_window)
            .collapsible(false)
            .show(ctx, |ui| {
                egui::Grid::new("config_grid")
                    .striped(true)
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label(format!("Library: {}", config.library_directory.as_str()));
                        if ui.button("Select library directory").clicked() {
                            if let Some(dir) = FileDialog::new().pick_folder() {
                                // TODO: let the user know when an error occured, with a pop-up or something like that.
                                config.library_directory =
                                    Utf8PathBuf::from_path_buf(dir).expect("Not a utf-8 path.");
                            }
                        }
                        ui.end_row();
                        ui.label("(Needs a restart to take effect)");
                    });
            });
    }

    pub fn open_window(&mut self) {
        self.show_window = true;
    }
}
