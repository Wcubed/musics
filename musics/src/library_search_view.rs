use crate::library::{Library, SongId};
use eframe::egui;
use eframe::egui::{Color32, Key, Modifiers, RichText, Sense, Ui, Widget};

pub struct LibrarySearchView {
    /// String used to filter song titles.
    filter_string: String,
    /// Ordered list of found songs that we can display.
    found_songs: Vec<SongId>,
    /// Whether to show the list of songs or not.
    show_results: bool,
    /// The song in the list that will be added when pressing "enter".
    highlighted_song_index: usize,
}

impl LibrarySearchView {
    pub fn new() -> Self {
        LibrarySearchView {
            filter_string: String::new(),
            found_songs: Vec::new(),
            show_results: false,
            highlighted_song_index: 0,
        }
    }

    pub fn should_show_results(&self) -> bool {
        self.show_results
    }

    #[must_use]
    pub fn show_search_box(&mut self, ui: &mut Ui, library: &Library) -> LibraryViewCommand {
        let mut command = LibraryViewCommand::None;
        ui.label(format!("{} songs", library.song_count()));

        let previous_filter = self.filter_string.clone();
        let search_response = egui::TextEdit::singleline(&mut self.filter_string)
            .hint_text("Search library")
            .show(ui)
            .response;

        // Ctrl-F focuses on the search bar, and clears it's contents.
        // TODO: Have an app-wide method of detecting shortcuts?
        if ui.input().key_pressed(Key::F) && ui.input().modifiers.matches(Modifiers::COMMAND) {
            search_response.request_focus();
            self.filter_string = String::new();
        }

        let search_gained_focus = search_response.gained_focus();
        let lost_focus = search_response.lost_focus();

        if search_gained_focus {
            self.show_results = true;
        }

        let close_clicked = if self.show_results {
            ui.button("Close").clicked()
        } else {
            false
        };

        if close_clicked || (lost_focus && ui.input().key_pressed(Key::Escape)) {
            // Pressing "escape" drops focus from the search bar. It makes sense to make it act as if the "close" button is pressed.
            self.show_results = false;
            self.filter_string = String::new();
        }
        if lost_focus && ui.input().key_pressed(Key::Enter) {
            // Pressing "enter" automatically drops focus from the search bar.
            // But we also want to add the current selected search result to the playlist.
            if let Some(id) = self.found_songs.get(self.highlighted_song_index) {
                command = LibraryViewCommand::AddSongToPlaylist(*id)
            }
            // Because it auto-drops focus, but we actually don't want to drop focus at all, we need to re-aquire focus.
            search_response.request_focus();
            self.filter_string = String::new();
            self.highlighted_song_index = 0;
        }

        if search_response.has_focus() && ui.input().key_pressed(Key::ArrowUp) {
            self.highlighted_song_index = (self.highlighted_song_index - 1).max(0);
        } else if search_response.has_focus() && ui.input().key_pressed(Key::ArrowDown) {
            self.highlighted_song_index =
                (self.highlighted_song_index + 1).min(self.found_songs.len() - 1);
        }

        if previous_filter != self.filter_string || search_gained_focus {
            self.update_found_songs(library);
        }

        command
    }

    /// The search is case-insensitive.
    fn update_found_songs(&mut self, library: &Library) {
        let lowercase_filter = self.filter_string.to_lowercase();
        self.found_songs = library
            .songs()
            .filter(|(_, song)| song.title.to_lowercase().contains(&lowercase_filter))
            .map(|(id, _)| id)
            .collect();

        self.highlighted_song_index = 0;
    }

    #[must_use]
    pub fn show_search_results(&self, ui: &mut Ui, library: &Library) -> LibraryViewCommand {
        let mut command = LibraryViewCommand::None;

        ui.label(format!(
            "{} / {} songs",
            self.found_songs.len(),
            library.song_count()
        ));

        let text_style = egui::TextStyle::Body;
        let row_height = ui.text_style_height(&text_style);

        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show_rows(ui, row_height, self.found_songs.len(), |ui, row_range| {
                for (index, id, song) in self
                    .found_songs
                    .iter()
                    .enumerate()
                    .skip(row_range.start)
                    .take(row_range.len())
                    .filter_map(|(index, id)| library.get_song(*id).map(|song| (index, id, song)))
                {
                    let mut title_text = RichText::new(&song.title);
                    if self.highlighted_song_index == index {
                        title_text = title_text.color(Color32::LIGHT_GREEN);
                    }

                    let song_response = egui::Label::new(title_text)
                        .wrap(false)
                        .sense(Sense::click())
                        .ui(ui);
                    if song_response.clicked() {
                        command = LibraryViewCommand::AddSongToPlaylist(*id);
                    }
                }
            });

        command
    }
}

pub enum LibraryViewCommand {
    None,
    AddSongToPlaylist(SongId),
}
