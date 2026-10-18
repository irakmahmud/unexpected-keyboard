#!/usr/bin/env bash
# ==============================================================================
# Script: check_circleci_status.sh
# Purpose: Check CircleCI remote pipeline execution status for Flutter Keyboard
# Workspace: /home/opc/keyboard
# ==============================================================================

set -euo pipefail

WORKSPACE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${WORKSPACE_DIR}"

REMOTE_URL=$(git remote get-url origin 2>/dev/null || echo "")
if [ -n "${1:-}" ]; then
    REPO="${1}"
elif [ -n "${REMOTE_URL}" ]; then
    REPO=$(echo "${REMOTE_URL}" | sed -E 's#.*(github\.com|gitlab\.com)[:/]([^/]+/[^/.]+)(\.git)?$#\2#')
else
    REPO="UNCONFIGURED"
fi

LATEST_COMMIT=$(git rev-parse HEAD 2>/dev/null || echo "unknown")

echo "===================================================================="
echo "🔍 Checking CircleCI Pipeline Status for Keyboard Project"
echo "Workspace:     ${WORKSPACE_DIR}"
echo "Remote Repo:   ${REPO}"
echo "Latest Commit: ${LATEST_COMMIT}"
echo "===================================================================="

if [ "${REPO}" = "UNCONFIGURED" ]; then
    echo "⚠️  No active Git remote 'origin' configured in ${WORKSPACE_DIR}."
    echo "👉 Run: ./scripts/connect_circleci_remote.sh <NEW_GIT_REPO_URL>"
    exit 0
fi

echo ""
echo "1. CircleCI Commit Statuses on GitHub:"
curl -s "https://api.github.com/repos/${REPO}/commits/${LATEST_COMMIT}/status" | jq -r '.statuses[] | "  - [\(.state | ascii_upcase)] \(.context): \(.description) (\(.target_url))"'

echo ""
echo "2. CircleCI Recent Builds (API v1.1):"
curl -s "https://circleci.com/api/v1.1/project/github/${REPO}?limit=5" | jq -r '.[] | "  - Build #\(.build_num) [\(.workflows.job_name // "job")]: status=\(.status), outcome=\(.outcome // "running"), commit=\(.vcs_revision[0:7]) (\(.build_url))"'

echo ""
echo "3. GitHub Actions Check-runs:"
curl -s "https://api.github.com/repos/${REPO}/commits/${LATEST_COMMIT}/check-runs" | jq -r '.check_runs[] | "  - [\(.status)/\(.conclusion // "running")] \(.name) (\(.app.name))"'
echo "===================================================================="
