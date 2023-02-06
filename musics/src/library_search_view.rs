use crate::library::{Library, SongId};
use eframe::egui;
use eframe::egui::{Key, Modifiers, Sense, Ui, Widget};

pub struct LibrarySearchView {
    /// String used to filter song titles.
    filter_string: String,
    /// Ordered list of found songs that we can display.
    found_songs: Vec<SongId>,
    /// Whether to show the list of songs or not.
    show_results: bool,
}

impl LibrarySearchView {
    pub fn new() -> Self {
        LibrarySearchView {
            filter_string: String::new(),
            found_songs: Vec::new(),
            show_results: false,
        }
    }

    pub fn should_show_results(&self) -> bool {
        self.show_results
    }

    pub fn show_search_box(&mut self, ui: &mut Ui, library: &Library) {
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

        if previous_filter != self.filter_string || search_gained_focus {
            self.update_found_songs(library);
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
    }

    /// The search is case-insensitive.
    fn update_found_songs(&mut self, library: &Library) {
        let lowercase_filter = self.filter_string.to_lowercase();
        self.found_songs = library
            .songs()
            .filter(|(_, song)| song.title.to_lowercase().contains(&lowercase_filter))
            .map(|(id, _)| id)
            .collect();
    }

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
                for (id, song) in self
                    .found_songs
                    .iter()
                    .skip(row_range.start)
                    .take(row_range.len())
                    .filter_map(|id| library.get_song(*id).map(|song| (id, song)))
                {
                    let song_response = egui::Label::new(&song.title)
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
