mod library;

use crate::library::Library;
use camino::{Utf8Path, Utf8PathBuf};
use eframe::egui::{Context, ProgressBar, Sense, Ui, Widget};
use eframe::{egui, App, Frame};
use sound::Player;
use std::time::Duration;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Musics",
        native_options,
        Box::new(|cc| Box::new(MusicsApp::new(cc))),
    );
}

struct MusicsApp {
    player: Player,
    library: Library,
}

impl MusicsApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut library = Library::new();
        library.insert_from_directory(Utf8Path::new("example_audio"));

        MusicsApp {
            player: Player::new(),
            library,
        }
    }

    fn play_next_song(&mut self) {
        self.player.play_file(Utf8Path::new(
            "example_audio/subfolder/dark_mystery_snippet.mp3",
        ));
    }

    fn play_previous_song(&mut self) {
        self.player
            .play_file(Utf8Path::new("example_audio/blank_holes_snippet.ogg"));
    }
}

impl App for MusicsApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        if self.player.song_finished_playing() {
            self.play_next_song();
        }

        egui::TopBottomPanel::bottom("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("|<<").clicked() {
                    self.play_previous_song();
                }

                if self.player.is_playing() {
                    if ui.button("||").clicked() {
                        self.player.pause();
                    }
                } else if ui.button(">").clicked() {
                    if self.player.empty() {
                        self.player
                            .play_file(Utf8Path::new("example_audio/blank_holes_snippet.ogg"));
                    } else {
                        self.player.resume();
                    }
                }

                if ui.button(">>|").clicked() {
                    self.play_next_song();
                }

                let duration = self.player.song_duration();
                let elapsed = self.player.time_elapsed();

                time_elapsed_widget(ui, duration, elapsed);

                let fraction = elapsed.as_secs_f32() / duration.as_secs_f32();

                let bar_response = ProgressBar::new(fraction).ui(ui);
                let response = bar_response.interact(Sense::click_and_drag());

                if let Some(interact_pos) = response.interact_pointer_pos() {
                    if response.drag_released() || response.clicked() {
                        let x_on_bar = interact_pos.x - response.rect.min.x;
                        let bar_width = response.rect.width();
                        let fraction = x_on_bar / bar_width;

                        let seek_duration = (duration.as_secs_f32() * fraction).max(0.);
                        self.player.seek(Duration::from_secs_f32(seek_duration));
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            for (_id, song) in self.library.songs() {
                ui.label(song.path.as_str());
            }
        });

        if self.player.is_playing() {
            // If we are playing music, we need to update the UI periodically,
            // otherwise the song progress will not be shown.
            // And we would not realize that a song has finished playing.
            ctx.request_repaint_after(Duration::from_secs(1));
        }
    }
}

fn time_elapsed_widget(ui: &mut Ui, duration: Duration, elapsed: Duration) {
    let text = format!(
        "{} / {}",
        duration_to_time_display(elapsed),
        duration_to_time_display(duration)
    );
    ui.label(text);
}

fn duration_to_time_display(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let remaining = total_seconds % 3600;
    let minutes = remaining / 60;
    let seconds = remaining % 60;

    format!("{hours}:{minutes:0>2}:{seconds:0>2}")
}
