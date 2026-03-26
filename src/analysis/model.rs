use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Band {
    pub name: String,
    pub low_hz: f32,
    pub high_hz: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    pub target_sample_rate: u32,
    pub window_size: usize,
    pub hop_size: usize,
    pub bands: Vec<Band>,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            target_sample_rate: 44_100,
            window_size: 2048,
            hop_size: 512,
            bands: vec![
                Band {
                    name: "sub".into(),
                    low_hz: 20.0,
                    high_hz: 60.0,
                },
                Band {
                    name: "bass".into(),
                    low_hz: 60.0,
                    high_hz: 150.0,
                },
                Band {
                    name: "low_mid".into(),
                    low_hz: 150.0,
                    high_hz: 400.0,
                },
                Band {
                    name: "mid".into(),
                    low_hz: 400.0,
                    high_hz: 2_000.0,
                },
                Band {
                    name: "high_mid".into(),
                    low_hz: 2_000.0,
                    high_hz: 6_000.0,
                },
                Band {
                    name: "high".into(),
                    low_hz: 6_000.0,
                    high_hz: 16_000.0,
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameFeatures {
    pub time_s: f32,
    pub rms: f32,
    pub spectral_flux: f32,
    pub spectral_flatness: f32,
    pub band_energy: Vec<f32>,
    pub band_flux: Vec<f32>,
}

impl FrameFeatures {
    pub fn lerp(&self, other: &FrameFeatures, frac: f32) -> FrameFeatures {
        FrameFeatures {
            time_s: self.time_s + (other.time_s - self.time_s) * frac,
            rms: self.rms + (other.rms - self.rms) * frac,
            spectral_flux: self.spectral_flux + (other.spectral_flux - self.spectral_flux) * frac,
            spectral_flatness: self.spectral_flatness
                + (other.spectral_flatness - self.spectral_flatness) * frac,
            band_energy: self
                .band_energy
                .iter()
                .zip(other.band_energy.iter())
                .map(|(a, b)| a + (b - a) * frac)
                .collect(),
            band_flux: self
                .band_flux
                .iter()
                .zip(other.band_flux.iter())
                .map(|(a, b)| a + (b - a) * frac)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackAnalysis {
    pub sample_rate: u32,
    pub duration_s: f32,
    pub estimated_bpm: Option<f32>,
    pub beat_times_s: Vec<f32>,
    pub frame_len: usize,
    pub frames: Vec<FrameFeatures>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameplayFrame {
    pub time_s: f32,

    // "lanes" principais
    pub lane_left: f32,
    pub lane_center: f32,
    pub lane_right: f32,

    // intensidade geral (densidade de gameplay)
    pub energy: f32,

    // eventos (spawn)
    pub event: f32,

    // textura (ruído/percussivo)
    pub texture: f32,

    // sincronização rítmica
    pub beat_strength: f32,
    pub frame: FrameFeatures,
}

impl GameplayFrame {
    pub fn normalize(&mut self) {
        self.lane_left /= 100_000.;
        self.lane_center /= 60_000.;
        self.lane_right /= 30_000.;
        self.event /= 20_000.;
        self.texture /= 10_000.;
        self.lane_left = self.lane_left.min(1.0);
        self.lane_center = self.lane_center.min(1.0);
        self.lane_right = self.lane_right.min(1.0);
        self.event = self.event.min(1.0);
        self.texture = self.texture.min(1.0);
    }

    pub fn lerp(&self, other: &GameplayFrame, frac: f32) -> GameplayFrame {
        GameplayFrame {
            time_s: self.time_s + (other.time_s - self.time_s) * frac,
            lane_left: self.lane_left + (other.lane_left - self.lane_left) * frac,
            lane_center: self.lane_center + (other.lane_center - self.lane_center) * frac,
            lane_right: self.lane_right + (other.lane_right - self.lane_right) * frac,
            energy: self.energy + (other.energy - self.energy) * frac,
            event: self.event + (other.event - self.event) * frac,
            texture: self.texture + (other.texture - self.texture) * frac,
            beat_strength: self.beat_strength + (other.beat_strength - self.beat_strength) * frac,
            frame: self.frame.lerp(&other.frame, frac),
        }
    }
}
