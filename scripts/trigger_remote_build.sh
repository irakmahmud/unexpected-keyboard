#!/usr/bin/env bash
# ==============================================================================
# REMOTE BUILD DISPATCHER: ORACLE VM -> CIRCLECI & GITHUB ACTIONS
# Triggers cloud compilation while guaranteeing zero local build load on VM
# ==============================================================================

set -euo pipefail

BRANCH=$(git rev-parse --abbrev-ref HEAD)
COMMIT_HASH=$(git rev-parse --short HEAD)

echo "=== REMOTE CLOUD BUILD DISPATCHER ==="
echo "Target Branch: ${BRANCH}"
echo "Current Commit: ${COMMIT_HASH}"
echo "Zero Local Build Policy: ACTIVE"
echo

if ! git remote get-url origin >/dev/null 2>&1; then
  echo "[WARNING] No git remote 'origin' configured yet."
  echo "To link your GitHub/CircleCI repository, run:"
  echo "  git remote add origin https://github.com/<user>/<repo>.git"
  echo "  git push -u origin ${BRANCH}"
  exit 0
fi

echo "[INFO] Pushing changes to remote repository to trigger CircleCI & GitHub Actions..."
git push origin "${BRANCH}"
echo "[SUCCESS] Remote build triggered in CircleCI / GitHub Actions cloud."
