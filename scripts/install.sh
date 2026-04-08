#!/usr/bin/env bash
# ccswitch installer
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/gautamkumar7atgit/ClaudeCodeSwitch/main/scripts/install.sh | bash
#
# What it does:
#   1. Detects architecture (arm64 / x86_64)
#   2. Downloads the universal binary from the latest GitHub Release
#   3. Verifies the SHA-256 checksum
#   4. Installs to /usr/local/bin/ccswitch (or $INSTALL_DIR if set)

set -euo pipefail

REPO="gautamkumar7atgit/ClaudeCodeSwitch"
BINARY="ccswitch"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# ── Helpers ──────────────────────────────────────────────────────────────────

info()  { printf '\033[36m→\033[0m  %s\n' "$*"; }
ok()    { printf '\033[32m✓\033[0m  %s\n' "$*"; }
err()   { printf '\033[31m✗\033[0m  %s\n' "$*" >&2; exit 1; }

need() {
    command -v "$1" &>/dev/null || err "Required tool not found: $1"
}

# ── Pre-flight ────────────────────────────────────────────────────────────────

need curl
need shasum

# macOS only
[[ "$(uname -s)" == "Darwin" ]] || err "ccswitch only runs on macOS"

# ── Architecture ──────────────────────────────────────────────────────────────

ARCH="$(uname -m)"
case "$ARCH" in
    arm64)   info "Detected architecture: Apple Silicon (arm64)" ;;
    x86_64)  info "Detected architecture: Intel (x86_64)" ;;
    *)       err "Unsupported architecture: $ARCH" ;;
esac

# ── Latest release ────────────────────────────────────────────────────────────

info "Fetching latest release from GitHub..."
RELEASE_URL="https://api.github.com/repos/${REPO}/releases/latest"
VERSION=$(curl -fsSL "$RELEASE_URL" | grep '"tag_name"' | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')

[[ -n "$VERSION" ]] || err "Could not determine latest release version"
info "Latest version: ${VERSION}"

BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"
BINARY_URL="${BASE_URL}/${BINARY}"
CHECKSUM_URL="${BASE_URL}/${BINARY}.sha256"

# ── Download ──────────────────────────────────────────────────────────────────

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

TMP_BIN="${TMP_DIR}/${BINARY}"
TMP_SHA="${TMP_DIR}/${BINARY}.sha256"

info "Downloading binary..."
curl -fsSL --progress-bar -o "$TMP_BIN" "$BINARY_URL"

info "Downloading checksum..."
curl -fsSL -o "$TMP_SHA" "$CHECKSUM_URL"

# ── Verify ────────────────────────────────────────────────────────────────────

info "Verifying checksum..."
EXPECTED_SUM="$(awk '{print $1}' "$TMP_SHA")"
ACTUAL_SUM="$(shasum -a 256 "$TMP_BIN" | awk '{print $1}')"

if [[ "$EXPECTED_SUM" != "$ACTUAL_SUM" ]]; then
    err "Checksum mismatch!
  Expected: ${EXPECTED_SUM}
  Got:      ${ACTUAL_SUM}"
fi
ok "Checksum verified"

# ── Verify it's a universal binary ────────────────────────────────────────────

ARCHS="$(lipo -info "$TMP_BIN" 2>/dev/null || true)"
if [[ "$ARCHS" == *"arm64"* && "$ARCHS" == *"x86_64"* ]]; then
    info "Universal binary confirmed (arm64 + x86_64)"
fi

# ── Install ───────────────────────────────────────────────────────────────────

chmod +x "$TMP_BIN"

if [[ -w "$INSTALL_DIR" ]]; then
    mv "$TMP_BIN" "${INSTALL_DIR}/${BINARY}"
else
    info "Installing to ${INSTALL_DIR}/ (may require sudo password)..."
    sudo mv "$TMP_BIN" "${INSTALL_DIR}/${BINARY}"
fi

ok "Installed ${BINARY} ${VERSION} to ${INSTALL_DIR}/${BINARY}"

# ── Post-install ──────────────────────────────────────────────────────────────

echo ""
echo "Get started:"
echo "  ccswitch init          — set up ccswitch and import existing credentials"
echo "  ccswitch add <name>    — save current credentials as a named profile"
echo "  ccswitch use <name>    — switch to a saved profile"
echo "  ccswitch --help        — see all commands"
