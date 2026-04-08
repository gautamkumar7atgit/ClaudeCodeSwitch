# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

---

## What This Project Is

`ccswitch` is a macOS CLI tool (Rust, single static binary) that switches between multiple Claude Code OAuth accounts by swapping credentials in the macOS Keychain. It also runs a background daemon (launchd) that watches the Keychain and syncs refreshed tokens back to the active profile.

**Read `docs/PLAN.md` before starting any work.** It contains all design decisions, the task breakdown, and progress tracking. All architectural decisions are final — do not deviate without user confirmation.

---

## Commands

```bash
# Build
cargo build
cargo build --release

# Check (fast compile without linking)
cargo check

# Lint
cargo clippy -- -W clippy::all

# Format
rustfmt --edition 2021 src/**/*.rs

# Run all tests
cargo test

# Run a single integration test by name
cargo test --test profile_tests test_name_here

# Run tests with output visible
cargo test -- --nocapture

# Build universal binary (after both targets are set up)
./scripts/build_universal.sh
```

---

## Architecture

### Data flow for `ccswitch use <name>`
1. Load `~/.claude-switcher/profiles/<name>.json` from disk
2. Kill `claude` and `claude-code` processes (SIGTERM → wait 2s → SIGKILL stragglers)
3. Write only the `claudeAiOauth` object to macOS Keychain (strip `_meta`)
4. Write `<name>` to `~/.claude-switcher/active`

### Data flow for the daemon (30s poll loop)
1. Read current Keychain credentials
2. Read active profile from `~/.claude-switcher/active` → load that profile JSON
3. If tokens differ: update `_meta.last_synced` on the active profile and save it (Claude Code performed a silent OAuth refresh — sync the new tokens back to disk)
4. If Keychain doesn't match active profile for other reasons: log WARN only, do nothing

### Profile JSON format
```json
{
  "claudeAiOauth": { "accessToken": "...", "refreshToken": "..." },
  "_meta": { "name": "work", "last_synced": "2026-04-08T10:30:00Z" }
}
```
When **writing to Keychain**, always strip `_meta` — only the `claudeAiOauth` object is written.

### Module responsibilities
- `config.rs` — all constants and path helpers (never hardcode `/Users/`)
- `keychain.rs` — shells out to `security` CLI; maps exit code 44 → error code 2
- `profiles.rs` — atomic file writes (`.tmp` → rename), chmod 600, all profile CRUD
- `process.rs` — `pgrep -x claude` + `pgrep -x claude-code`; `Ok(0)` when no processes found
- `output.rs` — all terminal output: ANSI colors, Unicode icons (→ ✓ ✗ !), table formatting, `confirm_prompt()`
- `daemon/mod.rs` — poll loop with SIGTERM handler and log rotation at 1 MB
- `daemon/launchd.rs` — plist generation + `launchctl load/unload`

### CLI output style
All user-facing output goes through `src/output.rs`. Use `print_success`, `print_error`, `print_warn`, `print_info`. Never `println!` directly from command handlers.

### Exit codes
| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Usage error |
| 2 | Keychain error |
| 3 | Profile not found |

---

## Key Conventions

- **Atomic writes:** write to `<path>.tmp`, then `fs::rename()` to target — always, for every profile or state file
- **Path resolution:** use `dirs::home_dir()` via `config::paths()` — never hardcode paths
- **Error handling:** `anyhow::Result<T>` everywhere; add `.context("what failed")` at each error site
- **Verbose flag:** global `-v / --verbose` on `Cli` struct; pass `verbose: bool` to all handlers; use `output::print_verbose()` for debug lines
- **Testing Keychain:** use service name `ccswitch-test` in integration tests, never `Claude Code-credentials`; each test must clean up after itself
- **Process kill target:** only `claude` (exact match) and `claude-code` — never `node` or other processes
- **Removing active profile:** require `--force` flag or `y/N` stdin prompt; never silently delete
- **`daemon start` idempotency:** if daemon already running, log error and prompt to confirm restart before unloading + reloading

---

## Runtime Paths

| Path | Purpose |
|------|---------|
| `~/.claude-switcher/profiles/<name>.json` | Profile credentials + metadata |
| `~/.claude-switcher/active` | Plain text: current active profile name |
| `~/.claude-switcher/daemon.log` | Current daemon log (max 1 MB, rotates to `.log.1`) |
| `~/Library/LaunchAgents/com.ccswitch.daemon.plist` | launchd daemon config |
