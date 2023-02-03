mod library;
mod playlist;

use crate::library::Library;
use crate::playlist::Playlist;
use camino::Utf8Path;
use eframe::egui::{Color32, Context, ProgressBar, RichText, Sense, Ui, Widget};
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
    playlist: Playlist,
}

impl MusicsApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // TODO (2023-02-03): There seems to be an issue, that when pixels_per_point is
        //    set to 1.0 (the default), the ui is instead shown at 2x the scale it should be.
        cc.egui_ctx.set_pixels_per_point(0.9999);

        let mut library = Library::new();
        library.insert_from_directory(Utf8Path::new("example_audio"));

        MusicsApp {
            player: Player::new(),
            library,
            playlist: Playlist::new(),
        }
    }

    fn play_next_song(&mut self) {
        if let Some(song) = self
            .playlist
            .select_next_song(true)
            .and_then(|id| self.library.get_song(id))
        {
            self.player.play_file(&song.path)
        }
    }

    fn play_previous_song(&mut self) {
        if let Some(song) = self
            .playlist
            .select_previous_song(true)
            .and_then(|id| self.library.get_song(id))
        {
            self.player.play_file(&song.path)
        }
    }

    fn play_song_by_playlists_index(&mut self, index: usize) {
        if let Some(song) = self
            .playlist
            .select_song(index)
            .and_then(|id| self.library.get_song(id))
        {
            self.player.play_file(&song.path)
        }
    }

    fn show_play_controls(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
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
                    self.play_next_song();
                } else {
                    self.player.resume();
                }
            }

            if ui.button(">>|").clicked() {
                self.play_next_song();
            }

            let mut volume = self.player.volume();
            egui::Slider::new(&mut volume, 0.0..=1.0)
                .fixed_decimals(1)
                .ui(ui);

            if volume != self.player.volume() {
                self.player.set_volume(volume);
            }

            if let Some(current_song) = self
                .playlist
                .current_song_id()
                .and_then(|id| self.library.get_song(id))
            {
                ui.label(&current_song.title);
            }
        });
    }

    fn show_playlist(&mut self, ui: &mut Ui) {
        let current_song = self.playlist.current_song_index();

        let mut maybe_song_index_to_play = None;

        for (index, id) in self.playlist.songs().enumerate() {
            if let Some(song) = self.library.get_song(*id) {
                let mut title_text = RichText::new(&song.title);

                if current_song == Some(index) {
                    title_text = title_text.color(Color32::LIGHT_BLUE);
                }

                if egui::Label::new(title_text)
                    .sense(Sense::click())
                    .ui(ui)
                    .clicked()
                {
                    maybe_song_index_to_play = Some(index);
                }
            }
        }

        if let Some(index) = maybe_song_index_to_play {
            self.play_song_by_playlists_index(index);
        }
    }
}

impl App for MusicsApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        if self.player.song_finished_playing() {
            self.play_next_song();
        }

        egui::TopBottomPanel::bottom("controls").show(ctx, |ui| {
            self.show_play_controls(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_playlist(ui);
            for (id, song) in self.library.songs() {
                if ui.button(&song.title).clicked() {
                    self.playlist.append_song(id);
                }
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
