use camino::{Utf8Path, Utf8PathBuf};
use slotmap::basic::Iter;
use slotmap::{new_key_type, SlotMap};

#[derive(Default)]
pub struct Library(SlotMap<SongId, Song>);

impl Library {
    pub fn new() -> Self {
        Default::default()
    }

    /// Scans the given directory and inserts any songs it encounters.
    /// TODO (2023-02-03): What should we do with potential duplicates?
    /// TODO (2023-02-03): Error handling and logging.
    pub fn insert_from_directory(&mut self, directory: &Utf8Path) {
        if !directory.is_dir() {
            return;
        }

        let paths = directory.read_dir().expect("Could not read dir");

        for entry in paths.filter_map(|entry| entry.ok()) {
            let path = Utf8PathBuf::from_path_buf(entry.path().to_owned())
                .expect("Path is not a utf-8 path");

            if path.is_dir() {
                self.insert_from_directory(&path);
            } else if let Some(extension) = path.extension() {
                if sound::SUPPORTED_EXTENSIONS.contains(&extension) {
                    // Found a song.
                    let song = Song::from_file(path);
                    self.0.insert(song);
                }
            }
        }
    }

    pub fn songs(&self) -> Iter<SongId, Song> {
        self.0.iter()
    }
}

new_key_type! { pub struct SongId; }

pub struct Song {
    pub title: String,
    pub path: Utf8PathBuf,
}

impl Song {
    fn from_file(path: Utf8PathBuf) -> Self {
        // TODO (2023-02-03): Load metadata from the file?
        let title = path.file_stem().unwrap_or("Unnamed").replace('_', " ");
        Self { title, path }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_directory() {
        let mut library = Library::new();
        library.insert_from_directory(Utf8Path::new("../example_audio"));

        let songs: Vec<&Song> = library.songs().map(|(_id, song)| song).collect();

        assert_eq!(songs.len(), 2);
    }
}
