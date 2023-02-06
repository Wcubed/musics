mod config;
mod library;
mod library_search_view;
mod playlist;

use crate::config::{Config, ConfigView};
use crate::library::Library;
use crate::library_search_view::{LibrarySearchView, LibraryViewCommand};
use crate::playlist::Playlist;
use eframe::egui::{
    Color32, Context, CursorIcon, Id, ProgressBar, RichText, Sense, Ui, Visuals, Widget,
};
use eframe::{egui, App, Frame, IconData, Storage};
use sound::Player;
use std::time::Duration;

fn main() {
    // TODO (2023-02-06): Package the icon with the executable?
    let icon = image::open("icon.png")
        .expect("Failed to open icon path")
        .to_rgba8();
    let (icon_width, icon_height) = icon.dimensions();

    let native_options = eframe::NativeOptions {
        icon_data: Some(IconData {
            rgba: icon.into_raw(),
            width: icon_width,
            height: icon_height,
        }),
        ..Default::default()
    };
    eframe::run_native(
        "Musics",
        native_options,
        Box::new(|cc| Box::new(MusicsApp::new(cc))),
    );
}

struct MusicsApp {
    config: Config,
    config_view: ConfigView,
    player: Player,
    library: Library,
    library_search_view: LibrarySearchView,
    playlist: Playlist,
    /// Records whether the user is currently dragging a song in the playlist.
    dragged_playlist_index: Option<usize>,
}

impl MusicsApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let config: Config = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };

        let visuals = Visuals::dark();
        cc.egui_ctx.set_visuals(visuals);

        // TODO (2023-02-03): There seems to be an issue, that when pixels_per_point is
        //    set to 1.0 (the default), the ui is instead shown at 2x the scale it should be.
        cc.egui_ctx.set_pixels_per_point(0.9999);

        let mut library = Library::new();
        if config.library_directory != "" {
            library.insert_from_directory(&config.library_directory);
        }

        MusicsApp {
            config,
            config_view: ConfigView::new(),
            player: Player::new(),
            library,
            library_search_view: LibrarySearchView::new(),
            playlist: Playlist::new(),
            dragged_playlist_index: None,
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

        if !ui.memory().is_anything_being_dragged() {
            self.dragged_playlist_index = None
        }

        if self.dragged_playlist_index.is_some() {
            ui.output().cursor_icon = CursorIcon::Grabbing;
        }

        let mut maybe_song_index_to_play = None;
        let mut move_dragged_song_to_target_index = None;
        let mut remove_song = None;

        // `interact_size` is the size of 1 button.
        let row_height = ui.spacing().interact_size.y;

        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show_rows(
                ui,
                row_height,
                self.playlist.song_count(),
                |ui, row_range| {
                    for (index, id) in self
                        .playlist
                        .songs()
                        .enumerate()
                        .skip(row_range.start)
                        .take(row_range.len())
                    {
                        if let Some(song) = self.library.get_song(*id) {
                            ui.horizontal(|ui| {
                                let id_source = "playlist_drag";
                                let drag_id = Id::new(id_source).with(index);

                                let drag_rect = ui.label("::").rect;
                                let drag_response = ui.interact(drag_rect, drag_id, Sense::drag());

                                if drag_response.drag_started() {
                                    self.dragged_playlist_index = Some(index);
                                } else if drag_response.hovered()
                                    && !ui.memory().is_anything_being_dragged()
                                {
                                    ui.output().cursor_icon = CursorIcon::Grab;
                                }

                                if let Some(dragged_index) = self.dragged_playlist_index {
                                    if dragged_index != index {
                                        if let Some(last_pos) = ui.input().pointer.hover_pos() {
                                            if last_pos.y >= drag_rect.top()
                                                && last_pos.y <= drag_rect.bottom()
                                            {
                                                move_dragged_song_to_target_index = Some(index);
                                            }
                                        }
                                    }
                                }

                                if ui.button("X").clicked() {
                                    remove_song = Some(index);
                                }

                                let mut title_text = RichText::new(&song.title);

                                if self.dragged_playlist_index == Some(index) {
                                    title_text = title_text.color(Color32::LIGHT_GREEN);
                                } else if current_song == Some(index) {
                                    title_text = title_text.color(Color32::LIGHT_BLUE);
                                }

                                if egui::Label::new(title_text)
                                    .wrap(false)
                                    .sense(Sense::click())
                                    .ui(ui)
                                    .clicked()
                                {
                                    maybe_song_index_to_play = Some(index);
                                }
                            });
                        }
                    }
                },
            );

        if let Some(index) = maybe_song_index_to_play {
            self.play_song_by_playlists_index(index);
        }

        if let (Some(source_index), Some(target_index)) = (
            self.dragged_playlist_index,
            move_dragged_song_to_target_index,
        ) {
            self.playlist
                .switch_songs_by_index(source_index, target_index);
            self.dragged_playlist_index = Some(target_index);
        }

        if let Some(remove_index) = remove_song {
            let removed_current_song = self.playlist.remove_song_by_index(remove_index);

            if removed_current_song {
                if let Some(index) = self.playlist.current_song_index() {
                    let was_playing = self.player.is_playing();

                    self.play_song_by_playlists_index(index);

                    if !was_playing {
                        self.player.pause();
                    }
                } else {
                    self.player.stop();
                }
            }
        }
    }

    fn handle_library_view_command(&mut self, command: LibraryViewCommand) {
        match command {
            LibraryViewCommand::None => {}
            LibraryViewCommand::AddSongToPlaylist(id) => {
                self.playlist.append_song(id);
            }
        }
    }
}

impl App for MusicsApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        if self.player.song_finished_playing() {
            self.play_next_song();
        }

        self.config_view.show(ctx, &mut self.config);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Config").clicked() {
                    self.config_view.open_window();
                }

                ui.separator();

                let command = self.library_search_view.show_search_box(ui, &self.library);
                self.handle_library_view_command(command);
            });
        });

        egui::TopBottomPanel::bottom("controls").show(ctx, |ui| {
            self.show_play_controls(ui);
        });

        if self.library_search_view.should_show_results() {
            egui::SidePanel::right("search_results").show(ctx, |ui| {
                let command = self
                    .library_search_view
                    .show_search_results(ui, &self.library);
                self.handle_library_view_command(command);
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_playlist(ui);
        });

        if self.player.is_playing() {
            // If we are playing music, we need to update the UI periodically,
            // otherwise the song progress will not be shown.
            // And we would not realize that a song has finished playing.
            ctx.request_repaint_after(Duration::from_secs(1));
        }
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.config);
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
