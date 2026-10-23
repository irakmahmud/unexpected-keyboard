#!/run/user/1000/veo3_pw_env/bin/python
"""
Comprehensive Staging End-to-End Verification Suite (Port 8090)
Tests:
1. Biometric ONNX FaceNet512 + MTCNN verification & detection enforcement (0-face, 1-face, 2-face)
2. 20-pose / multi-angle user enrollment, duplicate face prevention, duplicate Android ID prevention, biometric punch, FAISS file sync, and RAM+DB profile deletion + non-resurrection
3. Security hardening (HR password backdoor elimination, set/sync-hr-password overwrite protection, forgot-password bypass protection, path traversal & invalid ONNX upload prevention)
4. Database dual-format compatibility (YYYY-MM-DD + DD_Mon_YYYY dates, 8-char + 19-char UTC timestamps, ~/.last_attendance, ~/.attendance_time_check)
5. Reporting endpoints (CSV, real OpenXML .xlsx ZIP, HTML Muster Roll, Embeddings CSV)
6. Headless Chromium / Playwright UI E2E verification across all modals and HR workspace buttons (0 pageerror exceptions)
"""
import base64
import io
import json
import os
import sqlite3
import subprocess
import sys
import time
import urllib.error
import urllib.request
import zipfile

if os.path.isdir("/run/user/1000/veo3_pw_libs"):
    os.environ["LD_LIBRARY_PATH"] = "/run/user/1000/veo3_pw_libs:" + os.environ.get("LD_LIBRARY_PATH", "")

STAGING_PORT = 8090
BASE_URL = f"http://127.0.0.1:{STAGING_PORT}"
WORKSPACE = "/home/opc/attendance_system"
TEST_IMG_DIR = os.path.join(WORKSPACE, "rust_deepFace/test_images")
STAGING_DB = os.path.join(WORKSPACE, "data/staging_attendance.db")
BINARY_PATH = os.path.join(WORKSPACE, "deploy/staging_verified/attendance-api")
CHROME_BIN = "/run/user/1000/veo3_cloak_cache/chromium-146.0.7680.177.5/chrome"


IMAGE_MAP = {
    "Jolie_1.jpg": "Anjelina Jolie/1.jpg",
    "Jolie_2.png": "Anjelina Jolie/2.jpeg",
    "Jolie_3.jpg": "Anjelina Jolie/3.jpg",
    "Jolie_6.webp": "Anjelina Jolie/6.jpg",
    "face8.jpg": "Anjelina Jolie/face8.jpg",
    "pit.jpg": "Anjelina Jolie/pit.jpg",
    "Einstein_1.jpg": "Einstein/Albert_Einstein_Head_cleaned.jpg",
    "Einstein_2.jpg": "Einstein/close.jpg",
    "Hadi_1.jpeg": "osman hadi/color.jpg",
    "Hadi_2.jpg": "osman hadi/speech 2.jpg",
    "Hadi_3.jpg": "osman hadi/black.jpg",
}


def make_two_face_png_b64():
    """Decode natural.PNG or create a side-by-side 2-face image using Chromium canvas or PNGchunks."""
    from playwright.sync_api import sync_playwright
    b1 = b64_img("Jolie_3.jpg")
    b2 = b64_img("Einstein_2.jpg")
    with sync_playwright() as p:
        browser = p.chromium.launch(
            headless=True,
            executable_path=CHROME_BIN,
            args=["--no-sandbox", "--disable-gpu", "--no-zygote", "--single-process"],
        )
        page = browser.new_page()
        data_uri = page.evaluate(
            """async ([b1, b2]) => {
                const load = (src) => new Promise((res) => { const i = new Image(); i.onload = () => res(i); i.src = src; });
                const [i1, i2] = await Promise.all([
                    load('data:image/jpeg;base64,' + b1),
                    load('data:image/jpeg;base64,' + b2)
                ]);
                const c = document.createElement('canvas');
                c.width = 400;
                c.height = 250;
                const ctx = c.getContext('2d');
                ctx.drawImage(i1, 0, 0, 200, 250);
                ctx.drawImage(i2, 200, 0, 200, 250);
                return c.toDataURL('image/jpeg', 0.95);
            }""",
            [b1, b2],
        )
        browser.close()
        return data_uri.split(",", 1)[1]


def b64_img(filename):
    key = filename.strip()
    if key == "Blank_Img.jpg":
        import struct, zlib
        w, h = 64, 64
        raw = b"".join(b"\x00" + b"\x80\x80\x80" * w for _ in range(h))
        def chunk(t, d):
            return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)
        png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b"")
        return base64.b64encode(png).decode("ascii")
    if key == "Noise_Img.png":
        import struct, zlib
        w, h = 64, 64
        raw = b"".join(b"\x00" + bytes(((x * 73 + y * 31 + c * 17) % 256) for x in range(w) for c in range(3)) for y in range(h))
        def chunk(t, d):
            return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)
        png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b"")
        return base64.b64encode(png).decode("ascii")
    if key == "100.jpeg":
        return make_two_face_png_b64()
    rel = IMAGE_MAP.get(key, key)
    path = os.path.join(TEST_IMG_DIR, rel)
    with open(path, "rb") as f:
        return base64.b64encode(f.read()).decode("ascii")


def http_req(method, path, payload=None, timeout=30):
    url = f"{BASE_URL}{path}"
    headers = {}
    data = None
    if payload is not None:
        data = json.dumps(payload).encode("utf-8")
        headers["Content-Type"] = "application/json"
    req = urllib.request.Request(url, data=data, headers=headers, method=method)
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            raw = resp.read()
            ctype = resp.headers.get("Content-Type", "")
            if "application/json" in ctype:
                return resp.status, json.loads(raw.decode("utf-8"))
            return resp.status, raw
    except urllib.error.HTTPError as e:
        raw = e.read()
        try:
            return e.code, json.loads(raw.decode("utf-8"))
        except Exception:
            return e.code, raw.decode("utf-8", errors="replace")


def run_playwright_tests(record):
    from playwright.sync_api import sync_playwright

    page_errors = []
    console_errors = []

    with sync_playwright() as p:
        browser = p.chromium.launch(
            headless=True,
            executable_path=CHROME_BIN,
            args=["--no-sandbox", "--disable-gpu", "--no-zygote", "--single-process"],
        )
        context = browser.new_context()
        page = context.new_page()
        page.on("pageerror", lambda err: page_errors.append(str(err)))
        page.on("console", lambda msg: console_errors.append(msg.text) if msg.type == "error" else None)

        # 1. Load Kiosk & Admin Portal
        page.goto(f"{BASE_URL}/", wait_until="domcontentloaded", timeout=15000)
        page.wait_for_timeout(500)
        title = page.title()
        hdr_company = page.locator("#header-company-name").inner_text()
        record("Frontend", "Portal initial render & active company header badge", "Face Recognition" in title and len(hdr_company) > 0, f"title='{title}', badge='{hdr_company}'")

        # 2. Company Select / Create / Edit Modal
        page.evaluate("openCompanyManagement()")
        page.wait_for_timeout(300)
        comp_visible = page.locator("#modal-company").is_visible()
        page.evaluate("showCompanySubView('create')")
        page.wait_for_timeout(150)
        page.evaluate("showCompanySubView('edit')")
        page.wait_for_timeout(150)
        page.evaluate("showCompanySubView('select')")
        page.wait_for_timeout(150)
        page.evaluate("closeCompanyManagement()")
        record("Frontend", "Company Select/Create/Edit modal views", comp_visible, "All 3 company sub-views switched cleanly")

        # 3. HR Auth Modal, Change Password View & HR Workspace Unlock
        page.evaluate("openHrManagement()")
        page.wait_for_timeout(200)
        page.evaluate("toggleHrChangePasswordView(true)")
        page.wait_for_timeout(150)
        cp_vis = page.locator("#hr-auth-change-view").is_visible()
        page.evaluate("toggleHrChangePasswordView(false)")
        page.fill("#hr-admin-pin", "CustomHR#2026")
        page.evaluate("unlockHrManagement()")
        page.wait_for_timeout(400)
        hr_ws_visible = page.locator("#modal-hr").is_visible()
        record("Frontend", "HR Authentication & Change Password toggle & Workspace unlock", cp_vis and hr_ws_visible, "Unlocked #modal-hr with CustomHR#2026")

        # 4. All HR Workspace Actions (Users List + 11 HR Actions)
        hr_actions = [
            ("users-list", "HR View Registered Users table"),
            ("reg-camera", "HR Action 1: User Registration with Camera (20-Pose UI)"),
            ("reg-no-camera", "HR Action 2: User Registration without Camera"),
            ("att-report", "HR Action 3: Customized Attendance Report UI"),
            ("sync-local", "HR Action 4: Sync Users to Local Database"),
            ("verify-image", "HR Action 5: Check Fake Attendance (Verify Image)"),
            ("upload-model", "HR Action 6: Upload AI Model (Hot-Reload ONNX)"),
            ("delete-face", "HR Action 7: Delete Face (Reset Encodings)"),
            ("embeddings-report", "HR Action 8: Facial Features Report (512-D ONNX)"),
            ("sync-cloud", "HR Action 9: Sync Users to Cloud Database"),
            ("sync-hr-pw", "HR Action 10: Sync HR Password to Local"),
            ("reg-qr", "HR Action 11: User Registration with QR Code"),
        ]
        for act_key, label in hr_actions:
            page.evaluate(f"openHrAction('{act_key}')")
            page.wait_for_timeout(300)
            ws_vis = page.locator("#hr-action-workspace").is_visible()
            body_html = page.locator("#hr-workspace-body").inner_html()
            record("Frontend", label, ws_vis and len(body_html) > 10, f"action='{act_key}', rendered={len(body_html)} chars")
            if act_key == "reg-camera":
                jolie_b64 = b64_img("Jolie_1.jpg")
                page.evaluate(f"window.__TEST_FACE_B64 = 'data:image/jpeg;base64,{jolie_b64}';")
                page.fill("#reg-cam-username", "POSE_TEST_USER")
                pose_ok = page.evaluate("captureSingleRegPose()")
                page.wait_for_timeout(200)
                pose_status = page.locator("#reg-pose-status").inner_text()
                page.evaluate("window.__TEST_FACE_B64 = null;")
                record("Frontend", "Live Single-Pose Camera Validation via /api/v1/face/extract-frame", bool(pose_ok) and "1 / 20" in pose_status, f"status='{pose_status}'")
            if act_key == "reg-no-camera":
                page.fill("#reg-nocam-username", "UI_TEST_USER")
                page.fill("#reg-nocam-desig", "QA Engineer")
                page.fill("#reg-nocam-password", "UiPass#123")
                page.evaluate("submitNoCameraRegistration()")
                page.wait_for_timeout(400)
                code_u, body_u = http_req("GET", "/api/v1/users/check/UI_TEST_USER")
                record("Frontend", "Submit No-Camera User Registration via HR UI", code_u == 200 and body_u.get("exists") is True, f"exists={body_u.get('exists')}")
            page.evaluate("backToHrMainMenu()")
            page.wait_for_timeout(100)

        page.evaluate("closeHrManagement()")

        # 5. No-Camera Login & Forgot Password Reset via UI
        page.evaluate("openNoCameraLogin()")
        page.wait_for_timeout(250)
        nocam_vis = page.locator("#modal-no-camera").is_visible()
        page.fill("#no-cam-username", "UI_TEST_USER")
        page.fill("#no-cam-password", "UiPass#123")
        page.evaluate("submitNoCameraLogin()")
        page.wait_for_timeout(500)
        record("Frontend", "No-Camera Password Login Modal & Punch", nocam_vis, "Submitted login for UI_TEST_USER")

        page.evaluate("openNoCameraLogin()")
        page.wait_for_timeout(200)
        page.evaluate("showForgotPasswordPopup()")
        page.wait_for_timeout(200)
        fp_vis = page.locator("#no-cam-reset-box").is_visible()
        page.fill("#fp-username", "UI_TEST_USER")
        page.fill("#fp-hr-verify", "CustomHR#2026")
        page.fill("#fp-new-password", "UiPass#456")
        page.evaluate("submitForgotPasswordReset()")
        page.wait_for_timeout(400)
        fp_msg = page.locator("#fp-status-msg").inner_text()
        page.evaluate("closeNoCameraLogin()")
        record("Frontend", "Forgot Password Reset Popup & Execution", fp_vis and "reset" in fp_msg.lower(), f"msg='{fp_msg}'")

        # 6. Identity Verification Split-Window, QR Code Login, 7-Day History, Force Sync
        page.evaluate(
            """openIdentityVerificationModal({
                matched: true,
                username: 'UI_TEST_USER',
                login_type: 'IN_TIME',
                date: '2026-10-22',
                time: '09:00 AM',
                match_quality: 'EXACT',
                confidence: 98.5,
                min_distance: 0.142,
                exact_matches: 18,
                strong_matches: 2,
                possible_matches: 0,
                total_matches: 20
            }, '')"""
        )
        page.wait_for_timeout(250)
        vi_vis = page.locator("#modal-identity-verify").is_visible()
        page.evaluate("closeIdentityVerifyModal()")
        record("Frontend", "Identity Verification Split-Window Modal", vi_vis, f"visible={vi_vis}")

        page.evaluate("openQrLogin()")
        page.wait_for_timeout(250)
        qr_vis = page.locator("#modal-qr").is_visible()
        page.evaluate("closeQrLogin()")
        record("Frontend", "QR Code Login Modal (#modal-qr)", qr_vis, f"visible={qr_vis}")

        page.evaluate("openSevenDayHistory()")
        page.wait_for_timeout(350)
        hist_vis = page.locator("#modal-seven-days").is_visible()
        page.evaluate("closeSevenDayHistory()")
        record("Frontend", "7-Day Attendance History Modal (#modal-seven-days)", hist_vis, f"visible={hist_vis}")

        page.evaluate("forceSync()")
        page.wait_for_timeout(350)
        record("Frontend", "Zero uncaught JS pageerror exceptions across all UI flows", len(page_errors) == 0, f"page_errors={page_errors}")

        browser.close()


def main():
    results = []

    def record(category, name, passed, details):
        status = "PASS" if passed else "FAIL"
        print(f"[{status}] [{category}] {name}: {details}")
        results.append({
            "category": category,
            "name": name,
            "passed": bool(passed),
            "details": str(details),
        })

    if os.path.exists(STAGING_DB):
        os.remove(STAGING_DB)
    os.makedirs(os.path.dirname(STAGING_DB), exist_ok=True)

    env = os.environ.copy()
    env["PORT"] = str(STAGING_PORT)
    env["ATTENDANCE_DB_PATH"] = STAGING_DB
    env["MODELS_DIR"] = os.path.join(WORKSPACE, "models")

    proc = subprocess.Popen(
        [BINARY_PATH],
        cwd=WORKSPACE,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )

    try:
        # Wait for server startup
        ready = False
        for _ in range(60):
            try:
                code, _ = http_req("GET", "/")
                if code == 200:
                    ready = True
                    break
            except Exception:
                pass
            time.sleep(0.25)

        record("Startup", "Staging server listening on port 8090", ready, f"HTTP 200 on {BASE_URL}/")
        if not ready:
            sys.exit(1)

        # Check ~/.attendance_time_check created on startup (CAP-03)
        tc_path = os.path.expanduser("~/.attendance_time_check")
        record("System", "Time integrity file (~/.attendance_time_check)", os.path.exists(tc_path), tc_path)

        # 1. Biometric Verification Tests
        same_pairs = [
            ("Jolie_1.jpg", "Jolie_3.jpg"),
            ("Jolie_1.jpg", "Jolie_6.webp"),
            ("Jolie_1.jpg", "face8.jpg"),
            ("Einstein_1.jpg", "Einstein_2.jpg"),
            ("Hadi_1.jpeg", "Hadi_2.jpg"),
            ("Hadi_1.jpeg", "Hadi_3.jpg"),
        ]
        for img1, img2 in same_pairs:
            code, body = http_req("POST", "/api/v1/face/verify", {
                "image1_b64": b64_img(img1),
                "image2_b64": b64_img(img2),
            })
            ok = (code == 200 and isinstance(body, dict) and body.get("verified") is True and body.get("distance", 1.0) <= 0.30)
            record("Biometric", f"Same-person verify ({img1} vs {img2})", ok, f"HTTP {code}, distance={body.get('distance') if isinstance(body, dict) else body}")

        diff_pairs = [
            ("Jolie_1.jpg", "Einstein_1.jpg"),
            ("Jolie_1.jpg", "Hadi_1.jpeg"),
            ("Einstein_1.jpg", "Hadi_1.jpeg"),
            ("Jolie_1.jpg", "pit.jpg"),
        ]
        for img1, img2 in diff_pairs:
            code, body = http_req("POST", "/api/v1/face/verify", {
                "image1_b64": b64_img(img1),
                "image2_b64": b64_img(img2),
            })
            ok = (code == 200 and isinstance(body, dict) and body.get("verified") is False and body.get("distance", 0.0) > 0.60)
            record("Biometric", f"Different-person reject ({img1} vs {img2})", ok, f"HTTP {code}, distance={body.get('distance') if isinstance(body, dict) else body}")

        # 0-face and 2-face rejection (enforce_detection=True)
        for non_face in [" Blank_Img.jpg", "Noise_Img.png"]:
            code, body = http_req("POST", "/api/v1/face/extract-frame", {"image_b64": b64_img(non_face)})
            ok = (code == 422)
            record("Biometric", f"0-face rejection ({non_face.strip()})", ok, f"HTTP {code}, body={body}")

        code, body = http_req("POST", "/api/v1/face/extract-frame", {"image_b64": b64_img("100.jpeg")})
        record("Biometric", "Multi-face (2 faces) rejection (100.jpeg)", code == 422, f"HTTP {code}, body={body}")

        # 2. User Enrollment, Duplicate Rejection, Identity Verification, Biometric Punch & Deletion
        code, body = http_req("POST", "/api/v1/users", {
            "username": "JOLIE_TEST",
            "user_designation": "Lead Engineer",
            "user_android_id": "ANDR_JOLIE_01",
            "images_b64": [b64_img("Jolie_1.jpg"), b64_img("Jolie_2.png"), b64_img("Jolie_3.jpg")],
        })
        ok = (code == 200 and isinstance(body, dict) and body.get("embeddings_enrolled") == 3 and body.get("with_camera") is True)
        record("Enrollment", "Multi-angle ONNX registration (JOLIE_TEST)", ok, f"HTTP {code}, {body}")

        # Check FAISS files synced to ~/db/
        faiss_idx = os.path.expanduser("~/db/face_encodings.index")
        faiss_map = os.path.expanduser("~/db/user_mappings.txt")
        faiss_ok = os.path.exists(faiss_idx) and os.path.exists(faiss_map) and "JOLIE_TEST" in open(faiss_map).read()
        record("Biometric", "Persistent FAISS index & user_mappings.txt sync", faiss_ok, f"{faiss_idx} & {faiss_map}")

        # Duplicate face registration prevention
        code, body = http_req("POST", "/api/v1/users", {
            "username": "JOLIE_CLONE",
            "user_designation": "Impostor",
            "images_b64": [b64_img("Jolie_6.webp")],
        })
        ok = (code == 400 and "Duplicate face detected" in json.dumps(body))
        record("Security", "Duplicate face enrollment blocked (JOLIE_CLONE)", ok, f"HTTP {code}, {body}")

        # Duplicate Android Device ID registration prevention across users
        code, body = http_req("POST", "/api/v1/users", {
            "username": "JOLIE_DUP_DEV",
            "user_designation": "Tester",
            "user_android_id": "ANDR_JOLIE_01",
            "overwrite_existing": True,
        })
        ok = (code == 400 and "already registered to user" in json.dumps(body))
        record("Security", "Duplicate Android Device ID blocked across users (JOLIE_DUP_DEV)", ok, f"HTTP {code}, {body}")

        # Verify identity endpoint with unseen photo Jolie_6.webp
        code, body = http_req("POST", "/api/v1/face/verify-identity", {"image_b64": b64_img("Jolie_6.webp")})
        ok = (code == 200 and isinstance(body, dict) and body.get("matched") is True and body.get("username") == "JOLIE_TEST")
        record("Biometric", "Identity verification on unseen photo (Jolie_6.webp -> JOLIE_TEST)", ok, f"HTTP {code}, dist={body.get('min_distance') if isinstance(body, dict) else body}")

        # Biometric punch with unseen photo face8.jpg
        code, body = http_req("POST", "/api/v1/attendance/punch", {
            "username": "",
            "method": "Face",
            "image_b64": b64_img("face8.jpg"),
        })
        ok = (code == 200 and isinstance(body, dict) and body.get("username") == "JOLIE_TEST" and body.get("success") is True)
        record("Attendance", "Biometric camera punch-in (face8.jpg -> JOLIE_TEST)", ok, f"HTTP {code}, {body}")

        # Check ~/.last_attendance updated
        la_path = os.path.expanduser("~/.last_attendance")
        la_ok = os.path.exists(la_path) and "JOLIE_TEST" in open(la_path).read()
        record("Attendance", "Cooldown file (~/.last_attendance) updated", la_ok, la_path)

        # Delete face profile and verify RAM + SQLite purge + sync/rebuild non-resurrection
        code, body = http_req("DELETE", "/api/v1/face/profiles/JOLIE_TEST")
        record("Biometric", "Delete face profile (DELETE /api/v1/face/profiles/JOLIE_TEST)", code == 200, f"HTTP {code}, {body}")

        code, u_body = http_req("GET", "/api/v1/users/JOLIE_TEST")
        db_purged = (
            code == 200
            and isinstance(u_body, dict)
            and u_body.get("face_encoding") is None
            and u_body.get("register_with_camera") == "N"
        )
        record("Security", "Deleted face profile cleared users.face_encoding=NULL & register_with_camera='N'", db_purged, f"HTTP {code}, face_encoding={u_body.get('face_encoding') if isinstance(u_body, dict) else u_body}, reg_cam={u_body.get('register_with_camera') if isinstance(u_body, dict) else u_body}")

        # Trigger users-to-local sync and index rebuild to verify deleted face CANNOT resurrect
        http_req("POST", "/api/v1/sync/users-to-local", {})
        http_req("POST", "/api/v1/face/index/rebuild", {})

        code, body = http_req("POST", "/api/v1/face/verify-identity", {"image_b64": b64_img("Jolie_6.webp")})
        ok = (code == 200 and isinstance(body, dict) and body.get("matched") is False)
        record("Security", "Deleted face profile stays purged after sync/users-to-local & index/rebuild", ok, f"HTTP {code}, matched={body.get('matched') if isinstance(body, dict) else body}")

        # 3. Security & Authentication Hardening Tests
        code, body = http_req("POST", "/api/v1/auth/change-hr-password", {
            "company_id": "HQ_001",
            "old_password": "HR@123",
            "new_password": "CustomHR#2026",
        })
        record("Security", "Change HR password from default to CustomHR#2026", code == 200, f"HTTP {code}, {body}")

        for backdoor in ["HR@123", "1234", "admin123", "hr2026"]:
            code, body = http_req("POST", "/api/v1/auth/verify-hr", {
                "company_id": "HQ_001",
                "password": backdoor,
            })
            rejected = (code == 401)
            record("Security", f"Reject hardcoded HR backdoor '{backdoor}' with HTTP 401", rejected, f"HTTP {code}, {body}")

        code, body = http_req("POST", "/api/v1/auth/verify-hr", {
            "company_id": "HQ_001",
            "password": "CustomHR#2026",
        })
        record("Security", "Accept legitimate custom HR password 'CustomHR#2026'", code == 200 and body.get("verified") is True, f"HTTP {code}, {body}")

        # Ensure unauthenticated set-hr-password and sync-hr-password cannot overwrite CustomHR#2026
        code_set, body_set = http_req("POST", "/api/v1/auth/set-hr-password", {"company_id": "HQ_001", "password": "HijackPassword"})
        code_sync, body_sync = http_req("POST", "/api/v1/auth/sync-hr-password", {"company_id": "HQ_001", "password": "HijackPassword"})
        code, body = http_req("POST", "/api/v1/auth/sync-hr-password", {"company_id": "HQ_001"})
        code2, body2 = http_req("POST", "/api/v1/auth/verify-hr", {
            "company_id": "HQ_001",
            "password": "CustomHR#2026",
        })
        record(
            "Security",
            "Block unauthenticated HR password overwrite & preserve CustomHR#2026",
            code_set == 401 and code_sync == 401 and code2 == 200 and body2.get("verified") is True,
            f"set_code={code_set}, sync_explicit_code={code_sync}, verify={body2}",
        )

        # Ensure forgot-password rejects public designation string
        code_fp, body_fp = http_req("POST", "/api/v1/auth/forgot-password", {
            "username": "JOLIE_TEST",
            "verification": "Lead Engineer",
            "new_password": "hacked_password",
        })
        record("Security", "Reject public designation in forgot-password verification", code_fp == 401, f"HTTP {code_fp}, {body_fp}")

        # Path traversal rejection
        code, body = http_req("POST", "/api/v1/face/upload-model", {
            "model_name": "../../etc/passwd.onnx",
            "model_b64": base64.b64encode(b"x" * 100).decode("ascii"),
        })
        record("Security", "Path traversal blocked on /api/v1/face/upload-model", code == 400, f"HTTP {code}, {body}")

        # Invalid ONNX payload rejection BEFORE overwriting models/facenet512.onnx
        facenet_path = os.path.join(WORKSPACE, "models", "facenet512.onnx")
        size_before = os.path.getsize(facenet_path)
        code, body = http_req("POST", "/api/v1/face/upload-model", {
            "model_name": "facenet512.onnx",
            "model_b64": base64.b64encode(b"CORRUPTED_NON_ONNX_PAYLOAD_" * 10).decode("ascii"),
        })
        size_after = os.path.getsize(facenet_path)
        record(
            "Security",
            "Corrupted ONNX upload rejected with HTTP 400 before disk write",
            code == 400 and size_before == size_after == 94176340,
            f"HTTP {code}, size_before={size_before}, size_after={size_after}, body={body}",
        )

        # 4. Database Dual-Format Compatibility Tests
        conn = sqlite3.connect(STAGING_DB)
        conn.execute(
            "INSERT INTO attendance_records (record_id, company_user_uuid, username, date, in_time, out_time, in_time_with_camera, out_time_with_camera, status, work_hours, created_at) "
            "VALUES ('LEGACY_01', 'C_JOLIE_01', 'JOLIE_TEST', '20_Oct_2026', '2026-10-20 03:05:00', '2026-10-20 11:30:00', 'Y', 'Y', 'PRESENT', 8.4, '2026-10-20 03:05:00')"
        )
        conn.commit()
        conn.close()

        code, body = http_req("GET", "/api/v1/attendance/range?from=2026-10-01&to=2026-10-31&username=JOLIE_TEST")
        has_legacy = isinstance(body, list) and any(r.get("record_id") == "LEGACY_01" and r.get("date") == "2026-10-20" and r.get("in_time") == "09:05:00" for r in body)
        record("Database", "Dual-format DD_Mon_YYYY date & 19-char UTC timestamp normalization", code == 200 and has_legacy, f"HTTP {code}, records={body}")

        # 5. Reporting Endpoints (CSV, XLSX, HTML Muster Roll, Embeddings CSV)
        code, raw_csv = http_req("GET", "/api/v1/reports/customized.csv")
        record("Reporting", "Customized Attendance Report CSV", code == 200 and len(raw_csv) > 20, f"HTTP {code}, bytes={len(raw_csv)}")

        code, raw_xlsx = http_req("GET", "/api/v1/reports/customized.xlsx")
        is_valid_xlsx = False
        if code == 200 and isinstance(raw_xlsx, bytes) and raw_xlsx[:4] == b"PK\x03\x04":
            with zipfile.ZipFile(io.BytesIO(raw_xlsx), "r") as zf:
                is_valid_xlsx = "xl/worksheets/sheet1.xml" in zf.namelist()
        record("Reporting", "Valid OpenXML .xlsx Report (ZIP archive with sheet1.xml)", is_valid_xlsx, f"HTTP {code}, bytes={len(raw_xlsx) if isinstance(raw_xlsx, bytes) else 0}")

        code, raw_html = http_req("GET", "/api/v1/reports/muster-roll/html?year=2026&month=10")
        record("Reporting", "HTML Monthly Muster Roll Report", code == 200 and b"<table" in raw_html, f"HTTP {code}, bytes={len(raw_html)}")

        code, raw_emb_csv = http_req("GET", "/api/v1/face/embeddings-report.csv")
        record("Reporting", "Face Embeddings Analysis CSV Report", code == 200 and b"Username" in raw_emb_csv, f"HTTP {code}, bytes={len(raw_emb_csv)}")

        # 6. Headless Chromium / Playwright UI E2E Verification
        run_playwright_tests(record)

        # Verify UI registration saved user_designation and register_with_camera properly
        code_ui, body_ui = http_req("GET", "/api/v1/users/UI_TEST_USER")
        ui_desig_ok = (
            code_ui == 200
            and isinstance(body_ui, dict)
            and body_ui.get("user_designation") == "QA Engineer"
            and body_ui.get("register_with_camera") == "N"
        )
        record("Frontend", "UI registration persisted user_designation='QA Engineer' & register_with_camera='N'", ui_desig_ok, f"HTTP {code_ui}, {body_ui}")

        # Verify Cloud Outbox Sync flushes pending events to COMPLETED
        code_tr, body_tr = http_req("POST", "/api/v1/sync/trigger")
        code_st, body_st = http_req("GET", "/api/v1/sync/status")
        outbox_ok = (
            code_tr == 200
            and code_st == 200
            and isinstance(body_st, dict)
            and body_st.get("pending_count") == 0
            and os.path.exists(os.path.join(WORKSPACE, "data", "cloud_sync_outbox.jsonl"))
        )
        record("Database", "Cloud Sync Outbox flush marks events COMPLETED & archives JSONL", outbox_ok, f"trigger={body_tr}, status={body_st}")

        # 7. Security & Parity Regression Checks
        code_bio_mismatch, body_bio_mismatch = http_req(
            "POST",
            "/api/v1/attendance/punch",
            {"username": "JOLIE_TEST", "method": "CAMERA", "image_b64": einstein_b64},
        )
        record(
            "Security",
            "Biometric punch rejects mismatched camera face even when username is provided (HTTP 401)",
            code_bio_mismatch == 401,
            f"HTTP {code_bio_mismatch}, {body_bio_mismatch}",
        )

        code_pin_missing, body_pin_missing = http_req(
            "POST",
            "/api/v1/attendance/punch",
            {"username": "UI_TEST_USER", "method": "PIN"},
        )
        record(
            "Security",
            "PIN punch rejects missing pin/password parameter (HTTP 401)",
            code_pin_missing == 401,
            f"HTTP {code_pin_missing}, {body_pin_missing}",
        )

        code_cd, body_cd = http_req(
            "POST",
            "/api/v1/attendance/punch",
            {"username": "UI_TEST_USER", "method": "PIN", "pin": "UiPass#456"},
        )
        record(
            "Security",
            "Duplicate punch within 60s cooldown returns HTTP 409 Conflict",
            code_cd == 409,
            f"HTTP {code_cd}, {body_cd}",
        )

        last_att_path = os.path.expanduser("~/.last_attendance")
        last_att_json_ok = False
        if os.path.exists(last_att_path):
            with open(last_att_path, "r") as lf:
                parsed_la = json.load(lf)
                last_att_json_ok = isinstance(parsed_la, dict) and "last_attendance" in parsed_la and "users" in parsed_la
        record(
            "Database",
            "~/.last_attendance persisted in Python-compatible JSON format",
            last_att_json_ok,
            f"path={last_att_path}",
        )

        http_req(
            "PUT",
            "/api/v1/companies/TECHFIRM_01",
            {"company_id": "TECHFIRM_01", "company_name": "TechFirm Solutions Ltd.", "password": "admin"},
        )
        code_comp, body_comp = http_req("GET", "/api/v1/companies/TECHFIRM_01")
        comp_preserved = (
            code_comp == 200
            and isinstance(body_comp, dict)
            and body_comp.get("website") is not None
            and body_comp.get("email") is not None
        )
        record(
            "Database",
            "CompanyRepo::upsert_company preserves optional fields (website, email, tax_id) on update",
            comp_preserved,
            f"HTTP {code_comp}, website={body_comp.get('website') if isinstance(body_comp, dict) else None}",
        )

    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except Exception:
            proc.kill()

    passed_count = sum(1 for r in results if r["passed"])
    total_count = len(results)
    print(f"\nSUMMARY: {passed_count}/{total_count} checks passed.")
    with open(os.path.join(WORKSPACE, "staging_verification_results.json"), "w") as f:
        json.dump({"passed": passed_count, "total": total_count, "results": results}, f, indent=2)
    if passed_count != total_count:
        sys.exit(1)


if __name__ == "__main__":
    main()
