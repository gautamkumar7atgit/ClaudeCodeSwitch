# CCSwitch — Development Plan

> **UI-first approach.** Output styling and CLI structure are built before deep backend logic.
> Tasks marked ✅ / subtasks marked [✓] only after user-confirmed test run passes, with completion timestamp.

---

## Design Decisions (from interview)

| Topic | Decision |
|-------|----------|
| CLI output | Colored + Unicode icons (→ ✓ ✗ !) via ANSI |
| Profile security | Plaintext JSON, chmod 600, MVP only |
| Process targets | `claude` (exact) + `claude-code` |
| Active profile removal | Warn + require `--force` or `y/N` prompt |
| `init` with existing creds | Prompt user for profile name |
| Daemon on Keychain mismatch | Log warning, do nothing |
| `status` output | Full diagnostic block (PID, fingerprint, log tail) |
| `use` process shutdown | Wait up to 2s, then proceed regardless |
| `daemon start` when running | Log error + confirm to restart |
| `list` sort order | Active profile first, then alphabetical |
| `uninstall` Keychain | Leave Keychain untouched |
| Distribution | Homebrew tap + curl script simultaneously |
| `add` name collision | Prompt to overwrite |
| Verbose flag | Global `-v / --verbose` flag |
| `use` with no processes | Proceed silently (skip kill step) |
| Daemon management | launchd only (~/Library/LaunchAgents) |

---

## Phase 0 — Project Bootstrap

### Task 0.1: Initialize Rust Project
- [✓] Run `cargo init` in project root, set edition = 2021 — 2026-04-08
- [✓] Add all dependencies to `Cargo.toml` (`clap`, `serde`, `serde_json`, `anyhow`, `chrono`, `nix`, `plist`, `dirs`, `log`, `env_logger`) — 2026-04-08
- [✓] Create full directory skeleton (`src/cli/`, `src/commands/`, `src/daemon/`, `tests/integration/`, `tests/fixtures/`, `scripts/`) — 2026-04-08
- [✓] Verify `cargo check` passes with empty stubs — 2026-04-08

### Task 0.2: Constants & Config (`src/config.rs`)
- [✓] Define `KEYCHAIN_SERVICE = "Claude Code-credentials"` — 2026-04-08
- [✓] Define `KEYCHAIN_ACCOUNT = "credentials"` (or discover correct account key) — 2026-04-08
- [✓] Define `PROFILES_DIR = ~/.claude-switcher/profiles/` — 2026-04-08
- [✓] Define `ACTIVE_FILE = ~/.claude-switcher/active` — 2026-04-08
- [✓] Define `DAEMON_LOG = ~/.claude-switcher/daemon.log` — 2026-04-08
- [✓] Define `DAEMON_LOG_MAX_BYTES = 1_048_576` (1 MB) — 2026-04-08
- [✓] Define `POLL_INTERVAL_SECS = 30` — 2026-04-08
- [✓] Define exit code constants (0 success, 1 usage, 2 keychain, 3 not found) — 2026-04-08
- [✓] Implement `config::paths()` helper returning all resolved absolute paths using `dirs::home_dir()` — 2026-04-08

---

## Phase 1 — Core Infrastructure (M1)

### Task 1.1: Keychain Module (`src/keychain.rs`)
- [✓] Implement `read_keychain() -> anyhow::Result<KeychainCredentials>` — 2026-04-08
- [✓] Parse raw JSON output into `KeychainCredentials` struct — 2026-04-08
- [✓] Implement `write_keychain(creds: &OAuthCredentials) -> anyhow::Result<()>` — 2026-04-08
- [✓] Map exit codes to typed errors (exit 44 = not found → exit code 2) — 2026-04-08
- [✓] Implement `credential_fingerprint(creds: &OAuthCredentials) -> String` — 2026-04-08

### Task 1.2: Profile Module (`src/profiles.rs`)
- [✓] Define `Profile`, `OAuthCredentials`, `ProfileMeta` structs with correct serde renames — 2026-04-08
- [✓] Implement `list_profiles() -> anyhow::Result<Vec<Profile>>` — 2026-04-08
- [✓] Implement `load_profile(name: &str) -> anyhow::Result<Profile>` — 2026-04-08
- [✓] Implement `save_profile(profile: &Profile) -> anyhow::Result<()>` — atomic write (`.tmp` → rename), chmod 600 — 2026-04-08
- [✓] Implement `delete_profile(name: &str) -> anyhow::Result<()>` — 2026-04-08
- [✓] Implement `get_active_profile() -> anyhow::Result<Option<String>>` — 2026-04-08
- [✓] Implement `set_active_profile(name: &str) -> anyhow::Result<()>` — 2026-04-08
- [✓] Implement `clear_active_profile() -> anyhow::Result<()>` — 2026-04-08
- [✓] Implement `ensure_dirs() -> anyhow::Result<()>` — 2026-04-08

### Task 1.3: Process Module (`src/process.rs`)
- [✓] Implement `find_claude_pids() -> anyhow::Result<Vec<u32>>` — 2026-04-08
- [✓] Implement `kill_claude_processes() -> anyhow::Result<usize>` — SIGTERM → wait 2s → SIGKILL — 2026-04-08
- [✓] Return `Ok(0)` when no processes found — 2026-04-08

### Task 1.4: Output Utilities (`src/output.rs`) — **UI Priority**
- [✓] Define color constants using ANSI escape codes — 2026-04-08
- [✓] Implement `print_success(msg: &str)` — green ✓ prefix — 2026-04-08
- [✓] Implement `print_error(msg: &str)` — red ✗ prefix — 2026-04-08
- [✓] Implement `print_warn(msg: &str)` — yellow ! prefix — 2026-04-08
- [✓] Implement `print_info(msg: &str)` — cyan → prefix — 2026-04-08
- [✓] Implement `print_verbose(msg: &str, verbose: bool)` — 2026-04-08
- [✓] Implement `format_profile_row(profile: &Profile, is_active: bool) -> String` — 2026-04-08
- [✓] Implement `format_duration_ago(ts: DateTime<Utc>) -> String` — 2026-04-08
- [✓] Implement `confirm_prompt(msg: &str) -> anyhow::Result<bool>` — 2026-04-08

### Task 1.5: M1 Integration Tests
- [✓] `tests/integration/keychain_tests.rs` — 3 tests, all passing — 2026-04-08
- [✓] `tests/integration/profile_tests.rs` — 3 tests, all passing — 2026-04-08
- [✓] `tests/fixtures/sample_profile.json` — 2026-04-08

---

## Phase 2 — CLI Structure (UI First)

### Task 2.1: CLI Definitions (`src/cli/mod.rs`)
- [✓] Define top-level `Cli` struct with `#[command(author, version, about)]` — 2026-04-08
- [✓] Add global `--verbose / -v` flag on `Cli` struct — 2026-04-08
- [✓] Define `Commands` enum with all subcommands: `Add`, `Use`, `List`, `Remove`, `Status`, `Daemon`, `Init`, `Uninstall` — 2026-04-08
- [✓] Define `Add` args: `name: String`, `--overwrite` flag — 2026-04-08
- [✓] Define `Use` args: `name: String` — 2026-04-08
- [✓] Define `Remove` args: `name: String`, `--force` flag — 2026-04-08
- [✓] Define `Daemon` subcommand with `DaemonCommands` enum: `Start`, `Stop`, `Status` — 2026-04-08
- [✓] Verify `cargo build` + `ccswitch --help` renders correctly — 2026-04-08

### Task 2.2: Entry Point (`src/main.rs`)
- [✓] Parse args with clap, extract `verbose` flag — 2026-04-08
- [✓] Dispatch to each command handler, passing `verbose: bool` — 2026-04-08
- [✓] Map `anyhow::Error` to correct exit codes (2 for keychain, 3 for not found, 1 for other) — 2026-04-08
- [✓] Initialize `env_logger` respecting `RUST_LOG` env var — 2026-04-08

---

## Phase 3 — CLI Commands (M2)

### Task 3.1: `ccswitch list` (`src/commands/list.rs`) — **Build First (most visual)**
- [✓] Load all profiles + active profile name — 2026-04-08
- [✓] Sort: active profile first, then remaining alphabetically — 2026-04-08
- [✓] Print header `Profiles:` (or `No profiles found.` if empty) — 2026-04-08
- [✓] For each profile: print formatted row using `output::format_profile_row()` — 2026-04-08
- [✓] Verbose mode: also print profile file path — 2026-04-08

### Task 3.2: `ccswitch status` (`src/commands/status.rs`)
- [✓] Read active profile name and load it — 2026-04-08
- [✓] Read current Keychain credentials — 2026-04-08
- [✓] Compare Keychain access token vs active profile access token; show ✓ or ✗ — 2026-04-08
- [✓] Check daemon: PID from launchd — 2026-04-08
- [✓] Show full diagnostic block — 2026-04-08
- [✓] Handle "no active profile" gracefully — 2026-04-08
- [✓] Handle "daemon not running" gracefully — 2026-04-08

### Task 3.3: `ccswitch add <name>` (`src/commands/add.rs`)
- [✓] Read current Keychain credentials via `keychain::read_keychain()` — 2026-04-08
- [✓] Check if profile `<name>` already exists — 2026-04-08
- [✓] If exists: prompt `Profile "work" already exists. Overwrite? [y/N]: ` — 2026-04-08
- [✓] Build `Profile` with `_meta.name = name`, `_meta.last_synced = now` — 2026-04-08
- [✓] Save profile via `profiles::save_profile()` — 2026-04-08
- [✓] Print `✓ Profile "work" saved.` — 2026-04-08

### Task 3.4: `ccswitch use <name>` (`src/commands/use_profile.rs`)
- [✓] Load profile `<name>` (exit code 3 if not found) — 2026-04-08
- [✓] Kill running claude/claude-code processes (proceed silently if none found) — 2026-04-08
- [✓] Wait up to 2s for graceful shutdown (SIGKILL stragglers) — 2026-04-08
- [✓] Write profile's `OAuthCredentials` to Keychain (strip `_meta`) — 2026-04-08
- [✓] Set active profile to `<name>` — 2026-04-08
- [✓] Show elapsed time (`✓ Switched to "work" in 0.8s`) — 2026-04-08

### Task 3.5: `ccswitch remove <name>` (`src/commands/remove.rs`)
- [✓] Load profile `<name>` (exit code 3 if not found) — 2026-04-08
- [✓] Check if `<name>` is the currently active profile — 2026-04-08
- [✓] If active and no `--force`: warn + prompt `Remove anyway? [y/N]: ` — 2026-04-08
- [✓] If active and `--force`: skip prompt — 2026-04-08
- [✓] Delete profile file via `profiles::delete_profile()` — 2026-04-08
- [✓] If active profile was removed, clear active marker — 2026-04-08
- [✓] Print `✓ Profile "work" removed.` — 2026-04-08

### Task 3.6: M2 Integration Tests
- [✓] `tests/integration/cli_tests.rs` — 6 tests, all passing — 2026-04-08
- [✓] Edge cases: missing profile, active-profile lifecycle, delete cleanup — 2026-04-08
- [✓] Verify ANSI codes absent from `format_profile_row` output — 2026-04-08

---

## Phase 4 — Daemon (M3)

### Task 4.1: Daemon Poll Loop (`src/daemon/mod.rs`)
- [✓] Entry point `run_daemon()` called when binary launched with `--daemon` internal flag — 2026-04-08
- [✓] Log writing to `~/.claude-switcher/daemon.log` with **local-time** timestamps (`%Y-%m-%d %H:%M:%S %z`) — 2026-04-08 (timestamp format updated 2026-04-10)
- [✓] Log rotation: when file exceeds 1 MB, rename to `daemon.log.1`, start fresh — 2026-04-08
- [✓] Main loop: sleep 30s → read Keychain → compare → always sync back when any token differs (access-only rotation **or** full token rotation) — 2026-04-08 (sync logic fixed 2026-04-10)
- [✓] Handle SIGTERM gracefully via atomic flag — 2026-04-08

### Task 4.2: Launchd Integration (`src/daemon/launchd.rs`)
- [✓] Implement `generate_plist(binary_path: &Path) -> String` — 2026-04-08
- [✓] Implement `install_plist(binary_path: &Path) -> anyhow::Result<()>` — uses `launchctl bootstrap gui/<uid>` (macOS 13+ API); plist stays on disk for auto-load on every login — 2026-04-08 (fixed 2026-04-10)
- [✓] Implement `stop_daemon() -> anyhow::Result<()>` — uses `launchctl bootout gui/<uid>/<label>`; stops process without removing plist — 2026-04-10
- [✓] Implement `uninstall_plist() -> anyhow::Result<()>` — full uninstall: bootout + plist file removal — 2026-04-08 (fixed 2026-04-10)
- [✓] Implement `daemon_is_loaded() -> bool` — 2026-04-08
- [✓] Implement `get_daemon_pid() -> Option<u32>` — 2026-04-08

### Task 4.3: `ccswitch daemon {start|stop|status}` (`src/commands/daemon.rs`)
- [✓] `daemon start`: idempotency check, restart prompt, install + bootstrap — 2026-04-08 (fixed 2026-04-10)
- [✓] `daemon stop`: not-running guard, bootout only (plist kept for auto-start on next login) — 2026-04-08 (fixed 2026-04-10)
- [✓] `daemon status`: running/stopped, PID, log tail — 2026-04-08

### Task 4.4: M3 Integration Tests
- [✓] `tests/integration/daemon_tests.rs` — 6 tests, all passing — 2026-04-08
- [✓] Log rotation at threshold boundary — 2026-04-08
- [✓] Token refresh sync vs foreign credential distinction — 2026-04-08
- [✓] In-sync no-op verified — 2026-04-08

---

## Phase 5 — Setup & Cleanup (M4)

### Task 5.1: `ccswitch init` (`src/commands/init.rs`)
- [✓] Check if `~/.claude-switcher/` already exists; if so, warn and ask to re-init — 2026-04-08
- [✓] Run `profiles::ensure_dirs()` to create directory structure — 2026-04-08
- [✓] Attempt to read existing Keychain credentials — 2026-04-08
- [✓] If credentials found: prompt `Credentials found in Keychain. Save as profile [name]: ` — 2026-04-08
- [✓] Save captured credentials as named profile — 2026-04-08
- [✓] Set that profile as active — 2026-04-08
- [✓] Start daemon via `daemon::launchd::install_plist()` — 2026-04-08
- [✓] Print success summary with next steps — 2026-04-08

### Task 5.2: `ccswitch uninstall` (`src/commands/uninstall.rs`)
- [✓] Print warning about what will be deleted — 2026-04-08
- [✓] Prompt `This will remove all ccswitch data. Continue? [y/N]: ` — 2026-04-08
- [✓] Stop daemon: `launchctl unload` + delete plist — 2026-04-08
- [✓] Delete `~/.claude-switcher/` directory and all contents — 2026-04-08
- [✓] Leave Keychain untouched (per design decision) — 2026-04-08
- [✓] Print `✓ ccswitch uninstalled. Claude Code credentials are intact.` — 2026-04-08

### Task 5.3: M4 Integration Tests
- [✓] `tests/integration/init_tests.rs` — fresh install flow, verify dir structure and active profile — 2026-04-08
- [✓] `tests/integration/uninstall_tests.rs` — verify no leftover files after uninstall — 2026-04-08

---

## Phase 6 — Release Engineering (M5 + M6)

### Task 6.1: Universal Binary Build Script (`scripts/build_universal.sh`)
- [✓] Build arm64 target: `cargo build --release --target aarch64-apple-darwin` — 2026-04-08
- [✓] Build x86_64 target: `cargo build --release --target x86_64-apple-darwin` — 2026-04-08
- [✓] Combine with `lipo -create -output dist/ccswitch` — 2026-04-08
- [✓] Generate SHA256 checksum file — 2026-04-08

### Task 6.2: GitHub Actions CI (`.github/workflows/release.yml`)
- [✓] Trigger on git tag push (`v*`) — 2026-04-08
- [✓] Matrix build: `aarch64-apple-darwin` + `x86_64-apple-darwin` on `macos-latest` — 2026-04-08
- [✓] Run `cargo test` on native target before packaging — 2026-04-08
- [✓] Run `lipo` to create universal binary in release job — 2026-04-08
- [✓] Upload binary + checksum to GitHub Release via softprops/action-gh-release — 2026-04-08

### Task 6.3: Homebrew Tap Formula
- [✓] `scripts/homebrew/ccswitch.rb` template with correct URL pattern, SHA256 placeholder, `bin.install` — 2026-04-08
- [ ] Create separate repo `homebrew-ccswitch` and copy formula (post-first-release)
- [ ] Fill SHA256 from first release artifact and verify `brew install` end-to-end

### Task 6.4: Curl Install Script
- [✓] `scripts/install.sh` — detects arm64/x86_64, downloads universal binary from latest GitHub Release — 2026-04-08
- [✓] SHA-256 checksum verification — 2026-04-08
- [✓] `lipo -info` universal binary confirmation — 2026-04-08
- [ ] Live test on both architectures (after first release is published)

---

## Phase 7 — Documentation (M7)

### Task 7.1: README.md
- [✓] App overview (what/why/who) — 2026-04-08
- [✓] Install section: Homebrew + curl one-liner — 2026-04-08
- [✓] Quick start: `init` → `add` → `use` — 2026-04-08
- [✓] Commands reference table — 2026-04-08

### Task 7.2: Command Reference
- [✓] Full man-page-style reference for every command with all flags and examples — 2026-04-08
- [✓] Include exit code table — 2026-04-08

### Task 7.3: Troubleshooting Guide (`docs/TROUBLESHOOTING.md`)
- [✓] "Keychain access denied" (TCC prompt, `security` CLI fix) — 2026-04-08
- [✓] "Profile not found" after `ccswitch use` — 2026-04-08
- [✓] Daemon not syncing — 2026-04-08
- [✓] Binary not in PATH — 2026-04-08

### Task 7.4: CHANGELOG.md
- [✓] Create initial CHANGELOG.md with v0.1.0 entry — 2026-04-08

---

## Progress Tracker

| Milestone | Status | Completed |
|-----------|--------|-----------|
| M0: Project Bootstrap | ✅ Complete | 2026-04-08 |
| M1: Core Infrastructure | ✅ Complete | 2026-04-08 |
| M2: CLI Commands | ✅ Complete | 2026-04-08 |
| M3: Daemon | ✅ Complete | 2026-04-08 |
| M4: Setup & Cleanup | ✅ Complete | 2026-04-08 |
| M5: Universal Binary | ✅ Complete | 2026-04-08 |
| M6: Distribution | ✅ Complete | 2026-04-08 |
| M7: Documentation | ✅ Complete | 2026-04-08 |
| v0.1.1 Patch | ✅ Complete | 2026-04-10 |

---

## Patch History

### v0.1.1 — 2026-04-10

**Bug: daemon not syncing after full OAuth rotation**
- Claude Code rotates both access + refresh tokens every few days. The daemon previously only synced when the refresh token was unchanged, treating full rotation as "foreign credentials" and logging WARN instead of syncing. Fixed: daemon now always syncs keychain → active profile whenever any token differs.
- Affected: `src/daemon/mod.rs` — `poll_once()`

**Bug: daemon not auto-starting on login after `daemon stop`**
- `daemon stop` called `uninstall_plist()` which deleted the plist file from `~/Library/LaunchAgents/`. Without the plist, macOS had nothing to auto-load on the next login, requiring `ccswitch daemon start` after every reboot.
- Fixed: introduced `stop_daemon()` (bootout only, plist kept) vs `uninstall_plist()` (full removal, only used by `ccswitch uninstall`). `daemon stop` now uses `stop_daemon()`.
- Affected: `src/daemon/launchd.rs`, `src/commands/daemon.rs`

**Bug: deprecated `launchctl load/unload` on macOS 15 Sequoia**
- Replaced `launchctl load` → `launchctl bootstrap gui/<uid>` and `launchctl unload` → `launchctl bootout gui/<uid>/<label>`. The new API is stable on macOS 13+ and persists correctly across reboots.
- Affected: `src/daemon/launchd.rs`

**Improvement: daemon log timestamps in local time**
- Log timestamps were UTC (`2026-04-10T14:00:27Z`). Changed to local time with offset (`2026-04-10 19:30:27 +0530`) using `chrono::Local`.
- Affected: `src/daemon/mod.rs` — `daemon_log()`

---

> Tasks marked ✅ / subtasks checked [✓] only after user confirms test run passes.
> Format: `[✓] subtask description` with timestamp on same line when completed.
> Example: `[✓] Implement read_keychain() — 2026-04-08 15:42`
