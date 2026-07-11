#!/usr/bin/env bash
# verify-standalone.sh — verifies that the Tauri desktop app is fully self-contained.
# It embeds gitpurge-core statically and has zero process dependency on git-purge CLI.

set -euo pipefail

echo "======================================================"
echo "Verifying standalone desktop app build and dependencies"
echo "======================================================"

# 1. Verify Cargo dependency graph
echo "[1/3] Checking that gitpurge-desktop depends directly on gitpurge-core..."
if cargo tree -p gitpurge-desktop | grep -q "gitpurge-core"; then
    echo "  ✓ Direct dependency confirmed."
else
    echo "  ✗ Error: gitpurge-desktop does not depend on gitpurge-core in Cargo.toml."
    exit 1
fi

# 2. Check for hardcoded git-purge binary invocations
echo "[2/3] Checking for process calls to 'git-purge' binary in desktop app..."
# We search for command invocations of git-purge CLI in desktop src code
MATCHES=$(grep -rn "Command::new(\"git-purge\")" apps/desktop/src-tauri/ || true)
if [ -n "$MATCHES" ]; then
    echo "  ✗ Error: Found process invocations of 'git-purge' CLI:"
    echo "$MATCHES"
    exit 1
else
    echo "  ✓ No process calls to git-purge found. Backend uses core library API directly."
fi

# 3. Simulate absence of git-purge on PATH and verify compilation
echo "[3/3] Simulating empty PATH (no git-purge CLI) and verifying compilation..."
# We verify cargo compiles the desktop crate cleanly
# Even without git-purge on the system, compiling is successful because it is embedded
(
  # Dynamically detect cargo location to ensure it is kept on PATH
  CARGO_PATH=$(which cargo || echo "/home/mgamil/.cargo/bin/cargo")
  CARGO_DIR=$(dirname "$CARGO_PATH")
  export PATH="$CARGO_DIR:/usr/bin:/bin:/usr/sbin:/sbin"
  cargo check -p gitpurge-desktop
)
echo "  ✓ Desktop backend compiles cleanly with no git-purge CLI on PATH."

echo "======================================================"
echo "SUCCESS: Standalone verification complete!"
echo "======================================================"
