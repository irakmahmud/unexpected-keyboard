#!/usr/bin/env bash
# ==============================================================================
# Script: zero_local_build_guard.sh
# Purpose: Guard against unauthorized local Rust compilation on Oracle VM
# ==============================================================================

set -euo pipefail

echo "===================================================================="
echo "🛑 ZERO LOCAL RUST BUILD POLICY IS IN EFFECT ON ORACLE VM 🛑"
echo "===================================================================="
echo "Oracle VM hardware resources (RAM/CPU) are strictly reserved for"
echo "running runtime services, BigQuery sync, and attendance kiosks."
echo ""
echo "Heavy Rust compilation (cargo build, cargo build --release) must"
echo "EXCLUSIVELY be executed via CircleCI remote CI runners."
echo ""
echo "Workflow:"
echo " 1. Commit and push your code: git push origin master"
echo " 2. CircleCI builds artifacts remotely via .circleci/config.yml"
echo " 3. Deploy to Oracle VM via ./scripts/fetch_remote_binary.sh"
echo "===================================================================="
