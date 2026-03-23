use crate::analysis::model::Band;

pub fn rms(frame: &[f32]) -> f32 {
    if frame.is_empty() {
        return 0.0;
    }
    let mean_sq = frame.iter().map(|x| x * x).sum::<f32>() / frame.len() as f32;
    mean_sq.sqrt()
}

pub fn spectral_flux(curr_mag: &[f32], prev_mag: &[f32]) -> f32 {
    curr_mag
        .iter()
        .zip(prev_mag.iter())
        .map(|(c, p)| (c - p).max(0.0))
        .sum()
}

pub fn spectral_flatness(mag: &[f32]) -> f32 {
    let eps = 1e-12_f32;
    if mag.is_empty() {
        return 0.0;
    }

    let n = mag.len() as f32;
    let geo = (mag.iter().map(|x| (x + eps).ln()).sum::<f32>() / n).exp();
    let arith = mag.iter().sum::<f32>() / n;

    geo / (arith + eps)
}

pub fn band_energies(
    mag: &[f32],
    sample_rate: u32,
    window_size: usize,
    bands: &[Band],
) -> Vec<f32> {
    let bin_hz = sample_rate as f32 / window_size as f32;

    bands
        .iter()
        .map(|band| {
            mag.iter()
                .enumerate()
                .filter_map(|(bin, &m)| {
                    let freq = bin as f32 * bin_hz;
                    if freq >= band.low_hz && freq < band.high_hz {
                        Some(m * m)
                    } else {
                        None
                    }
                })
                .sum::<f32>()
        })
        .collect()
}

pub fn positive_deltas(curr: &[f32], prev: &[f32]) -> Vec<f32> {
    curr.iter()
        .zip(prev.iter())
        .map(|(c, p)| (c - p).max(0.0))
        .collect()
}
