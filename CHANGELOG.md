# Changelog

All notable changes to ccswitch are documented here.

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
