use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::cache;
use crate::ui;

const AUR_RPC: &str = "https://aur.archlinux.org/rpc/v5";
const AUR_CLONE_BASE: &str = "https://aur.archlinux.org";
const USER_AGENT: &str = concat!("yoru/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct AurPackage {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub num_votes: Option<u64>,
    pub popularity: Option<f64>,
    pub out_of_date: Option<u64>,
    pub depends: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct AurResponse {
    results: Vec<AurPackage>,
    error: Option<String>,
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .expect("Failed to build HTTP client")
}

pub async fn search(query: &str) -> Result<Vec<AurPackage>> {
    let url = format!("{}/search/{}?by=name-desc", AUR_RPC, query);
    let body: AurResponse = client()
        .get(&url)
        .send()
        .await
        .context("Failed to reach AUR RPC")?
        .json()
        .await
        .context("Failed to parse AUR response")?;

    if let Some(err) = body.error {
        bail!("AUR error: {}", err);
    }

    let mut results = body.results;
    results.sort_by(|a, b| {
        b.popularity
            .unwrap_or(0.0)
            .partial_cmp(&a.popularity.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(results)
}

pub async fn info(name: &str) -> Result<Option<AurPackage>> {
    let url = format!("{}/info?arg[]={}", AUR_RPC, name);
    let body: AurResponse = client()
        .get(&url)
        .send()
        .await
        .context("Failed to reach AUR RPC")?
        .json()
        .await
        .context("Failed to parse AUR response")?;

    if let Some(err) = body.error {
        bail!("AUR error: {}", err);
    }

    Ok(body.results.into_iter().next())
}

pub async fn install(name: &str, noconfirm: bool) -> Result<()> {
    let pkg = info(name)
        .await?
        .with_context(|| format!("Package '{}' not found in AUR", name))?;

    ui::header(&format!("Installing {} from AUR", ui::package_name(&pkg.name)));

    if let Some(ref desc) = pkg.description {
        ui::info(&format!("Description: {}", desc));
    }
    ui::info(&format!("Version: {}", ui::version_str(&pkg.version)));

    if let Some(ref deps) = pkg.depends {
        if !deps.is_empty() {
            ui::info(&format!("Dependencies: {}", deps.join(", ")));
        }
    }

    if !noconfirm && !ui::confirm("Proceed with build and install?") {
        ui::warn("Aborted.");
        return Ok(());
    }

    let build_dir = cache::build_dir(&pkg.name)?;
    clone_or_pull(&pkg.name, &build_dir)?;
    build_and_install(&build_dir, noconfirm)?;

    ui::success(&format!(
        "{} installed successfully.",
        ui::package_name(&pkg.name)
    ));

    Ok(())
}

fn clone_or_pull(name: &str, build_dir: &Path) -> Result<()> {
    let repo_url = format!("{}/{}.git", AUR_CLONE_BASE, name);
    let path_str = build_dir.to_str().unwrap();

    if build_dir.join(".git").exists() {
        ui::info("Updating existing build directory...");
        let status = Command::new("git")
            .args(["-C", path_str, "pull", "--rebase"])
            .status()
            .context("Failed to run git pull")?;

        if !status.success() {
            bail!("git pull failed");
        }
    } else {
        if build_dir.exists() {
            fs::remove_dir_all(build_dir)?;
        }
        ui::info("Cloning AUR repository...");
        let status = Command::new("git")
            .args(["clone", &repo_url, path_str])
            .status()
            .context("Failed to run git clone")?;

        if !status.success() {
            bail!("git clone failed");
        }
    }

    Ok(())
}

fn build_and_install(build_dir: &Path, noconfirm: bool) -> Result<()> {
    ui::info("Building package with makepkg...");

    let mut args = vec!["-si", "--cleanbuild"];
    if noconfirm {
        args.push("--noconfirm");
    }

    let status = Command::new("makepkg")
        .args(&args)
        .current_dir(build_dir)
        .status()
        .context("Failed to run makepkg")?;

    if !status.success() {
        bail!("makepkg failed — check the output above for errors");
    }

    Ok(())
}

pub fn get_aur_upgrades(aur_pkgs: &[AurPackage]) -> Vec<(&AurPackage, String)> {
    aur_pkgs
        .iter()
        .filter_map(|pkg| {
            let installed = get_installed_version(&pkg.name)?;
            if installed != pkg.version {
                Some((pkg, installed))
            } else {
                None
            }
        })
        .collect()
}

pub fn get_installed_version(name: &str) -> Option<String> {
    let output = Command::new("pacman").args(["-Q", name]).output().ok()?;

    if output.status.success() {
        let line = String::from_utf8_lossy(&output.stdout);
        Some(line.split_whitespace().nth(1)?.to_string())
    } else {
        None
    }
}

pub fn get_all_aur_installed() -> Vec<String> {
    Command::new("pacman")
        .args(["-Qm"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter_map(|line| line.split_whitespace().next().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

pub async fn get_aur_packages_info(names: &[String]) -> Result<Vec<AurPackage>> {
    if names.is_empty() {
        return Ok(vec![]);
    }

    let query = names
        .iter()
        .map(|n| format!("arg[]={}", n))
        .collect::<Vec<_>>()
        .join("&");

    let url = format!("{}/info?{}", AUR_RPC, query);
    let body: AurResponse = client()
        .get(&url)
        .send()
        .await
        .context("Failed to reach AUR RPC")?
        .json()
        .await
        .context("Failed to parse AUR response")?;

    Ok(body.results)
}
