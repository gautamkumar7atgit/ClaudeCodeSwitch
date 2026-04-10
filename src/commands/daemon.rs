use anyhow::Result;

use crate::cli::DaemonCommands;
use crate::daemon::launchd;
use crate::output::{confirm_prompt, print_error, print_success, print_warn};

pub fn run(cmd: &DaemonCommands, _verbose: bool) -> Result<()> {
    match cmd {
        DaemonCommands::Start => start(),
        DaemonCommands::Stop => stop(),
        DaemonCommands::Status => status(),
    }
}

fn start() -> Result<()> {
    let binary = std::env::current_exe()?;

    if launchd::daemon_is_loaded() {
        let pid_str = launchd::get_daemon_pid()
            .map(|p| format!(" (PID {p})"))
            .unwrap_or_default();
        print_error(&format!("Daemon already running{pid_str}."));
        let confirmed = confirm_prompt("Restart?")?;
        if !confirmed {
            return Ok(());
        }
        launchd::uninstall_plist()?;
    }

    launchd::install_plist(&binary)?;
    print_success("Daemon started.");
    Ok(())
}

fn stop() -> Result<()> {
    if !launchd::daemon_is_loaded() {
        print_warn("Daemon is not running.");
        return Ok(());
    }
    launchd::stop_daemon()?;
    print_success("Daemon stopped. It will restart automatically on next login.");
    Ok(())
}

fn status() -> Result<()> {
    if launchd::daemon_is_loaded() {
        let pid_str = launchd::get_daemon_pid()
            .map(|p| format!("running (PID {p})"))
            .unwrap_or_else(|| "running".to_string());
        println!("Daemon: {pid_str}");
    } else {
        println!("Daemon: stopped");
    }

    // Show last few log lines
    let log_path = crate::config::paths().daemon_log;
    if log_path.exists() {
        let content = std::fs::read_to_string(&log_path).unwrap_or_default();
        let lines: Vec<&str> = content.lines().rev().take(5).collect();
        if !lines.is_empty() {
            println!("Log (last 5 lines):");
            for line in lines.into_iter().rev() {
                println!("  {line}");
            }
        }
    }

    Ok(())
}
