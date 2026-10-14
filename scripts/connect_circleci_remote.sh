#!/usr/bin/env bash
# ==============================================================================
# Script: connect_circleci_remote.sh
# Purpose: Reconnect /home/opc/keyboard to a new GitHub/GitLab repository for CircleCI
# Usage: ./scripts/connect_circleci_remote.sh <NEW_REPO_URL> [BRANCH]
# ==============================================================================

set -euo pipefail

WORKSPACE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${WORKSPACE_DIR}"

NEW_REPO_URL="${1:-}"
BRANCH="${2:-main}"

if [ -z "${NEW_REPO_URL}" ]; then
    echo "===================================================================="
    echo "🔗 Keyboard CircleCI Remote Reconnection Utility"
    echo "===================================================================="
    echo "Usage:"
    echo "  ./scripts/connect_circleci_remote.sh https://<TOKEN>@github.com/<NEW_USER>/<NEW_REPO>.git"
    echo ""
    echo "Current Git Remote Status in ${WORKSPACE_DIR}:"
    git remote -v || echo "No remote configured."
    echo "===================================================================="
    exit 0
fi

echo "===================================================================="
echo "🔗 Connecting /home/opc/keyboard to Remote Repository"
echo "Target Branch: ${BRANCH}"
echo "===================================================================="

if git remote get-url origin >/dev/null 2>&1; then
    echo "🔄 Updating existing 'origin' remote..."
    git remote set-url origin "${NEW_REPO_URL}"
else
    echo "➕ Adding new 'origin' remote..."
    git remote add origin "${NEW_REPO_URL}"
fi

echo "🚀 Pushing keyboard codebase and .circleci/config.yml to '${BRANCH}'..."
git push -u origin "${BRANCH}"

echo "✅ Successfully pushed! Next step: Enable project in CircleCI dashboard (https://app.circleci.com/projects)."
echo "===================================================================="
