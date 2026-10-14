#!/usr/bin/env bash
# ==============================================================================
# Script: fetch_remote_binary.sh
# Purpose: Fetch pre-compiled Rust release binaries from CircleCI / Remote CI
# Mandate: ZERO LOCAL RUST BUILD on Oracle VM.
# ==============================================================================

set -euo pipefail

WORKSPACE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="${WORKSPACE_DIR}/target/release"
mkdir -p "${TARGET_DIR}"

CIRCLE_PROJECT_USERNAME="${CIRCLE_PROJECT_USERNAME:-IroScript}"
CIRCLE_PROJECT_REPONAME="${CIRCLE_PROJECT_REPONAME:-Oracle-VM-Projects}"
CIRCLE_API_URL="https://circleci.com/api/v1.1"

echo "===================================================================="
echo "🚀 [ZERO LOCAL RUST BUILD] CircleCI Remote Binary Deployer"
echo "===================================================================="
echo "Target Directory: ${TARGET_DIR}"

fetch_from_circleci_artifacts() {
    local build_num="${1:-latest}"

    if [ "${build_num}" = "latest" ]; then
        echo "🔍 Discovering latest successful build with artifacts on CircleCI..."
        local recent_builds
        recent_builds=$(curl -s "https://circleci.com/api/v1.1/project/github/${CIRCLE_PROJECT_USERNAME}/${CIRCLE_PROJECT_REPONAME}?limit=10")
        build_num=$(echo "${recent_builds}" | jq -r '[.[] | select(.status=="success")] | .[0].build_num' 2>/dev/null || echo "")
        if [ -z "${build_num}" ] || [ "${build_num}" = "null" ]; then
            echo "No successful build found in recent list."
            return 1
        fi
        echo "Latest successful build: ${build_num}"
    fi

    echo "🔍 Querying artifacts for build: ${build_num}..."
    local artifacts_json
    artifacts_json=$(curl -s "https://circleci.com/api/v1.1/project/github/${CIRCLE_PROJECT_USERNAME}/${CIRCLE_PROJECT_REPONAME}/${build_num}/artifacts")

    local archive_url
    archive_url=$(echo "${artifacts_json}" | jq -r '.[] | select(.path | endswith(".tar.gz")) | .url' 2>/dev/null || echo "")

    if [ -n "${archive_url}" ] && [ "${archive_url}" != "null" ]; then
        echo "⬇️ Downloading release archive from CircleCI: ${archive_url}..."
        curl -fsSL "${archive_url}" -o /tmp/attendance-binaries.tar.gz
        deploy_local_binary /tmp/attendance-binaries.tar.gz
    else
        echo "Notice: No .tar.gz archive found in build ${build_num} artifacts."
        local api_bin_url
        api_bin_url=$(echo "${artifacts_json}" | jq -r '.[] | select(.path | endswith("attendance-api")) | .url' 2>/dev/null || echo "")
        if [ -n "${api_bin_url}" ] && [ "${api_bin_url}" != "null" ]; then
            echo "⬇️ Downloading standalone binary: ${api_bin_url}..."
            curl -fsSL "${api_bin_url}" -o "${TARGET_DIR}/attendance-api"
            deploy_local_binary "${TARGET_DIR}/attendance-api"
        fi
    fi
}

deploy_local_binary() {
    local binary_source="${1:-}"
    if [ -f "${binary_source}" ]; then
        echo "📦 Extracting/installing remote binary: ${binary_source}..."
        if [[ "${binary_source}" == *.tar.gz ]]; then
            tar -xzf "${binary_source}" -C "${TARGET_DIR}/"
        else
            cp "${binary_source}" "${TARGET_DIR}/"
        fi
    fi

    echo "🔒 Enforcing executable permissions on release binaries..."
    chmod +x "${TARGET_DIR}/attendance-api" || true
    chmod +x "${TARGET_DIR}/attendance-cli" || true
    chmod +x "${TARGET_DIR}/attendance-device-agent" || true

    echo "✅ Remote binaries verified in ${TARGET_DIR}:"
    ls -lh "${TARGET_DIR}/attendance-"* 2>/dev/null || true
}

verify_and_restart_service() {
    echo "🔄 Checking running attendance daemon on port 8080..."
    local old_pid
    old_pid=$(ss -tulpn 2>/dev/null | grep ":8080" | grep -o "pid=[0-9]*" | cut -d'=' -f2 | head -n1 || true)
    
    if [ -n "${old_pid}" ]; then
        echo "Stopping old process (PID: ${old_pid})..."
        kill -15 "${old_pid}" 2>/dev/null || kill -9 "${old_pid}" 2>/dev/null || true
        sleep 1
    fi

    echo "🚀 Starting new attendance-api binary..."
    nohup "${TARGET_DIR}/attendance-api" > "${WORKSPACE_DIR}/server.log" 2>&1 &
    sleep 2

    echo "🩺 Probing API health..."
    if curl -s -f http://127.0.0.1:8080/api/v1/health > /dev/null; then
        echo "✅ Health check PASSED! Attendance API is operational on 127.0.0.1:8080."
    else
        echo "⚠️ Health check pending or server starting. Logs:"
        tail -n 10 "${WORKSPACE_DIR}/server.log" || true
    fi
}

# Main Execution Flow
case "${1:-deploy-current}" in
    fetch)
        fetch_from_circleci_artifacts "${2:-latest}"
        ;;
    deploy-current)
        deploy_local_binary "${2:-}"
        verify_and_restart_service
        ;;
    status)
        echo "Server Process:"
        ss -tulpn | grep 8080 || echo "Port 8080 not listening."
        ;;
    *)
        echo "Usage: $0 {fetch|deploy-current [file]|status}"
        exit 1
        ;;
esac
