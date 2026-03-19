mod analysis;

use anyhow::Result;
use clap::Parser;
use std::{fs, path::PathBuf};

use analysis::analyze_mono_pcm;
use analysis::audio_decode::{decode_audio_file, interleaved_to_mono};
use analysis::model::AnalysisConfig;
use analysis::util::resample_mono;

#[derive(Debug, Parser)]
struct Args {
    input: PathBuf,

    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(long, default_value_t = 44_100)]
    sample_rate: u32,

    #[arg(long, default_value_t = 2048)]
    window_size: usize,

    #[arg(long, default_value_t = 512)]
    hop_size: usize,

    #[arg(long, default_value_t = 4096)]
    resample_chunk_size: usize,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let decoded = decode_audio_file(&args.input)?;
    let mono = interleaved_to_mono(&decoded.samples_interleaved, decoded.channels);

    let mono = resample_mono(
        &mono,
        decoded.sample_rate,
        args.sample_rate,
        args.resample_chunk_size,
    )?;

    let mut config = AnalysisConfig::default();
    config.target_sample_rate = args.sample_rate;
    config.window_size = args.window_size;
    config.hop_size = args.hop_size;

    let analysis = analyze_mono_pcm(&mono, &config)?;

    let out_path = args.output.unwrap_or_else(|| {
        let mut p = args.input.clone();
        p.set_extension("analysis.json");
        p
    });

    let json = serde_json::to_string_pretty(&analysis)?;
    fs::write(&out_path, json)?;

    println!("wrote analysis to {}", out_path.display());
    Ok(())
}
