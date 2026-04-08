use anyhow::{Context, Result};
use std::process::Command;
use std::time::{Duration, Instant};

const SIGTERM_WAIT_SECS: u64 = 2;

fn pgrep_exact(name: &str) -> Result<Vec<u32>> {
    let output = Command::new("pgrep")
        .args(["-x", name])
        .output()
        .with_context(|| format!("failed to run pgrep for {name}"))?;

    if output.status.code() == Some(1) {
        // pgrep exits 1 when no processes found — not an error
        return Ok(vec![]);
    }

    let stdout = String::from_utf8(output.stdout)
        .with_context(|| format!("pgrep output for {name} is not valid UTF-8"))?;

    let pids = stdout
        .lines()
        .filter_map(|l| l.trim().parse::<u32>().ok())
        .collect();

    Ok(pids)
}

pub fn find_claude_pids() -> Result<Vec<u32>> {
    let mut pids = pgrep_exact("claude")?;
    pids.extend(pgrep_exact("claude-code")?);
    pids.sort_unstable();
    pids.dedup();
    Ok(pids)
}

pub fn kill_claude_processes() -> Result<usize> {
    let pids = find_claude_pids()?;
    if pids.is_empty() {
        return Ok(0);
    }

    // SIGTERM all
    for &pid in &pids {
        let _ = nix::sys::signal::kill(
            nix::unistd::Pid::from_raw(pid as i32),
            nix::sys::signal::Signal::SIGTERM,
        );
    }

    // Wait up to 2s for graceful exit
    let deadline = Instant::now() + Duration::from_secs(SIGTERM_WAIT_SECS);
    while Instant::now() < deadline {
        let remaining = find_claude_pids()?;
        if remaining.is_empty() {
            return Ok(pids.len());
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    // SIGKILL survivors
    let survivors = find_claude_pids()?;
    for &pid in &survivors {
        let _ = nix::sys::signal::kill(
            nix::unistd::Pid::from_raw(pid as i32),
            nix::sys::signal::Signal::SIGKILL,
        );
    }

    Ok(pids.len())
}
