use anyhow::Result;
use colored::Colorize;
use std::process::Command;

use crate::aur;
use crate::ui;

pub async fn search(query: &str) -> Result<()> {
    ui::header(&format!("Searching for '{}'", query));

    let (pacman_results, aur_results) = tokio::join!(search_pacman(query), aur::search(query));

    let pacman_out = pacman_results.unwrap_or_default();
    let aur_pkgs = aur_results.unwrap_or_default();

    if !pacman_out.is_empty() {
        println!("\n{} {}", "::".cyan().bold(), "Official Repositories".bold());
        print!("{}", pacman_out);
    }

    if !aur_pkgs.is_empty() {
        println!("\n{} {}", "::".cyan().bold(), "AUR".bold());
        for pkg in &aur_pkgs {
            let ood = if pkg.out_of_date.is_some() {
                format!(" {}", "[out-of-date]".red().bold())
            } else {
                String::new()
            };

            let votes = pkg
                .num_votes
                .map(|v| format!(" ({} votes)", v))
                .unwrap_or_default();

            println!(
                "{}/{} {}{}{}",
                ui::aur_label(),
                ui::package_name(&pkg.name),
                ui::version_str(&pkg.version),
                ood,
                votes
            );

            if let Some(ref desc) = pkg.description {
                println!("    {}", desc);
            }
        }
    }

    if pacman_out.is_empty() && aur_pkgs.is_empty() {
        ui::warn(&format!("No results found for '{}'", query));
    }

    Ok(())
}

async fn search_pacman(query: &str) -> Result<String> {
    let output = Command::new("pacman").args(["-Ss", query]).output()?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
