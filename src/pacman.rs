use anyhow::{bail, Context, Result};
use std::process::Command;

use crate::aur;
use crate::ui;

pub fn is_in_repos(name: &str) -> bool {
    Command::new("pacman")
        .args(["-Si", name])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub async fn install_many(packages: &[String], needed: bool, noconfirm: bool) -> Result<()> {
    let mut repo_pkgs: Vec<&str> = Vec::new();
    let mut aur_pkgs: Vec<&str> = Vec::new();

    for pkg in packages {
        if is_in_repos(pkg) {
            repo_pkgs.push(pkg);
        } else {
            aur_pkgs.push(pkg);
        }
    }

    if !repo_pkgs.is_empty() {
        ui::header(&format!(
            "Installing {} package(s) from official repos",
            repo_pkgs.len()
        ));
        pacman_install_many(&repo_pkgs, needed, noconfirm)?;
    }

    for pkg in aur_pkgs {
        ui::info(&format!(
            "{} not found in official repos, trying AUR...",
            ui::package_name(pkg)
        ));
        aur::install(pkg, noconfirm).await?;
    }

    Ok(())
}

fn pacman_install_many(names: &[&str], needed: bool, noconfirm: bool) -> Result<()> {
    let mut args = vec!["-S".to_string()];
    if needed {
        args.push("--needed".to_string());
    }
    if noconfirm {
        args.push("--noconfirm".to_string());
    }
    for name in names {
        args.push(name.to_string());
    }

    let status = Command::new("sudo")
        .arg("pacman")
        .args(&args)
        .status()
        .context("Failed to run pacman")?;

    if !status.success() {
        bail!("pacman install failed");
    }

    Ok(())
}

pub fn remove(name: &str, noconfirm: bool) -> Result<()> {
    ui::header(&format!("Removing {}", ui::package_name(name)));

    if !noconfirm && !ui::confirm(&format!("Remove {}?", name)) {
        ui::warn("Aborted.");
        return Ok(());
    }

    let mut args = vec!["-Rs".to_string()];
    if noconfirm {
        args.push("--noconfirm".to_string());
    }
    args.push(name.to_string());

    let status = Command::new("sudo")
        .arg("pacman")
        .args(&args)
        .status()
        .context("Failed to run pacman")?;

    if !status.success() {
        bail!("pacman remove failed");
    }

    ui::success(&format!("{} removed.", ui::package_name(name)));
    Ok(())
}

pub fn info(name: &str) -> Result<()> {
    let output = Command::new("pacman")
        .args(["-Qi", name])
        .output()
        .context("Failed to run pacman")?;

    if output.status.success() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
        return Ok(());
    }

    let output = Command::new("pacman")
        .args(["-Si", name])
        .output()
        .context("Failed to run pacman")?;

    if output.status.success() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
        return Ok(());
    }

    bail!("Package '{}' not found", name);
}

pub async fn sysupgrade(noconfirm: bool) -> Result<()> {
    ui::header("Starting full system upgrade");

    ui::info("Updating official repositories...");
    let mut args = vec!["-Syu".to_string()];
    if noconfirm {
        args.push("--noconfirm".to_string());
    }

    let status = Command::new("sudo")
        .arg("pacman")
        .args(&args)
        .status()
        .context("Failed to run pacman")?;

    if !status.success() {
        bail!("pacman system upgrade failed");
    }

    ui::info("Checking AUR packages for updates...");
    let installed = aur::get_all_aur_installed();

    if installed.is_empty() {
        ui::info("No AUR packages installed.");
        return Ok(());
    }

    let pb = ui::spinner(&format!(
        "Fetching AUR info for {} packages...",
        installed.len()
    ));
    let aur_pkgs = aur::get_aur_packages_info(&installed).await?;
    pb.finish_and_clear();

    let upgrades = aur::get_aur_upgrades(&aur_pkgs);

    if upgrades.is_empty() {
        ui::success("All AUR packages are up to date.");
        return Ok(());
    }

    ui::header(&format!(
        "{} AUR package(s) can be upgraded:",
        upgrades.len()
    ));
    for (pkg, installed_ver) in &upgrades {
        println!(
            "  {} {} -> {}",
            ui::package_name(&pkg.name),
            installed_ver.as_str(),
            ui::version_str(&pkg.version)
        );
    }

    if !noconfirm && !ui::confirm("Upgrade AUR packages?") {
        ui::warn("AUR upgrade skipped.");
        return Ok(());
    }

    for (pkg, _) in &upgrades {
        ui::header(&format!("Upgrading {}", ui::package_name(&pkg.name)));
        aur::install(&pkg.name, noconfirm).await?;
    }

    ui::success("System upgrade complete.");
    Ok(())
}
