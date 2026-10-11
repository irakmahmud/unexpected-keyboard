#!/usr/bin/env bash
# ==============================================================================
# Script: fetch_remote_apk.sh
# Purpose: Fetch remotely compiled Android APKs from CircleCI Cloud Artifacts
# Mandate: ZERO LOCAL FLUTTER / ANDROID BUILD on Oracle VM.
# Storage Guard: Requires > 5 GB free space before writing artifacts to Oracle VM.
# ==============================================================================

set -euo pipefail

WORKSPACE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="${WORKSPACE_DIR}/dist/apks"

echo "===================================================================="
echo "🚀 [ZERO LOCAL BUILD] CircleCI Remote APK Fetcher & Deployer"
echo "===================================================================="
echo "Target Artifacts Directory: ${TARGET_DIR}"

# 1. Storage Safety Gate (Phase 5 requirement: > 5 GB free)
FREE_KB=$(df -k / | awk 'NR==2 {print $4}')
FREE_GB=$(( FREE_KB / 1024 / 1024 ))
echo "Current Available Disk on Oracle VM: ${FREE_GB} GB"
if [ "${FREE_GB}" -lt 5 ]; then
    echo "❌ STORAGE SAFETY GATE VIOLATION: Available disk space (${FREE_GB} GB) is less than required 5 GB."
    echo "Aborting artifact download to protect critical runtime services."
    exit 1
fi
echo "✅ Storage safety gate PASSED (${FREE_GB} GB >= 5 GB)."

mkdir -p "${TARGET_DIR}"

fetch_and_verify_apk() {
    local apk_url="${1:-}"
    local expected_checksum="${2:-}"

    if [ -z "${apk_url}" ]; then
        echo "Usage: $0 fetch <APK_DIRECT_URL> [EXPECTED_SHA256]"
        echo "Example: $0 fetch https://output.circle-artifacts.com/.../unexpected-keyboard-release.apk <hash>"
        exit 1
    fi

    local filename
    filename=$(basename "${apk_url}")
    local dest="${TARGET_DIR}/${filename}"

    echo "⬇️ Downloading remote APK from CircleCI: ${apk_url}..."
    curl -fsSL "${apk_url}" -o "${dest}"

    echo "🔒 Calculating SHA-256 Checksum..."
    local actual_checksum
    actual_checksum=$(sha256sum "${dest}" | awk '{print $1}')
    echo "Actual Checksum:   ${actual_checksum}"

    if [ -n "${expected_checksum}" ]; then
        echo "Expected Checksum: ${expected_checksum}"
        if [ "${actual_checksum}" = "${expected_checksum}" ]; then
            echo "✅ Checksum verification PASSED! APK integrity guaranteed."
        else
            echo "❌ CHECKSUM MISMATCH! Deleting corrupted or untrusted artifact."
            rm -f "${dest}"
            exit 1
        fi
    fi

    echo "📦 Artifact stored successfully: ${dest} ($(du -h "${dest}" | cut -f1))"
}

case "${1:-status}" in
    fetch)
        fetch_and_verify_apk "${2:-}" "${3:-}"
        ;;
    status)
        echo "Existing Artifacts on Oracle VM:"
        ls -lh "${TARGET_DIR}" 2>/dev/null || echo "No local artifacts stored yet. All builds remain in CircleCI cloud."
        ;;
    *)
        echo "Usage: $0 {fetch <URL> [SHA256]|status}"
        exit 1
        ;;
esac
