pub mod analysis;
pub mod app;

pub const SAMPLE_RATE: u32 = 44_100;
pub const WINDOW_SIZE: usize = 2048;
pub const HOP_SIZE: usize = 512;
pub const RESAMPLE_CHUNK_SIZE: usize = 4096;
