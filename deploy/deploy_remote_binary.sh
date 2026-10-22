#!/usr/bin/env bash
# ==============================================================================
# Script: deploy_remote_binary.sh
# Architecture: ZERO LOCAL RUST BUILD — ORACLE VM + CIRCLECI REMOTE COMPILATION
# Role: Atomic Deployment & Rollback Engine for Pre-compiled Rust Binaries
# ==============================================================================

set -euo pipefail

WORKSPACE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEPLOY_DIR="${WORKSPACE_DIR}/deploy"
STAGING_DIR="${DEPLOY_DIR}/staging"
BACKUP_DIR="${DEPLOY_DIR}/backups"
TARGET_DIR="${WORKSPACE_DIR}/target/release"
MAX_BACKUPS=3

mkdir -p "${STAGING_DIR}" "${BACKUP_DIR}" "${TARGET_DIR}"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [DEPLOY] $1"
}

err() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [DEPLOY-ERROR] $1" >&2
}

# ------------------------------------------------------------------------------
# 1. Pre-flight Disk Space & Architecture Checks (Storage Protection)
# ------------------------------------------------------------------------------
check_storage_and_system() {
    log "Checking available storage on Oracle VM..."
    local avail_kb
    avail_kb=$(df -k "${WORKSPACE_DIR}" | awk 'NR==2 {print $4}')
    local min_kb=512000 # 500 MB minimum
    if [ "${avail_kb}" -lt "${min_kb}" ]; then
        err "CRITICAL: Insufficient disk space on Oracle VM (${avail_kb} KB available, required >= ${min_kb} KB). Aborting deployment!"
        exit 1
    fi
    log "Storage check PASSED: $((avail_kb / 1024)) MB available on Oracle root volume."

    # Verify Oracle Host Architecture
    local host_arch
    host_arch=$(uname -m)
    log "Oracle host architecture detected: ${host_arch}"
    if [ "${host_arch}" != "x86_64" ]; then
        err "CRITICAL: Expected host architecture x86_64, but found ${host_arch}."
        exit 1
    fi
}

# ------------------------------------------------------------------------------
# 2. Artifact Integrity & Compatibility Verification
# ------------------------------------------------------------------------------
verify_artifact() {
    local binary_file="$1"
    local checksum_file="${2:-}"

    if [ ! -f "${binary_file}" ]; then
        err "Binary file not found: ${binary_file}"
        exit 1
    fi

    # SHA256 Checksum Validation
    if [ -n "${checksum_file}" ] && [ -f "${checksum_file}" ]; then
        log "Verifying SHA-256 checksum against ${checksum_file}..."
        local base_name
        base_name=$(basename "${binary_file}")
        local expected_sum
        expected_sum=$(grep "${base_name}" "${checksum_file}" | awk '{print $1}')
        if [ -n "${expected_sum}" ]; then
            local actual_sum
            actual_sum=$(sha256sum "${binary_file}" | awk '{print $1}')
            if [ "${expected_sum}" != "${actual_sum}" ]; then
                err "SHA-256 MISMATCH for ${base_name}!"
                err "Expected: ${expected_sum}"
                err "Actual:   ${actual_sum}"
                exit 1
            fi
            log "SHA-256 verification SUCCESS: ${actual_sum}"
        else
            log "Notice: Checksum for ${base_name} not found in ${checksum_file}. Skipping strict hash match."
        fi
    fi

    # Architecture & ELF Verification
    log "Verifying binary ELF format and architecture..."
    local file_type
    file_type=$(file -b "${binary_file}")
    if [[ "${file_type}" != *"ELF 64-bit"* ]] || [[ "${file_type}" != *"x86-64"* ]]; then
        err "Incompatible binary format: ${file_type}. Expected ELF 64-bit x86-64 executable."
        exit 1
    fi
    log "ELF format confirmed: ${file_type}"

    # Runtime Shared Library Link Check
    log "Validating dynamic runtime library linkages (ldd)..."
    if ldd "${binary_file}" | grep -i "not found"; then
        err "Missing dynamic library dependencies detected in ${binary_file}!"
        exit 1
    fi
    log "Runtime library dependencies verified."
}

# ------------------------------------------------------------------------------
# 3. Rotate Old Backups (Storage Protection - Max 3 Backups)
# ------------------------------------------------------------------------------
rotate_backups() {
    log "Maintaining max ${MAX_BACKUPS} historical backups in ${BACKUP_DIR}..."
    local backup_count
    backup_count=$(ls -1 "${BACKUP_DIR}"/attendance-api.* 2>/dev/null | wc -l || true)
    if [ "${backup_count}" -gt "${MAX_BACKUPS}" ]; then
        ls -1t "${BACKUP_DIR}"/attendance-api.* | tail -n +"$((MAX_BACKUPS + 1))" | while read -r old_backup; do
            log "Removing aged backup: ${old_backup}"
            rm -f "${old_backup}"
        done
    fi
}

# ------------------------------------------------------------------------------
# 4. Atomic Replacement and Service Management
# ------------------------------------------------------------------------------
atomic_deploy() {
    local staged_binary="$1"
    local target_binary="${TARGET_DIR}/attendance-api"
    local timestamp
    timestamp=$(date '+%Y%m%d_%H%M%S')
    local backup_target="${BACKUP_DIR}/attendance-api.${timestamp}"

    # Backup current active binary if present
    if [ -f "${target_binary}" ]; then
        log "Creating safe backup of current binary: ${backup_target}"
        cp "${target_binary}" "${backup_target}"
        rotate_backups
    fi

    # Atomic deployment: copy to temporary file on same filesystem and mv
    log "Executing atomic binary replacement into ${target_binary}..."
    cp "${staged_binary}" "${target_binary}.tmp"
    chmod +x "${target_binary}.tmp"
    mv -f "${target_binary}.tmp" "${target_binary}"
    log "Atomic file swap completed."

    # Service Restart
    restart_service "${backup_target}"
}

# ------------------------------------------------------------------------------
# 5. Service Restart & Health Verification with Automatic Rollback
# ------------------------------------------------------------------------------
restart_service() {
    local backup_file="${1:-}"
    log "Restarting Axum production attendance service..."

    local old_pid
    old_pid=$(pgrep -f "target/release/attendance-api" | head -n1 || /usr/sbin/ss -tulpn 2>/dev/null | grep ":8080" | grep -o "pid=[0-9]*" | cut -d'=' -f2 | head -n1 || true)
    if [ -n "${old_pid}" ]; then
        log "Gracefully stopping previous process (PID: ${old_pid})..."
        kill -15 "${old_pid}" 2>/dev/null || kill -9 "${old_pid}" 2>/dev/null || true
        sleep 1
    fi

    log "Launching updated attendance-api runtime..."
    nohup "${TARGET_DIR}/attendance-api" </dev/null > "${WORKSPACE_DIR}/server.log" 2>&1 &
    local new_pid=$!
    disown "${new_pid}" 2>/dev/null || true
    log "Attendance API started with PID: ${new_pid}"

    # Health Probe
    log "Conducting post-deployment health check against http://127.0.0.1:8080/api/v1/health..."
    local attempts=10
    local healthy=false

    for i in $(seq 1 ${attempts}); do
        sleep 1
        if curl -s -f http://127.0.0.1:8080/api/v1/health > /dev/null 2>&1; then
            healthy=true
            break
        fi
        log "Health check probe ${i}/${attempts} pending..."
    done

    if [ "${healthy}" = "true" ]; then
        log "🎉 DEPLOYMENT SUCCESS: Axum attendance service is HEALTHY on port 8080!"
        curl -s http://127.0.0.1:8080/api/v1/health | jq . 2>/dev/null || curl -s http://127.0.0.1:8080/api/v1/health
    else
        err "CRITICAL: Health check FAILED after ${attempts} attempts! Initiating automatic rollback..."
        perform_rollback "${backup_file}"
    fi
}

perform_rollback() {
    local backup_file="${1:-}"
    if [ -n "${backup_file}" ] && [ -f "${backup_file}" ]; then
        log "🚨 Rolling back to previous working binary: ${backup_file}..."
        cp "${backup_file}" "${TARGET_DIR}/attendance-api"
        chmod +x "${TARGET_DIR}/attendance-api"

        local current_pid
        current_pid=$(ss -tulpn 2>/dev/null | grep ":8080" | grep -o "pid=[0-9]*" | cut -d'=' -f2 | head -n1 || true)
        if [ -n "${current_pid}" ]; then
            kill -9 "${current_pid}" 2>/dev/null || true
        fi

        nohup "${TARGET_DIR}/attendance-api" > "${WORKSPACE_DIR}/server.log" 2>&1 &
        sleep 2

        if curl -s -f http://127.0.0.1:8080/api/v1/health > /dev/null 2>&1; then
            log "✅ Rollback verified: Previous stable binary restored and operational."
        else
            err "FATAL: Rollback binary failed to start. Inspect server.log!"
        fi
        exit 1
    else
        err "No rollback binary available! Deployment failed."
        exit 1
    fi
}

# ------------------------------------------------------------------------------
# Entrypoint
# ------------------------------------------------------------------------------
main() {
    local source_binary="${1:-${TARGET_DIR}/attendance-api}"
    local checksum_file="${2:-}"

    echo "===================================================================="
    echo "  ORACLE VM ZERO-LOCAL-BUILD PRODUCTION DEPLOYMENT ENGINE"
    echo "===================================================================="
    check_storage_and_system
    verify_artifact "${source_binary}" "${checksum_file}"
    atomic_deploy "${source_binary}"
}

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
