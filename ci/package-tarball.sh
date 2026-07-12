#!/usr/bin/env bash
set -euo pipefail

TARGET=$1
EXT=$2

# Determine version
VERSION=$(grep -m1 'version =' Cargo.toml | cut -d '"' -f2)
DIR_NAME="git-purge-${VERSION}-${TARGET}"
TMP_DIR="target/tmp/${DIR_NAME}"

mkdir -p target/tmp
rm -rf "${TMP_DIR}"
mkdir -p "${TMP_DIR}"

# Find the binary
BIN_NAME="git-purge"
if [[ "${TARGET}" == *"windows"* ]]; then
    BIN_NAME="git-purge.exe"
fi

# Check target-specific output or fallback to default release dir
SRC_BIN="target/${TARGET}/release/${BIN_NAME}"
if [ ! -f "${SRC_BIN}" ]; then
    SRC_BIN="target/release/${BIN_NAME}"
fi

if [ ! -f "${SRC_BIN}" ]; then
    echo "Error: Binary not found at target/${TARGET}/release/${BIN_NAME} or target/release/${BIN_NAME}"
    exit 1
fi

cp "${SRC_BIN}" "${TMP_DIR}/"
cp LICENSE-MIT "${TMP_DIR}/" || true
cp LICENSE-APACHE "${TMP_DIR}/" || true
cp README.md "${TMP_DIR}/" || true

if [[ "${TARGET}" == *"windows"* ]]; then
    # Create install.ps1
    cat << 'EOF' > "${TMP_DIR}/install.ps1"
& "$PSScriptRoot\git-purge.exe" install-cli $args
EOF
else
    # Create install.sh
    cat << 'EOF' > "${TMP_DIR}/install.sh"
#!/bin/sh
exec "$(dirname "$0")/git-purge" install-cli "$@"
EOF
    chmod +x "${TMP_DIR}/install.sh"
fi

mkdir -p dist

if [ "${EXT}" = "zip" ]; then
    (cd target/tmp && zip -r "../../dist/${DIR_NAME}.zip" "${DIR_NAME}")
else
    tar -czf "dist/${DIR_NAME}.tar.gz" -C target/tmp "${DIR_NAME}"
fi

rm -rf "target/tmp/${DIR_NAME}"
echo "Packaged to dist/${DIR_NAME}.${EXT}"
