use anyhow::Result;
use tracing::instrument;
use features::{band_energies, positive_deltas, rms, spectral_flatness, spectral_flux};
use fft::Stft;
use model::{AnalysisConfig, FrameFeatures, TrackAnalysis};
use beat::estimate_bpm_and_beats;
use crate::analysis::audio_decode::interleaved_to_mono;
use crate::analysis::util::resample_mono;
use crate::{HOP_SIZE, RESAMPLE_CHUNK_SIZE, SAMPLE_RATE, WINDOW_SIZE};

pub mod audio_decode;
pub mod features;
pub mod fft;
pub mod model;
pub mod util;
pub mod beat;
pub mod gameplay;

#[instrument(skip(samples, config))]
pub fn analyze_mono_pcm(samples: &[f32], config: &AnalysisConfig) -> Result<TrackAnalysis> {
    let stft = Stft::new(config.window_size, config.hop_size);

    let mut frames = Vec::new();
    let mut prev_mag = vec![0.0f32; stft.num_bins()];
    let mut prev_band_energy = vec![0.0f32; config.bands.len()];

    let duration_s = samples.len() as f32 / config.target_sample_rate as f32;

    let mut start = 0usize;
    while start < samples.len() {
        let end = (start + config.window_size).min(samples.len());
        let time_s = start as f32 / config.target_sample_rate as f32;

        let mag = stft.process_frame(samples, start)?;
        let band_energy =
            band_energies(&mag, config.target_sample_rate, config.window_size, &config.bands);
        let band_flux = positive_deltas(&band_energy, &prev_band_energy);

        let frame_rms = rms(&samples[start..end]);
        let flux = spectral_flux(&mag, &prev_mag);
        let flatness = spectral_flatness(&mag);

        frames.push(FrameFeatures {
            time_s,
            rms: frame_rms,
            spectral_flux: flux,
            spectral_flatness: flatness,
            band_energy: band_energy.clone(),
            band_flux: band_flux.clone(),
        });

        prev_mag = mag;
        prev_band_energy = band_energy;

        start += config.hop_size;
    }

    let (bpm, beats) = estimate_bpm_and_beats(
        &frames,
        config.hop_size,
        config.target_sample_rate,
    );

    Ok(TrackAnalysis {
        sample_rate: config.target_sample_rate,
        duration_s,
        estimated_bpm: bpm,
        beat_times_s: beats,
        frame_len: frames.len(),
        frames,
    })
}

#[instrument]
pub fn analyze_file(path: &std::path::Path) -> Result<TrackAnalysis> {
    let decoded = audio_decode::decode_audio_file(path)?;
    let mono = interleaved_to_mono(&decoded.samples_interleaved, decoded.channels);

    let mono = resample_mono(
        &mono,
        decoded.sample_rate,
        SAMPLE_RATE,
        RESAMPLE_CHUNK_SIZE,
    )?;

    let mut config = AnalysisConfig::default();
    config.target_sample_rate = SAMPLE_RATE;
    config.window_size = WINDOW_SIZE;
    config.hop_size = HOP_SIZE;

    let analysis = analyze_mono_pcm(&mono, &config)?;
    Ok(analysis)   
}
