use crate::analysis::model::FrameFeatures;
use tracing::instrument;

#[instrument(skip(frames))]
pub fn estimate_bpm_and_beats(
    frames: &[FrameFeatures],
    hop_size: usize,
    sample_rate: u32,
) -> (Option<f32>, Vec<f32>) {
    if frames.is_empty() {
        return (None, vec![]);
    }

    let seconds_per_frame = hop_size as f32 / sample_rate as f32;

    // 1. construir novelty (onset strength)
    let mut novelty: Vec<f32> = frames
        .iter()
        .map(|f| {
            let low_flux = f.band_flux.get(0).copied().unwrap_or(0.0)
                + f.band_flux.get(1).copied().unwrap_or(0.0);

            let mid_flux = f.band_flux.get(2).copied().unwrap_or(0.0)
                + f.band_flux.get(3).copied().unwrap_or(0.0);

            0.5 * low_flux + 0.3 * mid_flux + 0.2 * f.spectral_flux
        })
        .collect();

    // 2. suavizar
    smooth_ema(&mut novelty, 0.1);

    // 3. remover tendência lenta (high-pass simples)
    remove_local_mean(&mut novelty, 32);

    // 4. normalizar
    normalize(&mut novelty);

    // 5. estimar BPM via autocorrelação
    let bpm = estimate_bpm_autocorr(&novelty, seconds_per_frame);

    // 6. alinhar batidas
    let beats = if let Some(bpm) = bpm {
        align_beats(&novelty, bpm, seconds_per_frame)
    } else {
        vec![]
    };

    (bpm, beats)
}

fn smooth_ema(signal: &mut [f32], alpha: f32) {
    let mut prev = 0.0;
    for x in signal.iter_mut() {
        prev = alpha * *x + (1.0 - alpha) * prev;
        *x = prev;
    }
}

fn remove_local_mean(signal: &mut [f32], window: usize) {
    let len = signal.len();

    for i in 0..len {
        let start = i.saturating_sub(window);
        let end = (i + window).min(len - 1);

        let mean = signal[start..=end].iter().sum::<f32>()
            / (end - start + 1) as f32;

        signal[i] -= mean;
    }
}

fn normalize(signal: &mut [f32]) {
    let max = signal
        .iter()
        .cloned()
        .fold(0.0_f32, f32::max);

    if max > 0.0 {
        for x in signal.iter_mut() {
            *x /= max;
        }
    }
}

fn estimate_bpm_autocorr(
    novelty: &[f32],
    seconds_per_frame: f32,
) -> Option<f32> {
    let min_bpm = 70.0;
    let max_bpm = 180.0;

    let min_lag = (60.0 / max_bpm / seconds_per_frame) as usize;
    let max_lag = (60.0 / min_bpm / seconds_per_frame) as usize;

    if max_lag >= novelty.len() {
        return None;
    }

    let mut best_lag = 0;
    let mut best_score = 0.0;

    for lag in min_lag..max_lag {
        let mut score = 0.0;

        for i in 0..(novelty.len() - lag) {
            score += novelty[i] * novelty[i + lag];
        }

        if score > best_score {
            best_score = score;
            best_lag = lag;
        }
    }

    if best_lag == 0 {
        return None;
    }

    let period_s = best_lag as f32 * seconds_per_frame;
    Some(60.0 / period_s)
}

fn align_beats(
    novelty: &[f32],
    bpm: f32,
    seconds_per_frame: f32,
) -> Vec<f32> {
    let period_s = 60.0 / bpm;
    let period_frames = period_s / seconds_per_frame;

    let search_window = period_frames as usize;

    // encontrar melhor fase inicial
    let mut best_offset = 0;
    let mut best_score = 0.0;

    for offset in 0..search_window {
        let mut score = 0.0;
        let mut i = offset;

        while i < novelty.len() {
            score += novelty[i];
            i += search_window;
        }

        if score > best_score {
            best_score = score;
            best_offset = offset;
        }
    }

    // gerar batidas
    let mut beats = Vec::new();
    let mut i = best_offset as f32;

    while (i as usize) < novelty.len() {
        let idx = i.round() as usize;
        let time = idx as f32 * seconds_per_frame;
        beats.push(time);

        i += period_frames;
    }

    beats
}
