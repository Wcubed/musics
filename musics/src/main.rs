use camino::Utf8Path;
use eframe::egui::{Context, ProgressBar, Sense, Widget};
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
}

impl MusicsApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        MusicsApp {
            player: Player::new(),
        }
    }
}

impl App for MusicsApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        if self.player.is_playing() {
            // If we are playing music, we need to update the UI periodically,
            // otherwise the song progress will not be shown.
            // And we would not realize that a song has finished playing.
            ctx.request_repaint_after(Duration::from_secs(1));
        }

        egui::TopBottomPanel::bottom("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
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

                let duration = self.player.song_duration();
                let elapsed = self.player.time_elapsed();
                let fraction = elapsed.as_secs_f32() / duration.as_secs_f32();

                let bar_response = ProgressBar::new(fraction).ui(ui);
                let response = bar_response.interact(Sense::click_and_drag());

                if let Some(interact_pos) = response.interact_pointer_pos() {
                    if response.drag_released() || response.clicked() {
                        let x_on_bar = interact_pos.x - response.rect.min.x;
                        let bar_width = response.rect.width();
                        let fraction = x_on_bar / bar_width;

                        let seek_duration = duration.as_secs_f32() * fraction;
                        self.player.seek(Duration::from_secs_f32(seek_duration));
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {});
    }
}
