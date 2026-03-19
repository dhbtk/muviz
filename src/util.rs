use anyhow::{bail, Result};
use audioadapter::Adapter;
use audioadapter_buffers::direct::{SequentialSlice, SequentialSliceOfVecs};
use rubato::{FixedSync, Resampler};

pub fn resample_mono(
    input: &[f32],
    in_rate: u32,
    out_rate: u32,
    chunk_size: usize,
) -> Result<Vec<f32>> {
    if in_rate == out_rate {
        return Ok(input.to_vec());
    }

    let ratio = out_rate as f64 / in_rate as f64;

    let mut resampler = rubato::Fft::<f32>::new(
        in_rate as usize,
        out_rate as usize,
        chunk_size,
        1,
        1,
        FixedSync::Input
    )?;

    let mut output = Vec::new();
    let mut pos = 0;

    while pos < input.len() {
        let end = (pos + chunk_size).min(input.len());
        let mut chunk = input[pos..end].to_vec();

        if chunk.len() < chunk_size {
            chunk.resize(chunk_size, 0.0);
        }

        let waves_in = SequentialSlice::new(&chunk, 1, chunk.len())?;
        let waves_out = resampler.process(&waves_in, 0, None)?;

        if waves_out.channels() != 1 {
            bail!("unexpected number of channels after resample");
        }

        output.extend_from_slice(&waves_out.take_data());
        pos += chunk_size;
    }

    let expected_len = ((input.len() as f64) * ratio).round() as usize;
    output.truncate(expected_len);
    Ok(output)
}
