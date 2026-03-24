use crate::analysis::model::{GameplayFrame, TrackAnalysis};

pub fn derive_gameplay(analysis: &TrackAnalysis) -> Vec<GameplayFrame> {
    let frames = &analysis.frames;
    let mut out = Vec::with_capacity(frames.len());

    for f in frames {
        let sub = f.band_energy.first().copied().unwrap_or(0.0);
        let bass = f.band_energy.get(1).copied().unwrap_or(0.0);
        let low_mid = f.band_energy.get(2).copied().unwrap_or(0.0);
        let mid = f.band_energy.get(3).copied().unwrap_or(0.0);
        let high_mid = f.band_energy.get(4).copied().unwrap_or(0.0);
        let high = f.band_energy.get(5).copied().unwrap_or(0.0);

        // Lanes (bem importantes visualmente)
        let lane_left = sub + bass;
        let lane_center = low_mid + mid;
        let lane_right = high_mid + high;

        // Energia global
        let energy = f.rms;

        // Eventos (ataques)
        let event = 0.6 * f.spectral_flux
            + 0.3 * (f.band_flux.first().unwrap_or(&0.0) + f.band_flux.get(1).unwrap_or(&0.0))
            + 0.1 * f.band_flux.get(4).unwrap_or(&0.0);

        // Textura (ruído / distorção / pratos)
        let texture = 0.7 * f.spectral_flatness + 0.3 * high;

        let beat_strength = gaussian_beat_strength(f.time_s, &analysis.beat_times_s, 0.06);

        out.push(GameplayFrame {
            time_s: f.time_s,
            lane_left,
            lane_center,
            lane_right,
            energy,
            event,
            texture,
            beat_strength,
            frame: f.clone(),
        });
    }

    normalize_gameplay(&mut out);

    out
}

fn gaussian_beat_strength(time_s: f32, beat_times_s: &[f32], sigma_s: f32) -> f32 {
    if beat_times_s.is_empty() {
        return 0.0;
    }

    let nearest_dist = beat_times_s
        .iter()
        .map(|b| (time_s - *b).abs())
        .fold(f32::INFINITY, f32::min);

    (-nearest_dist.powi(2) / (2.0 * sigma_s.powi(2))).exp()
}

fn normalize_gameplay(frames: &mut [GameplayFrame]) {
    for f in frames {
        f.normalize();
    }
}
