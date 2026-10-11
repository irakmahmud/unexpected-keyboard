#!/usr/bin/env bash
# ==============================================================================
# Script: check_circleci_status.sh
# Purpose: Check CircleCI remote pipeline execution status for Flutter Keyboard
# ==============================================================================

set -euo pipefail

REPO="IroScript/Oracle-VM-Projects"
LATEST_COMMIT=$(cd /home/opc/Oracle-VM-Projects && git rev-parse HEAD)

echo "===================================================================="
echo "🔍 Checking CircleCI Pipeline Status for ${REPO}"
echo "Latest Monorepo Commit: ${LATEST_COMMIT}"
echo "===================================================================="

echo ""
echo "1. GitHub Commit Status (Check-runs):"
CHECK_RUNS=$(curl -s "https://api.github.com/repos/${REPO}/commits/${LATEST_COMMIT}/check-runs")
TOTAL_CHECKS=$(echo "${CHECK_RUNS}" | grep -o '"total_count": [0-9]*' | head -n1 || echo "0")
echo "GitHub Check-runs: ${TOTAL_CHECKS}"

if echo "${CHECK_RUNS}" | grep -q '"name":'; then
    echo "Active / Completed Checks:"
    echo "${CHECK_RUNS}" | grep -E '"name"|"status"|"conclusion"' | head -n 16
else
    echo "No check-runs reported on GitHub commit yet."
fi

echo ""
echo "2. CircleCI v2 Public Pipelines:"
CIRCLE_STATUS=$(curl -s "https://circleci.com/api/v2/project/gh/${REPO}/pipeline")
echo "${CIRCLE_STATUS}" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    items = data.get('items', [])
    for it in items[:3]:
        print(f\"Pipeline #{it.get('number')} ({it.get('id')}): State={it.get('state')}, Created={it.get('created_at')}\")
except Exception as e:
    print('Failed to parse CircleCI status:', e)
"

echo "===================================================================="
