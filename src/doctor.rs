use anyhow::Result;
use colored::Colorize;
use std::process::Command;
use which::which;

use crate::cache;
use crate::ui;

struct Check {
    label: &'static str,
    result: CheckResult,
}

enum CheckResult {
    Ok(String),
    Warn(String),
    Fail(String),
}

pub fn run() -> Result<()> {
    ui::header("Running system health checks");
    println!();

    let checks = vec![
        check_binary("pacman"),
        check_binary("makepkg"),
        check_binary("git"),
        check_binary("gcc"),
        check_binary("sudo"),
        check_pacman_db(),
        check_arch_linux(),
        check_cache(),
    ];

    let mut all_ok = true;

    for check in &checks {
        let status = match &check.result {
            CheckResult::Ok(msg) => {
                format!("{} {} - {}", "[ok]".green().bold(), check.label, msg.dimmed())
            }
            CheckResult::Warn(msg) => {
                all_ok = false;
                format!("{} {} - {}", "[warn]".yellow().bold(), check.label, msg)
            }
            CheckResult::Fail(msg) => {
                all_ok = false;
                format!("{} {} - {}", "[fail]".red().bold(), check.label, msg)
            }
        };
        println!("  {}", status);
    }

    println!();

    if all_ok {
        ui::success("All checks passed. yoru is ready.");
    } else {
        ui::warn("Some checks failed. Review the output above.");
    }

    Ok(())
}

fn check_binary(name: &'static str) -> Check {
    let result = match which(name) {
        Ok(path) => CheckResult::Ok(path.display().to_string()),
        Err(_) => CheckResult::Fail(format!("'{}' not found in PATH", name)),
    };
    Check { label: name, result }
}

fn check_pacman_db() -> Check {
    let output = Command::new("pacman").args(["-Q"]).output();
    let result = match output {
        Ok(o) if o.status.success() => CheckResult::Ok("pacman database is readable".into()),
        _ => CheckResult::Fail("Cannot read pacman database".into()),
    };
    Check { label: "pacman db", result }
}

fn check_arch_linux() -> Check {
    let has_os_release = std::path::Path::new("/etc/arch-release").exists();
    let result = if has_os_release {
        CheckResult::Ok("Running on Arch Linux".into())
    } else {
        CheckResult::Warn("Not running on Arch Linux — some features may not work".into())
    };
    Check { label: "arch linux", result }
}

fn check_cache() -> Check {
    let size = cache::cache_size();
    let result = CheckResult::Ok(format!("cache size: {}", size));
    Check { label: "build cache", result }
}
