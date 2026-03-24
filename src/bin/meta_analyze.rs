use anyhow::{Context, Result};
use clap::Parser;
use muviz::analysis::model::{AnalysisConfig, GameplayFrame};
use muviz::app::analyze::PersistedAnalysis;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// List of .analysis.json files
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

struct Stats {
    min: f32,
    max: f32,
    sum: f64,
    count: usize,
    values: Vec<f32>,
}

impl Stats {
    fn new() -> Self {
        Self {
            min: f32::INFINITY,
            max: f32::NEG_INFINITY,
            sum: 0.0,
            count: 0,
            values: Vec::new(),
        }
    }

    fn update(&mut self, val: f32) {
        if val < self.min {
            self.min = val;
        }
        if val > self.max {
            self.max = val;
        }
        self.sum += val as f64;
        self.count += 1;
        self.values.push(val);
    }

    fn avg(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            (self.sum / self.count as f64) as f32
        }
    }

    fn median(&mut self) -> f32 {
        if self.count == 0 {
            return 0.0;
        }
        self.values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = self.count / 2;
        if self.count % 2 == 0 {
            (self.values[mid - 1] + self.values[mid]) / 2.0
        } else {
            self.values[mid]
        }
    }

    fn push_values(&mut self, row: &mut Vec<String>) {
        let avg = self.avg();
        let med = self.median();
        row.push(format!("{:.4}", self.min));
        row.push(format!("{:.4}", self.max));
        row.push(format!("{:.4}", avg));
        row.push(format!("{:.4}", med));
    }
}

struct CollectionStats {
    rms: Stats,
    spectral_flux: Stats,
    spectral_flatness: Stats,
    band_energy: Vec<Stats>,
    band_flux: Vec<Stats>,
    lane_left: Stats,
    lane_center: Stats,
    lane_right: Stats,
    energy: Stats,
    event: Stats,
    texture: Stats,
    beat_strength: Stats,
}

impl CollectionStats {
    fn new() -> Self {
        Self {
            rms: Stats::new(),
            spectral_flux: Stats::new(),
            spectral_flatness: Stats::new(),
            band_energy: Vec::new(),
            band_flux: Vec::new(),
            lane_left: Stats::new(),
            lane_center: Stats::new(),
            lane_right: Stats::new(),
            energy: Stats::new(),
            event: Stats::new(),
            texture: Stats::new(),
            beat_strength: Stats::new(),
        }
    }

    fn update(&mut self, gameplay_frame: &GameplayFrame) {
        let f = &gameplay_frame.frame;
        self.rms.update(f.rms);
        self.spectral_flux.update(f.spectral_flux);
        self.spectral_flatness.update(f.spectral_flatness);

        if self.band_energy.len() < f.band_energy.len() {
            self.band_energy
                .resize_with(f.band_energy.len(), Stats::new);
        }
        for (i, &val) in f.band_energy.iter().enumerate() {
            self.band_energy[i].update(val);
        }

        if self.band_flux.len() < f.band_flux.len() {
            self.band_flux.resize_with(f.band_flux.len(), Stats::new);
        }
        for (i, &val) in f.band_flux.iter().enumerate() {
            self.band_flux[i].update(val);
        }

        self.lane_left.update(gameplay_frame.lane_left);
        self.lane_center.update(gameplay_frame.lane_center);
        self.lane_right.update(gameplay_frame.lane_right);
        self.energy.update(gameplay_frame.energy);
        self.event.update(gameplay_frame.event);
        self.texture.update(gameplay_frame.texture);
        self.beat_strength.update(gameplay_frame.beat_strength);
    }

    fn csv_header(&self) -> Vec<String> {
        let mut headers = vec!["filename".to_string()];

        let config = AnalysisConfig::default();

        for field in &["rms", "spectral_flux", "spectral_flatness"] {
            for suffix in &["min", "max", "avg", "med"] {
                headers.push(format!("{}_{}", field, suffix));
            }
        }

        for i in 0..self.band_energy.len() {
            let band_name = config
                .bands
                .get(i)
                .map(|b| b.name.as_str())
                .unwrap_or("unknown");
            for suffix in &["min", "max", "avg", "med"] {
                headers.push(format!("band_energy_{}_{}", band_name, suffix));
            }
        }

        for i in 0..self.band_flux.len() {
            let band_name = config
                .bands
                .get(i)
                .map(|b| b.name.as_str())
                .unwrap_or("unknown");
            for suffix in &["min", "max", "avg", "med"] {
                headers.push(format!("band_flux_{}_{}", band_name, suffix));
            }
        }

        for field in &[
            "lane_left",
            "lane_center",
            "lane_right",
            "energy",
            "event",
            "texture",
            "beat_strength",
        ] {
            for suffix in &["min", "max", "avg", "med"] {
                headers.push(format!("{}_{}", field, suffix));
            }
        }

        headers
    }

    fn csv_row(&mut self, filename: &str) -> Vec<String> {
        let mut row = vec![filename.to_string()];
        self.rms.push_values(&mut row);
        self.spectral_flux.push_values(&mut row);
        self.spectral_flatness.push_values(&mut row);
        for stat in &mut self.band_energy {
            stat.push_values(&mut row);
        }
        for stat in &mut self.band_flux {
            stat.push_values(&mut row);
        }
        self.lane_left.push_values(&mut row);
        self.lane_center.push_values(&mut row);
        self.lane_right.push_values(&mut row);
        self.energy.push_values(&mut row);
        self.event.push_values(&mut row);
        self.texture.push_values(&mut row);
        self.beat_strength.push_values(&mut row);
        row
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut global_stats = CollectionStats::new();
    let mut songs_stats = Vec::new();

    for file_path in &args.files {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
        let analysis: PersistedAnalysis = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON from file: {}", file_path.display()))?;

        let mut song_stats = CollectionStats::new();
        for frame in &analysis.frames {
            song_stats.update(frame);
            global_stats.update(frame);
        }

        songs_stats.push((file_path.to_string_lossy().to_string(), song_stats));
    }

    let mut wtr = csv::Writer::from_writer(io::stdout());

    if let Some((_, first_song)) = songs_stats.get(0) {
        wtr.write_record(first_song.csv_header())?;
    }

    for (name, mut stats) in songs_stats {
        wtr.write_record(stats.csv_row(&name))?;
    }

    if args.files.len() > 1 {
        wtr.write_record(global_stats.csv_row("_global"))?;
    }

    wtr.flush()?;

    Ok(())
}
