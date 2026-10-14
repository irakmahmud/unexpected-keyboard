#!/usr/bin/env python3
"""
auto_deploy_daemon.py — Autonomous Continuous Deployment Daemon for Oracle VM
Role: Polls CircleCI for successful remote Rust builds, downloads artifacts,
      and triggers zero-downtime atomic deployment without ANY local compilation.
"""

import os
import sys
import time
import json
import signal
import logging
import urllib.request
import urllib.error
import subprocess
from pathlib import Path

# Paths
BASE_DIR = Path("/home/opc/attendance_system")
DATA_DIR = BASE_DIR / "data"
DEPLOY_DIR = BASE_DIR / "deploy"
STAGING_DIR = DEPLOY_DIR / "staging"
STATE_FILE = DATA_DIR / "last_deployed_build.txt"
DEPLOY_SCRIPT = DEPLOY_DIR / "deploy_remote_binary.sh"
LOG_FILE = BASE_DIR / "auto_deploy.log"

GITHUB_REPO = "IroScript/Oracle-VM-Projects"
POLL_INTERVAL_SECONDS = 20

# Setup Logging
logging.basicConfig(
    level=logging.INFO,
    format="[%(asctime)s] [%(levelname)s] [AUTO-DEPLOY] %(message)s",
    handlers=[
        logging.FileHandler(LOG_FILE),
        logging.StreamHandler(sys.stdout)
    ]
)

running = True

def handle_sigterm(signum, frame):
    global running
    logging.info("Received termination signal. Gracefully exiting auto_deploy daemon...")
    running = False

signal.signal(signal.SIGTERM, handle_sigterm)
signal.signal(signal.SIGINT, handle_sigterm)

def get_last_deployed_build():
    if STATE_FILE.exists():
        try:
            return int(STATE_FILE.read_text().strip())
        except Exception:
            return 0
    return 0

def save_last_deployed_build(build_num):
    DATA_DIR.mkdir(parents=True, exist_ok=True)
    STATE_FILE.write_text(str(build_num))

def fetch_json(url):
    req = urllib.request.Request(
        url,
        headers={"User-Agent": "OracleVM-AutoDeployDaemon/1.0", "Accept": "application/json"}
    )
    with urllib.request.urlopen(req, timeout=15) as resp:
        return json.loads(resp.read().decode("utf-8"))

def download_file(url, target_path):
    req = urllib.request.Request(url, headers={"User-Agent": "OracleVM-AutoDeployDaemon/1.0"})
    with urllib.request.urlopen(req, timeout=30) as resp, open(target_path, "wb") as out_file:
        out_file.write(resp.read())

def process_build(build_num):
    logging.info(f"Checking artifacts for successful CircleCI build #{build_num}...")
    artifacts_url = f"https://circleci.com/api/v1.1/project/github/{GITHUB_REPO}/{build_num}/artifacts"
    try:
        artifacts = fetch_json(artifacts_url)
    except Exception as e:
        logging.warning(f"Failed to fetch artifacts for build #{build_num}: {e}")
        return False

    tarball_url = None
    checksum_url = None
    api_bin_url = None

    for art in artifacts:
        path = art.get("path", "")
        url = art.get("url", "")
        if path.endswith("attendance-system-linux-x86_64.tar.gz"):
            tarball_url = url
        elif path.endswith("SHA256SUMS.txt"):
            checksum_url = url
        elif path.endswith("attendance-api"):
            api_bin_url = url

    if not (tarball_url or api_bin_url):
        # Build was not an attendance_system build (likely skipped or keyboard job)
        logging.debug(f"Build #{build_num} does not contain attendance-api artifacts. Skipping.")
        return False

    logging.info(f"🎯 New attendance release artifacts detected in CircleCI build #{build_num}!")
    STAGING_DIR.mkdir(parents=True, exist_ok=True)

    staged_binary = STAGING_DIR / "attendance-api"
    staged_checksum = STAGING_DIR / "SHA256SUMS.txt"

    try:
        if checksum_url:
            logging.info(f"Downloading SHA256 checksums from {checksum_url}...")
            download_file(checksum_url, staged_checksum)

        if tarball_url:
            staged_tar = STAGING_DIR / "release.tar.gz"
            logging.info(f"Downloading release archive from {tarball_url}...")
            download_file(tarball_url, staged_tar)
            subprocess.run(["tar", "-xzf", str(staged_tar), "-C", str(STAGING_DIR)], check=True)
            staged_tar.unlink(missing_ok=True)
        elif api_bin_url:
            logging.info(f"Downloading standalone attendance-api binary from {api_bin_url}...")
            download_file(api_bin_url, staged_binary)

        if not staged_binary.exists():
            logging.error("Failed to locate staged attendance-api binary after download.")
            return False

        os.chmod(staged_binary, 0o755)

        # Execute safe atomic deploy
        logging.info("Executing deploy_remote_binary.sh...")
        cmd = [str(DEPLOY_SCRIPT), str(staged_binary)]
        if staged_checksum.exists():
            cmd.append(str(staged_checksum))

        deploy_proc = subprocess.run(cmd, capture_output=True, text=True)
        if deploy_proc.returncode == 0:
            logging.info(f"🎉 SUCCESS! CircleCI build #{build_num} deployed and verified live on Oracle VM!")
            save_last_deployed_build(build_num)
            return True
        else:
            logging.error(f"Deployment script failed for build #{build_num}:\n{deploy_proc.stderr}")
            return False

    except Exception as e:
        logging.error(f"Exception during artifact download and deploy for build #{build_num}: {e}")
        return False

def check_for_updates():
    last_deployed = get_last_deployed_build()
    api_url = f"https://circleci.com/api/v1.1/project/github/{GITHUB_REPO}?limit=10"
    try:
        builds = fetch_json(api_url)
    except Exception as e:
        logging.warning(f"Error querying CircleCI API: {e}")
        return

    # Find successful finished builds
    successful_builds = [
        b for b in builds
        if b.get("status") == "success" and b.get("lifecycle") == "finished"
    ]

    for build in reversed(successful_builds):
        build_num = build.get("build_num", 0)
        if build_num > last_deployed:
            if process_build(build_num):
                last_deployed = build_num

def main():
    logging.info("====================================================================")
    logging.info("🚀 Oracle VM Continuous Deployment Daemon Started")
    logging.info(f"Repository: {GITHUB_REPO} | Poll Interval: {POLL_INTERVAL_SECONDS}s")
    logging.info(f"Initial Last Deployed Build: {get_last_deployed_build()}")
    logging.info("Zero local build policy strictly enforced. Only artifacts deployed.")
    logging.info("====================================================================")

    while running:
        try:
            check_for_updates()
        except Exception as e:
            logging.error(f"Unexpected error in polling loop: {e}")
        time.sleep(POLL_INTERVAL_SECONDS)

    logging.info("Auto-deploy daemon terminated cleanly.")

if __name__ == "__main__":
    main()
