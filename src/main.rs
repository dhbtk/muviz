mod analysis;
mod app;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use crate::app::run_app;
use app::Args;

const SAMPLE_RATE: u32 = 44_100;
const WINDOW_SIZE: usize = 2048;
const HOP_SIZE: usize = 512;
const RESAMPLE_CHUNK_SIZE: usize = 4096;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new(
                "info,symphonia_core=warn,symphonia_bundle_mp3=warn,wgpu=error,muviz=debug",
            )
        }))
        .init();
    let args = Args::parse();

    run_app(args);

    Ok(())
}
