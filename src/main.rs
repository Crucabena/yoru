mod aur;
mod cache;
mod cli;
mod doctor;
mod pacman;
mod search;
mod ui;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use colored::Colorize;

fn print_help() {
    println!();
    println!("  {}  {}", "yoru".cyan().bold(), env!("CARGO_PKG_VERSION").dimmed());
    println!("  {}", "A modern AUR helper for Arch Linux".dimmed());
    println!();
    println!("  {}", "Usage".white().bold());
    println!("    {} {}", "yoru".cyan(), "<command> [options]".dimmed());
    println!();
    println!("  {}", "Commands".white().bold());
    println!("    {}    {}", "-S <pkg...>".cyan().bold(),  "Install one or more packages");
    println!("    {}  {}", "-Ss <query>".cyan().bold(),    "Search AUR and official repos");
    println!("    {}      {}", "-Syu".cyan().bold(),       "Upgrade all installed packages");
    println!("    {}    <pkg>  {}", "-R".cyan().bold(),    "Remove an installed package");
    println!("    {}   <pkg>  {}", "-Qi".cyan().bold(),    "Show package information");
    println!("    {}    {}", "clean".cyan().bold(),         "Clean the local build cache");
    println!("    {}   {}", "doctor".cyan().bold(),         "Check system health and dependencies");
    println!();
    println!("  {}", "Options".white().bold());
    println!("    {}   {}", "--noconfirm".cyan(), "Skip confirmation prompts");
    println!("    {}      {}", "--needed".cyan(),  "Skip reinstalling up-to-date packages");
    println!("    {}         {}", "--all".cyan(),   "Used with clean to wipe entire cache");
    println!();
}

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::args().len() == 1 {
        print_help();
        return Ok(());
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::S { packages, needed, noconfirm } => {
            pacman::install_many(&packages, needed, noconfirm).await?;
        }
        Commands::Ss { query } => {
            search::search(&query).await?;
        }
        Commands::Syu { noconfirm } => {
            pacman::sysupgrade(noconfirm).await?;
        }
        Commands::R { package, noconfirm } => {
            pacman::remove(&package, noconfirm)?;
        }
        Commands::Qi { package } => {
            pacman::info(&package)?;
        }
        Commands::Clean { all } => {
            cache::clean(all)?;
        }
        Commands::Doctor => {
            doctor::run()?;
        }
    }

    Ok(())
}
