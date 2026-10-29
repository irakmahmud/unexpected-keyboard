#!/usr/bin/env bash
# ==============================================================================
# Script: restore_original_design.sh
# Purpose: Instant 1-Second Rollback to Original Classic V8 Design (or back to 3D Futuristic)
# Usage:
#   ./scripts/restore_original_design.sh          # Restores Original Classic V8 UI
#   ./scripts/restore_original_design.sh 3d       # Switches to 3D Futuristic Cyber-HUD UI
#   ./scripts/restore_original_design.sh hard     # Restores original Rust files & binary backup
# ==============================================================================

set -euo pipefail

WORKSPACE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BACKUP_DIR="${WORKSPACE_DIR}/deploy/backups/ui_v8_classic_original"
MODE="${1:-classic}"

mkdir -p "${WORKSPACE_DIR}/data"

if [ "${MODE}" = "3d" ] || [ "${MODE}" = "futuristic" ]; then
    echo "futuristic_3d" > "${WORKSPACE_DIR}/data/ui_design_mode.txt"
    rm -f "${WORKSPACE_DIR}/data/ui_override.html"
    echo "✅ 3D Futuristic Biometric Cyber-HUD Design is now ACTIVE on http://127.0.0.1:8080/"
elif [ "${MODE}" = "hard" ]; then
    echo "⏪ Performing full source & binary rollback to Original Classic V8..."
    echo "classic" > "${WORKSPACE_DIR}/data/ui_design_mode.txt"
    rm -f "${WORKSPACE_DIR}/data/ui_override.html"
    if [ -d "${BACKUP_DIR}" ]; then
        cp -p "${BACKUP_DIR}/styles.rs" "${WORKSPACE_DIR}/crates/attendance-api/src/ui/styles.rs"
        cp -p "${BACKUP_DIR}/kiosk_view.rs" "${WORKSPACE_DIR}/crates/attendance-api/src/ui/kiosk_view.rs"
        cp -p "${BACKUP_DIR}/scripts.rs" "${WORKSPACE_DIR}/crates/attendance-api/src/ui/scripts.rs"
        if [ -f "${BACKUP_DIR}/attendance-api.v8_classic" ]; then
            "${WORKSPACE_DIR}/deploy/deploy_remote_binary.sh" "${BACKUP_DIR}/attendance-api.v8_classic"
        fi
    fi
    echo "✅ Full hard rollback to Original Classic V8 completed!"
else
    echo "classic" > "${WORKSPACE_DIR}/data/ui_design_mode.txt"
    echo "✅ Server default UI switched to Original Classic V8 Design (0ms instant switch)!"
fi
