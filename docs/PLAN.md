# CCSwitch — Development Plan

> **How to read this file.**
> Top-level sections map to released (or in-progress) versions. Each version section contains
> its own motivation, design decisions, and task breakdown. Tasks are marked `[✓]` only after
> a user-confirmed test run passes, with a completion date. The global design decisions at the
> top apply across all versions unless a version section overrides them.

---

## Global Design Decisions

These decisions were locked in during the initial design interview and apply to every version
unless a specific version section notes an override.

| Topic | Decision |
|-------|----------|
| CLI output | Colored + Unicode icons (→ ✓ ✗ !) via ANSI codes |
| Profile storage | Plaintext JSON, chmod 600, `~/.claude-switcher/profiles/` |
| Keychain writes | Only the `claudeAiOauth` object — strip `_meta` before writing |
| Process targets | `claude` (exact match) + `claude-code` — never `node` or others |
| Active profile removal | Warn + require `--force` or `y/N` prompt |
| `init` with existing creds | Prompt user for profile name |
| Daemon on Keychain mismatch | Log WARN only, do nothing |
| `status` output | Full diagnostic block (PID, fingerprint, log tail) |
| `use` process shutdown | SIGTERM → wait 2 s → SIGKILL stragglers, proceed regardless |
| `daemon start` when running | Log error + confirm to restart |
| `list` sort order | Active profile first, then alphabetical |
| `uninstall` Keychain | Leave Keychain untouched |
| Distribution | Homebrew tap + curl script simultaneously |
| `add` name collision | Prompt to overwrite (or `--overwrite` flag to skip) |
| Verbose flag | Global `-v / --verbose` flag passed to all handlers |
| Daemon management | launchd only — `~/Library/LaunchAgents/` |
| Atomic writes | `.tmp` → `fs::rename()` for every profile/state file |
| Path resolution | `dirs::home_dir()` via `config::paths()` — never hardcode `~` |
| Error handling | `anyhow::Result<T>` everywhere + `.context("what failed")` |
| Testing Keychain | Service name `ccswitch-test` in tests, never the real service |

---

## Progress Tracker

| Version | Status | Released |
|---------|--------|----------|
| v0.1.0 — Initial release | ✅ Complete | 2026-04-08 |
| v0.1.1 — Daemon + launchd fixes | ✅ Complete | 2026-04-10 |
| v0.1.2 — Install script + formula fixes | ✅ Complete | 2026-04-13 |
| v1.0.0 — Profile export / import | ✅ Complete | 2026-04-13 |

---

---

## v0.1.0 — Initial Release

**Released:** 2026-04-08
**Scope:** Full working CLI + daemon, Homebrew tap, curl installer, documentation.

### What was shipped

- `ccswitch init / add / use / list / remove / status` — all core account-switching commands
- `ccswitch daemon start/stop/status` — launchd-managed background sync daemon
- `ccswitch uninstall` — full cleanup, Keychain untouched
- 30-second poll daemon: detects silent OAuth token refreshes, syncs back to active profile
- Universal binary (arm64 + x86_64) via `lipo`; CI on GitHub Actions
- Homebrew tap formula template + curl install script with SHA-256 verification

---

### Phase 0 — Project Bootstrap

#### Task 0.1: Initialize Rust Project
- [✓] Run `cargo init` in project root, set edition = 2021 — 2026-04-08
- [✓] Add all dependencies to `Cargo.toml` (`clap`, `serde`, `serde_json`, `anyhow`, `chrono`, `nix`, `plist`, `dirs`, `log`, `env_logger`) — 2026-04-08
- [✓] Create full directory skeleton (`src/cli/`, `src/commands/`, `src/daemon/`, `tests/integration/`, `tests/fixtures/`, `scripts/`) — 2026-04-08
- [✓] Verify `cargo check` passes with empty stubs — 2026-04-08

#### Task 0.2: Constants & Config (`src/config.rs`)
- [✓] Define `KEYCHAIN_SERVICE = "Claude Code-credentials"` — 2026-04-08
- [✓] Define `KEYCHAIN_ACCOUNT = "credentials"` — 2026-04-08
- [✓] Define `PROFILES_DIR = ~/.claude-switcher/profiles/` — 2026-04-08
- [✓] Define `ACTIVE_FILE = ~/.claude-switcher/active` — 2026-04-08
- [✓] Define `DAEMON_LOG = ~/.claude-switcher/daemon.log` — 2026-04-08
- [✓] Define `DAEMON_LOG_MAX_BYTES = 1_048_576` (1 MB) — 2026-04-08
- [✓] Define `POLL_INTERVAL_SECS = 30` — 2026-04-08
- [✓] Define exit code constants (0 success, 1 usage, 2 keychain, 3 not found) — 2026-04-08
- [✓] Implement `config::paths()` helper returning resolved absolute paths via `dirs::home_dir()` — 2026-04-08

---

### Phase 1 — Core Infrastructure

#### Task 1.1: Keychain Module (`src/keychain.rs`)
- [✓] Implement `read_keychain() -> Result<KeychainCredentials>` — 2026-04-08
- [✓] Parse raw JSON output into `KeychainCredentials` struct — 2026-04-08
- [✓] Implement `write_keychain(creds: &OAuthCredentials) -> Result<()>` — 2026-04-08
- [✓] Map exit codes to typed errors (exit 44 → error code 2) — 2026-04-08
- [✓] Implement `credential_fingerprint(creds: &OAuthCredentials) -> String` — 2026-04-08

#### Task 1.2: Profile Module (`src/profiles.rs`)
- [✓] Define `Profile`, `OAuthCredentials`, `ProfileMeta` structs with correct serde renames — 2026-04-08
- [✓] Implement `list_profiles() -> Result<Vec<Profile>>` — 2026-04-08
- [✓] Implement `load_profile(name: &str) -> Result<Profile>` — 2026-04-08
- [✓] Implement `save_profile(profile: &Profile) -> Result<()>` — atomic write (`.tmp` → rename), chmod 600 — 2026-04-08
- [✓] Implement `delete_profile(name: &str) -> Result<()>` — 2026-04-08
- [✓] Implement `get_active_profile() -> Result<Option<String>>` — 2026-04-08
- [✓] Implement `set_active_profile(name: &str) -> Result<()>` — 2026-04-08
- [✓] Implement `clear_active_profile() -> Result<()>` — 2026-04-08
- [✓] Implement `ensure_dirs() -> Result<()>` — 2026-04-08

#### Task 1.3: Process Module (`src/process.rs`)
- [✓] Implement `find_claude_pids() -> Result<Vec<u32>>` — 2026-04-08
- [✓] Implement `kill_claude_processes() -> Result<usize>` — SIGTERM → wait 2s → SIGKILL — 2026-04-08
- [✓] Return `Ok(0)` when no processes found — 2026-04-08

#### Task 1.4: Output Utilities (`src/output.rs`)
- [✓] Define ANSI color constants — 2026-04-08
- [✓] Implement `print_success / print_error / print_warn / print_info / print_verbose` — 2026-04-08
- [✓] Implement `format_profile_row(profile, is_active) -> String` — 2026-04-08
- [✓] Implement `format_duration_ago(ts: DateTime<Utc>) -> String` — 2026-04-08
- [✓] Implement `confirm_prompt(msg: &str) -> Result<bool>` — 2026-04-08

#### Task 1.5: Integration Tests (M1)
- [✓] `tests/integration/keychain_tests.rs` — 3 tests passing — 2026-04-08
- [✓] `tests/integration/profile_tests.rs` — 3 tests passing — 2026-04-08
- [✓] `tests/fixtures/sample_profile.json` — 2026-04-08

---

### Phase 2 — CLI Structure

#### Task 2.1: CLI Definitions (`src/cli/mod.rs`)
- [✓] Define `Cli` struct with `#[command(author, version, about)]` — 2026-04-08
- [✓] Add global `--verbose / -v` flag — 2026-04-08
- [✓] Define `Commands` enum: `Add`, `Use`, `List`, `Remove`, `Status`, `Daemon`, `Init`, `Uninstall` — 2026-04-08
- [✓] Verify `ccswitch --help` renders correctly — 2026-04-08

#### Task 2.2: Entry Point (`src/main.rs`)
- [✓] Parse args, extract `verbose` flag, dispatch to handlers — 2026-04-08
- [✓] Map `anyhow::Error` to correct exit codes (2 = keychain, 3 = not found, 1 = other) — 2026-04-08
- [✓] Initialize `env_logger` — 2026-04-08

---

### Phase 3 — CLI Commands

#### Task 3.1: `ccswitch list` (`src/commands/list.rs`)
- [✓] Load all profiles + active profile name — 2026-04-08
- [✓] Sort: active first, then alphabetical — 2026-04-08
- [✓] `No profiles found.` if empty; verbose mode also shows file paths — 2026-04-08

#### Task 3.2: `ccswitch status` (`src/commands/status.rs`)
- [✓] Full diagnostic block: active profile, daemon PID, Keychain match, token fingerprints, log tail — 2026-04-08
- [✓] Graceful handling of no active profile / daemon not running — 2026-04-08

#### Task 3.3: `ccswitch add <name>` (`src/commands/add.rs`)
- [✓] Read Keychain → build Profile → save; overwrite prompt if name exists — 2026-04-08

#### Task 3.4: `ccswitch use <name>` (`src/commands/use_profile.rs`)
- [✓] Load profile (exit 3 if missing); kill processes; write Keychain; set active — 2026-04-08

#### Task 3.5: `ccswitch remove <name>` (`src/commands/remove.rs`)
- [✓] Guard for active profile removal (`--force` or `y/N`); clear active marker if needed — 2026-04-08

#### Task 3.6: Integration Tests (M2)
- [✓] `tests/integration/cli_tests.rs` — 6 tests passing — 2026-04-08

---

### Phase 4 — Daemon

#### Task 4.1: Daemon Poll Loop (`src/daemon/mod.rs`)
- [✓] `run_daemon()` entry point via `--daemon` internal flag — 2026-04-08
- [✓] Log to `daemon.log` with local-time timestamps — 2026-04-08
- [✓] Log rotation at 1 MB (`daemon.log` → `daemon.log.1`) — 2026-04-08
- [✓] 30-second poll loop: read Keychain → compare → sync if any token differs — 2026-04-08
- [✓] SIGTERM handler via atomic flag for clean shutdown — 2026-04-08

#### Task 4.2: Launchd Integration (`src/daemon/launchd.rs`)
- [✓] `generate_plist(binary_path)` — plist content as `String` — 2026-04-08
- [✓] `install_plist(binary_path)` — write plist + `launchctl bootstrap` — 2026-04-08
- [✓] `stop_daemon()` — `launchctl bootout` only, plist kept for auto-start on login — 2026-04-08
- [✓] `uninstall_plist()` — full removal: bootout + delete plist file — 2026-04-08
- [✓] `daemon_is_loaded() -> bool` — 2026-04-08
- [✓] `get_daemon_pid() -> Option<u32>` — 2026-04-08

#### Task 4.3: `ccswitch daemon {start|stop|status}` (`src/commands/daemon.rs`)
- [✓] `start` — idempotency check, restart prompt, bootstrap — 2026-04-08
- [✓] `stop` — not-running guard, `bootout` only, plist preserved — 2026-04-08
- [✓] `status` — running/stopped, PID, log tail — 2026-04-08

#### Task 4.4: Integration Tests (M3)
- [✓] `tests/integration/daemon_tests.rs` — 6 tests passing — 2026-04-08

---

### Phase 5 — Setup & Cleanup

#### Task 5.1: `ccswitch init` (`src/commands/init.rs`)
- [✓] Re-init guard; create dirs; import existing Keychain creds as named profile; start daemon — 2026-04-08

#### Task 5.2: `ccswitch uninstall` (`src/commands/uninstall.rs`)
- [✓] Warning + confirm prompt; stop daemon + delete plist; delete `~/.claude-switcher/`; leave Keychain untouched — 2026-04-08

#### Task 5.3: Integration Tests (M4)
- [✓] `tests/integration/init_tests.rs` — 2026-04-08
- [✓] `tests/integration/uninstall_tests.rs` — 2026-04-08

---

### Phase 6 — Release Engineering

#### Task 6.1: Universal Binary (`scripts/build_universal.sh`)
- [✓] Build arm64 + x86_64 targets; combine with `lipo`; generate SHA256 — 2026-04-08

#### Task 6.2: GitHub Actions CI (`.github/workflows/release.yml`)
- [✓] Trigger on `v*` tag; matrix build; `cargo test`; `lipo`; upload to GitHub Release — 2026-04-08

#### Task 6.3: Homebrew Tap Formula (`scripts/homebrew/ccswitch.rb`)
- [✓] Formula template with correct URL, SHA256, `bin.install` — 2026-04-08

#### Task 6.4: Curl Install Script (`scripts/install.sh`)
- [✓] Arch detection; download from latest Release; SHA-256 verify; `lipo -info` check — 2026-04-08

---

### Phase 7 — Documentation

#### Task 7.1–7.4: Docs
- [✓] `README.md` — overview, install, quick start, full command reference, how it works — 2026-04-08
- [✓] `docs/TROUBLESHOOTING.md` — Keychain denied, profile not found, daemon not syncing, PATH issues — 2026-04-08
- [✓] `CHANGELOG.md` — v0.1.0 entry created — 2026-04-08

---

---

## v0.1.1 — Daemon & Launchd Fixes

**Released:** 2026-04-10
**Scope:** Three bugs discovered during live use, no new commands.

### Bug fixes

#### Daemon not syncing after full OAuth token rotation
- **Root cause:** Daemon only synced when the refresh token was unchanged. Claude Code
  periodically rotates both access _and_ refresh tokens; the daemon treated that as "foreign
  credentials" and logged WARN instead of syncing.
- **Fix:** `poll_once()` now syncs whenever _any_ token differs (access-only or full rotation).
- **File:** `src/daemon/mod.rs`

#### Daemon not auto-starting on login after `ccswitch daemon stop`
- **Root cause:** `daemon stop` called `uninstall_plist()`, which deleted the plist from
  `~/Library/LaunchAgents/`. Without the plist, macOS had nothing to load on the next login,
  requiring `ccswitch daemon start` after every reboot.
- **Fix:** Introduced `stop_daemon()` (bootout only, plist kept) separate from
  `uninstall_plist()` (full removal, used only by `ccswitch uninstall`). `daemon stop` now
  calls `stop_daemon()`.
- **Files:** `src/daemon/launchd.rs`, `src/commands/daemon.rs`

#### Deprecated `launchctl load/unload` on macOS 13+ (Sequoia)
- **Root cause:** `launchctl load / unload` are deprecated and behave unreliably on macOS 13+.
- **Fix:** Replaced with `launchctl bootstrap gui/<uid>` (start) and
  `launchctl bootout gui/<uid>/<label>` (stop) — the stable modern API.
- **File:** `src/daemon/launchd.rs`

### Improvement

#### Daemon log timestamps in local time
- Timestamps were UTC (`2026-04-10T14:00:27Z`). Changed to local time with UTC offset
  (`2026-04-10 19:30:27 +0530`) using `chrono::Local`.
- **File:** `src/daemon/mod.rs` — `daemon_log()`

---

---

## v0.1.2 — Install Script & Formula Fixes

**Released:** 2026-04-13
**Scope:** Distribution fixes only — no code changes to the binary itself.

### Changes

#### `scripts/install.sh` — Homebrew conflict detection
- Detect existing Homebrew-managed `ccswitch` binary in `$PATH` (`/opt/homebrew/` or `/usr/local/` paths).
- Abort with instructions (`brew uninstall ccswitch`) instead of silently overwriting the
  Homebrew-managed file (which would leave two conflicting copies in `$PATH`).
- Warn (but proceed) when overwriting an existing curl-installed binary.
- **File:** `scripts/install.sh`

#### `scripts/homebrew/ccswitch.rb` — Formula corrections
- Version placeholder `1.0.0` corrected to `0.1.1`.
- Real SHA256 filled in; URL and tap references updated to `gautamkumar7atgit`.
- Removed deprecated `bottle :unneeded` line (causes `ArgumentError` in Homebrew 4.x).
- **File:** `scripts/homebrew/ccswitch.rb`

#### Homebrew tap repo created
- `gautamkumar7atgit/homebrew-ccswitch` published on GitHub (public, required by Homebrew).
- Formula pushed; `brew tap gautamkumar7atgit/ccswitch && brew install ccswitch` verified
  end-to-end on a live machine.

#### README.md URL fixes
- Both the Homebrew install command and the curl URL had stale `gautam/ccswitch` placeholders.
  Updated to the correct `gautamkumar7atgit` GitHub username.

---

---

## v1.0.0 — Profile Export / Import (Team Sharing)

**Released:** 2026-04-13
**Scope:** New `export` and `import` commands enabling teams to share credentials without
re-authenticating through Claude Code's `/login` flow.

### Motivation

Every team member previously had to complete Claude Code's interactive OAuth `/login` flow
independently. This meant credentials could not be centrally managed, and onboarding required
each person to authenticate themselves. With export/import:
- An admin exports one or more profiles to a portable `.ccspack` file (encrypted).
- Teammates import the file using a shared passphrase — no browser auth needed.
- The passphrase acts as the access-control gate; the admin controls who receives it.

### New commands

| Command | Description |
|---------|-------------|
| `ccswitch export <name>` | Bundle a single profile into `<name>.ccspack` |
| `ccswitch export --all` | Bundle every saved profile into `ccswitch-export.ccspack` |
| `ccswitch import <file>` | Restore profiles from a `.ccspack` bundle |

**Export flags:** `--output <file>`, `--no-encrypt` (plaintext, prints a warning)
**Import flags:** `--as <name>` (rename; single-profile bundles only), `--overwrite`

### Bundle file format (`.ccspack`)

```jsonc
// Encrypted variant (default):
{
  "version": 1,
  "created_at": "2026-04-13T10:00:00Z",
  "encrypted": true,
  "payload": "<base64 of: version(4B) || salt(32B) || nonce(12B) || AES-256-GCM ciphertext>"
}

// Plaintext variant (--no-encrypt):
{
  "version": 1,
  "created_at": "2026-04-13T10:00:00Z",
  "encrypted": false,
  "profiles": [{ "claudeAiOauth": {...}, "_meta": {...} }]
}
```

The encrypted payload decrypts to a JSON array of full `Profile` objects (same schema as
on-disk `.json` files). The binary blob inside base64 is:
```
[0..4]   version  u32 little-endian (= 1)
[4..36]  salt     32 bytes random   (Argon2id KDF input)
[36..48] nonce    12 bytes random   (AES-256-GCM nonce)
[48..]   AES-256-GCM authenticated ciphertext
```

### Cryptographic design decisions (v1.0.0)

| Choice | Decision | Reason |
|--------|----------|--------|
| KDF | Argon2id (m=64 MB, t=3, p=1) | Memory-hard, OWASP/RFC 9106 recommended |
| Cipher | AES-256-GCM | AEAD — detects tampering; hardware-accelerated on Apple Silicon |
| Encoding | Standard Base64 | JSON-safe, no extra dependency |
| Passphrase input | `rpassword` (no terminal echo) | Prevents shoulder-surfing |
| Output file perms | chmod 600 (atomic write) | Consistent with profile files |
| `--no-encrypt` | Allowed with visible warning | Enables scripted / CI use cases |

### New dependencies added to `Cargo.toml`

| Crate | Version | Purpose |
|-------|---------|---------|
| `aes-gcm` | 0.10 | AES-256-GCM authenticated encryption |
| `argon2` | 0.5 | Argon2id passphrase-to-key derivation |
| `rand` | 0.8 | Cryptographically secure random bytes (salt, nonce) |
| `base64` | 0.22 | Encode encrypted payload for JSON storage |
| `rpassword` | 7 | Read passphrase from terminal without echo |

### New files

| File | Purpose |
|------|---------|
| `src/bundle.rs` | `ExportBundle` struct; `encrypt_profiles`, `decrypt_profiles`, `read_bundle`, `write_bundle` |
| `src/commands/export.rs` | Export command handler |
| `src/commands/import.rs` | Import command handler |

### Modified files

| File | Change |
|------|--------|
| `Cargo.toml` | Version → `1.0.0`; added 5 crypto dependencies |
| `src/lib.rs` | Registered `pub mod bundle` |
| `src/cli/mod.rs` | Added `Export` and `Import` variants to `Commands`; imported `PathBuf` |
| `src/main.rs` | Added match arms for `Export` and `Import` |
| `src/commands/mod.rs` | Registered `pub mod export` and `pub mod import` |
| `CHANGELOG.md` | v1.0.0 section added |
| `README.md` | `export` / `import` command docs + team-sharing workflow section |

### Task breakdown

#### Task 1: Dependencies + version bump
- [✓] Add `aes-gcm`, `argon2`, `rand`, `base64`, `rpassword` to `Cargo.toml` — 2026-04-13
- [✓] Bump version `0.1.2` → `1.0.0` — 2026-04-13

#### Task 2: Bundle module (`src/bundle.rs`)
- [✓] Define `ExportBundle` struct with `#[serde(skip_serializing_if = "Option::is_none")]` fields — 2026-04-13
- [✓] Implement `encrypt_profiles(profiles, passphrase) -> Result<String>` — 2026-04-13
- [✓] Implement `decrypt_profiles(payload, passphrase) -> Result<Vec<Profile>>` — 2026-04-13
- [✓] Implement `write_bundle(bundle, path)` — atomic write, chmod 600 — 2026-04-13
- [✓] Implement `read_bundle(path) -> Result<ExportBundle>` — 2026-04-13

#### Task 3: Export command (`src/commands/export.rs`)
- [✓] Validate: name XOR `--all`, bail if neither — 2026-04-13
- [✓] Load profile(s) via `profiles::load_profile` / `profiles::list_profiles` — 2026-04-13
- [✓] Passphrase prompt twice + match check via `rpassword::prompt_password` — 2026-04-13
- [✓] Encrypt via `bundle::encrypt_profiles` — 2026-04-13
- [✓] Default output paths: `<name>.ccspack` or `ccswitch-export.ccspack` — 2026-04-13
- [✓] Plaintext path: print prominent warning before writing — 2026-04-13
- [✓] Print success with recipient usage hint — 2026-04-13

#### Task 4: Import command (`src/commands/import.rs`)
- [✓] Read bundle via `bundle::read_bundle` — 2026-04-13
- [✓] Prompt passphrase and decrypt if `bundle.encrypted` — 2026-04-13
- [✓] `--as` rename guard (single-profile only) — 2026-04-13
- [✓] Per-profile overwrite prompt (or `--overwrite` to skip) — 2026-04-13
- [✓] Save via `profiles::save_profile` — 2026-04-13
- [✓] Print imported count + skipped count — 2026-04-13

#### Task 5: CLI wiring
- [✓] `Export` + `Import` variants in `src/cli/mod.rs` — 2026-04-13
- [✓] Match arms in `src/main.rs` — 2026-04-13
- [✓] Module registrations in `src/commands/mod.rs` and `src/lib.rs` — 2026-04-13

#### Task 6: Documentation
- [✓] `CHANGELOG.md` — v1.0.0 entry — 2026-04-13
- [✓] `README.md` — export/import command docs + team-sharing workflow — 2026-04-13

#### Task 7: Build verification
- [✓] `cargo build` — clean, no warnings — 2026-04-13

### Security notes

- Encrypted bundles use AES-256-GCM (authenticated encryption) — any tampering invalidates decryption.
- The Argon2id KDF (64 MB RAM, 3 iterations) makes brute-force attacks against the passphrase
  expensive even on modern hardware.
- Bundle files are written with chmod 600 (same as profile files).
- **Caveat — shared tokens:** Multiple team members holding the same OAuth token may trigger
  session conflicts if Claude Code invalidates concurrent sessions. If this happens, one member
  must re-authenticate and the admin should re-export.
- `--no-encrypt` is for scripted/CI pipelines in fully trusted environments only. Never
  transmit a plaintext bundle over an untrusted channel.

---

> Task format: `[✓] description — YYYY-MM-DD`
> Subtasks are checked only after a user-confirmed test run passes.
