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

# 1. Storage Safety Gate (>= 100 MB free when overwriting 47MB APKs; skipped for tmpfiles upload)
FREE_KB=$(df -k / | awk 'NR==2 {print $4}')
FREE_MB=$(( FREE_KB / 1024 ))
echo "Current Available Disk on Oracle VM: ${FREE_MB} MB"
if [ "${1:-}" != "tmpfiles" ] && [ "${FREE_MB}" -lt 100 ]; then
    echo "❌ STORAGE SAFETY GATE VIOLATION: Available disk space (${FREE_MB} MB) is less than required 100 MB."
    echo "Aborting artifact download to protect critical runtime services."
    exit 1
fi
echo "✅ Storage safety gate PASSED (${FREE_MB} MB available)."

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
    gh run download "${run_id}" --repo "${repo}" -n unexpected-keyboard-arm64-apk -D "${tmp_dir}/arm64" 2>/dev/null || true
    gh run download "${run_id}" --repo "${repo}" -n unexpected-keyboard-release-apk -D "${tmp_dir}/release"
    if [ -f "${tmp_dir}/arm64/unexpected-keyboard-arm64.apk" ]; then
        cp -f "${tmp_dir}/arm64/unexpected-keyboard-arm64.apk" "${TARGET_DIR}/unexpected-keyboard-arm64.apk"
    fi
    if [ -f "${tmp_dir}/release/unexpected-keyboard-release.apk" ]; then
        cp -f "${tmp_dir}/release/unexpected-keyboard-release.apk" "${TARGET_DIR}/unexpected-keyboard-release.apk"
    elif [ -f "${tmp_dir}/release/app-release.apk" ]; then
        cp -f "${tmp_dir}/release/app-release.apk" "${TARGET_DIR}/unexpected-keyboard-release.apk"
    fi
    rm -rf "${tmp_dir}"
    echo "✅ Successfully updated APKs in ${TARGET_DIR}:"
    ls -lh "${TARGET_DIR}"
    sha256sum "${TARGET_DIR}"/*
    upload_to_tmpfiles
}

upload_to_tmpfiles() {
    echo "===================================================================="
    echo "⚡ Uploading APKs to tmpfiles.org Fast Download Server..."
    echo "===================================================================="
    for apk_name in "unexpected-keyboard-arm64.apk" "unexpected-keyboard-release.apk" "unexpected-keyboard-arm32.apk"; do
        local apk_path="${TARGET_DIR}/${apk_name}"
        if [ -f "${apk_path}" ]; then
            local resp page_url direct_url
            resp=$(curl -fsSL -F "file=@${apk_path}" https://tmpfiles.org/api/v1/upload 2>/dev/null || true)
            page_url=$(echo "${resp}" | jq -r '.data.url // empty' | sed 's|^http://|https://|')
            if [ -n "${page_url}" ]; then
                direct_url=$(curl -fsSL "${page_url}" 2>/dev/null | grep -oE 'https://tmpfiles\.org/dl/[^"]+' | head -n1 || true)
                echo "📦 ${apk_name}:"
                echo "   - Fast Direct Download URL: ${direct_url}"
                echo "   - Download Page URL:        ${page_url}"
            fi
        fi
    done
}

fetch_from_circleci() {
    local repo="${1:-irakmahmud/unexpected-keyboard}"
    echo "⬇️ Fetching latest successful build artifacts from CircleCI (${repo})..."
    local artifacts_json
    artifacts_json=$(curl -fsSL "https://circleci.com/api/v1.1/project/github/${repo}/latest/artifacts?branch=main&filter=successful")
    local arm64_url arm32_url release_url
    arm64_url=$(echo "${artifacts_json}" | jq -r '.[] | select(.path | endswith("unexpected-keyboard-arm64.apk")) | .url' | head -n1)
    arm32_url=$(echo "${artifacts_json}" | jq -r '.[] | select(.path | endswith("unexpected-keyboard-arm32.apk")) | .url' | head -n1)
    release_url=$(echo "${artifacts_json}" | jq -r '.[] | select(.path | endswith("unexpected-keyboard-release.apk")) | .url' | head -n1)
    if [ -z "${release_url}" ] || [ "${release_url}" = "null" ]; then
        echo "❌ No CircleCI release APK artifact found for ${repo}."
        exit 1
    fi
    if [ -n "${arm64_url}" ] && [ "${arm64_url}" != "null" ]; then
        fetch_and_verify_apk "${arm64_url}"
    fi
    if [ -n "${arm32_url}" ] && [ "${arm32_url}" != "null" ]; then
        fetch_and_verify_apk "${arm32_url}"
    fi
    fetch_and_verify_apk "${release_url}"
    echo "✅ Successfully updated APKs from CircleCI in ${TARGET_DIR}:"
    ls -lh "${TARGET_DIR}"
    sha256sum "${TARGET_DIR}"/*
    upload_to_tmpfiles
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
    tmpfiles)
        upload_to_tmpfiles
        ;;
    status)
        echo "Existing Artifacts on Oracle VM:"
        ls -lh "${TARGET_DIR}" 2>/dev/null || echo "No local artifacts stored yet."
        ;;
    *)
        echo "Usage: $0 {fetch <URL> [SHA256]|circleci_latest [OWNER/REPO]|gh_latest [OWNER/REPO]|tmpfiles|status}"
        exit 1
        ;;
esac
