#!/usr/bin/env bash
# ==============================================================================
# Script: check_circleci_status.sh
# Purpose: Check CircleCI remote pipeline execution status via GitHub API & CircleCI API
# ==============================================================================

set -euo pipefail

REPO="irakmahmud/unexpected-keyboard"
LATEST_COMMIT=$(cd /home/opc/attendance_system && git rev-parse HEAD)

echo "===================================================================="
echo "🔍 Checking CircleCI Pipeline Status for ${REPO}"
echo "Latest Commit SHA: ${LATEST_COMMIT}"
echo "===================================================================="

echo ""
echo "1. GitHub Commit Status (Check-runs & Status API):"
CHECK_RUNS=$(curl -s "https://api.github.com/repos/${REPO}/commits/${LATEST_COMMIT}/check-runs")
TOTAL_CHECKS=$(echo "${CHECK_RUNS}" | grep -o '"total_count": [0-9]*' | head -n1 || echo "0")
echo "GitHub Check-runs: ${TOTAL_CHECKS}"

if echo "${CHECK_RUNS}" | grep -q '"name":'; then
    echo "Active / Completed Checks:"
    echo "${CHECK_RUNS}" | grep -E '"name"|"status"|"conclusion"' | head -n 12
else
    echo "No check-runs reported on GitHub commit yet."
fi

echo ""
echo "2. CircleCI Public API Probe (v1.1 Live Builds on attendance-system branch):"
LATEST_BUILDS=$(curl -s "https://circleci.com/api/v1.1/project/github/${REPO}/tree/attendance-system?limit=5")
if echo "${LATEST_BUILDS}" | jq . >/dev/null 2>&1; then
    echo "${LATEST_BUILDS}" | jq '.[0:5][] | {build_num, status, stop_time, subject, build_url}'
else
    echo "Unable to parse CircleCI build response."
fi

echo ""
echo "===================================================================="

