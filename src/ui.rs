use colored::Colorize;
use console::Term;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{self, Write};
use std::time::Duration;

pub fn header(text: &str) {
    println!("{} {}", "::".cyan().bold(), text.bold());
}

pub fn success(text: &str) {
    println!("{} {}", "==>".green().bold(), text);
}

pub fn info(text: &str) {
    println!("{} {}", "-->".blue().bold(), text);
}

pub fn warn(text: &str) {
    println!("{} {}", "warning:".yellow().bold(), text);
}

pub fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["-", "\\", "|", "/"]),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

pub fn confirm(prompt: &str) -> bool {
    let term = Term::stdout();
    print!("{} {} [Y/n] ", "::".cyan().bold(), prompt.bold());
    io::stdout().flush().unwrap();
    let input = term.read_line().unwrap_or_default();
    let trimmed = input.trim().to_lowercase();
    trimmed.is_empty() || trimmed == "y" || trimmed == "yes"
}

pub fn package_name(name: &str) -> String {
    name.cyan().bold().to_string()
}

pub fn version_str(ver: &str) -> String {
    ver.green().to_string()
}

pub fn aur_label() -> String {
    "AUR".magenta().bold().to_string()
}
