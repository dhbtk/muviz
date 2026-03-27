use crate::SAMPLE_RATE;

pub struct Compressor {
    pub threshold_db: f32,
    pub ratio: f32,
    pub attack: f32,
    pub release: f32,
    pub makeup_gain_db: f32,

    envelope: f32,
    dt: f32,
}

fn db_to_linear(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

fn linear_to_db(x: f32) -> f32 {
    20.0 * x.max(1e-6).log10()
}

impl Default for Compressor {
    fn default() -> Self {
        Self::new()
    }
}

impl Compressor {
    pub fn new() -> Self {
        Self {
            threshold_db: -12.0,
            ratio: 3.0,
            attack: 10.0,
            release: 2.0,
            makeup_gain_db: 3.0,
            envelope: 0.0,
            dt: 0.0,
        }
    }
    pub fn process_sample(&mut self, input: f32, dt: f32) -> f32 {
        let input_abs = input.abs();

        // envelope follower
        let coeff = if input_abs > self.envelope {
            self.attack
        } else {
            self.release
        };

        self.envelope += (input_abs - self.envelope) * coeff * dt;

        let env_db = linear_to_db(self.envelope);

        // ganho
        let gain_db = if env_db > self.threshold_db {
            let excess = env_db - self.threshold_db;
            -excess * (1.0 - 1.0 / self.ratio)
        } else {
            0.0
        };

        let gain = db_to_linear(gain_db + self.makeup_gain_db);

        input * gain
    }

    pub fn process_samples(&mut self, input: &[f32]) -> Vec<f32> {
        input
            .iter()
            .enumerate()
            .map(|(i, &sample)| self.process_sample(sample, i as f32 / SAMPLE_RATE as f32))
            .collect()
    }
}
