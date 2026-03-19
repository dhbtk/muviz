use crate::analysis::model::{FrameFeatures, GameplayFrame};

pub fn derive_gameplay(frames: &[FrameFeatures]) -> Vec<GameplayFrame> {
    let mut out = Vec::with_capacity(frames.len());

    for f in frames {
        let sub  = f.band_energy.get(0).copied().unwrap_or(0.0);
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
        let event =
            0.6 * f.spectral_flux +
                0.3 * (f.band_flux.get(0).unwrap_or(&0.0)
                    + f.band_flux.get(1).unwrap_or(&0.0)) +
                0.1 * f.band_flux.get(4).unwrap_or(&0.0);

        // Textura (ruído / distorção / pratos)
        let texture =
            0.7 * f.spectral_flatness +
                0.3 * high;

        // (placeholder até você ligar com beat.rs)
        let beat_strength = 0.0;

        out.push(GameplayFrame {
            time_s: f.time_s,
            lane_left,
            lane_center,
            lane_right,
            energy,
            event,
            texture,
            beat_strength,
        });
    }

    normalize_gameplay(&mut out);

    out
}

fn normalize_gameplay(frames: &mut [GameplayFrame]) {
    fn norm(vals: &mut [f32]) {
        let max = vals.iter().cloned().fold(0.0, f32::max);
        if max > 0.0 {
            for v in vals {
                *v /= max;
            }
        }
    }

    let mut left: Vec<_> = frames.iter().map(|f| f.lane_left).collect();
    let mut center: Vec<_> = frames.iter().map(|f| f.lane_center).collect();
    let mut right: Vec<_> = frames.iter().map(|f| f.lane_right).collect();
    let mut energy: Vec<_> = frames.iter().map(|f| f.energy).collect();
    let mut event: Vec<_> = frames.iter().map(|f| f.event).collect();
    let mut texture: Vec<_> = frames.iter().map(|f| f.texture).collect();

    norm(&mut left);
    norm(&mut center);
    norm(&mut right);
    norm(&mut energy);
    norm(&mut event);
    norm(&mut texture);

    for (i, f) in frames.iter_mut().enumerate() {
        f.lane_left = left[i];
        f.lane_center = center[i];
        f.lane_right = right[i];
        f.energy = energy[i];
        f.event = event[i];
        f.texture = texture[i];
    }
}
