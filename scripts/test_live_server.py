import urllib.request
import urllib.parse
import json
import time
import sys
import subprocess
import os

PORT = 8089
BASE_URL = f"http://127.0.0.1:{PORT}"

env = os.environ.copy()
env["PORT"] = str(PORT)
env["DATABASE_URL"] = "sqlite:///home/opc/attendance_system/data/test_attendance.db"

# Remove any old test db and clear SHAKIL from cooldown state for repeatable test runs
for suffix in ("", "-wal", "-shm"):
    p = f"/home/opc/attendance_system/data/test_attendance.db{suffix}"
    if os.path.exists(p):
        os.remove(p)
for cd_path in (os.path.expanduser("~/.last_attendance"), "/home/opc/attendance_system/data/.last_attendance"):
    if os.path.exists(cd_path):
        try:
            with open(cd_path, "r") as f:
                cd_data = json.load(f)
            if isinstance(cd_data, dict) and "users" in cd_data and "SHAKIL" in cd_data["users"]:
                del cd_data["users"]["SHAKIL"]
                with open(cd_path, "w") as f:
                    json.dump(cd_data, f)
        except Exception:
            pass

print(f"Starting Attendance System Rust Binary on port {PORT}...")
proc = subprocess.Popen(
    ["/home/opc/attendance_system/target/release/attendance-api"],
    env=env,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE
)

def req(path, method="GET", body=None):
    url = f"{BASE_URL}{path}"
    headers = {"Content-Type": "application/json"}
    data = json.dumps(body).encode("utf-8") if body else None
    r = urllib.request.Request(url, data=data, headers=headers, method=method)
    try:
        with urllib.request.urlopen(r, timeout=5) as response:
            status = response.status
            content = response.read().decode("utf-8")
            try:
                parsed = json.loads(content)
            except:
                parsed = content
            return status, parsed
    except urllib.error.HTTPError as e:
        content = e.read().decode("utf-8")
        try:
            parsed = json.loads(content)
        except:
            parsed = content
        return e.code, parsed

for _ in range(30):
    try:
        st, _ = req("/api/v1/health")
        if st == 200:
            break
    except Exception:
        time.sleep(0.3)

try:
    print("\n--- TEST 1: Health Check ---")
    status, data = req("/api/v1/health")
    print(f"Status: {status}, Response: {data}")
    assert status == 200
    assert data["status"] == "healthy"
    assert data["system"] == "Pure Rust Axum Attendance Core"

    print("\n--- TEST 2: List Seeded Users ---")
    status, users = req("/api/v1/users")
    print(f"Status: {status}, Total Users: {len(users)}")
    usernames = [u["username"] for u in users]
    print(f"Usernames: {usernames}")
    assert status == 200
    assert "RAHIM" in usernames

    print("\n--- TEST 3: Create New User (SHAKIL) ---")
    new_user_payload = {
        "username": "SHAKIL",
        "user_designation": "Rust Engineer",
        "company_name": "Antigravity HQ",
        "user_android_id": "ANDR_SHAKIL_777"
    }
    status, res = req("/api/v1/users", method="POST", body=new_user_payload)
    print(f"Status: {status}, Response: {res}")
    assert status == 200
    assert res["success"] is True
    assert res["username"] == "SHAKIL"

    print("\n--- TEST 4: Punch Attendance IN_TIME for SHAKIL ---")
    punch_payload = {
        "username": "SHAKIL",
        "method": "CAMERA"
    }
    status, punch_res = req("/api/v1/attendance/punch", method="POST", body=punch_payload)
    print(f"Status: {status}, Response: {punch_res}")
    assert status == 200
    assert punch_res["login_type"] == "IN_TIME"
    assert punch_res["success"] is True

    print("\n--- TEST 5: Verify Cooldown Prevents Duplicate Punch ---")
    status, cd_res = req("/api/v1/attendance/punch", method="POST", body=punch_payload)
    print(f"Status: {status} (Expected 409 Conflict), Response: {cd_res}")
    assert status == 409
    assert "Cooldown active" in cd_res["error"]

    print("\n--- TEST 6: Get Today's Live Attendance Feed ---")
    status, feed = req("/api/v1/attendance/today")
    print(f"Status: {status}, Today's punches: {len(feed)}")
    assert status == 200
    assert any(p["username"] == "SHAKIL" for p in feed)

    print("\n--- TEST 7: Dashboard Summary Metrics ---")
    status, summary = req("/api/v1/attendance/summary")
    print(f"Status: {status}, Summary: {summary}")
    assert status == 200
    assert summary["present_today"] >= 1
    assert summary["checked_in"] >= 1

    print("\n--- TEST 8: Face Vector Matching (512-D L2 Euclidean) ---")
    # Enroll a valid non-constant 512-D embedding for RAHIM, then match with slight noise
    base_vec = [0.0] * 512
    base_vec[0] = 0.99
    base_vec[1] = 0.14
    enr_status, enr_res = req("/api/v1/face/enroll", method="POST", body={"username": "RAHIM", "embedding": base_vec})
    assert enr_status == 200 and enr_res.get("success") is True
    test_vec = list(base_vec)
    test_vec[2] = 0.01
    status, face_res = req("/api/v1/face/match", method="POST", body={"embedding": test_vec})
    print(f"Status: {status}, Face Match: {face_res}")
    assert status == 200
    assert face_res["matched"] is True
    assert face_res["username"] == "RAHIM"
    assert face_res["distance"] < 0.60

    print("\n--- TEST 9: Android ID Registration & Session Pairing ---")
    status, sess = req("/api/v1/devices/session", method="POST")
    sid = sess["session_id"]
    print(f"Created Session ID: {sid}")
    status, pair_res = req("/api/receive_android_id", method="POST", body={
        "session_id": sid,
        "android_id": "MOCK_SAMSUNG_GALAXY_ID_883"
    })
    print(f"Pairing Status: {status}, Response: {pair_res}")
    assert status == 200
    assert pair_res["status"] == "success"

    print("\n--- TEST 10: CSV Report Generation ---")
    status, csv_data = req("/api/v1/reports/export.csv")
    print(f"CSV Header & Content Preview:\n{csv_data[:150]}")
    assert status == 200
    assert "Username,Date,In_Time" in csv_data
    assert "SHAKIL" in csv_data

    print("\n--- TEST 11: Trigger BigQuery Outbox Sync ---")
    status, sync_res = req("/api/v1/sync/trigger", method="POST")
    print(f"Sync Status: {status}, Response: {sync_res}")
    assert status == 200
    assert sync_res["success"] is True

    print("\n--- TEST 12: Web Kiosk HTML UI Rendering ---")
    status, html_content = req("/")
    print(f"UI Status: {status}, HTML Length: {len(html_content)} bytes")
    assert status == 200
    assert "Face Recognition System" in html_content or "স্মার্ট উপস্থিতি ব্যবস্থা" in html_content

    print("\n🎉 ALL 12 INTEGRATION TESTS PASSED WITH 100% SUCCESS!")

finally:
    proc.terminate()
    proc.wait()
    print("Test server shutdown completed cleanly.")
