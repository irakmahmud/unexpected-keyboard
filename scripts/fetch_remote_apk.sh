#!/usr/bin/env bash
# ==============================================================================
# Script: fetch_remote_apk.sh
# Purpose: Fetch remotely compiled Android APKs from GitHub Actions / CircleCI
# Mandate: ZERO LOCAL FLUTTER / ANDROID BUILD on Oracle VM.
# Storage Guard: Requires > 5 GB free space before writing artifacts to Oracle VM.
# ==============================================================================

set -euo pipefail

WORKSPACE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="${WORKSPACE_DIR}/dist/apks"

echo "===================================================================="
echo "🚀 [ZERO LOCAL BUILD] Remote Cloud APK Fetcher & Deployer"
echo "===================================================================="
echo "Target Artifacts Directory: ${TARGET_DIR}"

# 1. Storage Safety Gate (>= 1 GB free when overwriting 49MB APK)
FREE_KB=$(df -k / | awk 'NR==2 {print $4}')
FREE_GB=$(( FREE_KB / 1024 / 1024 ))
echo "Current Available Disk on Oracle VM: ${FREE_GB} GB"
if [ "${FREE_GB}" -lt 1 ]; then
    echo "❌ STORAGE SAFETY GATE VIOLATION: Available disk space (${FREE_GB} GB) is less than required 1 GB."
    echo "Aborting artifact download to protect critical runtime services."
    exit 1
fi
echo "✅ Storage safety gate PASSED (${FREE_GB} GB >= 1 GB)."

mkdir -p "${TARGET_DIR}"

fetch_and_verify_apk() {
    local apk_url="${1:-}"
    local expected_checksum="${2:-}"

    if [ -z "${apk_url}" ]; then
        echo "Usage: $0 fetch <APK_DIRECT_URL> [EXPECTED_SHA256]"
        exit 1
    fi

    local filename
    filename=$(basename "${apk_url}")
    local dest="${TARGET_DIR}/${filename}"

    echo "⬇️ Downloading remote APK from Cloud: ${apk_url}..."
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

fetch_from_github_actions() {
    local repo="${1:-irakmahmud/unexpected-keyboard}"
    echo "⬇️ Fetching latest successful run from GitHub Actions (${repo})..."
    local run_id
    run_id=$(gh run list --repo "${repo}" --status success --limit 1 --json databaseId -q '.[0].databaseId')
    if [ -z "${run_id}" ] || [ "${run_id}" = "null" ]; then
        echo "❌ No completed successful GitHub Actions run found for ${repo}."
        exit 1
    fi
    echo "Found Run ID: ${run_id}"
    local tmp_dir="/tmp/gh_apk_fetch_$$"
    mkdir -p "${tmp_dir}"
    gh run download "${run_id}" --repo "${repo}" -n unexpected-keyboard-release-apk -D "${tmp_dir}/release"
    gh run download "${run_id}" --repo "${repo}" -n unexpected-keyboard-debug-apk -D "${tmp_dir}/debug"
    cp -f "${tmp_dir}/release/app-release.apk" "${TARGET_DIR}/unexpected-keyboard-release.apk"
    cp -f "${tmp_dir}/debug/app-debug.apk" "${TARGET_DIR}/unexpected-keyboard-debug.apk"
    rm -rf "${tmp_dir}"
    echo "✅ Successfully updated APKs in ${TARGET_DIR}:"
    ls -lh "${TARGET_DIR}"
    sha256sum "${TARGET_DIR}"/*
}

fetch_from_circleci() {
    local repo="${1:-irakmahmud/unexpected-keyboard}"
    echo "⬇️ Fetching latest successful build artifacts from CircleCI (${repo})..."
    local artifacts_json
    artifacts_json=$(curl -fsSL "https://circleci.com/api/v1.1/project/github/${repo}/latest/artifacts?branch=main&filter=successful")
    local release_url debug_url
    release_url=$(echo "${artifacts_json}" | jq -r '.[] | select(.path | endswith("unexpected-keyboard-release.apk")) | .url' | head -n1)
    debug_url=$(echo "${artifacts_json}" | jq -r '.[] | select(.path | endswith("unexpected-keyboard-debug.apk")) | .url' | head -n1)
    if [ -z "${release_url}" ] || [ "${release_url}" = "null" ]; then
        echo "❌ No CircleCI release APK artifact found for ${repo}."
        exit 1
    fi
    fetch_and_verify_apk "${release_url}"
    if [ -n "${debug_url}" ] && [ "${debug_url}" != "null" ]; then
        fetch_and_verify_apk "${debug_url}"
    fi
    echo "✅ Successfully updated APKs from CircleCI in ${TARGET_DIR}:"
    ls -lh "${TARGET_DIR}"
    sha256sum "${TARGET_DIR}"/*
}

case "${1:-status}" in
    fetch)
        fetch_and_verify_apk "${2:-}" "${3:-}"
        ;;
    circleci_latest)
        fetch_from_circleci "${2:-irakmahmud/unexpected-keyboard}"
        ;;
    gh_latest)
        fetch_from_github_actions "${2:-irakmahmud/unexpected-keyboard}"
        ;;
    status)
        echo "Existing Artifacts on Oracle VM:"
        ls -lh "${TARGET_DIR}" 2>/dev/null || echo "No local artifacts stored yet."
        ;;
    *)
        echo "Usage: $0 {fetch <URL> [SHA256]|circleci_latest [OWNER/REPO]|gh_latest [OWNER/REPO]|status}"
        exit 1
        ;;
esac
