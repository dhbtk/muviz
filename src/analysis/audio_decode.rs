use anyhow::{Context, Result, anyhow};
use std::{fs::File, path::Path};
use tracing::instrument;

use symphonia::core::codecs::CodecRegistry;
use symphonia::core::{
    audio::{AudioBufferRef, Signal},
    codecs::DecoderOptions,
    errors::Error as SymphoniaError,
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};
use symphonia::default::get_probe;
use symphonia_adapter_libopus::OpusDecoder;

#[derive(Debug, Clone)]
pub struct DecodedAudio {
    pub sample_rate: u32,
    pub channels: usize,
    pub samples_interleaved: Vec<f32>,
}

pub fn codec_registry() -> CodecRegistry {
    let mut codecs = CodecRegistry::new();
    codecs.register_all::<OpusDecoder>();
    codecs
}

#[instrument(skip(path))]
pub fn decode_audio_file(path: &Path) -> Result<DecodedAudio> {
    let file = File::open(path).with_context(|| format!("failed to open {:?}", path))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        hint.with_extension(ext);
    }

    let probed = get_probe().format(
        &hint,
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;

    let mut format = probed.format;

    let track = format
        .default_track()
        .ok_or_else(|| anyhow!("no default audio track found"))?;

    let codec_params = &track.codec_params;
    let sample_rate = codec_params
        .sample_rate
        .ok_or_else(|| anyhow!("unknown sample rate"))?;
    let channels = codec_params
        .channels
        .ok_or_else(|| anyhow!("unknown channel layout"))?
        .count();

    let mut decoder = codec_registry().make(codec_params, &DecoderOptions::default())?;

    let mut samples_interleaved = Vec::<f32>::new();

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(_)) => break,
            Err(err) => return Err(err.into()),
        };

        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(err) => return Err(err.into()),
        };

        match decoded {
            AudioBufferRef::F32(buf) => {
                let frames = buf.frames();
                for i in 0..frames {
                    for ch in 0..channels {
                        samples_interleaved.push(buf.chan(ch)[i]);
                    }
                }
            }
            other => {
                let spec = *other.spec();
                let duration = other.capacity() as u64;
                let mut sample_buf =
                    symphonia::core::audio::SampleBuffer::<f32>::new(duration, spec);
                sample_buf.copy_interleaved_ref(other);
                samples_interleaved.extend_from_slice(sample_buf.samples());
            }
        }
    }

    Ok(DecodedAudio {
        sample_rate,
        channels,
        samples_interleaved,
    })
}

#[instrument(skip(samples))]
pub fn interleaved_to_mono(samples: &[f32], channels: usize) -> Vec<f32> {
    if channels == 1 {
        return samples.to_vec();
    }

    samples
        .chunks_exact(channels)
        .map(|frame| frame.iter().copied().sum::<f32>() / channels as f32)
        .collect()
}
