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
echo "1. GitHub Commit Status (Check-runs):"
HTTP_CODE=$(curl -s -o /tmp/gh_check_runs.json -w "%{http_code}" "https://api.github.com/repos/${REPO}/commits/${LATEST_COMMIT}/check-runs" || echo "000")
if [ "${HTTP_CODE}" = "200" ]; then
    TOTAL_CHECKS=$(grep -o '"total_count": [0-9]*' /tmp/gh_check_runs.json | head -n1 || echo "0")
    echo "GitHub Check-runs: ${TOTAL_CHECKS}"
else
    echo "GitHub API returned HTTP ${HTTP_CODE} for ${REPO}."
fi

echo ""
echo "2. CircleCI v2 Public Pipelines:"
CIRCLE_STATUS=$(curl -s "https://circleci.com/api/v2/project/gh/${REPO}/pipeline" || echo "{}")
echo "${CIRCLE_STATUS}" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    if 'message' in data:
        print('CircleCI API Message:', data['message'])
    items = data.get('items', [])
    for it in items[:5]:
        print(f\"Pipeline #{it.get('number')} ({it.get('id')}): State={it.get('state')}, Created={it.get('created_at')}\")
except Exception as e:
    print('Failed to parse CircleCI status:', e)
"
echo "===================================================================="
