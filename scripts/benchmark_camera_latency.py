#!/usr/bin/env python3
"""
Benchmarks camera & ONNX face verification latency and startup speed on the pure Native Rust attendance-api.
Compares Before-Optimization vs After-Optimization across low-res (mobile/downscaled) and high-res (desktop/upload) frames.
"""
import base64
import json
import os
import subprocess
import time
import urllib.request

API_BASE = "http://127.0.0.1:8080"

BEFORE_LATENCIES = {
    "40.jpg (194x260) [Cold 1st Frame]": 1186.0,
    "40.jpg (194x260) [Warm Frame]": 123.2,
    "osman hadi/speech 2.jpg (750x450)": 428.5,
    "Anjelina Jolie/1.jpg (860x670)": 942.3,
    "Anjelina Jolie/100.jpeg (1075x1600)": 2043.7,
    "Einstein/Albert_Einstein_Head_cleaned.jpg (1280x1584)": 8012.5,
}

TEST_CASES = [
    ("40.jpg (194x260) [Warm Frame]", "/home/opc/attendance_system/40.jpg"),
    ("osman hadi/speech 2.jpg (750x450)", "/home/opc/attendance_system/faces_db/osman hadi/speech 2.jpg"),
    ("Anjelina Jolie/1.jpg (860x670)", "/home/opc/attendance_system/faces_db/Anjelina Jolie/1.jpg"),
    ("Anjelina Jolie/100.jpeg (1075x1600)", "/home/opc/attendance_system/faces_db/Anjelina Jolie/100.jpeg"),
    ("Einstein/Albert_Einstein_Head_cleaned.jpg (1280x1584)", "/home/opc/attendance_system/faces_db/Einstein/Albert_Einstein_Head_cleaned.jpg"),
]


def post_json(path: str, payload: dict) -> tuple[float, dict]:
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        f"{API_BASE}{path}",
        data=data,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    t0 = time.perf_counter()
    with urllib.request.urlopen(req, timeout=30) as resp:
        body = json.loads(resp.read().decode("utf-8"))
    dt_ms = (time.perf_counter() - t0) * 1000.0
    return dt_ms, body


def main() -> None:
    print("=" * 88)
    print("🚀 PURE NATIVE RUST ATTENDANCE-API: CAMERA & ONNX LATENCY BENCHMARK")
    print("=" * 88)

    # 1. Check health and engine type
    with urllib.request.urlopen(f"{API_BASE}/api/v1/health", timeout=5) as resp:
        health = json.loads(resp.read().decode("utf-8"))
    print(f"Health Check: status={health.get('status')}, service={health.get('service')}, engine={health.get('engine')}")

    # 2. Check running binary & process tree (prove zero Python/TensorFlow in server)
    ps_out = subprocess.check_output(
        ["ps", "-C", "attendance-api", "-o", "pid,comm,rss,args"], text=True
    ).strip()
    print("\nActive Native Rust Server Process:")
    print(ps_out)

    # 3. Benchmark verify-identity across all test images
    print("\n" + "-" * 88)
    print(f"{'Image / Resolution':<52} | {'Before (ms)':>11} | {'After (ms)':>10} | {'Speedup':>8}")
    print("-" * 88)

    for idx, (label, path) in enumerate(TEST_CASES):
        with open(path, "rb") as f:
            b64 = "data:image/jpeg;base64," + base64.b64encode(f.read()).decode("ascii")

        if idx == 0:
            # First measure first frame
            dt_first, res_first = post_json("/api/v1/face/verify-identity", {"image_base64": b64})
            before_cold = BEFORE_LATENCIES["40.jpg (194x260) [Cold 1st Frame]"]
            speedup_cold = before_cold / max(dt_first, 0.1)
            print(f"{'40.jpg (194x260) [Cold 1st Frame]':<52} | {before_cold:>9.1f}ms | {dt_first:>8.1f}ms | {speedup_cold:>7.1f}x")

        # Measure warm frame (median of 3 runs)
        runs = []
        last_res = {}
        for _ in range(3):
            dt, last_res = post_json("/api/v1/face/verify-identity", {"image_base64": b64})
            runs.append(dt)
        runs.sort()
        median_ms = runs[len(runs) // 2]
        before_ms = BEFORE_LATENCIES.get(label, 0.0)
        speedup = before_ms / max(median_ms, 0.1)
        matched = last_res.get("user_id") or ("detected" if last_res.get("detected") else "none")
        print(f"{label:<52} | {before_ms:>9.1f}ms | {median_ms:>8.1f}ms | {speedup:>7.1f}x  (match={matched})")

    print("-" * 88)


if __name__ == "__main__":
    main()
