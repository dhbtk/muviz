use anyhow::Result;
use realfft::{RealFftPlanner, RealToComplex};
use std::sync::Arc;

pub struct Stft {
    window_size: usize,
    hop_size: usize,
    hann: Vec<f32>,
    rfft: Arc<dyn RealToComplex<f32>>,
}

impl Stft {
    pub fn new(window_size: usize, hop_size: usize) -> Self {
        let mut planner = RealFftPlanner::<f32>::new();
        let rfft = planner.plan_fft_forward(window_size);

        let hann = (0..window_size)
            .map(|n| {
                let x = n as f32 / window_size as f32;
                (std::f32::consts::PI * 2.0 * x).sin().powi(2)
            })
            .collect();

        Self {
            window_size,
            hop_size,
            hann,
            rfft,
        }
    }

    pub fn window_size(&self) -> usize {
        self.window_size
    }

    pub fn hop_size(&self) -> usize {
        self.hop_size
    }

    pub fn num_bins(&self) -> usize {
        self.window_size / 2 + 1
    }

    pub fn process_frame(&self, signal: &[f32], start: usize) -> Result<Vec<f32>> {
        let mut input = vec![0.0f32; self.window_size];
        let mut output = self.rfft.make_output_vec();

        for i in 0..self.window_size {
            let sample = signal.get(start + i).copied().unwrap_or(0.0);
            input[i] = sample * self.hann[i];
        }

        self.rfft.process(&mut input, &mut output)?;

        let mag = output
            .iter()
            .map(|c| (c.re * c.re + c.im * c.im).sqrt())
            .collect();

        Ok(mag)
    }
}
