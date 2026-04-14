
## v1.0.0 Full Test Suite

### Setup — build the binary first

```bash
# Build release binary
cargo build --release

# Put it in your PATH for the session
export PATH="$(pwd)/target/release:$PATH"

# Confirm version
ccswitch --version
# Expected: ccswitch 1.0.0
```

---

## Block 0 — CLI skeleton

```bash
# 0.1  Global help
ccswitch --help
# Expected: lists all subcommands including export, import

# 0.2  Subcommand help pages
ccswitch export --help
ccswitch import --help
ccswitch daemon --help

# 0.3  Unknown subcommand → exit 1
ccswitch bogus 2>&1; echo "exit: $?"
# Expected: error message, exit 1

# 0.4  No subcommand → exit 1
ccswitch 2>&1; echo "exit: $?"
# Expected: usage error
```

---

## Block 1 — init

```bash
# 1.1  First-time init
ccswitch init
# Expected: creates ~/.claude-switcher/, prompts for profile name (uses current Keychain creds),
#           starts daemon, prints success

# 1.2  Re-init guard (run again immediately)
ccswitch init
# Expected: error or "already initialized" — must NOT silently overwrite

# 1.3  Verbose init
ccswitch -v init
# Expected: same flow with extra debug lines
```

---

## Block 2 — list

```bash
# 2.1  List with at least one profile
ccswitch list
# Expected: active profile shown first (→ marker), others alphabetically

# 2.2  Verbose list (shows file paths)
ccswitch -v list

# 2.3  List on fresh machine (no profiles dir)
rm -rf ~/.claude-switcher && ccswitch list
# Expected: "No profiles found." gracefully
```

---

## Block 3 — add

```bash
# 3.1  Add current creds as a new profile
ccswitch add work
# Expected: "✓ Profile 'work' saved"

# 3.2  Add another profile name
ccswitch add personal
ccswitch list
# Expected: both 'work' and 'personal' in list

# 3.3  Name collision — interactive prompt
ccswitch add work
# Expected: "Profile 'work' already exists — overwrite? [y/N]"
# Reply: n → profile unchanged
# Run again, reply y → profile overwritten

# 3.4  Name collision with --overwrite flag (no prompt)
ccswitch add work --overwrite
# Expected: overwrites silently, no prompt

# 3.5  Verbose add
ccswitch -v add verbose-test
```

---

## Block 4 — use

```bash
# 4.1  Switch to a valid profile (Claude processes should stop + Keychain updated)
ccswitch use personal
# Expected: processes killed (or "no processes running"), Keychain updated,
#           active set to 'personal', success message

# 4.2  Verify active changed
ccswitch list
# Expected: 'personal' has → marker, 'work' does not

# 4.3  Non-existent profile → exit 3
ccswitch use ghost 2>&1; echo "exit: $?"
# Expected: "profile not found: ghost", exit 3

# 4.4  Verbose use
ccswitch -v use work
```

---

## Block 5 — status

```bash
# 5.1  Full status block
ccswitch status
# Expected: active profile name, daemon PID (or "not running"),
#           Keychain fingerprint match/mismatch, log tail

# 5.2  Status with no active profile
# First clear it:
rm ~/.claude-switcher/active 2>/dev/null; ccswitch status
# Expected: "No active profile" — no crash

# 5.3  Verbose status
ccswitch -v status
```

---

## Block 6 — remove

```bash
# 6.1  Remove a non-active profile (no prompt needed)
ccswitch use work          # make work active
ccswitch remove personal
# Expected: "✓ Profile 'personal' removed" — no prompt

# 6.2  Remove the ACTIVE profile without --force → prompt
ccswitch remove work
# Expected: "Profile 'work' is active — are you sure? [y/N]"
# Reply: n → aborted
# Reply: y → removed, active marker cleared

# 6.3  Remove active profile with --force (no prompt)
ccswitch add work          # re-add first
ccswitch use work
ccswitch remove work --force
# Expected: removed without prompt

# 6.4  Remove non-existent profile → exit 3
ccswitch remove ghost 2>&1; echo "exit: $?"
# Expected: "profile not found: ghost", exit 3
```

---

## Block 7 — daemon

```bash
# Re-add profiles for remaining tests
ccswitch add work
ccswitch use work

# 7.1  Start daemon
ccswitch daemon start
# Expected: "✓ Daemon started" — plist written to ~/Library/LaunchAgents/

# 7.2  Daemon status
ccswitch daemon status
# Expected: "Running", PID shown, log tail shown

# 7.3  Start when already running → restart prompt
ccswitch daemon start
# Expected: "Daemon is already running — restart? [y/N]"
# Reply: n → no action
# Reply: y → restarts

# 7.4  Stop daemon
ccswitch daemon stop
# Expected: "✓ Daemon stopped"
# Plist MUST still exist (boot-persistence):
ls ~/Library/LaunchAgents/com.ccswitch.daemon.plist
# Expected: file present

# 7.5  Status after stop
ccswitch daemon status
# Expected: "Not running"

# 7.6  Start again (plist still present, should work)
ccswitch daemon start

# 7.7  Verify log rotation: check log exists
ls -lh ~/.claude-switcher/daemon.log
```

---

## Block 8 — export (v1.0.0 new)

```bash
# Setup: ensure we have two profiles
ccswitch add work --overwrite
ccswitch add personal

# 8.1  Export single profile (encrypted, default output path)
ccswitch export work
# Expected: prompts "Passphrase:" and "Confirm passphrase:" twice
#           → writes work.ccspack
#           → prints recipient usage hint
ls -la work.ccspack          # must be chmod 600
file work.ccspack             # should be JSON text

# 8.2  Export with custom output path
ccswitch export personal --output /tmp/test-personal.ccspack
ls -la /tmp/test-personal.ccspack

# 8.3  Export --all
ccswitch export --all
# Expected: prompts for passphrase, writes ccswitch-export.ccspack
ls -la ccswitch-export.ccspack

# 8.4  Export --all with custom output
ccswitch export --all --output /tmp/all-profiles.ccspack

# 8.5  Passphrase mismatch → error
ccswitch export work
# At "Passphrase:" enter: abc123
# At "Confirm passphrase:" enter: wrongpass
# Expected: "passphrases do not match" error

# 8.6  Empty passphrase → error
ccswitch export work
# At both prompts, press Enter (empty)
# Expected: "passphrase must not be empty" error

# 8.7  Export --no-encrypt (plaintext warning)
ccswitch export work --no-encrypt --output /tmp/plain.ccspack
# Expected: prominent WARNING printed, file written
cat /tmp/plain.ccspack
# Expected: profiles visible in plaintext JSON

# 8.8  Cannot specify name AND --all
ccswitch export work --all 2>&1; echo "exit: $?"
# Expected: "cannot use both a profile name and --all", exit 1

# 8.9  Export with no name and no --all
ccswitch export 2>&1; echo "exit: $?"
# Expected: "specify a profile name or use --all", exit 1

# 8.10  Export non-existent profile → exit 3
ccswitch export ghost 2>&1; echo "exit: $?"
# Expected: "profile not found: ghost", exit 3

# 8.11  Export --all when no profiles exist
rm -rf ~/.claude-switcher/profiles/*.json
ccswitch export --all 2>&1; echo "exit: $?"
# Expected: "no profiles found to export", exit 1

# Re-add for import tests
ccswitch add work
ccswitch add personal
ccswitch export work --no-encrypt --output /tmp/plain.ccspack
ccswitch export personal --output /tmp/personal.ccspack
# (remember passphrase you used for personal.ccspack)
ccswitch export --all --output /tmp/all.ccspack
# (remember passphrase for all.ccspack)
```

---

## Block 9 — import (v1.0.0 new)

```bash
# 9.1  Import plaintext single-profile bundle
rm ~/.claude-switcher/profiles/work.json 2>/dev/null
ccswitch import /tmp/plain.ccspack
# Expected: "Imported 1 profile"
ccswitch list  # work should appear

# 9.2  Import encrypted single-profile bundle
rm ~/.claude-switcher/profiles/personal.json 2>/dev/null
ccswitch import /tmp/personal.ccspack
# Expected: prompts "Passphrase:", enter correct passphrase → "Imported 1 profile"
ccswitch list  # personal should appear

# 9.3  Wrong passphrase → error
ccswitch import /tmp/personal.ccspack
# Enter wrong passphrase
# Expected: "decryption failed — wrong passphrase?" error

# 9.4  Import with profile already existing → overwrite prompt
ccswitch import /tmp/plain.ccspack
# Expected: "Profile 'work' already exists — overwrite? [y/N]"
# Reply: n → "Skipped 'work'", "No profiles were imported."
# Reply: y → profile overwritten, "Imported 1 profile"

# 9.5  Import with --overwrite flag (no prompt on conflict)
ccswitch import /tmp/plain.ccspack --overwrite
# Expected: overwrites silently, "Imported 1 profile"

# 9.6  Import --all bundle
rm ~/.claude-switcher/profiles/*.json 2>/dev/null
ccswitch import /tmp/all.ccspack
# Enter passphrase for all.ccspack
# Expected: "Imported 2 profiles"
ccswitch list  # both work + personal present

# 9.7  Import with --as rename (single-profile bundle only)
ccswitch import /tmp/plain.ccspack --as renamed-work
# Expected: profile saved as 'renamed-work', not 'work'
ccswitch list  # should show renamed-work

# 9.8  --as with multi-profile bundle → error
ccswitch import /tmp/all.ccspack --as bad 2>&1; echo "exit: $?"
# Expected: "--as can only be used with single-profile bundles", exit 1

# 9.9  Non-existent bundle file → error
ccswitch import /tmp/does-not-exist.ccspack 2>&1; echo "exit: $?"
# Expected: file read error, exit 1

# 9.10  Corrupted bundle → error
echo "not valid json" > /tmp/bad.ccspack
ccswitch import /tmp/bad.ccspack 2>&1; echo "exit: $?"
# Expected: "failed to parse bundle", exit 1

# 9.11  Verbose import
ccswitch import /tmp/plain.ccspack --overwrite -v
# Expected: per-profile "saved profile 'work'" lines
```

---

## Block 10 — export → import round-trip

```bash
# 10.1  Encrypted round-trip: export then delete then import
ccswitch export work --output /tmp/rt-work.ccspack
# passphrase: testpass123

rm ~/.claude-switcher/profiles/work.json
ccswitch list  # work should be gone

ccswitch import /tmp/rt-work.ccspack
# passphrase: testpass123
ccswitch list  # work should be back

# 10.2  Verify credentials match (check via status fingerprint)
ccswitch use work
ccswitch status
# Expected: "Keychain match" line shows ✓
```

---

## Block 11 — uninstall

```bash
# 11.1  Uninstall (interactive confirm)
ccswitch uninstall
# Expected: big warning, "Are you sure? [y/N]"
# Reply: n → aborted

# 11.2  Uninstall confirmed
ccswitch uninstall
# Reply: y
# Expected: daemon stopped, plist deleted, ~/.claude-switcher/ deleted
# Keychain UNTOUCHED (verify: security find-generic-password -s "Claude Code-credentials")
ls ~/.claude-switcher 2>&1     # must be gone
ls ~/Library/LaunchAgents/com.ccswitch.daemon.plist 2>&1  # must be gone

# 11.3  After uninstall: ccswitch list → no crash
ccswitch list
# Expected: "No profiles found." (dirs don't exist yet)
```

---

## Block 12 — exit code verification

```bash
# 12.1  Exit 0 on success
ccswitch init; echo "exit: $?"

# 12.2  Exit 1 on usage error
ccswitch export 2>&1; echo "exit: $?"   # no name, no --all

# 12.3  Exit 3 on not-found
ccswitch use ghost 2>&1; echo "exit: $?"

# 12.4  Exit 2 on Keychain error (simulate by revoking access in Keychain Access app,
#        then run ccswitch add test)
# Note: hard to automate, but worth a manual spot-check
```

---

## Block 13 — unit + integration tests (automated)

```bash
# Run the full test suite (does NOT touch the real Keychain)
cargo test -- --nocapture 2>&1 | tee test-output.txt

# Check for failures
grep -E "FAILED|error\[" test-output.txt

# Bundle-specific unit tests
cargo test bundle 2>&1
```

---

## Pass criteria

| Command | Check |
|---------|-------|
| `init` | Dirs created, profile saved, daemon started |
| `add` | Profile file exists, chmod 600, collision prompt works |
| `use` | Keychain updated, active file updated, processes killed |
| `list` | Active first, alphabetical, verbose shows paths |
| `remove` | Guard for active, `--force` skips prompt, clears active |
| `status` | PID shown, fingerprint match/mismatch shown |
| `daemon start/stop/status` | Plist persists after stop, PID shown, restart prompt |
| `export` | `.ccspack` written, chmod 600, passphrase mismatch caught, `--no-encrypt` warns |
| `import` | Decrypts correctly, `--as` rename, `--overwrite`, conflict prompt, multi-profile |
| Round-trip | Export→delete→import restores working credentials |
| `uninstall` | Dirs gone, plist gone, Keychain untouched |
| Exit codes | 0/1/2/3 correct in all paths |