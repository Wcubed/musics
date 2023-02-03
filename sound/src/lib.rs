#![deny(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

mod decoder;

use crate::decoder::{SymphoniaDecoder, TimeControl};
use camino::Utf8Path;
use rodio::{OutputStream, Sink};
use std::fs::File;
use std::time::Duration;
use symphonia::core::io::MediaSourceStream;

pub struct Player {
    /// Hard reference kept to prevent it from going out of scope.
    _stream: OutputStream,
    sink: Sink,
    /// [`None`] if there is no song queued or playing at the moment.
    time_control: Option<TimeControl>,
}

impl Default for Player {
    fn default() -> Self {
        // TODO (2023-02-03): Proper error handling.
        let (_stream, stream_handle) =
            OutputStream::try_default().expect("Could not get output stream");
        let sink = Sink::try_new(&stream_handle).expect("Could not create sink");

        Player {
            _stream,
            sink,
            time_control: None,
        }
    }
}

impl Player {
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads and plays the given file, replacing anything else that is currently playing.
    pub fn play_file(&mut self, path: &Utf8Path) {
        // TODO (2023-02-03): Error handling.
        let audio_file = File::open(path).expect("Could not open file");
        let stream = MediaSourceStream::new(Box::new(audio_file), Default::default());

        let decoder = SymphoniaDecoder::new(stream).expect("Creating decoder should work");
        self.time_control = Some(decoder.get_control());

        self.sink.empty();
        self.sink.append(decoder);
        self.sink.play();
    }

    /// Duration of the current song.
    /// Returns 0 if there is no current song.
    pub fn song_duration(&self) -> Duration {
        if let Some(control) = &self.time_control {
            control.total_duration()
        } else {
            Duration::from_secs(0)
        }
    }

    /// Seeks on the currently playing audio.
    /// Seeks to the end if the given time is longer than the total duration of the song.
    /// Does nothing if no song is playing or queued.
    pub fn seek(&self, time: Duration) {
        if let Some(control) = &self.time_control {
            control.seek(time)
        }
    }

    pub fn pause(&self) {
        self.sink.pause();
    }

    /// Resumes after having been paused. Does nothing if there is no song queued.
    pub fn resume(&self) {
        self.sink.play()
    }

    pub fn is_playing(&self) -> bool {
        !self.sink.is_paused() && !self.sink.empty()
    }

    /// Gives the elapsed time in the current song.
    /// Returns 0 if there is no song.
    pub fn time_elapsed(&self) -> Duration {
        if let Some(control) = &self.time_control {
            control.time_elapsed()
        } else {
            Duration::from_secs(0)
        }
    }

    pub fn empty(&self) -> bool {
        self.sink.empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Actually plays about a second of audio.
    /// TODO (2023-02-03): Figure out how to make this work without requiring an audio output.
    #[test]
    fn playing_test() {
        let mut player = Player::new();

        player.play_file(Utf8Path::new("../example_audio/blank_holes_snippet.ogg"));
        let duration = player.song_duration().as_secs();
        assert_eq!(duration, 17);

        // Test starting elapsed.
        std::thread::sleep(Duration::from_millis(100));
        let elapsed = player.time_elapsed().as_secs_f32();
        assert!(0. < elapsed && elapsed < 1.);

        // Test seeking into the middle of the song.
        player.seek(Duration::from_secs(10));
        std::thread::sleep(Duration::from_millis(100));
        let elapsed = player.time_elapsed().as_secs_f32();
        assert!(10.0 < elapsed && elapsed < 11.0);

        // Test seeking beyond the song.
        player.seek(Duration::from_secs(20));
        std::thread::sleep(Duration::from_millis(100));
        let elapsed = player.time_elapsed().as_secs();
        assert_eq!(elapsed, 17);
        assert!(player.empty(), "Player should be empty, because the song is only 17 seconds, and we asked it to seek beyond that.")
    }
}
