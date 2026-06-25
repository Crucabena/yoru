use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::ui;

pub fn cache_root() -> Result<PathBuf> {
    let base = dirs::cache_dir().context("Could not determine cache directory")?;
    let path = base.join("yoru").join("builds");
    fs::create_dir_all(&path)?;
    Ok(path)
}

pub fn build_dir(pkg_name: &str) -> Result<PathBuf> {
    let path = cache_root()?.join(pkg_name);
    fs::create_dir_all(&path)?;
    Ok(path)
}

pub fn clean(all: bool) -> Result<()> {
    let root = cache_root()?;

    if !root.exists() {
        ui::info("Build cache is already empty.");
        return Ok(());
    }

    if all {
        ui::header("Cleaning entire build cache");
        let entries: Vec<_> = fs::read_dir(&root)?.collect();
        let count = entries.len();
        fs::remove_dir_all(&root)?;
        fs::create_dir_all(&root)?;
        ui::success(&format!("Removed {} cached build(s).", count));
    } else {
        ui::header("Cleaning stale build cache (older than 7 days)");
        let threshold = SystemTime::now() - Duration::from_secs(7 * 24 * 3600);
        let mut removed = 0;

        for entry in fs::read_dir(&root)? {
            let entry = entry?;
            let meta = entry.metadata()?;
            let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);

            if modified < threshold {
                let path = entry.path();
                if path.is_dir() {
                    fs::remove_dir_all(&path)?;
                } else {
                    fs::remove_file(&path)?;
                }
                ui::info(&format!("Removed: {}", entry.file_name().to_string_lossy()));
                removed += 1;
            }
        }

        if removed == 0 {
            ui::info("No stale builds found.");
        } else {
            ui::success(&format!("Removed {} stale build(s).", removed));
        }
    }

    ui::info(&format!("Cache location: {}", root.display()));
    Ok(())
}

pub fn cache_size() -> String {
    let root = match cache_root() {
        Ok(r) => r,
        Err(_) => return "unknown".to_string(),
    };
    dir_size(&root)
        .map(|b| human_size(b))
        .unwrap_or_else(|_| "unknown".to_string())
}

fn dir_size(path: &Path) -> Result<u64> {
    let mut total = 0u64;
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let meta = entry.metadata()?;
            if meta.is_dir() {
                total += dir_size(&entry.path())?;
            } else {
                total += meta.len();
            }
        }
    }
    Ok(total)
}

fn human_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut i = 0;
    while size >= 1024.0 && i < UNITS.len() - 1 {
        size /= 1024.0;
        i += 1;
    }
    format!("{:.1} {}", size, UNITS[i])
}
