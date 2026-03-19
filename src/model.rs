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
                Band { name: "sub".into(),      low_hz: 20.0,   high_hz: 60.0 },
                Band { name: "bass".into(),     low_hz: 60.0,   high_hz: 150.0 },
                Band { name: "low_mid".into(),  low_hz: 150.0,  high_hz: 400.0 },
                Band { name: "mid".into(),      low_hz: 400.0,  high_hz: 2_000.0 },
                Band { name: "high_mid".into(), low_hz: 2_000.0, high_hz: 6_000.0 },
                Band { name: "high".into(),     low_hz: 6_000.0, high_hz: 16_000.0 },
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackAnalysis {
    pub sample_rate: u32,
    pub duration_s: f32,
    pub estimated_bpm: Option<f32>,
    pub beat_times_s: Vec<f32>,
    pub frame_len: usize,
    pub frames: Vec<FrameFeatures>,
}
