#![deny(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

mod decoder;

use crate::decoder::SymphoniaDecoder;
use rodio::{OutputStream, Sink, Source};
use std::fs::File;
use symphonia::core::io::MediaSourceStream;

pub fn symphonia_tryout() {
    let audio_file =
        File::open("../example_audio/blank_holes_snippet.ogg").expect("Could not open file");
    let stream = MediaSourceStream::new(Box::new(audio_file), Default::default());

    let decoder = SymphoniaDecoder::new(stream).expect("Creating decoder should work");

    println!(
        "Duration: {}",
        decoder.total_duration().unwrap().as_secs_f32()
    );

    let (_stream, stream_handle) =
        OutputStream::try_default().expect("Could not get output stream");
    let sink = Sink::try_new(&stream_handle).expect("Could not create stream handle");

    sink.append(decoder);
    sink.play();
    std::thread::sleep(std::time::Duration::from_secs(1));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        symphonia_tryout()
    }
}
