#!/usr/bin/env bash
# ==============================================================================
# GUARD SCRIPT: STRICT ZERO LOCAL FLUTTER BUILD ENFORCEMENT
# Host Environment: Oracle Cloud VM (Development & Version Control Only)
# Remote Infrastructure: CircleCI Cloud Runners + GitHub Actions
# ==============================================================================

set -euo pipefail

RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

COMMAND_ATTEMPTED="${*:-unknown}"

echo -e "${RED}╔══════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${RED}║      ⛔ MANDATORY ZERO LOCAL FLUTTER BUILD POLICY VIOLATION BLOCKED     ║${NC}"
echo -e "${RED}╚══════════════════════════════════════════════════════════════════════════╝${NC}"
echo
echo -e "${YELLOW}Attempted Command:${NC} ${COMMAND_ATTEMPTED}"
echo -e "${YELLOW}Target Node:${NC} Oracle Cloud VM ($(hostname) - $(uname -m))"
echo
echo -e "${RED}[POLICY ENFORCEMENT]${NC} Local Flutter / Gradle compilation is strictly PROHIBITED on this Oracle VM."
echo -e "Reason: VM resources are dedicated solely to logic authoring, git synchronization, and telemetry."
echo
echo -e "${BLUE}────────────────── REMOTE BUILD ARCHITECTURE OVERVIEW ──────────────────${NC}"
echo -e "1. ${GREEN}CircleCI Remote Build Pipeline:${NC}"
echo -e "   - Config: .circleci/config.yml"
echo -e "   - Cloud Runner: ghcr.io/cirruslabs/flutter:stable (Docker Executor)"
echo -e "   - Automated Workflows: validate-and-test -> build-remote-apks"
echo -e "   - Artifacts: unexpected-keyboard-debug.apk, unexpected-keyboard-release.apk"
echo
echo -e "2. ${GREEN}GitHub Actions Remote Pipeline:${NC}"
echo -e "   - Config: .github/workflows/build_apk.yml"
echo -e "   - Cloud Runner: ubuntu-latest (Temurin Java 17 + Flutter SDK)"
echo
echo -e "${BLUE}────────────────── HOW TO INITIATE A REMOTE BUILD ──────────────────${NC}"
echo -e "Simply push your commits to git:"
echo -e "   ${GREEN}git add .${NC}"
echo -e "   ${GREEN}git commit -m \"feat: update keyboard logic\"${NC}"
echo -e "   ${GREEN}git push origin main${NC}"
echo
echo -e "CircleCI and GitHub Actions will automatically compile the APKs in the cloud."
echo -e "────────────────────────────────────────────────────────────────────────"
exit 1
