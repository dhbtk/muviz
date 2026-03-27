use crate::analysis::audio_decode::codec_registry;
use crate::analysis::compressor::Compressor;
use crate::SAMPLE_RATE;
use bevy::audio::{AddAudioSource, Decodable, Source};
use bevy::prelude::*;
use std::fs::File;
use std::path::Path;
use std::time::Duration;
use symphonia::core::audio::{AudioBufferRef, SampleBuffer, Signal};
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;

#[derive(Asset, TypePath)]
pub struct SongAsset {
    pub path: String,
}

pub struct SongDecoder {
    reader: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    sample_buf: Option<SampleBuffer<f32>>,
    sample_idx: usize,
    channels: u16,
    sample_rate: u32,
    total_duration: Option<Duration>,
    left_chan_compressor: Compressor,
    right_chan_compressor: Compressor,
}

pub struct PlaybackPlugin;

impl Plugin for PlaybackPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_source::<SongAsset>()
            .init_asset::<SongAsset>();
    }
}

impl SongDecoder {
    pub fn new(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file = File::open(path)?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let hint = Hint::new();
        let probed = get_probe().format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )?;

        let reader = probed.format;
        let track = reader
            .default_track()
            .ok_or_else(|| anyhow::anyhow!("no default audio track found"))?;

        let codec_params = &track.codec_params;
        let sample_rate = codec_params.sample_rate.unwrap_or(44100);
        let channels = codec_params.channels.map(|c| c.count() as u16).unwrap_or(2);
        let total_duration = codec_params
            .n_frames
            .map(|f| Duration::from_secs_f64(f as f64 / sample_rate as f64));

        let decoder = codec_registry().make(codec_params, &DecoderOptions::default())?;

        Ok(Self {
            reader,
            decoder,
            sample_buf: None,
            sample_idx: 0,
            channels,
            sample_rate,
            total_duration,
            left_chan_compressor: Compressor::new(),
            right_chan_compressor: Compressor::new(),
        })
    }

    pub fn skip_to_duration(&mut self, duration: Duration) {
        let _ = self.reader.seek(
            symphonia::core::formats::SeekMode::Accurate,
            symphonia::core::formats::SeekTo::Time {
                time: symphonia::core::units::Time::from(duration.as_secs_f64()),
                track_id: None,
            },
        );
        self.decoder.reset();
        self.sample_buf = None;
        self.sample_idx = 0;
    }

    fn refill_buffer(&mut self) -> bool {
        loop {
            let packet = match self.reader.next_packet() {
                Ok(packet) => packet,
                Err(symphonia::core::errors::Error::IoError(_)) => return false,
                Err(_) => return false,
            };

            let decoded = match self.decoder.decode(&packet) {
                Ok(decoded) => decoded,
                Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
                Err(_) => return false,
            };
            let sample_buf = if let AudioBufferRef::F32(mut buf) = decoded {
                let buf = buf.to_mut();
                let (left, right) = buf.chan_pair_mut(0, 1);
                for sample in left.iter_mut() {
                    *sample = self
                        .left_chan_compressor
                        .process_sample(*sample, 1.0 / SAMPLE_RATE as f32);
                }
                for sample in right.iter_mut() {
                    *sample = self
                        .right_chan_compressor
                        .process_sample(*sample, 1.0 / SAMPLE_RATE as f32);
                }

                let spec = *buf.spec();
                let duration = buf.capacity() as u64;
                let mut sample_buf = SampleBuffer::<f32>::new(duration, spec);
                sample_buf
                    .copy_interleaved_ref(AudioBufferRef::F32(std::borrow::Cow::Borrowed(buf)));
                sample_buf
            } else {
                return false;
            };

            self.sample_buf = Some(sample_buf);
            self.sample_idx = 0;
            return true;
        }
    }
}

impl Iterator for SongDecoder {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if (self.sample_buf.is_none()
            || self.sample_idx >= self.sample_buf.as_ref().unwrap().samples().len())
            && !self.refill_buffer()
        {
            return None;
        }

        let sample = self.sample_buf.as_ref().unwrap().samples()[self.sample_idx];
        self.sample_idx += 1;
        Some(sample)
    }
}

impl Source for SongDecoder {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        self.total_duration
    }
}

impl Decodable for SongAsset {
    type DecoderItem = <Self::Decoder as Iterator>::Item;
    type Decoder = SongDecoder;

    fn decoder(&self) -> Self::Decoder {
        SongDecoder::new(&self.path).expect("Failed to create SongDecoder")
    }
}
