#!/usr/bin/env bash
# verify-release.sh: Verifies release tarball/zip using SHA256SUMS and SHA256SUMS.sig (minisign).
# Usage: ./scripts/verify-release.sh <release-archive-path> [<sha256sums-path> <signature-path>]

set -euo pipefail

if [ "$#" -lt 1 ]; then
  echo "Usage: $0 <release-archive-path> [<sha256sums-path> <signature-path>]"
  exit 1
fi

ARCHIVE="$1"
DIR="$(dirname "$ARCHIVE")"
BASENAME="$(basename "$ARCHIVE")"

SHA256SUMS="${2:-$DIR/SHA256SUMS}"
SIG="${3:-$DIR/SHA256SUMS.sig}"
PUBKEY="$(dirname "$0")/../.scratch/minisign_keys/minisign.pub"

if [ ! -f "$ARCHIVE" ]; then
  echo "Error: Release archive '$ARCHIVE' not found."
  exit 1
fi

if [ ! -f "$SHA256SUMS" ]; then
  echo "Error: SHA256SUMS file '$SHA256SUMS' not found."
  exit 1
fi

if [ ! -f "$SIG" ]; then
  echo "Error: Signature file '$SIG' not found."
  exit 1
fi

if [ ! -f "$PUBKEY" ]; then
  echo "Error: Public key '$PUBKEY' not found."
  exit 1
fi

echo "=== Verifying Minisign Signature ==="
# Verify the SHA256SUMS file signature
if ! minisign -V -p "$PUBKEY" -m "$SHA256SUMS" -x "$SIG"; then
  echo "Error: Signature verification failed!"
  exit 2
fi
echo "✓ Signature verified successfully."

echo "=== Verifying SHA256 Checksum ==="
# Extract expected hash from SHA256SUMS
EXPECTED_HASH=$(grep "$BASENAME" "$SHA256SUMS" | awk '{print $1}')
if [ -z "$EXPECTED_HASH" ]; then
  echo "Error: Archive '$BASENAME' not listed in SHA256SUMS."
  exit 3
fi

# Compute actual hash
if command -v sha256sum >/dev/null 2>&1; then
  ACTUAL_HASH=$(sha256sum "$ARCHIVE" | awk '{print $1}')
else
  ACTUAL_HASH=$(shasum -a 256 "$ARCHIVE" | awk '{print $1}')
fi

if [ "$EXPECTED_HASH" != "$ACTUAL_HASH" ]; then
  echo "Error: Checksum mismatch!"
  echo "Expected: $EXPECTED_HASH"
  echo "Actual:   $ACTUAL_HASH"
  exit 3
fi

echo "✓ Checksum verified successfully."
echo "=== ALL CHECKS PASSED FOR $BASENAME ==="
exit 0
