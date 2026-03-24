use anyhow::Result;
use clap::Parser;

use muviz::app::analyze::perform_analysis;
use muviz::app::run_app;
use muviz::app::Args;

fn main() -> Result<()> {
    let args = Args::parse();
    if args.analyze_only {
        tracing_subscriber::fmt::init();
        perform_analysis(&args).expect("analysis failed");
        return Ok(());
    }

    run_app(args);

    Ok(())
}
