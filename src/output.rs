use chrono::{DateTime, Utc};
use std::io::{self, Write};

// ANSI color codes
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

pub fn green(s: &str) -> String {
    if ansi_enabled() { format!("{GREEN}{s}{RESET}") } else { s.to_string() }
}

pub fn red(s: &str) -> String {
    if ansi_enabled() { format!("{RED}{s}{RESET}") } else { s.to_string() }
}

pub fn yellow(s: &str) -> String {
    if ansi_enabled() { format!("{YELLOW}{s}{RESET}") } else { s.to_string() }
}

pub fn cyan(s: &str) -> String {
    if ansi_enabled() { format!("{CYAN}{s}{RESET}") } else { s.to_string() }
}

pub fn bold(s: &str) -> String {
    if ansi_enabled() { format!("{BOLD}{s}{RESET}") } else { s.to_string() }
}

pub fn dim(s: &str) -> String {
    if ansi_enabled() { format!("{DIM}{s}{RESET}") } else { s.to_string() }
}

fn ansi_enabled() -> bool {
    // Only emit ANSI when stdout is a TTY
    use std::io::IsTerminal;
    io::stdout().is_terminal()
}

pub fn print_success(msg: &str) {
    if ansi_enabled() {
        println!("{GREEN}✓{RESET} {msg}");
    } else {
        println!("✓ {msg}");
    }
}

pub fn print_error(msg: &str) {
    if ansi_enabled() {
        eprintln!("{RED}✗{RESET} {msg}");
    } else {
        eprintln!("✗ {msg}");
    }
}

pub fn print_warn(msg: &str) {
    if ansi_enabled() {
        println!("{YELLOW}!{RESET} {msg}");
    } else {
        println!("! {msg}");
    }
}

pub fn print_info(msg: &str) {
    if ansi_enabled() {
        println!("{CYAN}→{RESET} {msg}");
    } else {
        println!("→ {msg}");
    }
}

pub fn print_verbose(msg: &str, verbose: bool) {
    if !verbose {
        return;
    }
    if ansi_enabled() {
        println!("{DIM}{msg}{RESET}");
    } else {
        println!("{msg}");
    }
}

pub fn format_duration_ago(ts: DateTime<Utc>) -> String {
    let now = Utc::now();
    let secs = (now - ts).num_seconds().max(0) as u64;
    if secs < 60 {
        format!("{secs}s ago")
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86400)
    }
}

use crate::profiles::Profile;

pub fn format_profile_row(profile: &Profile, is_active: bool, verbose: bool) -> String {
    use crate::keychain::credential_fingerprint;

    let arrow = if is_active { "→" } else { " " };
    let active_badge = if is_active { " (active)" } else { "" };
    let synced = format_duration_ago(profile.meta.last_synced);
    let name = &profile.meta.name;
    let token = credential_fingerprint(&profile.claude_ai_oauth);

    let mut row = format!("  {arrow} {name:<20}{active_badge:<10}  {token}  synced {synced}");

    if verbose {
        let p = crate::config::paths();
        let path = p.profiles_dir.join(format!("{name}.json"));
        row.push_str(&format!("\n       {}", path.display()));
    }

    row
}

pub fn confirm_prompt(msg: &str) -> anyhow::Result<bool> {
    print!("{msg} [y/N]: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(matches!(input.trim().to_lowercase().as_str(), "y" | "yes"))
}
