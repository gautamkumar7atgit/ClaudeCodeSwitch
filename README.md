# ccswitch

Switch between multiple Claude Code OAuth accounts on macOS by swapping credentials in the system Keychain. A background daemon watches for silent token refreshes and keeps your saved profiles in sync automatically.

---

## Install

**Homebrew (recommended)**

```bash
brew tap gautamkumar7atgit/ccswitch
brew install ccswitch
# One-time: enable auto-updates for this tap
brew tap --force-auto-update gautamkumar7atgit/ccswitch
```

**curl one-liner**

```bash
curl -fsSL https://raw.githubusercontent.com/gautamkumar7atgit/ClaudeCodeSwitch/main/scripts/install.sh | bash
```

Both methods install a universal binary (arm64 + x86_64).

---

## Update

```bash
ccswitch update
```

Detects your install method automatically:
- **Homebrew:** runs `brew upgrade ccswitch`
- **curl install:** downloads the latest binary from GitHub, verifies SHA256, and replaces the current binary in-place

---

## Quick start

```bash
# 1. One-time setup — imports your current credentials, starts the daemon
ccswitch init

# 2. Log into a second Claude Code account in the browser, then snapshot it
ccswitch add personal

# 3. Switch accounts
ccswitch use personal
ccswitch use work
```

---

## Commands

### `ccswitch init`

First-time setup. Creates `~/.claude-switcher/`, imports credentials already in your Keychain as a named profile, and registers the background daemon with launchd (auto-starts on every login).

```
ccswitch init
```

Only needs to be run once, right after installing the binary.

---

### `ccswitch add <name>`

Snapshot the credentials currently in your Keychain as a new profile named `<name>`. Run this after logging into a Claude Code account you want to save.

```
ccswitch add work
ccswitch add personal
ccswitch add --overwrite work   # overwrite without prompting
```

| Flag | Description |
|------|-------------|
| `--overwrite` | Skip the overwrite confirmation prompt |

---

### `ccswitch use <name>`

Switch to a saved profile. Terminates running `claude` / `claude-code` processes, writes the profile's credentials to the Keychain, and sets the profile as active.

```
ccswitch use work
```

Exits with code **3** if the profile is not found.

---

### `ccswitch list`

List all saved profiles. The active profile is shown first with an arrow (`→`).

```
ccswitch list
ccswitch list -v    # also show profile file paths
```

---

### `ccswitch remove <name>`

Delete a saved profile. If the profile is currently active, prompts for confirmation (or use `--force` to skip).

```
ccswitch remove old-account
ccswitch remove work --force
```

| Flag | Description |
|------|-------------|
| `--force` | Skip confirmation when removing the active profile |

---

### `ccswitch status`

Show a diagnostic snapshot of the current state.

```
ccswitch status
```

Output includes:
- Active profile name
- Daemon status and PID
- Last token sync time
- Whether Keychain credentials match the active profile
- Access token fingerprints (last 10 chars) for both profile and Keychain
- Last 3 lines of the daemon log

---

### `ccswitch daemon start`

Register and start the background sync daemon via launchd. The daemon auto-restarts on login — you only need to run this once (or after `ccswitch daemon stop`).

```
ccswitch daemon start
```

If the daemon is already running, you will be prompted to confirm a restart.

---

### `ccswitch daemon stop`

Unload the daemon from launchd and remove the plist. The daemon will not restart on next login until `ccswitch daemon start` is run again.

```
ccswitch daemon stop
```

---

### `ccswitch daemon status`

Show daemon running state, PID, and the last few log lines.

```
ccswitch daemon status
```

---

### `ccswitch export <name>`

Export a saved profile (or all profiles) to a portable `.ccspack` bundle. The bundle is
encrypted by default — you will be prompted for a passphrase.

```
ccswitch export work                         # single profile → work.ccspack
ccswitch export work --output /tmp/w.ccspack # custom output path
ccswitch export --all                        # all profiles → ccswitch-export.ccspack
ccswitch export work --no-encrypt            # plaintext bundle (trusted env only)
```

| Flag | Description |
|------|-------------|
| `--all` | Export every saved profile into one bundle |
| `--output <file>` | Write bundle to this path instead of the default |
| `--no-encrypt` | Skip passphrase encryption (prints a warning) |

---

### `ccswitch import <file>`

Import profiles from a `.ccspack` bundle. If the bundle is encrypted, you will be
prompted for the passphrase.

```
ccswitch import work.ccspack                   # import all profiles in the bundle
ccswitch import work.ccspack --as personal     # rename on import (single-profile only)
ccswitch import work.ccspack --overwrite       # skip overwrite confirmation
```

| Flag | Description |
|------|-------------|
| `--as <name>` | Rename the imported profile (single-profile bundles only) |
| `--overwrite` | Overwrite existing profiles without prompting |

---

### `ccswitch update`

Update ccswitch to the latest release. Detects your install method automatically.

```
ccswitch update
```

Homebrew users: the first time after a new release, run `brew upgrade ccswitch`. For future releases, `ccswitch update` handles it automatically.

---

### `ccswitch uninstall`

Remove all ccswitch data: `~/.claude-switcher/` and the launchd plist. **Your Keychain credentials are never touched.**

```
ccswitch uninstall
```

---

## Global flags

| Flag | Description |
|------|-------------|
| `-v`, `--verbose` | Print extra debug lines |
| `--version` | Print version and exit |
| `--help` | Print help and exit |

---

## How it works

**Switching accounts (`ccswitch use <name>`)**
1. Load profile from `~/.claude-switcher/profiles/<name>.json`
2. Send SIGTERM to any running `claude` / `claude-code` processes; wait up to 2 s, then SIGKILL any stragglers
3. Write only the `claudeAiOauth` object to the macOS Keychain
4. Write `<name>` to `~/.claude-switcher/active`

**Background daemon (30 s poll loop)**

Claude Code silently refreshes OAuth tokens in the background. The daemon detects when the Keychain access token has rotated (while the refresh token stays the same) and writes the updated tokens back to the active profile on disk so they aren't lost on the next switch.

**Sharing credentials with a team (`ccswitch export` / `ccswitch import`)**

1. Admin runs `ccswitch export work` — profile is serialized to JSON, encrypted with
   AES-256-GCM (key derived via Argon2id from the passphrase), and written to `work.ccspack`
2. Admin sends `work.ccspack` to teammates and shares the passphrase through a separate
   secure channel (e.g. password manager, Signal — never in the same Slack message as the file)
3. Each teammate runs `ccswitch import work.ccspack`, enters the passphrase, and the profile
   is saved to `~/.claude-switcher/profiles/work.json` (chmod 600)
4. Teammate runs `ccswitch use work` — no browser re-auth needed

> **Note:** Multiple people sharing the same OAuth token may occasionally trigger session
> conflicts if Claude Code invalidates concurrent sessions. If that happens, one user must
> re-authenticate and the admin should re-export.

---

## Runtime files

| Path | Purpose |
|------|---------|
| `~/.claude-switcher/profiles/<name>.json` | Saved credentials + metadata |
| `~/.claude-switcher/active` | Name of the currently active profile |
| `~/.claude-switcher/daemon.log` | Daemon log (max 1 MB, rotates to `.log.1`) |
| `~/Library/LaunchAgents/com.ccswitch.daemon.plist` | launchd registration |

---

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Usage / general error |
| 2 | Keychain error |
| 3 | Profile not found |

---

## License

MIT
