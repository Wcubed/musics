use crate::library::SongId;
use std::slice::Iter;

// TODO (2023-02-03): Add tests.

#[derive(Default)]
pub struct Playlist {
    songs: Vec<SongId>,
    current_song_index: Option<usize>,
}

impl Playlist {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn songs(&self) -> Iter<SongId> {
        self.songs.iter()
    }

    pub fn append_song(&mut self, song: SongId) {
        self.songs.push(song)
    }

    pub fn select_next_song(&mut self, wrap: bool) -> Option<SongId> {
        // TODO (2023-02-03): Refactor this set of if statements.
        self.current_song_index = if self.songs.is_empty() {
            None
        } else if let Some(index) = self.current_song_index {
            if index + 1 >= self.songs.len() {
                if wrap {
                    Some(0)
                } else {
                    None
                }
            } else {
                Some(index + 1)
            }
        } else if !self.songs.is_empty() {
            Some(0)
        } else {
            // Shouldn't get here.
            None
        };

        self.current_song_index
            .and_then(|index| self.songs.get(index))
            .cloned()
    }

    pub fn select_previous_song(&mut self, wrap: bool) -> Option<SongId> {
        // TODO (2023-02-03): Refactor this set of if statements.
        self.current_song_index = if self.songs.is_empty() {
            None
        } else if let Some(index) = self.current_song_index {
            if index == 0 {
                if wrap {
                    Some(self.songs.len() - 1)
                } else {
                    None
                }
            } else {
                Some(index - 1)
            }
        } else if !self.songs.is_empty() {
            Some(self.songs.len() - 1)
        } else {
            // Shouldn't get here.
            None
        };

        self.current_song_index
            .and_then(|index| self.songs.get(index))
            .cloned()
    }

    pub fn current_song_index(&self) -> Option<usize> {
        self.current_song_index
    }
}
