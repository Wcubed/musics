//! Combined from the following sources, then edited to suit this programs needs:
//! - https://github.com/tramhao/termusic/blob/master/src/player/rusty_backend/decoder/mod.rs
//! - https://github.com/RustAudio/rodio/blob/master/src/decoder/symphonia.rs

use rodio::decoder::DecoderError;
use rodio::Source;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use symphonia::{
    core::{
        audio::{AudioBufferRef, SampleBuffer, SignalSpec},
        codecs::{self, CodecParameters},
        errors::Error,
        formats::{FormatOptions, FormatReader, SeekMode, SeekTo},
        io::MediaSourceStream,
        meta::MetadataOptions,
        probe::Hint,
        units::{Time, TimeBase},
    },
    default::get_probe,
};

#[derive(Clone)]
pub struct DecoderControl {
    total_duration: Duration,
    time_elapsed: Arc<RwLock<Duration>>,

    /// If this is set to [`Some`], the next time the decoder is asked for a
    /// sample, it will first seek to the specified time, and then set this value to [`None`].
    seek_request: Arc<RwLock<Option<Duration>>>,
}

impl DecoderControl {
    pub fn seek(&self, elapsed: Duration) {
        *self.seek_request.write().unwrap() = Some(elapsed);
        *self.time_elapsed.write().unwrap() = elapsed;
    }

    pub fn get_seek_request(&self) -> Option<Duration> {
        self.seek_request.read().unwrap().clone()
    }

    pub fn time_elapsed(&self) -> Duration {
        self.time_elapsed.read().unwrap().clone()
    }

    pub fn total_duration(&self) -> Duration {
        self.total_duration
    }

    fn set_elapsed(&self, elapsed: Duration) {
        *self.time_elapsed.write().unwrap() = elapsed;
    }

    fn clear_seek_request(&self) {
        *self.seek_request.write().unwrap() = None;
    }
}

// Decoder errors are not considered fatal.
// The correct action is to just get a new packet and try again.
// But a decode error in more than 3 consecutive packets is fatal.
const MAX_DECODE_ERRORS: usize = 3;

pub struct SymphoniaDecoder {
    decoder: Box<dyn codecs::Decoder>,
    current_frame_offset: usize,
    format: Box<dyn FormatReader>,
    buffer: SampleBuffer<i16>,
    spec: SignalSpec,
    control: DecoderControl,
}

impl SymphoniaDecoder {
    pub fn new(mss: MediaSourceStream) -> Result<Self, DecoderError> {
        match Self::init(mss) {
            Err(e) => match e {
                Error::IoError(e) => Err(DecoderError::IoError(e.to_string())),
                Error::DecodeError(e) => Err(DecoderError::DecodeError(e)),
                Error::SeekError(_) => {
                    unreachable!("Seek errors should not occur during initialization")
                }
                Error::Unsupported(_) => Err(DecoderError::UnrecognizedFormat),
                Error::LimitError(e) => Err(DecoderError::LimitError(e)),
                Error::ResetRequired => Err(DecoderError::ResetRequired),
            },
            Ok(Some(decoder)) => Ok(decoder),
            Ok(None) => Err(DecoderError::NoStreams),
        }
    }

    /// Hands out controllers, so that other threads can get info / control this decoder while
    /// it is playing.
    pub fn get_control(&self) -> DecoderControl {
        self.control.clone()
    }

    fn init(mss: MediaSourceStream) -> symphonia::core::errors::Result<Option<Self>> {
        let mut probed = get_probe().format(
            &Hint::default(),
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )?;

        let track = match probed.format.default_track() {
            Some(stream) => stream,
            None => return Ok(None),
        };

        let mut decoder = symphonia::default::get_codecs().make(
            &track.codec_params,
            &codecs::DecoderOptions { verify: true },
        )?;

        let duration = Self::get_duration(&track.codec_params);

        let mut decode_errors: usize = 0;
        let decode_result = loop {
            let current_frame = probed.format.next_packet()?;
            match decoder.decode(&current_frame) {
                Ok(result) => break result,
                Err(e) => match e {
                    Error::DecodeError(_) => {
                        decode_errors += 1;
                        if decode_errors > MAX_DECODE_ERRORS {
                            return Err(e);
                        }
                    }
                    _ => return Err(e),
                },
            }
        };
        let spec = *decode_result.spec();
        let buffer = Self::get_buffer(decode_result, spec);

        let control = DecoderControl {
            total_duration: duration,
            time_elapsed: Arc::new(RwLock::new(Duration::from_secs(0))),
            seek_request: Arc::new(RwLock::new(None)),
        };

        Ok(Some(Self {
            decoder,
            current_frame_offset: 0,
            format: probed.format,
            buffer,
            spec,
            control,
        }))
    }

    fn get_duration(params: &CodecParameters) -> Duration {
        params.n_frames.map_or_else(
            || {
                // TODO: Return a nice error?
                // panic!("no n_frames");
                Duration::from_secs(99)
            },
            |n_frames| {
                params.time_base.map_or_else(
                    || {
                        // TODO: Return a nice error?
                        // panic!("no time base?");
                        Duration::from_secs(199)
                    },
                    |tb| {
                        let time = tb.calc_time(n_frames);
                        Duration::from_secs(time.seconds) + Duration::from_secs_f64(time.frac)
                    },
                )
            },
        )
    }

    fn get_buffer(decoded: AudioBufferRef<'_>, spec: SignalSpec) -> SampleBuffer<i16> {
        let duration = decoded.capacity() as u64;
        let mut buffer = SampleBuffer::<i16>::new(duration, spec);
        buffer.copy_interleaved_ref(decoded);
        buffer
    }

    fn seek(&mut self, time: Duration) -> Option<Duration> {
        let nanos_per_sec = 1_000_000_000.0;
        match self.format.seek(
            SeekMode::Coarse,
            SeekTo::Time {
                time: Time::new(
                    time.as_secs(),
                    f64::from(time.subsec_nanos()) / nanos_per_sec,
                ),
                track_id: None,
            },
        ) {
            Ok(seeked_to) => {
                let base = TimeBase::new(1, self.sample_rate());
                let time = base.calc_time(seeked_to.actual_ts);

                Some(Duration::from_millis(
                    time.seconds * 1000 + ((time.frac * 60. * 1000.).round() as u64),
                ))
            }
            Err(_) => None,
        }
    }
}

impl Source for SymphoniaDecoder {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.buffer.samples().len())
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn channels(&self) -> u16 {
        self.spec.channels.count() as u16
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.spec.rate
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        Some(self.control.total_duration)
    }
}

impl Iterator for SymphoniaDecoder {
    type Item = i16;

    #[inline]
    fn next(&mut self) -> Option<i16> {
        if let Some(duration) = self.control.get_seek_request() {
            self.seek(duration);
            self.control.clear_seek_request();
        }

        if self.current_frame_offset == self.buffer.len() {
            let mut decode_errors: usize = 0;
            let decoded = loop {
                match self.format.next_packet() {
                    Ok(packet) => match self.decoder.decode(&packet) {
                        Ok(decoded) => {
                            let ts = packet.ts();
                            if let Some(track) = self.format.default_track() {
                                if let Some(tb) = track.codec_params.time_base {
                                    let t = tb.calc_time(ts);
                                    self.control.set_elapsed(
                                        Duration::from_secs(t.seconds)
                                            + Duration::from_secs_f64(t.frac),
                                    );
                                }
                            }
                            break decoded;
                        }
                        Err(e) => match e {
                            Error::DecodeError(_) => {
                                decode_errors += 1;
                                if decode_errors > MAX_DECODE_ERRORS {
                                    return None;
                                }
                            }
                            _ => return None,
                        },
                    },
                    Err(_) => return None,
                }
            };
            self.spec = *decoded.spec();
            self.buffer = Self::get_buffer(decoded, self.spec);
            self.current_frame_offset = 0;
        }

        if self.buffer.samples().is_empty() {
            return None;
        }

        let sample = self.buffer.samples()[self.current_frame_offset];
        self.current_frame_offset += 1;

        Some(sample)
    }
}
