use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "yoru",
    version = env!("CARGO_PKG_VERSION"),
    about = "A modern AUR helper for Arch Linux",
    long_about = None,
    arg_required_else_help = true,
    subcommand_required = true,
    disable_help_flag = true,
    disable_version_flag = true,
    disable_help_subcommand = true,
    override_usage = "yoru <COMMAND> [OPTIONS]",
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(name = "-S", about = "Install one or more packages from the AUR or official repos")]
    S {
        #[arg(required = true, num_args = 1..)]
        packages: Vec<String>,

        #[arg(long, help = "Do not reinstall already installed packages")]
        needed: bool,

        #[arg(long, help = "Skip confirmation prompts")]
        noconfirm: bool,
    },

    #[command(name = "-Ss", about = "Search for packages in the AUR and official repos")]
    Ss {
        query: String,
    },

    #[command(name = "-Syu", about = "Upgrade all installed packages")]
    Syu {
        #[arg(long, help = "Skip confirmation prompts")]
        noconfirm: bool,
    },

    #[command(name = "-R", about = "Remove an installed package")]
    R {
        package: String,

        #[arg(long, help = "Skip confirmation prompts")]
        noconfirm: bool,
    },

    #[command(name = "-Qi", about = "Show detailed information about a package")]
    Qi {
        package: String,
    },

    #[command(name = "clean", about = "Clean the local build cache")]
    Clean {
        #[arg(long, help = "Remove all cached builds, not just old ones")]
        all: bool,
    },

    #[command(name = "doctor", about = "Check system health and yoru dependencies")]
    Doctor,
}
