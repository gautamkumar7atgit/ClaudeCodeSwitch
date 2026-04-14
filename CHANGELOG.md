# Changelog

All notable changes to ccswitch are documented here.

---

## [1.0.0] — 2026-04-13

### Added

- `ccswitch export <name|--all>` — bundle one or all saved profiles into a portable
  `.ccspack` file; encrypted by default using AES-256-GCM with an Argon2id-derived
  passphrase key (no plaintext tokens ever written without explicit `--no-encrypt`)
- `ccswitch import <file>` — restore profiles from a `.ccspack` bundle; prompts for the
  passphrase on encrypted bundles; supports `--as <name>` rename and `--overwrite`
- `--output <file>` flag on export to control the output path
- `--no-encrypt` flag for plaintext bundles in scripted/trusted environments (prints a
  visible warning before writing)
- Team-sharing workflow: admin exports → shares file + passphrase separately → teammates
  import, skipping the interactive `/login` re-auth step entirely

---

## [0.1.2] — 2026-04-13

### Fixed

- `install.sh`: detect existing Homebrew-managed install and abort with instructions to `brew uninstall ccswitch` first, preventing conflicting binaries in `$PATH`
- `install.sh`: warn when overwriting an existing curl-installed binary instead of silently replacing it

### Changed

- `scripts/homebrew/ccswitch.rb`: corrected version (`1.0.0` → `0.1.1`), filled real SHA256, updated URLs and tap references to `gautamkumar7atgit`
- Homebrew tap repo (`gautamkumar7atgit/homebrew-ccswitch`) created and published

---

## [0.1.1] — 2026-04-10

### Fixed

- Daemon: sync active profile whenever any token differs (access-only rotation was previously misclassified as foreign credentials and skipped)
- `daemon stop`: use `launchctl bootout` only — keep plist in `~/Library/LaunchAgents/` so daemon auto-starts on next login
- `daemon start`: replaced deprecated `launchctl load/unload` with `launchctl bootstrap/bootout gui/<uid>` (macOS 13+ API)

### Changed

- Daemon log timestamps now use local time with UTC offset (e.g. `2026-04-10 19:30:27 +0530`) instead of UTC

---

## [0.1.0] — 2026-04-08

Initial release.

### Added

- `ccswitch init` — one-time setup: creates `~/.claude-switcher/`, imports existing Keychain credentials as a named profile, and registers the launchd daemon
- `ccswitch add <name>` — snapshot current Keychain credentials as a named profile
- `ccswitch use <name>` — switch active account (terminates Claude processes, writes credentials to Keychain)
- `ccswitch list` — list all profiles with active marker, token fingerprint, and last-synced time
- `ccswitch remove <name>` — delete a profile with active-profile guard and `--force` bypass
- `ccswitch status` — colorized diagnostic block: active profile, daemon state, Keychain match, token fingerprints, log tail
- `ccswitch daemon start/stop/status` — manage the background launchd sync daemon
- `ccswitch uninstall` — remove all ccswitch data (Keychain untouched)
- Background daemon with 30 s poll loop: detects silent OAuth token refreshes and syncs them back to the active profile on disk
- SIGTERM handler for clean daemon shutdown
- Log rotation at 1 MB (`daemon.log` → `daemon.log.1`)
- Universal binary (arm64 + x86_64) via `lipo`
- GitHub Actions CI: matrix build, `cargo test`, universal binary assembly, GitHub Release upload
- Homebrew tap formula template
- curl install script with SHA-256 verification
