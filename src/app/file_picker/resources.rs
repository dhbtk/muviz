use bevy::prelude::*;
use directories::UserDirs;
use std::path::{Path, PathBuf};

#[derive(Resource)]
pub struct FilePicker {
    pub current_dir: PathBuf,
    pub selected_file: Option<PathBuf>,
    pub entries: Vec<DirectoryEntry>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DirectoryEntry {
    pub path: PathBuf,
    pub is_dir: bool,
}

impl PartialOrd for DirectoryEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DirectoryEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.is_dir && !other.is_dir {
            return std::cmp::Ordering::Less;
        }
        if !self.is_dir && other.is_dir {
            return std::cmp::Ordering::Greater;
        }
        self.path.cmp(&other.path)
    }
}

impl Default for FilePicker {
    fn default() -> Self {
        let mut instance = Self {
            current_dir: UserDirs::new()
                .map(|dirs| dirs.home_dir().to_owned())
                .unwrap_or_else(|| PathBuf::from(Path::new("/"))),
            selected_file: None,
            entries: Vec::new(),
        };
        let _ = instance.refresh();
        instance
    }
}

const PLAYABLE_EXTENSIONS: [&str; 5] = ["mp3", "ogg", "wav", "flac", "opus"];

impl FilePicker {
    pub fn refresh(&mut self) -> Result {
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(&self.current_dir)?.flatten() {
            let path = entry.path();
            if hf::is_hidden(&path)? {
                continue;
            }
            if !path.is_dir() && !self.is_playable_file(&path) {
                continue;
            }
            entries.push(DirectoryEntry {
                is_dir: path.is_dir(),
                path,
            });
        }
        self.entries = entries;
        self.entries.sort();
        self.selected_file = None;
        Ok(())
    }

    pub fn select_file(&mut self, path: PathBuf) {
        self.selected_file = Some(path);
    }

    fn is_playable_file(&self, path: &Path) -> bool {
        path.extension().is_some_and(|ext| {
            let ext = ext.to_str().unwrap_or("").to_lowercase();
            PLAYABLE_EXTENSIONS.contains(&ext.as_str())
        })
    }
}
