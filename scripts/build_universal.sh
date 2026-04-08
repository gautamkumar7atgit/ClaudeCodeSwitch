#!/usr/bin/env bash
# Build a universal macOS binary (arm64 + x86_64) for ccswitch.
# Requires both targets to be installed:
#   rustup target add aarch64-apple-darwin x86_64-apple-darwin
#
# Output:
#   dist/ccswitch              — universal binary
#   dist/ccswitch.sha256       — SHA-256 checksum

set -euo pipefail

BINARY="ccswitch"
DIST="dist"

# ── Targets ─────────────────────────────────────────────────────────────────

ARM_TARGET="aarch64-apple-darwin"
X86_TARGET="x86_64-apple-darwin"

ARM_BIN="target/${ARM_TARGET}/release/${BINARY}"
X86_BIN="target/${X86_TARGET}/release/${BINARY}"

# ── Build ────────────────────────────────────────────────────────────────────

echo "==> Building ${ARM_TARGET}..."
cargo build --release --target "${ARM_TARGET}"

echo "==> Building ${X86_TARGET}..."
cargo build --release --target "${X86_TARGET}"

# ── Combine ──────────────────────────────────────────────────────────────────

mkdir -p "${DIST}"
UNIVERSAL="${DIST}/${BINARY}"

echo "==> Creating universal binary with lipo..."
lipo -create \
    "${ARM_BIN}" \
    "${X86_BIN}" \
    -output "${UNIVERSAL}"

lipo -info "${UNIVERSAL}"

# ── Checksum ─────────────────────────────────────────────────────────────────

echo "==> Generating SHA-256 checksum..."
shasum -a 256 "${UNIVERSAL}" | tee "${DIST}/${BINARY}.sha256"

echo ""
echo "Done. Artifacts:"
echo "  ${UNIVERSAL}"
echo "  ${DIST}/${BINARY}.sha256"
