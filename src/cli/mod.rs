use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "ccswitch",
    author,
    version,
    about = "Switch between multiple Claude Code OAuth accounts"
)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Save current Keychain credentials as a named profile
    Add {
        /// Profile name
        name: String,
        /// Overwrite existing profile without prompting
        #[arg(long)]
        overwrite: bool,
    },
    /// Switch to a saved profile
    Use {
        /// Profile name
        name: String,
    },
    /// List all saved profiles
    List,
    /// Remove a saved profile
    Remove {
        /// Profile name
        name: String,
        /// Skip confirmation prompt when removing the active profile
        #[arg(long)]
        force: bool,
    },
    /// Show current status
    Status,
    /// Manage the background sync daemon
    Daemon {
        #[command(subcommand)]
        command: DaemonCommands,
    },
    /// Initialize ccswitch (create dirs, import existing credentials)
    Init,
    /// Remove all ccswitch data
    Uninstall,
    /// Export one or all profiles to a portable encrypted bundle
    Export {
        /// Profile name to export (omit when using --all)
        name: Option<String>,
        /// Export every saved profile
        #[arg(long)]
        all: bool,
        /// Output file path (default: <name>.ccspack or ccswitch-export.ccspack)
        #[arg(long, short)]
        output: Option<PathBuf>,
        /// Write a plaintext bundle without passphrase encryption
        /// (use only in fully trusted environments)
        #[arg(long)]
        no_encrypt: bool,
    },
    /// Import profiles from an exported bundle
    Import {
        /// Path to the .ccspack bundle file
        file: PathBuf,
        /// Rename the imported profile — only valid for single-profile bundles
        #[arg(long, value_name = "NAME")]
        r#as: Option<String>,
        /// Overwrite existing profiles without prompting
        #[arg(long)]
        overwrite: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum DaemonCommands {
    /// Start the sync daemon
    Start,
    /// Stop the sync daemon
    Stop,
    /// Show daemon status
    Status,
}
