# Troubleshooting

---

## "Keychain access denied" / `security` command fails

**Symptom:** `ccswitch` exits with error code 2, or you see a message like `keychain error: ...`.

**Cause:** macOS Transparency, Consent, and Control (TCC) has not granted the terminal access to the Keychain item written by Claude Code, or the Keychain item does not exist yet.

**Fix:**

1. Open **System Settings → Privacy & Security → Full Disk Access** and make sure your terminal app (Terminal.app, iTerm2, etc.) is listed and enabled.
2. Verify the Keychain item exists:
   ```bash
   security find-generic-password -s "Claude Code-credentials" -a "credentials" -w
   ```
   If this fails, Claude Code has not been logged in yet. Launch Claude Code, complete the OAuth flow, then try again.
3. If the item exists but `ccswitch` still fails, try deleting and re-adding it via the `security` CLI:
   ```bash
   security delete-generic-password -s "Claude Code-credentials" -a "credentials"
   # Then log back in through Claude Code to recreate it
   ```

---

## "Profile not found" after `ccswitch use`

**Symptom:** Exit code 3, message like `Profile "work" not found`.

**Fix:**

```bash
ccswitch list          # see what profiles actually exist
ccswitch add work      # re-add the profile if it's missing
```

Profiles live in `~/.claude-switcher/profiles/`. If you moved or renamed files manually, the name must match the filename (without `.json`).

---

## Daemon not syncing / tokens going stale

**Symptom:** After Claude Code silently refreshes its token, `ccswitch use <name>` reverts to old credentials and Claude Code fails to authenticate.

**Checks:**

```bash
ccswitch status                 # check "Keychain match" and "Daemon" rows
ccswitch daemon status          # check running state and log tail
cat ~/.claude-switcher/daemon.log
```

**Common causes:**

- **Daemon is stopped.** Run `ccswitch daemon start` and verify it shows a PID.
- **Daemon binary path changed.** If you moved or reinstalled `ccswitch`, the plist still points to the old path. Fix:
  ```bash
  ccswitch daemon stop
  ccswitch daemon start
  ```
- **Both tokens differ (foreign credentials).** The daemon intentionally does nothing when both the access token AND refresh token in the Keychain differ from the active profile — this means a different account is active. Run `ccswitch use <name>` to re-assert the correct profile.

---

## Binary not in PATH

**Symptom:** `command not found: ccswitch`

**Fix (curl install):**

The install script places the binary at `~/.local/bin/ccswitch`. Ensure that directory is in your PATH:

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

**Fix (Homebrew):**

```bash
brew link ccswitch
```

---

## Daemon starts but immediately stops

**Symptom:** `ccswitch daemon status` shows stopped, log shows the daemon exited right after starting.

**Fix:**

Check the log for the cause:
```bash
cat ~/.claude-switcher/daemon.log
```

Common reasons:
- The binary was deleted or moved after the plist was created. Run `ccswitch daemon stop && ccswitch daemon start` to regenerate the plist with the current binary path.
- Permission error on `~/.claude-switcher/`. Verify ownership:
  ```bash
  ls -la ~/.claude-switcher/
  ```

---

## `ccswitch use` doesn't kill Claude Code

**Symptom:** Claude Code keeps running with the old credentials after a switch.

**Cause:** `ccswitch` sends SIGTERM to processes named exactly `claude` or `claude-code`. If Claude Code is running under a different process name (can happen with some wrapper scripts), it won't be targeted.

**Fix:** Quit Claude Code manually before running `ccswitch use <name>`.

---

## Re-initializing from scratch

If your setup is in a broken state:

```bash
ccswitch uninstall          # removes ~/.claude-switcher/ and the plist
ccswitch init               # fresh setup
```

Your Keychain credentials are never removed by `uninstall`.
