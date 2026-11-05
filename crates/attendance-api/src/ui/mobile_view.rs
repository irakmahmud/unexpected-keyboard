pub fn render_mobile_manifest_json() -> String {
    r###"{
  "name": "Smart Attendance Mobile — Millions Scale & Geofence",
  "short_name": "AttendanceMobile",
  "start_url": "/mobile",
  "display": "standalone",
  "background_color": "#090d16",
  "theme_color": "#00f2fe",
  "orientation": "portrait",
  "icons": [
    {
      "src": "data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><rect width='100' height='100' rx='22' fill='%230f172a'/><text y='68' x='18' font-size='60'>⚡</text></svg>",
      "sizes": "192x192",
      "type": "image/svg+xml"
    }
  ]
}"###
    .to_string()
}

pub fn render_mobile_app_html() -> String {
    r###"<!DOCTYPE html>
<html lang="bn">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
    <meta name="theme-color" content="#090d16">
    <link rel="manifest" href="/manifest.json">
    <title>Smart Attendance Mobile — Millions-Scale Office Control (Mandatory Location + Wi-Fi)</title>
    <style>
        :root {
            --bg-dark: #090d16;
            --card-bg: rgba(18, 26, 45, 0.92);
            --accent-cyan: #00f2fe;
            --accent-green: #00e676;
            --accent-red: #ff5252;
            --accent-amber: #ffab00;
            --accent-purple: #7c4dff;
            --text-main: #f1f5f9;
            --text-sub: #94a3b8;
        }
        * { box-sizing: border-box; margin: 0; padding: 0; font-family: 'Segoe UI', system-ui, -apple-system, sans-serif; }
        body { background: var(--bg-dark); color: var(--text-main); min-height: 100vh; padding-bottom: 90px; overflow-x: hidden; }
        .mobile-header {
            background: linear-gradient(135deg, #0f172a 0%, #1e1b4b 100%);
            border-bottom: 1px solid rgba(0, 242, 254, 0.3);
            padding: 12px 16px;
            position: sticky; top: 0; z-index: 100;
            display: flex; justify-content: space-between; align-items: center;
        }
        .brand-box { display: flex; align-items: center; gap: 10px; }
        .brand-icon { width: 38px; height: 38px; border-radius: 10px; background: linear-gradient(135deg, #00f2fe, #4facfe); display: flex; align-items: center; justify-content: center; font-size: 1.3rem; color: #051923; font-weight: 900; }
        .brand-title { font-size: 0.95rem; font-weight: 800; letter-spacing: 0.5px; color: #fff; }
        .brand-sub { font-size: 0.7rem; color: var(--accent-cyan); display: block; }
        .security-ribbon {
            background: rgba(15, 23, 42, 0.95);
            border-bottom: 1px solid rgba(255,255,255,0.08);
            padding: 8px 14px;
            font-size: 0.74rem;
            display: flex; flex-wrap: wrap; gap: 6px; justify-content: space-between; align-items: center;
        }
        .badge { padding: 4px 8px; border-radius: 999px; font-weight: 700; font-size: 0.7rem; display: inline-flex; align-items: center; gap: 4px; }
        .badge-ok { background: rgba(0, 230, 118, 0.16); color: #00e676; border: 1px solid rgba(0, 230, 118, 0.4); }
        .badge-err { background: rgba(255, 82, 82, 0.2); color: #ff5252; border: 1px solid rgba(255, 82, 82, 0.5); }
        .badge-warn { background: rgba(255, 171, 0, 0.18); color: #ffab00; border: 1px solid rgba(255, 171, 0, 0.45); }
        .container { padding: 12px; max-width: 680px; margin: 0 auto; }
        .card {
            background: var(--card-bg);
            border: 1px solid rgba(0, 242, 254, 0.18);
            border-radius: 14px;
            padding: 15px;
            margin-bottom: 14px;
            box-shadow: 0 8px 24px rgba(0,0,0,0.35);
        }
        .card-title { font-size: 0.94rem; font-weight: 800; color: var(--accent-cyan); margin-bottom: 10px; display: flex; align-items: center; justify-content: space-between; gap: 6px; }
        .form-group { margin-bottom: 10px; }
        .form-label { font-size: 0.76rem; color: var(--text-sub); font-weight: 700; display: block; margin-bottom: 4px; }
        .form-input {
            width: 100%; padding: 9px 12px; border-radius: 10px;
            background: #0b1120; border: 1px solid rgba(255,255,255,0.16);
            color: #fff; font-size: 0.88rem; outline: none;
        }
        .form-input:focus { border-color: var(--accent-cyan); }
        .btn {
            width: 100%; padding: 11px; border-radius: 10px; border: none;
            font-weight: 800; font-size: 0.86rem; cursor: pointer;
            display: inline-flex; align-items: center; justify-content: center; gap: 7px;
            transition: transform 0.1s, opacity 0.2s; text-decoration: none;
        }
        .btn:active { transform: scale(0.98); }
        .btn-cyan { background: linear-gradient(135deg, #00f2fe, #4facfe); color: #041421; }
        .btn-green { background: linear-gradient(135deg, #00e676, #00b0ff); color: #041421; }
        .btn-purple { background: linear-gradient(135deg, #7c4dff, #b388ff); color: #fff; }
        .btn-red { background: linear-gradient(135deg, #ff5252, #d50000); color: #fff; }
        .btn-amber { background: linear-gradient(135deg, #ffab00, #ff6d00); color: #041421; }
        .btn-outline { background: rgba(255,255,255,0.06); border: 1px solid rgba(0, 242, 254, 0.4); color: #e2e8f0; }

        /* MANDATORY LOCATION HARD LOCK OVERLAY */
        #mobile-location-gate {
            position: fixed; inset: 0; z-index: 9999;
            background: radial-gradient(circle at top, #1e1b4b 0%, #090d16 75%);
            display: flex; flex-direction: column; justify-content: center; align-items: center;
            padding: 18px; overflow-y: auto;
        }
        .gate-box {
            width: 100%; max-width: 480px;
            background: rgba(15, 23, 42, 0.96);
            border: 2px solid var(--accent-red);
            border-radius: 18px;
            padding: 20px;
            text-align: center;
            box-shadow: 0 0 40px rgba(255, 82, 82, 0.25);
        }
        .gate-icon { font-size: 3rem; margin-bottom: 8px; }
        .bottom-nav {
            position: fixed; bottom: 0; left: 0; right: 0; z-index: 200;
            background: rgba(11, 17, 32, 0.98);
            border-top: 1px solid rgba(0, 242, 254, 0.25);
            display: grid; grid-template-columns: repeat(5, 1fr);
            padding: 6px 2px;
        }
        .nav-btn {
            background: none; border: none; color: var(--text-sub);
            display: flex; flex-direction: column; align-items: center; gap: 3px;
            padding: 5px 2px; font-size: 0.66rem; font-weight: 700; cursor: pointer;
        }
        .nav-btn.active { color: var(--accent-cyan); }
        .nav-btn span.icon { font-size: 1.15rem; }
        .tab-view { display: none; }
        .tab-view.active { display: block; }
        .user-row {
            background: rgba(255,255,255,0.03);
            border: 1px solid rgba(255,255,255,0.08);
            border-radius: 10px;
            padding: 10px 12px;
            margin-bottom: 8px;
            display: flex; justify-content: space-between; align-items: center; gap: 8px;
        }
        .stat-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-bottom: 12px; }
        .stat-box { background: rgba(0, 242, 254, 0.06); border: 1px solid rgba(0, 242, 254, 0.2); border-radius: 10px; padding: 10px; text-align: center; }
        .stat-val { font-size: 1.3rem; font-weight: 900; color: #fff; }
        .stat-lbl { font-size: 0.7rem; color: var(--text-sub); }
        .nine-cards-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; }
        .mini-att-card {
            background: rgba(9, 13, 22, 0.9);
            border: 1px solid rgba(255,255,255,0.09);
            border-left: 3px solid var(--accent-cyan);
            border-radius: 8px; padding: 8px;
        }
        .mini-att-lbl { font-size: 0.66rem; color: var(--text-sub); margin-bottom: 3px; }
        .mini-att-val { font-size: 0.82rem; font-weight: 800; color: #fff; word-break: break-word; }
        table.mob-table { width: 100%; border-collapse: collapse; font-size: 0.76rem; }
        table.mob-table th, table.mob-table td { padding: 7px 6px; border-bottom: 1px solid rgba(255,255,255,0.08); text-align: left; }
        table.mob-table th { color: var(--accent-cyan); font-weight: 800; }
    </style>
</head>
<body>

    <!-- ===================================================================== -->
    <!-- MANDATORY GPS LOCATION & WI-FI ENFORCEMENT GATE (BLOCKS WITHOUT GPS)  -->
    <!-- ===================================================================== -->
    <div id="mobile-location-gate">
        <div class="gate-box" id="gate-card">
            <div class="gate-icon" id="gate-icon">📍</div>
            <h2 style="font-size: 1.2rem; color: #ff5252; margin-bottom: 6px;" id="gate-title">
                লোকেশন ছাড়া মোবাইল অ্যাপ কাজ করবে না!
            </h2>
            <p style="font-size: 0.84rem; color: #cbd5e1; margin-bottom: 14px; line-height: 1.45;" id="gate-desc">
                নিরাপত্তা পলিসি অনুযায়ী এই মোবাইল অ্যাপ ব্যবহার করতে হলে আপনার ফোনের <b>GPS Location</b> চালু থাকতে হবে এবং আপনাকে নির্ধারিত বাসা/অফিসের লোকেশনে থাকতে হবে।
            </p>

            <div style="background: #090d16; border: 1px solid rgba(255,255,255,0.12); border-radius: 12px; padding: 12px; text-align: left; font-size: 0.8rem; margin-bottom: 14px;">
                <div style="display:flex; justify-content:space-between; margin-bottom:5px;">
                    <span style="color:#94a3b8;">📍 Location Mandatory:</span>
                    <strong style="color:#00e676;">বাধ্যতামূলক (ALWAYS ON)</strong>
                </div>
                <div style="display:flex; justify-content:space-between; margin-bottom:5px;">
                    <span style="color:#94a3b8;">🏠 নির্ধারিত স্থান:</span>
                    <strong id="gate-target-name" style="color:#00f2fe;">Loading...</strong>
                </div>
                <div style="display:flex; justify-content:space-between; margin-bottom:5px;">
                    <span style="color:#94a3b8;">🎯 নির্ধারিত GPS (Lat, Lon):</span>
                    <strong id="gate-target-coords" style="color:#e2e8f0;">23.8103, 90.4125</strong>
                </div>
                <div style="display:flex; justify-content:space-between; margin-bottom:5px;">
                    <span style="color:#94a3b8;">📡 আপনার বর্তমান GPS:</span>
                    <strong id="gate-current-coords" style="color:#ffab00;">যাচাই করা হচ্ছে...</strong>
                </div>
                <div style="display:flex; justify-content:space-between; margin-bottom:5px;">
                    <span style="color:#94a3b8;">📏 দূরত্ব (Distance):</span>
                    <strong id="gate-distance" style="color:#ff5252;">---</strong>
                </div>
                <div style="display:flex; justify-content:space-between;">
                    <span style="color:#94a3b8;">📶 Wi-Fi অপশন:</span>
                    <strong id="gate-wifi-status" style="color:#cbd5e1;">Checking...</strong>
                </div>
            </div>

            <!-- Wi-Fi SSID Input when Wi-Fi option is active or user wants to specify connected Wi-Fi -->
            <div class="form-group" style="text-align:left; margin-bottom:12px;">
                <label class="form-label">📶 সংযুক্ত Wi-Fi এর নাম (Connected Wi-Fi SSID — অপশনাল/পলিসি অনুযায়ী):</label>
                <input type="text" id="gate-wifi-input" class="form-input" placeholder="যেমন: Home_WiFi বা Office_WiFi" value="">
            </div>

            <div style="display: flex; flex-direction: column; gap: 9px;">
                <button class="btn btn-cyan" onclick="requestAndVerifyMobileLocation()">
                    📍 ফোনের GPS Location চালু ও যাচাই করুন (Verify Now)
                </button>
                <button class="btn btn-green" onclick="lockGeofenceToCurrentHouseLocation()">
                    🏠 আমার বর্তমান বাসার/অফিসের GPS লোকেশনটি সেট করুন
                </button>
                <a href="/" style="color: #94a3b8; font-size: 0.78rem; text-decoration: none; margin-top: 4px;">
                    ← ডেক্সটপ পোর্টাল ভিউতে ফিরে যান (Return to Desktop View)
                </a>
            </div>
        </div>
    </div>

    <!-- HEADER -->
    <header class="mobile-header">
        <div class="brand-box">
            <div class="brand-icon">📱</div>
            <div>
                <span class="brand-title">SMART ATTENDANCE v8.0</span>
                <span class="brand-sub" id="mob-header-company">Company: Smart Attendance HQ</span>
            </div>
        </div>
        <div style="display:flex; gap:6px; align-items:center;">
            <a href="/" style="background: rgba(0, 242, 254, 0.14); border: 1px solid rgba(0,242,254,0.4); color: #00f2fe; padding: 6px 9px; border-radius: 8px; font-size: 0.72rem; font-weight: 800; text-decoration: none;">
                🖥️ 3D Web
            </a>
            <a href="/api/v1/downloads/android-apk" style="background: rgba(0, 230, 118, 0.2); border: 1px solid #00e676; color: #00e676; padding: 6px 9px; border-radius: 8px; font-size: 0.72rem; font-weight: 800; text-decoration: none;">
                ⬇️ APK
            </a>
        </div>
    </header>

    <!-- LIVE GPS & WI-FI TELEMETRY RIBBON -->
    <div class="security-ribbon">
        <span id="ribbon-gps-badge" class="badge badge-err">📍 GPS: লকড</span>
        <span id="ribbon-dist-badge" class="badge badge-warn">📏 দূরত্ব: ---</span>
        <span id="ribbon-wifi-badge" class="badge badge-ok">📶 Wi-Fi: Ready</span>
    </div>

    <div class="container">
        <!-- ================================================================= -->
        <!-- TAB 1: FULL KIOSK (CAMERA + SPLIT VERIFY, PIN + FORGOT PW, QR,    -->
        <!--        9 ATTENDANCE CARDS & 7-DAY HISTORY — attendance_v8.py)     -->
        <!-- ================================================================= -->
        <section id="tab-kiosk" class="tab-view active">
            <div class="stat-grid">
                <div class="stat-box">
                    <div class="stat-val" id="mob-stat-users">0</div>
                    <div class="stat-lbl">মোট রেজিস্টার্ড ইউজার</div>
                </div>
                <div class="stat-box">
                    <div class="stat-val" id="mob-stat-present">0</div>
                    <div class="stat-lbl">আজকের উপস্থিতি</div>
                </div>
            </div>

            <!-- 1A. CAMERA ATTENDANCE + SPLIT VERIFICATION WINDOW -->
            <div class="card">
                <div class="card-title">
                    <span>📷 ১. In &amp; Out with Camera (FaceNet512 ONNX)</span>
                    <span class="badge badge-ok">512-D</span>
                </div>
                <p style="font-size: 0.76rem; color: var(--text-sub); margin-bottom: 8px;">
                    মোবাইল ক্যামেরায় মুখের ছবি তুলে FaceNet512 + MTCNN যাচাই করে উপস্থিতি দিন (GPS Location বাধ্যতামূলক):
                </p>
                <div style="background: #000; border-radius: 12px; overflow: hidden; height: 210px; position: relative; margin-bottom: 10px; border: 1px solid rgba(0,242,254,0.3);">
                    <video id="mob-video" autoplay playsinline muted style="width:100%; height:100%; object-fit:cover; transform: scaleX(-1);"></video>
                    <canvas id="mob-canvas" width="360" height="270" style="display:none;"></canvas>
                </div>
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px; margin-bottom: 8px;">
                    <button class="btn btn-outline" onclick="startMobileCamera()">🎥 ক্যামেরা চালু</button>
                    <button class="btn btn-cyan" onclick="verifyAndPreviewMobileCamera()">🔍 Verify &amp; Punch</button>
                </div>
                <button class="btn btn-green" onclick="submitMobileCameraPunch()">📸 সরাসরি ছবি তুলে উপস্থিতি দিন</button>

                <!-- Split Verification Preview Box (CAP-15 Parity) -->
                <div id="mob-verify-split-box" style="display:none; margin-top:10px; background:#090d16; border:1px solid #00e676; border-radius:10px; padding:10px;">
                    <div style="font-weight:800; color:#00e676; font-size:0.86rem; margin-bottom:6px;">
                        ✅ Identity Verification Split-Window (<span id="mob-ver-countdown">5s</span>)
                    </div>
                    <div style="font-size:0.78rem; color:#e2e8f0; margin-bottom:8px;">
                        👤 User: <b id="mob-ver-user">---</b> | Tier: <b id="mob-ver-tier">---</b><br>
                        🎯 Confidence: <b id="mob-ver-conf">---</b> | Distance: <b id="mob-ver-dist">---</b>
                    </div>
                    <div style="display:grid; grid-template-columns:1fr 1fr; gap:8px;">
                        <button class="btn btn-green" onclick="confirmVerifiedCameraPunch()">✅ হ্যাঁ, আমিই (Confirm)</button>
                        <button class="btn btn-red" onclick="cancelVerifiedCameraPunch()">🔄 না, আবার তুলুন</button>
                    </div>
                </div>
            </div>

            <!-- 1B. WITHOUT CAMERA (USERNAME + PASSWORD/PIN + FORGOT PASSWORD) -->
            <div class="card">
                <div class="card-title">
                    <span>🚪 ২. In &amp; Out without Camera (PIN / Password)</span>
                    <button onclick="toggleMobileForgotPw()" style="background:none; border:none; color:#ffab00; font-size:0.75rem; font-weight:700; cursor:pointer;">🔑 Forgot Password?</button>
                </div>
                <div class="form-group">
                    <label class="form-label">ইউজারনেম (USERNAME)</label>
                    <input type="text" id="mob-punch-user" class="form-input" placeholder="যেমন: RAHIM বা ADMIN" style="text-transform: uppercase;">
                </div>
                <div class="form-group">
                    <label class="form-label">পাসওয়ার্ড / পিন (PIN / Password)</label>
                    <input type="password" id="mob-punch-pin" class="form-input" placeholder="ডিফল্ট: admin123 বা 1234" value="1234">
                </div>
                <button class="btn btn-green" onclick="submitMobilePinPunch()">
                    ✅ মোবাইল থেকে উপস্থিতি (IN / OUT) রেকর্ড করুন
                </button>
                <div id="mob-punch-result" style="margin-top: 10px; font-size: 0.84rem; font-weight: 700; text-align: center;"></div>

                <!-- Forgot Password Reset Sub-Panel (CAP-19 Parity) -->
                <div id="mob-forgot-pw-box" style="display:none; margin-top:12px; padding-top:10px; border-top:1px solid rgba(255,255,255,0.12);">
                    <div style="font-size:0.84rem; font-weight:800; color:#ffab00; margin-bottom:8px;">🔑 Reset User Password (HR পাসওয়ার্ড বা Android ID দিয়ে)</div>
                    <div class="form-group">
                        <label class="form-label">ইউজারনেম (Username) *</label>
                        <input type="text" id="mob-fp-user" class="form-input" placeholder="যেমন: RAHIM" style="text-transform:uppercase;">
                    </div>
                    <div class="form-group">
                        <label class="form-label">HR পাসওয়ার্ড অথবা ইউজারের Android ID *</label>
                        <input type="password" id="mob-fp-auth" class="form-input" placeholder="যেমন: HR@123 বা ANDR_DEV001">
                    </div>
                    <div class="form-group">
                        <label class="form-label">নতুন পাসওয়ার্ড (New Password) *</label>
                        <input type="password" id="mob-fp-newpw" class="form-input" placeholder="নতুন পাসওয়ার্ড লিখুন">
                    </div>
                    <button class="btn btn-amber" onclick="submitMobileForgotPassword()">🔄 পাসওয়ার্ড রিসেট করুন</button>
                    <div id="mob-fp-status" style="margin-top:6px; font-size:0.8rem; font-weight:700;"></div>
                </div>
            </div>

            <!-- 1C. QR CODE IN & OUT (ONLINE + ANDROID ID) -->
            <div class="card">
                <div class="card-title">
                    <span>📲 ৩. In &amp; Out with QR Code (Online / Android ID)</span>
                </div>
                <div style="display:grid; grid-template-columns:1fr 1fr; gap:8px; margin-bottom:8px;">
                    <input type="text" id="mob-qr-android-id" class="form-input" placeholder="Android ID (যেমন: ANDR_DEV001)" value="ANDR_DEV001">
                    <button class="btn btn-purple" onclick="submitMobileQrPunch()">📲 QR / Android ID Punch</button>
                </div>
                <button class="btn btn-outline" onclick="generateMobileQrSession()">🔄 অনলাইন QR Session তৈরি করুন</button>
                <div id="mob-qr-session-box" style="display:none; margin-top:10px; text-align:center; background:#090d16; padding:10px; border-radius:10px;">
                    <div id="mob-qr-svg" style="background:#fff; padding:8px; border-radius:8px; display:inline-block; margin-bottom:6px;"></div>
                    <div id="mob-qr-session-txt" style="font-size:0.75rem; color:#00f2fe;">Waiting for scan...</div>
                </div>
            </div>

            <!-- 1D. TODAY'S ATTENDANCE — EXACT 9 KIOSK INFO CARDS + 7 DAYS HISTORY (CAP-22 & CAP-23) -->
            <div class="card">
                <div class="card-title">
                    <span>📋 TODAY'S ATTENDANCE (9 Cards)</span>
                    <button onclick="loadMobileSevenDayHistory()" style="background:#5E0037; color:#fff; border:none; padding:5px 10px; border-radius:6px; font-size:0.75rem; font-weight:800; cursor:pointer;">📈 7 Days History</button>
                </div>
                <div class="nine-cards-grid">
                    <div class="mini-att-card" style="border-left-color:#6c5ce7;">
                        <div class="mini-att-lbl">👤 User Name</div>
                        <div class="mini-att-val" id="mob-card-username">---</div>
                    </div>
                    <div class="mini-att-card" style="border-left-color:#fdcb6e;">
                        <div class="mini-att-lbl">⏱️ Record Time</div>
                        <div class="mini-att-val" id="mob-card-record-time">---</div>
                    </div>
                    <div class="mini-att-card" style="border-left-color:#00b894;">
                        <div class="mini-att-lbl">🔐 In/Out</div>
                        <div class="mini-att-val" id="mob-card-login-type">---</div>
                    </div>
                    <div class="mini-att-card" style="border-left-color:#dfe6e9;">
                        <div class="mini-att-lbl">📋 Status</div>
                        <div class="mini-att-val" id="mob-card-attendance-type">---</div>
                    </div>
                    <div class="mini-att-card" style="border-left-color:#e17055;">
                        <div class="mini-att-lbl">📅 Date</div>
                        <div class="mini-att-val" id="mob-card-date">---</div>
                    </div>
                    <div class="mini-att-card" style="border-left-color:#a29bfe;">
                        <div class="mini-att-lbl">📱 Method</div>
                        <div class="mini-att-val" id="mob-card-attendance-method">---</div>
                    </div>
                    <div class="mini-att-card" style="border-left-color:#ff7675;">
                        <div class="mini-att-lbl">⚠️ Monthly Lates</div>
                        <div class="mini-att-val" id="mob-card-monthly-lates">0</div>
                    </div>
                    <div class="mini-att-card" style="border-left-color:#d63031;">
                        <div class="mini-att-lbl">🚨 Late Absents</div>
                        <div class="mini-att-val" id="mob-card-late-absents">0</div>
                    </div>
                    <div class="mini-att-card" style="border-left-color:#fab1a0;">
                        <div class="mini-att-lbl">🏃 Early Leaves</div>
                        <div class="mini-att-val" id="mob-card-early-leaves">0</div>
                    </div>
                </div>

                <!-- 7-Day Attendance History Table -->
                <div id="mob-7day-box" style="display:none; margin-top:12px; padding-top:10px; border-top:1px solid rgba(255,255,255,0.12); overflow-x:auto;">
                    <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:6px;">
                        <strong style="font-size:0.82rem; color:#00f2fe;">📈 বিগত ৭ দিনের উপস্থিতি রেকর্ড (Last 7 Days)</strong>
                        <button onclick="document.getElementById('mob-7day-box').style.display='none'" style="background:none; border:none; color:#ff5252; font-weight:800; cursor:pointer;">✕</button>
                    </div>
                    <table class="mob-table">
                        <thead>
                            <tr><th>User</th><th>Date</th><th>In</th><th>Out</th><th>Status</th></tr>
                        </thead>
                        <tbody id="mob-7day-tbody"></tbody>
                    </table>
                </div>
            </div>
        </section>

        <!-- ================================================================= -->
        <!-- TAB 2: MILLIONS-OF-USERS SHARDED DIRECTORY (64 SHARDS + DELETE)   -->
        <!-- ================================================================= -->
        <section id="tab-millions" class="tab-view">
            <div class="card">
                <div class="card-title">👥 কোটি/লক্ষ ইউজার ম্যানেজমেন্ট (64 Physical Shards)</div>
                <p style="font-size: 0.76rem; color: var(--text-sub); margin-bottom: 10px;">
                    পিসি না থাকলেও মোবাইল থেকেই O(1) Keyset Cursor Pagination এর মাধ্যমে লক্ষ/কোটি ইউজার খুঁজুন ও পরিচালনা করুন:
                </p>
                <div style="display: grid; grid-template-columns: 2fr 1fr; gap: 8px; margin-bottom: 10px;">
                    <input type="text" id="mob-user-search" class="form-input" placeholder="ইউজারনেম বা পদবি খুঁজুন..." oninput="resetAndLoadMobileUsers()">
                    <select id="mob-shard-select" class="form-input" onchange="resetAndLoadMobileUsers()">
                        <option value="">সব ৬৪ শার্ড (All)</option>
                    </select>
                </div>
                <div id="mob-cursor-meta" style="font-size: 0.75rem; color: var(--accent-cyan); margin-bottom: 10px;"></div>
                <div id="mob-users-list" style="max-height: 380px; overflow-y: auto;"></div>
                <div style="display: flex; gap: 8px; margin-top: 10px;">
                    <button class="btn btn-outline" onclick="resetAndLoadMobileUsers()">⏮️ প্রথম পেজ</button>
                    <button class="btn btn-cyan" onclick="loadNextMobileUsersPage()">পরবর্তী পেজ (O(1) Cursor) ⏭️</button>
                </div>
            </div>
        </section>

        <!-- ================================================================= -->
        <!-- TAB 3: USER REGISTRATION (20-POSE CAMERA + NO-CAM + QR + BULK)    -->
        <!-- ================================================================= -->
        <section id="tab-provision" class="tab-view">
            <!-- 3A. 20-POSE CAMERA FACE REGISTRATION (CAP-11 Parity) -->
            <div class="card">
                <div class="card-title">
                    <span>📷 ১. ২০-পোজ ক্যামেরা ফেস রেজিস্ট্রেশন (20-Pose ONNX)</span>
                    <span class="badge badge-ok" id="mob-reg20-badge">0 / 20</span>
                </div>
                <div style="display:grid; grid-template-columns:1fr 1fr; gap:8px; margin-bottom:8px;">
                    <input type="text" id="mob-camreg-user" class="form-input" placeholder="Username (যেমন: KARIM_02)" style="text-transform:uppercase;">
                    <input type="text" id="mob-camreg-desig" class="form-input" placeholder="Designation" value="Engineer">
                </div>
                <input type="password" id="mob-camreg-pw" class="form-input" placeholder="Password" value="1234" style="margin-bottom:8px;">
                <div style="background:#000; border-radius:10px; overflow:hidden; height:180px; margin-bottom:8px; border:1px solid rgba(0,242,254,0.3);">
                    <video id="mob-reg-video" autoplay playsinline muted style="width:100%; height:100%; object-fit:cover; transform:scaleX(-1);"></video>
                </div>
                <div id="mob-reg20-prompt" style="font-size:0.78rem; color:#ffab00; font-weight:700; margin-bottom:8px;">
                    Pose 1/20: ক্যামেরা চালু করে ছবি তুলুন বা Auto Capture চাপুন
                </div>
                <div style="display:grid; grid-template-columns:1fr 1fr; gap:8px; margin-bottom:8px;">
                    <button class="btn btn-outline" onclick="startMobileRegCamera()">🎥 ক্যামেরা চালু</button>
                    <button class="btn btn-cyan" onclick="captureNextMobilePose()">📸 পরের পোজ তুলুন</button>
                </div>
                <div style="display:grid; grid-template-columns:1fr 1fr; gap:8px;">
                    <button class="btn btn-purple" onclick="autoCaptureAll20MobilePoses()">⚡ Auto 20-Pose Capture</button>
                    <button class="btn btn-green" onclick="submitMobile20PoseRegistration()">✅ ২০-পোজ সেভ করুন</button>
                </div>
                <div id="mob-camreg-status" style="margin-top:8px; font-size:0.8rem; font-weight:700;"></div>
            </div>

            <!-- 3B. WITHOUT CAMERA REGISTRATION (CAP-17 Parity) -->
            <div class="card">
                <div class="card-title">👤 ২. ক্যামেরা ছাড়া নতুন ইউজার রেজিস্ট্রেশন (512-Zero Vector)</div>
                <div class="form-group">
                    <label class="form-label">ইউজারনেম (Username) *</label>
                    <input type="text" id="mob-reg-username" class="form-input" placeholder="যেমন: TANVIR_01" style="text-transform: uppercase;">
                </div>
                <div class="form-group">
                    <label class="form-label">পদবি (Designation)</label>
                    <input type="text" id="mob-reg-desig" class="form-input" placeholder="যেমন: Senior Engineer" value="Staff">
                </div>
                <div class="form-group">
                    <label class="form-label">পাসওয়ার্ড (Password)</label>
                    <input type="password" id="mob-reg-pw" class="form-input" value="1234">
                </div>
                <button class="btn btn-green" onclick="registerSingleUserFromMobile()">
                    👤 ইউজার রেজিস্টার করুন (No Camera)
                </button>
                <div id="mob-reg-status" style="margin-top:8px; font-size:0.82rem; font-weight:700;"></div>
            </div>

            <!-- 3C. QR CODE / ANDROID ID USER REGISTRATION (CAP-20 Parity) -->
            <div class="card">
                <div class="card-title">📲 ৩. QR Code / Android ID সহ ইউজার রেজিস্ট্রেশন</div>
                <div style="display:grid; grid-template-columns:1fr 1fr; gap:8px; margin-bottom:8px;">
                    <input type="text" id="mob-qrreg-user" class="form-input" placeholder="Username (যেমন: FIELD_01)" style="text-transform:uppercase;">
                    <input type="text" id="mob-qrreg-aid" class="form-input" placeholder="Android ID (যেমন: ANDR_991)">
                </div>
                <div style="display:grid; grid-template-columns:1fr 1fr; gap:8px; margin-bottom:8px;">
                    <input type="text" id="mob-qrreg-desig" class="form-input" value="Field Executive">
                    <input type="password" id="mob-qrreg-pw" class="form-input" value="1234">
                </div>
                <button class="btn btn-cyan" onclick="registerQrUserFromMobile()">📲 QR / Android ID ইউজার সেভ করুন</button>
                <div id="mob-qrreg-status" style="margin-top:8px; font-size:0.8rem; font-weight:700;"></div>
            </div>

            <!-- 3D. BULK SHARDED PROVISIONING -->
            <div class="card">
                <div class="card-title">⚡ ৪. বাল্ক ইউজার জেনারেটর (64-Shard Bulk Provisioning)</div>
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px; margin-bottom: 10px;">
                    <div>
                        <label class="form-label">প্রিফিক্স (Prefix)</label>
                        <input type="text" id="mob-bulk-prefix" class="form-input" value="OFFICE_EMP">
                    </div>
                    <div>
                        <label class="form-label">সংখ্যা (Batch Count)</label>
                        <select id="mob-bulk-count" class="form-input">
                            <option value="50">50 Users</option>
                            <option value="250">250 Users</option>
                            <option value="1000">1,000 Users</option>
                            <option value="5000">5,000 Users</option>
                        </select>
                    </div>
                </div>
                <button class="btn btn-purple" onclick="bulkProvisionFromMobile()">
                    🚀 ৬৪ শার্ডে বাল্ক ইউজার তৈরি করুন
                </button>
                <div id="mob-bulk-status" style="margin-top:8px; font-size:0.82rem; font-weight:700;"></div>
            </div>
        </section>

        <!-- ================================================================= -->
        <!-- TAB 4: HR MANAGEMENT (ALL 12 FUNCTIONS) & COMPANY INFO            -->
        <!-- ================================================================= -->
        <section id="tab-hr" class="tab-view">
            <!-- 4A. COMPANY INFO (SELECT / CREATE 7-FIELD / EDIT — CAP-05,06,07) -->
            <div class="card">
                <div class="card-title">
                    <span>🏢 Company Management (Select / Create / Edit)</span>
                </div>
                <div style="font-size:0.78rem; color:#00e676; margin-bottom:8px;">
                    সক্রিয় কোম্পানি: <b id="mob-comp-active-lbl">Smart Attendance HQ (HQ_001)</b>
                </div>
                <div style="display:grid; grid-template-columns:1.3fr 1fr auto; gap:6px; margin-bottom:10px;">
                    <select id="mob-comp-select" class="form-input"></select>
                    <input type="password" id="mob-comp-verify-pw" class="form-input" placeholder="Password (admin123)" value="admin123">
                    <button class="btn btn-cyan" style="width:auto; padding:0 12px;" onclick="switchMobileCompany()">Switch</button>
                </div>
                <details style="background:#090d16; padding:10px; border-radius:10px; border:1px solid rgba(255,255,255,0.1);">
                    <summary style="cursor:pointer; font-weight:800; font-size:0.82rem; color:#00f2fe;">➕ নতুন কোম্পানি তৈরি বা এডিট করুন (7 Fields)</summary>
                    <div style="margin-top:10px; display:grid; grid-template-columns:1fr 1fr; gap:8px;">
                        <input type="text" id="mob-c-name" class="form-input" placeholder="Company Name *">
                        <input type="text" id="mob-c-id" class="form-input" placeholder="Company ID (e.g. BR_02) *">
                        <input type="text" id="mob-c-addr" class="form-input" placeholder="Company Address *">
                        <input type="text" id="mob-c-person" class="form-input" placeholder="Contact Person Name *">
                        <input type="text" id="mob-c-desig" class="form-input" placeholder="Designation *">
                        <input type="text" id="mob-c-phone" class="form-input" placeholder="Phone Number *">
                    </div>
                    <input type="password" id="mob-c-pw" class="form-input" placeholder="Company Password *" style="margin-top:8px;">
                    <div style="display:grid; grid-template-columns:1fr 1fr; gap:8px; margin-top:8px;">
                        <button class="btn btn-green" onclick="createMobileCompany()">➕ Create Company</button>
                        <button class="btn btn-amber" onclick="editMobileCompany()">✏️ Update Company</button>
                    </div>
                </details>
                <div id="mob-comp-status" style="margin-top:6px; font-size:0.78rem; font-weight:700;"></div>
            </div>

            <!-- 4B. HR AUTHENTICATION & ALL 12 HR TOOLS (CAP-24..34) -->
            <div class="card">
                <div class="card-title">
                    <span>👔 HR Management Portal (attendance_v8.py All 12 Tools)</span>
                    <span id="mob-hr-lock-badge" class="badge badge-warn">🔒 Locked</span>
                </div>
                <div id="mob-hr-login-box">
                    <div style="display:grid; grid-template-columns:1fr auto; gap:8px; margin-bottom:8px;">
                        <input type="password" id="mob-hr-pw" class="form-input" placeholder="HR Password (ডিফল্ট: HR@123)" value="HR@123">
                        <button class="btn btn-cyan" style="width:auto; padding:0 16px;" onclick="unlockMobileHrPortal()">🔓 Unlock HR</button>
                    </div>
                </div>

                <div id="mob-hr-tools-box" style="display:none;">
                    <!-- Customized Reports (CSV, XLSX, HTML Muster Roll) -->
                    <div style="background:#090d16; padding:10px; border-radius:10px; margin-bottom:10px; border:1px solid rgba(0,242,254,0.2);">
                        <div style="font-weight:800; font-size:0.82rem; color:#00f2fe; margin-bottom:8px;">📊 Customized Attendance Reports (CSV / Excel .xlsx / Muster Roll)</div>
                        <div style="display:grid; grid-template-columns:repeat(3, 1fr); gap:6px;">
                            <a href="/api/v1/reports/customized.csv" target="_blank" class="btn btn-cyan" style="font-size:0.75rem; padding:8px;">⬇️ CSV</a>
                            <a href="/api/v1/reports/customized.xlsx" target="_blank" class="btn btn-green" style="font-size:0.75rem; padding:8px;">📗 Excel .xlsx</a>
                            <a href="/api/v1/reports/muster-roll/html" target="_blank" class="btn btn-purple" style="font-size:0.75rem; padding:8px;">📄 Muster Roll</a>
                        </div>
                    </div>

                    <!-- Embeddings Report & FAISS Index Inspect -->
                    <div style="background:#090d16; padding:10px; border-radius:10px; margin-bottom:10px; border:1px solid rgba(124,77,255,0.3);">
                        <div style="font-weight:800; font-size:0.82rem; color:#b388ff; margin-bottom:8px;">🧬 12-Col Facial Embeddings Report &amp; FAISS Index</div>
                        <div style="display:grid; grid-template-columns:1fr 1fr; gap:6px; margin-bottom:6px;">
                            <button class="btn btn-outline" style="font-size:0.75rem; padding:8px;" onclick="inspectMobileEmbeddingsReport()">🔍 View 12-Col Report</button>
                            <button class="btn btn-outline" style="font-size:0.75rem; padding:8px;" onclick="inspectMobileFakeAttendance()">🕵️ Check Fake Attendance</button>
                        </div>
                        <div style="display:grid; grid-template-columns:repeat(3, 1fr); gap:6px;">
                            <a href="/api/v1/face/embeddings-report.csv" target="_blank" class="btn btn-outline" style="font-size:0.7rem; padding:6px;">📄 Analysis CSV</a>
                            <a href="/api/v1/face/embeddings-summary.csv" target="_blank" class="btn btn-outline" style="font-size:0.7rem; padding:6px;">📄 Summary CSV</a>
                            <a href="/api/v1/face/embeddings-vectors.csv" target="_blank" class="btn btn-outline" style="font-size:0.7rem; padding:6px;">📄 512-D Vectors</a>
                        </div>
                    </div>

                    <!-- Sync Local / Cloud / HR Password / Delete Face -->
                    <div style="display:grid; grid-template-columns:1fr 1fr; gap:8px; margin-bottom:10px;">
                        <button class="btn btn-outline" style="font-size:0.76rem;" onclick="runMobileHrSync('/api/v1/sync/users-to-local', 'Local SQLite & FAISS Sync')">🔄 Sync Users to Local</button>
                        <button class="btn btn-outline" style="font-size:0.76rem;" onclick="runMobileHrSync('/api/v1/sync/users-to-cloud', 'Cloud Outbox Sync')">☁️ Sync Users to Cloud</button>
                        <button class="btn btn-outline" style="font-size:0.76rem;" onclick="runMobileHrSync('/api/v1/auth/sync-hr-password', 'HR Password Sync')">🔑 Sync HR Password</button>
                        <button class="btn btn-red" style="font-size:0.76rem;" onclick="deleteMobileUserFacePrompt()">🗑️ Delete Face Data</button>
                    </div>

                    <!-- Change HR Password -->
                    <details style="background:#090d16; padding:10px; border-radius:10px; border:1px solid rgba(255,255,255,0.1);">
                        <summary style="cursor:pointer; font-weight:800; font-size:0.8rem; color:#ffab00;">🔑 Change HR Password</summary>
                        <div style="display:grid; grid-template-columns:1fr 1fr auto; gap:6px; margin-top:8px;">
                            <input type="password" id="mob-hrcp-old" class="form-input" placeholder="Current HR PW">
                            <input type="password" id="mob-hrcp-new" class="form-input" placeholder="New HR PW">
                            <button class="btn btn-amber" style="width:auto; padding:0 12px;" onclick="changeMobileHrPassword()">Save</button>
                        </div>
                    </details>

                    <div id="mob-hr-output" style="margin-top:10px; font-size:0.78rem; color:#00e676; background:#090d16; padding:10px; border-radius:8px; display:none; max-height:240px; overflow-y:auto;"></div>
                </div>
            </div>
        </section>

        <!-- ================================================================= -->
        <!-- TAB 5: MANDATORY LOCATION & WI-FI POLICY + ALL PLATFORM DOWNLOADS -->
        <!-- ================================================================= -->
        <section id="tab-policy" class="tab-view">
            <div class="card">
                <div class="card-title">📍 বাধ্যতামূলক লোকেশন ও Wi-Fi পলিসি সেটিংস</div>
                <div style="background: rgba(0, 230, 118, 0.12); border: 1px solid rgba(0, 230, 118, 0.4); padding: 10px; border-radius: 10px; font-size: 0.78rem; margin-bottom: 12px;">
                    🔒 <b>বাধ্যতামূলক নিয়ম:</b> মোবাইল অ্যাপে <b>GPS Location</b> সর্বদা বাধ্যতামূলক (Mandatory)। লোকেশন ছাড়া অ্যাপ কখনোই কাজ করবে না। পাশাপাশি আপনি নিচের <b>Wi-Fi অপশনটি</b> চালু বা কাস্টমাইজ করতে পারবেন।
                </div>

                <div class="form-group">
                    <label class="form-label">🏠 নির্ধারিত বাসা বা অফিসের নাম (Location Name)</label>
                    <input type="text" id="pol-loc-name" class="form-input" value="Home / Office Authorized Geofence">
                </div>
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px;">
                    <div class="form-group">
                        <label class="form-label">Latitude (অক্ষাংশ) *</label>
                        <input type="number" step="any" id="pol-lat" class="form-input" value="23.8103">
                    </div>
                    <div class="form-group">
                        <label class="form-label">Longitude (দ্রাঘিমাংশ) *</label>
                        <input type="number" step="any" id="pol-lon" class="form-input" value="90.4125">
                    </div>
                </div>
                <div class="form-group">
                    <label class="form-label">📏 অনুমোদিত সীমানা ব্যাসার্ধ (Allowed Radius in Meters)</label>
                    <input type="number" id="pol-radius" class="form-input" value="150">
                </div>

                <button class="btn btn-outline" style="margin-bottom: 12px;" onclick="fillCurrentGpsIntoPolicyForm()">
                    📍 আমার ফোনের বর্তমান GPS স্থানাঙ্ক বসিয়ে নিন
                </button>

                <hr style="border: none; border-top: 1px solid rgba(255,255,255,0.1); margin: 12px 0;">

                <div class="form-group" style="display: flex; align-items: center; justify-content: space-between; background: rgba(124, 77, 255, 0.12); padding: 10px 12px; border-radius: 10px; border: 1px solid rgba(124, 77, 255, 0.35);">
                    <div>
                        <div style="font-weight: 800; font-size: 0.84rem; color: #fff;">📶 নির্ধারিত Wi-Fi অপশন (Wi-Fi Enforcement)</div>
                        <div style="font-size: 0.72rem; color: #cbd5e1;">চালু করলে নির্ধারিত লোকেশন + নির্ধারিত Wi-Fi দুটোই লাগবে</div>
                    </div>
                    <input type="checkbox" id="pol-wifi-enabled" style="width: 22px; height: 22px; accent-color: #00e676;">
                </div>

                <div class="form-group">
                    <label class="form-label">📶 অনুমোদিত Wi-Fi এর নামসমূহ (Allowed Wi-Fi SSIDs, কমা দিয়ে)</label>
                    <input type="text" id="pol-wifi-ssids" class="form-input" value="Office_WiFi,Home_WiFi,SmartAttendance_5G">
                </div>

                <button class="btn btn-green" onclick="saveMobilePolicySettings()">
                    💾 লোকেশন ও Wi-Fi পলিসি সেভ করুন
                </button>
                <div id="pol-save-status" style="margin-top: 8px; font-size: 0.82rem; font-weight: 700; text-align: center;"></div>
            </div>

            <!-- ALL PLATFORM DOWNLOAD LINKS (WINDOWS, LINUX, MAC, APK) -->
            <div class="card">
                <div class="card-title">💻 সকল প্ল্যাটফর্ম ডাউনলোড (Windows / Linux / Mac / APK)</div>
                <div style="display:grid; grid-template-columns:1fr 1fr; gap:8px;">
                    <a href="/api/v1/downloads/windows-installer" class="btn btn-cyan" style="font-size:0.78rem;">🪟 Windows Setup (.exe)</a>
                    <a href="/api/v1/downloads/windows-portable" class="btn btn-outline" style="font-size:0.78rem;">📦 Windows Portable (.zip)</a>
                    <a href="/api/v1/downloads/linux-desktop" class="btn btn-green" style="font-size:0.78rem;">🐧 Linux Desktop (.tar.gz)</a>
                    <a href="/api/v1/downloads/macos-desktop" class="btn btn-purple" style="font-size:0.78rem;">🍎 macOS Universal (.zip)</a>
                </div>
                <a href="/api/v1/downloads/android-apk" class="btn btn-amber" style="margin-top:8px; font-size:0.84rem;">
                    🤖 Download Android Mobile App (.apk)
                </a>
            </div>
        </section>
    </div>

    <!-- BOTTOM NAVIGATION BAR (5 TABS) -->
    <nav class="bottom-nav">
        <button class="nav-btn active" id="nav-kiosk" onclick="switchMobileTab('kiosk')">
            <span class="icon">📷</span>
            <span>কিয়স্ক (9-Card)</span>
        </button>
        <button class="nav-btn" id="nav-millions" onclick="switchMobileTab('millions')">
            <span class="icon">👥</span>
            <span>লক্ষ ইউজার</span>
        </button>
        <button class="nav-btn" id="nav-provision" onclick="switchMobileTab('provision')">
            <span class="icon">➕</span>
            <span>রেজিস্ট্রেশন</span>
        </button>
        <button class="nav-btn" id="nav-hr" onclick="switchMobileTab('hr')">
            <span class="icon">👔</span>
            <span>HR ও কোম্পানি</span>
        </button>
        <button class="nav-btn" id="nav-policy" onclick="switchMobileTab('policy')">
            <span class="icon">📍</span>
            <span>পলিসি ও অ্যাপস</span>
        </button>
    </nav>

    <script>
        let mobileState = {
            lat: null,
            lon: null,
            accuracy: null,
            wifiSsid: "",
            wifiBssid: "",
            verified: false,
            policy: null,
            nextCursorUsername: "",
            stream: null,
            regStream: null,
            watchId: null,
            activeCompanyId: "HQ_001",
            activeCompanyName: "Smart Attendance HQ",
            pendingVerifiedUser: null,
            pendingVerifiedB64: null,
            verifyTimer: null,
            qrPollTimer: null,
            regPosesB64: []
        };

        // Populate 64 shards dropdown
        const shardSel = document.getElementById('mob-shard-select');
        for (let i = 0; i < 64; i++) {
            const opt = document.createElement('option');
            opt.value = String(i);
            opt.textContent = `Shard #${String(i).padStart(2, '0')}`;
            shardSel.appendChild(opt);
        }

        async function loadPolicyFromServer() {
            try {
                const r = await fetch('/api/v1/mobile/policy');
                const p = await r.json();
                mobileState.policy = p;
                document.getElementById('gate-target-name').textContent = p.location_name;
                document.getElementById('gate-target-coords').textContent = `${Number(p.target_latitude).toFixed(4)}, ${Number(p.target_longitude).toFixed(4)} (${p.allowed_radius_meters}m)`;
                document.getElementById('gate-wifi-status').textContent = p.wifi_option_enabled
                    ? `বাধ্যতামূলক (${p.allowed_wifi_ssids})`
                    : `ঐচ্ছিক (${p.allowed_wifi_ssids})`;

                document.getElementById('pol-loc-name').value = p.location_name || '';
                document.getElementById('pol-lat').value = p.target_latitude;
                document.getElementById('pol-lon').value = p.target_longitude;
                document.getElementById('pol-radius').value = p.allowed_radius_meters;
                document.getElementById('pol-wifi-enabled').checked = !!p.wifi_option_enabled;
                document.getElementById('pol-wifi-ssids').value = p.allowed_wifi_ssids || '';
            } catch (e) {
                console.error('Failed to load mobile policy:', e);
            }
        }

        async function loadMobileCompanies() {
            try {
                const rp = await fetch('/api/v1/companies/profile');
                if (rp.ok) {
                    const prof = await rp.json();
                    if (prof.company_id) {
                        mobileState.activeCompanyId = prof.company_id;
                        mobileState.activeCompanyName = prof.company_name || prof.company_id;
                        document.getElementById('mob-header-company').textContent = `Company: ${mobileState.activeCompanyName}`;
                        document.getElementById('mob-comp-active-lbl').textContent = `${mobileState.activeCompanyName} (${mobileState.activeCompanyId})`;
                        document.getElementById('mob-c-name').value = prof.company_name || '';
                        document.getElementById('mob-c-id').value = prof.company_id || '';
                        document.getElementById('mob-c-addr').value = prof.address_road || '';
                        document.getElementById('mob-c-person').value = prof.contact_person_name || '';
                        document.getElementById('mob-c-desig').value = prof.contact_person_designation || '';
                        document.getElementById('mob-c-phone').value = prof.contact_person_number || '';
                    }
                }
                const rl = await fetch('/api/v1/companies/list');
                if (rl.ok) {
                    const list = await rl.json();
                    const sel = document.getElementById('mob-comp-select');
                    if (sel && Array.isArray(list)) {
                        sel.innerHTML = list.map(c => `<option value="${c.company_id}" ${c.company_id === mobileState.activeCompanyId ? 'selected' : ''}>${c.company_name} (${c.company_id})</option>`).join('');
                    }
                }
            } catch (_) {}
        }

        function detectNativeAndroidBridge() {
            if (window.AndroidNativeBridge) {
                try {
                    const wifiJson = JSON.parse(window.AndroidNativeBridge.getWifiInfoJson());
                    const rawSsid = (wifiJson && wifiJson.ssid) ? String(wifiJson.ssid).trim() : "";
                    const cleanSsid = (rawSsid.toLowerCase() === "<unknown ssid>") ? "" : rawSsid;
                    const rawBssid = (wifiJson && wifiJson.bssid) ? String(wifiJson.bssid).trim() : "";
                    const cleanBssid = (rawBssid === "02:00:00:00:00:00") ? "" : rawBssid;

                    document.getElementById('gate-wifi-input').value = cleanSsid;
                    mobileState.wifiSsid = cleanSsid;
                    mobileState.wifiBssid = cleanBssid;

                    const locJson = JSON.parse(window.AndroidNativeBridge.getHardwareLocationJson());
                    if (locJson && locJson.gps_enabled && locJson.has_fix && locJson.latitude && locJson.longitude) {
                        mobileState.lat = Number(locJson.latitude);
                        mobileState.lon = Number(locJson.longitude);
                        mobileState.accuracy = Number(locJson.accuracy || 15);
                        return true;
                    } else {
                        mobileState.lat = null;
                        mobileState.lon = null;
                        mobileState.accuracy = null;
                        return false;
                    }
                } catch (e) {
                    console.warn('Android bridge probe error:', e);
                }
            }
            return false;
        }

        function startContinuousGpsWatch() {
            if (mobileState.watchId !== null || !navigator.geolocation) return;
            mobileState.watchId = navigator.geolocation.watchPosition(
                async (pos) => {
                    mobileState.lat = pos.coords.latitude;
                    mobileState.lon = pos.coords.longitude;
                    mobileState.accuracy = pos.coords.accuracy;
                    if (mobileState.verified) {
                        await verifyWithBackend();
                    }
                },
                () => {
                    if (!detectNativeAndroidBridge()) {
                        mobileState.lat = null;
                        mobileState.lon = null;
                        mobileState.accuracy = null;
                        showGateLocked('GPS Location বন্ধ হয়ে গেছে! লোকেশন ছাড়া মোবাইল অ্যাপ কাজ করবে না।');
                    }
                },
                { enableHighAccuracy: true, maximumAge: 5000, timeout: 10000 }
            );
        }

        async function requestAndVerifyMobileLocation() {
            mobileState.wifiSsid = (document.getElementById('gate-wifi-input').value || '').trim();
            document.getElementById('gate-current-coords').textContent = 'GPS সিগন্যাল খোঁজা হচ্ছে...';

            if (detectNativeAndroidBridge() && mobileState.lat !== null && mobileState.lon !== null) {
                await verifyWithBackend();
                return;
            }

            if (!navigator.geolocation) {
                showGateLocked('আপনার ব্রাউজারে Geolocation সাপোর্ট নেই! লোকেশন ছাড়া মোবাইল অ্যাপ কাজ করবে না।');
                return;
            }

            navigator.geolocation.getCurrentPosition(
                async (pos) => {
                    mobileState.lat = pos.coords.latitude;
                    mobileState.lon = pos.coords.longitude;
                    mobileState.accuracy = pos.coords.accuracy;
                    startContinuousGpsWatch();
                    await verifyWithBackend();
                },
                () => {
                    mobileState.lat = null;
                    mobileState.lon = null;
                    mobileState.accuracy = null;
                    showGateLocked('লোকেশন পারমিশন পাওয়া যায়নি বা GPS বন্ধ! লোকেশন ছাড়া মোবাইল অ্যাপ কাজ করবে না। অনুগ্রহ করে GPS চালু করুন।');
                },
                { enableHighAccuracy: true, timeout: 10000, maximumAge: 0 }
            );
        }

        async function verifyWithBackend() {
            try {
                if (!window.AndroidNativeBridge) {
                    mobileState.wifiSsid = (document.getElementById('gate-wifi-input').value || '').trim();
                }
                const resp = await fetch('/api/v1/mobile/verify-access', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        latitude: mobileState.lat,
                        longitude: mobileState.lon,
                        gps_accuracy_meters: mobileState.accuracy,
                        wifi_ssid: mobileState.wifiSsid,
                        wifi_bssid: mobileState.wifiBssid,
                        device_id: 'MOBILE_WEB_OR_APK'
                    })
                });
                const res = await resp.json();
                if (mobileState.lat !== null && mobileState.lon !== null) {
                    document.getElementById('gate-current-coords').textContent = `${mobileState.lat.toFixed(5)}, ${mobileState.lon.toFixed(5)}`;
                } else {
                    document.getElementById('gate-current-coords').textContent = 'GPS বন্ধ / পাওয়া যায়নি';
                }
                if (res.distance_meters !== null && res.distance_meters !== undefined) {
                    document.getElementById('gate-distance').textContent = `${Number(res.distance_meters).toFixed(1)} মিটার (অনুমোদিত: ${res.allowed_radius_meters}m)`;
                    document.getElementById('ribbon-dist-badge').textContent = `📏 ${Number(res.distance_meters).toFixed(0)}m`;
                }

                if (res.allowed) {
                    mobileState.verified = true;
                    document.getElementById('mobile-location-gate').style.display = 'none';
                    document.getElementById('ribbon-gps-badge').className = 'badge badge-ok';
                    document.getElementById('ribbon-gps-badge').textContent = `📍 GPS যাচাইকৃত (${mobileState.lat.toFixed(3)}, ${mobileState.lon.toFixed(3)})`;
                    document.getElementById('ribbon-wifi-badge').className = 'badge badge-ok';
                    document.getElementById('ribbon-wifi-badge').textContent = `📶 Wi-Fi: ${mobileState.wifiSsid || 'Optional OK'}`;
                    loadMobileDashboardStats();
                    resetAndLoadMobileUsers();
                    loadMobileCompanies();
                } else {
                    showGateLocked(res.reason || 'লোকেশন বা Wi-Fi যাচাই ব্যর্থ হয়েছে!');
                }
            } catch (e) {
                showGateLocked('সার্ভারের সাথে সংযোগ করা যায়নি: ' + e.message);
            }
        }

        function showGateLocked(reasonText) {
            mobileState.verified = false;
            document.getElementById('mobile-location-gate').style.display = 'flex';
            document.getElementById('gate-desc').innerHTML = `<span style="color:#ffab00; font-weight:700;">${reasonText}</span>`;
            document.getElementById('ribbon-gps-badge').className = 'badge badge-err';
            document.getElementById('ribbon-gps-badge').textContent = '📍 GPS: লকড';
        }

        async function lockGeofenceToCurrentHouseLocation() {
            if (detectNativeAndroidBridge() && mobileState.lat !== null && mobileState.lon !== null) {
                await savePolicyCoords(mobileState.lat, mobileState.lon, 'আমার নির্ধারিত বাসা / অফিস লোকেশন');
                return;
            }
            if (!navigator.geolocation) {
                showGateLocked('আপনার ডিভাইসে Geolocation সাপোর্ট নেই! লোকেশন ছাড়া মোবাইল অ্যাপ কাজ করবে না।');
                return;
            }
            navigator.geolocation.getCurrentPosition(
                async (pos) => {
                    mobileState.lat = pos.coords.latitude;
                    mobileState.lon = pos.coords.longitude;
                    mobileState.accuracy = pos.coords.accuracy;
                    mobileState.wifiSsid = (document.getElementById('gate-wifi-input').value || '').trim();
                    startContinuousGpsWatch();
                    await savePolicyCoords(mobileState.lat, mobileState.lon, 'আমার নির্ধারিত বাসা / অফিস লোকেশন');
                },
                () => {
                    mobileState.lat = null;
                    mobileState.lon = null;
                    mobileState.accuracy = null;
                    showGateLocked('লোকেশন ছাড়া বাসার/অফিসের লোকেশন সেট বা অ্যাপ আনলক করা যাবে না! অনুগ্রহ করে আগে ফোনের GPS Location চালু ও Allow করুন।');
                },
                { enableHighAccuracy: true, timeout: 8000, maximumAge: 0 }
            );
        }

        async function savePolicyCoords(lat, lon, locName) {
            const resp = await fetch('/api/v1/mobile/policy', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    location_name: locName,
                    target_latitude: lat,
                    target_longitude: lon,
                    allowed_radius_meters: 200.0
                })
            });
            if (resp.ok) {
                await loadPolicyFromServer();
                await verifyWithBackend();
            }
        }

        function switchMobileTab(tab) {
            if (!mobileState.verified || mobileState.lat === null || mobileState.lon === null) {
                showGateLocked('লোকেশন ছাড়া মোবাইল অ্যাপ কাজ করবে না! অনুগ্রহ করে আগে লোকেশন যাচাই করুন।');
                return;
            }
            document.querySelectorAll('.tab-view').forEach(el => el.classList.remove('active'));
            document.querySelectorAll('.nav-btn').forEach(el => el.classList.remove('active'));
            document.getElementById('tab-' + tab).classList.add('active');
            document.getElementById('nav-' + tab).classList.add('active');
        }

        async function updateMobileNineCards(punchRes) {
            if (!punchRes) return;
            const uname = punchRes.username || '---';
            document.getElementById('mob-card-username').textContent = uname;
            document.getElementById('mob-card-record-time').textContent = punchRes.timestamp || punchRes.time || '---';
            document.getElementById('mob-card-login-type').textContent = punchRes.login_type || punchRes.status || '---';
            document.getElementById('mob-card-attendance-type').textContent = punchRes.attendance_type || punchRes.late_status || 'Present';
            document.getElementById('mob-card-date').textContent = punchRes.date || new Date().toISOString().slice(0, 10);
            document.getElementById('mob-card-attendance-method').textContent = punchRes.method || 'MOBILE_GPS';
            try {
                const stResp = await fetch(`/api/v1/attendance/user-stats/${encodeURIComponent(uname)}`);
                if (stResp.ok) {
                    const st = await stResp.json();
                    document.getElementById('mob-card-monthly-lates').textContent = st.total_lates ?? 0;
                    document.getElementById('mob-card-late-absents').textContent = st.late_absents ?? 0;
                    document.getElementById('mob-card-early-leaves').textContent = st.early_leaves ?? 0;
                }
            } catch (_) {}
        }

        async function loadMobileSevenDayHistory() {
            const box = document.getElementById('mob-7day-box');
            const tbody = document.getElementById('mob-7day-tbody');
            box.style.display = 'block';
            tbody.innerHTML = '<tr><td colspan="5">Loading 7-day history...</td></tr>';
            try {
                const r = await fetch('/api/v1/attendance/history-7days');
                const d = await r.json();
                const rows = d.history || d.records || (Array.isArray(d) ? d : []);
                if (!rows.length) {
                    tbody.innerHTML = '<tr><td colspan="5">কোনো রেকর্ড পাওয়া যায়নি</td></tr>';
                    return;
                }
                tbody.innerHTML = rows.slice(0, 30).map(item => `
                    <tr>
                        <td><b>${item.username || '---'}</b></td>
                        <td>${item.date || '---'}</td>
                        <td>${item.in_time || '---'}</td>
                        <td>${item.out_time || '---'}</td>
                        <td style="color:#00e676;">${item.status || item.attendance_type || 'Present'}</td>
                    </tr>
                `).join('');
            } catch (e) {
                tbody.innerHTML = `<tr><td colspan="5">Error: ${e.message}</td></tr>`;
            }
        }

        async function loadMobileDashboardStats() {
            if (!mobileState.verified) return;
            try {
                const url = `/api/v1/mobile/scale-dashboard?latitude=${mobileState.lat}&longitude=${mobileState.lon}&wifi_ssid=${encodeURIComponent(mobileState.wifiSsid)}&wifi_bssid=${encodeURIComponent(mobileState.wifiBssid)}`;
                const r = await fetch(url);
                if (!r.ok) {
                    const errData = await r.json().catch(() => ({}));
                    showGateLocked(errData.error || 'লোকেশন যাচাই ব্যর্থ হয়েছে!');
                    return;
                }
                const d = await r.json();
                if (d.daily_summary) {
                    document.getElementById('mob-stat-users').textContent = d.daily_summary.total_employees;
                    document.getElementById('mob-stat-present').textContent = d.daily_summary.present_today;
                }
            } catch (e) {}
        }

        async function resetAndLoadMobileUsers() {
            mobileState.nextCursorUsername = "";
            await loadNextMobileUsersPage();
        }

        async function loadNextMobileUsersPage() {
            if (!mobileState.verified) return;
            const search = document.getElementById('mob-user-search').value.trim();
            const shard = document.getElementById('mob-shard-select').value;
            let url = `/api/v1/mobile/users-cursor?latitude=${mobileState.lat}&longitude=${mobileState.lon}&wifi_ssid=${encodeURIComponent(mobileState.wifiSsid)}&wifi_bssid=${encodeURIComponent(mobileState.wifiBssid)}&limit=25`;
            if (search) url += `&search=${encodeURIComponent(search)}`;
            if (shard !== "") url += `&shard_bucket=${shard}`;
            if (mobileState.nextCursorUsername) url += `&after_username=${encodeURIComponent(mobileState.nextCursorUsername)}`;

            const r = await fetch(url);
            const d = await r.json();
            if (!r.ok) {
                showGateLocked(d.error || 'অ্যাক্সেস রিজেক্টেড');
                return;
            }
            document.getElementById('mob-stat-users').textContent = d.total_registered_users || 0;
            document.getElementById('mob-cursor-meta').textContent =
                `মোট ইউজার: ${d.total_registered_users} | কুয়েরি সময়: ${d.execution_time_us} µs | 64-Shard O(1) Cursor`;
            mobileState.nextCursorUsername = d.next_cursor_username || "";

            const box = document.getElementById('mob-users-list');
            const users = d.users || [];
            if (users.length === 0) {
                box.innerHTML = `<div style="text-align:center; color:#94a3b8; padding:18px;">কোনো ইউজার পাওয়া যায়নি</div>`;
                return;
            }
            box.innerHTML = users.map(u => `
                <div class="user-row">
                    <div>
                        <div style="font-weight:800; color:#fff; font-size:0.88rem;">👤 ${u.username}</div>
                        <div style="font-size:0.72rem; color:#94a3b8;">${u.user_designation || 'Employee'} • ${u.company_name || 'HQ'} • Cam: ${u.register_with_camera || 'N'}</div>
                    </div>
                    <div style="display:flex; align-items:center; gap:6px;">
                        <span class="badge badge-ok">Shard #${String(u.shard_bucket || 0).padStart(2, '0')}</span>
                        <button onclick="deleteMobileUser('${u.username}')" style="background:rgba(255,82,82,0.2); border:1px solid #ff5252; color:#ff5252; border-radius:6px; padding:4px 8px; font-size:0.72rem; font-weight:800; cursor:pointer;">🗑️</button>
                    </div>
                </div>
            `).join('');
        }

        async function deleteMobileUser(username) {
            if (!confirm(`আপনি কি নিশ্চিতভাবে ইউজার '${username}' মুছে ফেলতে চান?`)) return;
            const r = await fetch(`/api/v1/users/${encodeURIComponent(username)}`, { method: 'DELETE' });
            if (r.ok) {
                resetAndLoadMobileUsers();
                loadMobileDashboardStats();
            }
        }

        async function submitMobilePinPunch() {
            const uname = document.getElementById('mob-punch-user').value.trim().toUpperCase();
            const pin = document.getElementById('mob-punch-pin').value.trim();
            const resBox = document.getElementById('mob-punch-result');
            if (!uname) {
                resBox.style.color = '#ff5252';
                resBox.textContent = 'অনুগ্রহ করে ইউজারনেম লিখুন!';
                return;
            }
            if (window.AndroidNativeBridge) {
                detectNativeAndroidBridge();
            }
            resBox.style.color = '#00f2fe';
            resBox.textContent = 'GPS লোকেশন ও উপস্থিতি যাচাই করা হচ্ছে...';

            const r = await fetch('/api/v1/attendance/punch', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    username: uname,
                    pin: pin,
                    method: 'PIN',
                    device_id: 'MOBILE_ANDROID_APP',
                    client_platform: 'ANDROID_MOBILE',
                    latitude: mobileState.lat,
                    longitude: mobileState.lon,
                    wifi_ssid: mobileState.wifiSsid,
                    wifi_bssid: mobileState.wifiBssid,
                    gps_accuracy_meters: mobileState.accuracy
                })
            });
            const d = await r.json();
            if (r.ok && d.success) {
                resBox.style.color = '#00e676';
                resBox.textContent = `✔ ${d.message} (GPS Verified)`;
                await updateMobileNineCards(d);
                loadMobileDashboardStats();
            } else {
                resBox.style.color = '#ff5252';
                resBox.textContent = `❌ ${d.error || 'পাঞ্চ ব্যর্থ হয়েছে'}`;
                if (r.status === 403) {
                    showGateLocked(d.error || 'লোকেশন ছাড়া মোবাইল অ্যাপ কাজ করবে না!');
                }
            }
        }

        function toggleMobileForgotPw() {
            const el = document.getElementById('mob-forgot-pw-box');
            el.style.display = (el.style.display === 'none') ? 'block' : 'none';
        }

        async function submitMobileForgotPassword() {
            const uname = document.getElementById('mob-fp-user').value.trim().toUpperCase();
            const authVal = document.getElementById('mob-fp-auth').value.trim();
            const newPw = document.getElementById('mob-fp-newpw').value.trim();
            const st = document.getElementById('mob-fp-status');
            if (!uname || !authVal || !newPw) {
                st.style.color = '#ff5252';
                st.textContent = 'সবগুলো ঘর পূরণ করুন!';
                return;
            }
            const r = await fetch('/api/v1/auth/reset-password', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    username: uname,
                    hr_password: authVal,
                    android_id: authVal,
                    new_password: newPw,
                    company_id: mobileState.activeCompanyId
                })
            });
            const d = await r.json();
            if (r.ok && d.success) {
                st.style.color = '#00e676';
                st.textContent = `✔ ${d.message || 'পাসওয়ার্ড সফলভাবে রিসেট হয়েছে!'}`;
            } else {
                st.style.color = '#ff5252';
                st.textContent = `❌ ${d.error || 'পাসওয়ার্ড রিসেট ব্যর্থ'}`;
            }
        }

        async function submitMobileQrPunch() {
            if (window.AndroidNativeBridge) detectNativeAndroidBridge();
            const aid = document.getElementById('mob-qr-android-id').value.trim();
            const resBox = document.getElementById('mob-punch-result');
            if (!aid) return;
            const r = await fetch('/api/v1/attendance/punch', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    username: '',
                    method: 'QR',
                    android_id: aid,
                    device_id: 'MOBILE_QR_SCANNER',
                    client_platform: 'ANDROID_MOBILE',
                    latitude: mobileState.lat,
                    longitude: mobileState.lon,
                    wifi_ssid: mobileState.wifiSsid,
                    wifi_bssid: mobileState.wifiBssid,
                    gps_accuracy_meters: mobileState.accuracy
                })
            });
            const d = await r.json();
            if (r.ok && d.success) {
                resBox.style.color = '#00e676';
                resBox.textContent = `✔ QR Punch: ${d.username} (${d.login_type})`;
                await updateMobileNineCards(d);
                loadMobileDashboardStats();
            } else {
                resBox.style.color = '#ff5252';
                resBox.textContent = `❌ ${d.error || 'QR পাঞ্চ ব্যর্থ'}`;
                if (r.status === 403) showGateLocked(d.error);
            }
        }

        async function generateMobileQrSession() {
            const box = document.getElementById('mob-qr-session-box');
            box.style.display = 'block';
            const r = await fetch('/api/v1/devices/session', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ purpose: 'LOGIN' })
            });
            const d = await r.json();
            if (d.qr_svg) {
                document.getElementById('mob-qr-svg').innerHTML = d.qr_svg;
            }
            document.getElementById('mob-qr-session-txt').textContent = `Session: ${d.session_id} (স্ক্যানের অপেক্ষায়...)`;
        }

        async function startMobileCamera() {
            try {
                const video = document.getElementById('mob-video');
                if (!mobileState.stream || !mobileState.stream.active) {
                    const s = await navigator.mediaDevices.getUserMedia({
                        video: { facingMode: 'user', width: { ideal: 480 }, height: { ideal: 360 }, frameRate: { ideal: 30 } },
                        audio: false
                    });
                    mobileState.stream = s;
                }
                if (video && video.srcObject !== mobileState.stream) {
                    video.srcObject = mobileState.stream;
                }
                return true;
            } catch (e) {
                alert('ক্যামেরা চালু করা যায়নি: ' + e.message);
                return false;
            }
        }

        async function verifyAndPreviewMobileCamera() {
            if (window.AndroidNativeBridge) detectNativeAndroidBridge();
            const video = document.getElementById('mob-video');
            if (!mobileState.stream || !mobileState.stream.active || !video.srcObject) {
                const ok = await startMobileCamera();
                if (ok) await new Promise(r => setTimeout(r, 180));
            }
            const canvas = document.getElementById('mob-canvas');
            const ctx = canvas.getContext('2d', { alpha: false });
            ctx.drawImage(video, 0, 0, canvas.width, canvas.height);
            const b64 = canvas.toDataURL('image/jpeg', 0.85);
            const resBox = document.getElementById('mob-punch-result');
            resBox.style.color = '#00f2fe';
            resBox.textContent = '⚡ FaceNet512 ONNX ভেরিফিকেশন চলছে...';

            const r = await fetch('/api/v1/face/verify-identity', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ image_b64: b64 })
            });
            const d = await r.json();
            if (r.ok && d.matched) {
                mobileState.pendingVerifiedUser = d.username;
                mobileState.pendingVerifiedB64 = b64;
                mobileState.pendingVerifiedEmbedding = d.embedding || null;
                document.getElementById('mob-verify-split-box').style.display = 'block';
                document.getElementById('mob-ver-user').textContent = d.username;
                document.getElementById('mob-ver-tier').textContent = d.match_quality || 'EXACT';
                document.getElementById('mob-ver-conf').textContent = `${Number(d.confidence || 98).toFixed(1)}%`;
                document.getElementById('mob-ver-dist').textContent = Number(d.min_distance || 0.12).toFixed(4);
                let sec = 2;
                document.getElementById('mob-ver-countdown').textContent = `${sec}s`;
                if (mobileState.verifyTimer) clearInterval(mobileState.verifyTimer);
                mobileState.verifyTimer = setInterval(() => {
                    sec -= 1;
                    document.getElementById('mob-ver-countdown').textContent = `${sec}s`;
                    if (sec <= 0) {
                        clearInterval(mobileState.verifyTimer);
                        confirmVerifiedCameraPunch();
                    }
                }, 700);
            } else {
                resBox.style.color = '#ff5252';
                resBox.textContent = `❌ মুখ শনাক্ত হয়নি: ${d.error || d.reason || 'No match'}`;
            }
        }

        async function confirmVerifiedCameraPunch() {
            if (mobileState.verifyTimer) clearInterval(mobileState.verifyTimer);
            document.getElementById('mob-verify-split-box').style.display = 'none';
            if (!mobileState.pendingVerifiedUser) return;
            const resBox = document.getElementById('mob-punch-result');
            const r = await fetch('/api/v1/attendance/punch', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    username: mobileState.pendingVerifiedUser,
                    method: 'CAMERA',
                    device_id: 'MOBILE_ANDROID_APP',
                    client_platform: 'ANDROID_MOBILE',
                    latitude: mobileState.lat,
                    longitude: mobileState.lon,
                    wifi_ssid: mobileState.wifiSsid,
                    wifi_bssid: mobileState.wifiBssid,
                    gps_accuracy_meters: mobileState.accuracy,
                    face_embedding: mobileState.pendingVerifiedEmbedding || null,
                    image_b64: mobileState.pendingVerifiedB64
                })
            });
            const d = await r.json();
            if (r.ok && d.success) {
                resBox.style.color = '#00e676';
                resBox.textContent = `✔ ${d.message}`;
                await updateMobileNineCards(d);
                loadMobileDashboardStats();
            } else {
                resBox.style.color = '#ff5252';
                resBox.textContent = `❌ ${d.error || 'পাঞ্চ ব্যর্থ'}`;
            }
        }

        function cancelVerifiedCameraPunch() {
            if (mobileState.verifyTimer) clearInterval(mobileState.verifyTimer);
            document.getElementById('mob-verify-split-box').style.display = 'none';
            mobileState.pendingVerifiedUser = null;
        }

        async function submitMobileCameraPunch() {
            if (window.AndroidNativeBridge) {
                detectNativeAndroidBridge();
            }
            const video = document.getElementById('mob-video');
            if (!mobileState.stream || !mobileState.stream.active || !video.srcObject) {
                const ok = await startMobileCamera();
                if (ok) await new Promise(r => setTimeout(r, 180));
            }
            const canvas = document.getElementById('mob-canvas');
            const ctx = canvas.getContext('2d', { alpha: false });
            ctx.drawImage(video, 0, 0, canvas.width, canvas.height);
            const b64 = canvas.toDataURL('image/jpeg', 0.85);
            const resBox = document.getElementById('mob-punch-result');
            resBox.style.color = '#00f2fe';
            resBox.textContent = 'FaceNet512 ONNX ও GPS লোকেশন যাচাই হচ্ছে...';

            const r = await fetch('/api/v1/attendance/punch', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    username: document.getElementById('mob-punch-user').value.trim().toUpperCase(),
                    method: 'CAMERA',
                    device_id: 'MOBILE_ANDROID_APP',
                    client_platform: 'ANDROID_MOBILE',
                    latitude: mobileState.lat,
                    longitude: mobileState.lon,
                    wifi_ssid: mobileState.wifiSsid,
                    wifi_bssid: mobileState.wifiBssid,
                    gps_accuracy_meters: mobileState.accuracy,
                    image_b64: b64
                })
            });
            const d = await r.json();
            if (r.ok && d.success) {
                resBox.style.color = '#00e676';
                resBox.textContent = `✔ ${d.message}`;
                await updateMobileNineCards(d);
                loadMobileDashboardStats();
            } else {
                resBox.style.color = '#ff5252';
                resBox.textContent = `❌ ${d.error || 'ফেস ভেরিফিকেশন ব্যর্থ'}`;
                if (r.status === 403) {
                    showGateLocked(d.error || 'লোকেশন ছাড়া মোবাইল অ্যাপ কাজ করবে না!');
                }
            }
        }

        // 20-Pose Mobile Registration Functions
        async function startMobileRegCamera() {
            try {
                const s = await navigator.mediaDevices.getUserMedia({ video: { facingMode: 'user' } });
                mobileState.regStream = s;
                document.getElementById('mob-reg-video').srcObject = s;
                mobileState.regPosesB64 = [];
                document.getElementById('mob-reg20-badge').textContent = '0 / 20';
            } catch (e) {
                alert('ক্যামেরা চালু করা যায়নি: ' + e.message);
            }
        }

        async function captureNextMobilePose() {
            if (mobileState.regPosesB64.length >= 20) return;
            const video = document.getElementById('mob-reg-video');
            const canvas = document.getElementById('mob-canvas');
            const ctx = canvas.getContext('2d');
            ctx.drawImage(video, 0, 0, canvas.width, canvas.height);
            const b64 = canvas.toDataURL('image/jpeg', 0.85);
            const poseIdx = mobileState.regPosesB64.length;
            const r = await fetch('/api/v1/face/extract-frame', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ image_b64: b64, pose_index: poseIdx, enforce_pose: true })
            });
            const d = await r.json();
            if (r.ok && d.quality_ok !== false) {
                mobileState.regPosesB64.push(b64);
                document.getElementById('mob-reg20-badge').textContent = `${mobileState.regPosesB64.length} / 20`;
                document.getElementById('mob-reg20-prompt').textContent =
                    `✔ Pose ${mobileState.regPosesB64.length}/20 Captured (${d.pose_verified ? '✓ Verified' : '✗ Saved'})`;
            } else {
                document.getElementById('mob-reg20-prompt').textContent =
                    `⚠️ Retake Pose ${poseIdx + 1}: ${d.quality_reason || d.error || 'মুখ স্পষ্ট নয়'}`;
            }
        }

        async function autoCaptureAll20MobilePoses() {
            for (let i = mobileState.regPosesB64.length; i < 20; i++) {
                await captureNextMobilePose();
                await new Promise(res => setTimeout(res, 250));
            }
        }

        async function submitMobile20PoseRegistration() {
            const uname = document.getElementById('mob-camreg-user').value.trim().toUpperCase();
            const desig = document.getElementById('mob-camreg-desig').value.trim();
            const pw = document.getElementById('mob-camreg-pw').value.trim();
            const st = document.getElementById('mob-camreg-status');
            if (!uname || mobileState.regPosesB64.length === 0) {
                st.style.color = '#ff5252';
                st.textContent = 'ইউজারনেম এবং অন্তত ১-২০টি পোজ ক্যাপচার আবশ্যক!';
                return;
            }
            st.style.color = '#00f2fe';
            st.textContent = '২০-পোজ FaceNet512 এমবেডিং সেভ হচ্ছে...';
            const r = await fetch('/api/v1/users', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    username: uname,
                    designation: desig,
                    password: pw,
                    images_b64: mobileState.regPosesB64,
                    image_b64: mobileState.regPosesB64[0],
                    overwrite_existing: true
                })
            });
            const d = await r.json();
            if (r.ok) {
                st.style.color = '#00e676';
                st.textContent = `✔ ইউজার '${uname}' (${mobileState.regPosesB64.length} poses) সফলভাবে রেজিস্টার হয়েছে!`;
                resetAndLoadMobileUsers();
                loadMobileDashboardStats();
            } else {
                st.style.color = '#ff5252';
                st.textContent = `❌ ${d.error || 'রেজিস্ট্রেশন ব্যর্থ'}`;
            }
        }

        async function registerSingleUserFromMobile() {
            if (window.AndroidNativeBridge) {
                detectNativeAndroidBridge();
            }
            const uname = document.getElementById('mob-reg-username').value.trim().toUpperCase();
            const desig = document.getElementById('mob-reg-desig').value.trim();
            const pw = document.getElementById('mob-reg-pw').value.trim();
            const st = document.getElementById('mob-reg-status');
            if (!uname) {
                st.style.color = '#ff5252';
                st.textContent = 'ইউজারনেম আবশ্যক!';
                return;
            }
            const r = await fetch('/api/v1/mobile/register-user', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    latitude: mobileState.lat,
                    longitude: mobileState.lon,
                    gps_accuracy_meters: mobileState.accuracy,
                    wifi_ssid: mobileState.wifiSsid,
                    wifi_bssid: mobileState.wifiBssid,
                    username: uname,
                    user_designation: desig,
                    password: pw
                })
            });
            const d = await r.json();
            if (r.ok && d.success) {
                st.style.color = '#00e676';
                st.textContent = `✔ ইউজার '${uname}' সফলভাবে Shard #${d.shard_bucket}-এ রেজিস্টার হয়েছে!`;
                resetAndLoadMobileUsers();
                loadMobileDashboardStats();
            } else {
                st.style.color = '#ff5252';
                st.textContent = `❌ ${d.error || 'ত্রুটি'}`;
                if (r.status === 403) {
                    showGateLocked(d.error || 'লোকেশন ছাড়া মোবাইল অ্যাপ কাজ করবে না!');
                }
            }
        }

        async function registerQrUserFromMobile() {
            const uname = document.getElementById('mob-qrreg-user').value.trim().toUpperCase();
            const aid = document.getElementById('mob-qrreg-aid').value.trim();
            const desig = document.getElementById('mob-qrreg-desig').value.trim();
            const pw = document.getElementById('mob-qrreg-pw').value.trim();
            const st = document.getElementById('mob-qrreg-status');
            if (!uname || !aid) {
                st.style.color = '#ff5252';
                st.textContent = 'Username এবং Android ID আবশ্যক!';
                return;
            }
            const r = await fetch('/api/v1/users', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    username: uname,
                    designation: desig,
                    password: pw,
                    user_android_id: aid
                })
            });
            const d = await r.json();
            if (r.ok) {
                st.style.color = '#00e676';
                st.textContent = `✔ QR ইউজার '${uname}' (${aid}) সফলভাবে রেজিস্টার হয়েছে!`;
                resetAndLoadMobileUsers();
            } else {
                st.style.color = '#ff5252';
                st.textContent = `❌ ${d.error || 'ডুপ্লিকেট বা ত্রুটি'}`;
            }
        }

        async function bulkProvisionFromMobile() {
            if (window.AndroidNativeBridge) {
                detectNativeAndroidBridge();
            }
            const prefix = document.getElementById('mob-bulk-prefix').value.trim();
            const count = parseInt(document.getElementById('mob-bulk-count').value, 10) || 50;
            const st = document.getElementById('mob-bulk-status');
            st.style.color = '#00f2fe';
            st.textContent = `${count} জন ইউজার ৬৪ শার্ডে তৈরি করা হচ্ছে...`;

            const r = await fetch('/api/v1/mobile/bulk-provision', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    latitude: mobileState.lat,
                    longitude: mobileState.lon,
                    gps_accuracy_meters: mobileState.accuracy,
                    wifi_ssid: mobileState.wifiSsid,
                    wifi_bssid: mobileState.wifiBssid,
                    prefix: prefix,
                    count: count,
                    designation: 'Mobile Provisioned Staff'
                })
            });
            const d = await r.json();
            if (r.ok && d.success) {
                st.style.color = '#00e676';
                st.textContent = `✔ ${d.message} (${Number(d.execution_time_ms).toFixed(1)} ms)`;
                resetAndLoadMobileUsers();
                loadMobileDashboardStats();
            } else {
                st.style.color = '#ff5252';
                st.textContent = `❌ ${d.error || 'বাল্ক প্রোভিশন ব্যর্থ'}`;
                if (r.status === 403) {
                    showGateLocked(d.error || 'লোকেশন ছাড়া মোবাইল অ্যাপ কাজ করবে না!');
                }
            }
        }

        // Company & HR Functions
        async function switchMobileCompany() {
            const cid = document.getElementById('mob-comp-select').value;
            const pw = document.getElementById('mob-comp-verify-pw').value.trim();
            const st = document.getElementById('mob-comp-status');
            const r = await fetch('/api/v1/companies/verify', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ company_id: cid, password: pw, set_active: true })
            });
            const d = await r.json();
            if (r.ok && d.success) {
                st.style.color = '#00e676';
                st.textContent = `✔ Active Company switched to ${d.company.company_name}`;
                await loadMobileCompanies();
            } else {
                st.style.color = '#ff5252';
                st.textContent = `❌ ${d.error || 'Invalid company password'}`;
            }
        }

        async function createMobileCompany() {
            const st = document.getElementById('mob-comp-status');
            const payload = {
                company_name: document.getElementById('mob-c-name').value.trim(),
                company_id: document.getElementById('mob-c-id').value.trim(),
                company_address: document.getElementById('mob-c-addr').value.trim(),
                contact_person_name: document.getElementById('mob-c-person').value.trim(),
                contact_person_designation: document.getElementById('mob-c-desig').value.trim(),
                contact_person_number: document.getElementById('mob-c-phone').value.trim(),
                password: document.getElementById('mob-c-pw').value.trim()
            };
            const r = await fetch('/api/v1/companies/create', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload)
            });
            const d = await r.json();
            if (r.ok && d.success) {
                st.style.color = '#00e676';
                st.textContent = `✔ Company created: ${payload.company_name}`;
                await loadMobileCompanies();
            } else {
                st.style.color = '#ff5252';
                st.textContent = `❌ ${d.error || 'Create failed'}`;
            }
        }

        async function editMobileCompany() {
            const cid = document.getElementById('mob-c-id').value.trim() || mobileState.activeCompanyId;
            const st = document.getElementById('mob-comp-status');
            const payload = {
                company_name: document.getElementById('mob-c-name').value.trim(),
                company_id: cid,
                company_address: document.getElementById('mob-c-addr').value.trim(),
                contact_person_name: document.getElementById('mob-c-person').value.trim(),
                contact_person_designation: document.getElementById('mob-c-desig').value.trim(),
                contact_person_number: document.getElementById('mob-c-phone').value.trim(),
                password: document.getElementById('mob-c-pw').value.trim() || 'admin123'
            };
            const r = await fetch(`/api/v1/companies/${encodeURIComponent(cid)}`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload)
            });
            const d = await r.json();
            if (r.ok && d.success) {
                st.style.color = '#00e676';
                st.textContent = `✔ Company updated: ${payload.company_name}`;
                await loadMobileCompanies();
            } else {
                st.style.color = '#ff5252';
                st.textContent = `❌ ${d.error || 'Update failed'}`;
            }
        }

        async function unlockMobileHrPortal() {
            const pw = document.getElementById('mob-hr-pw').value.trim();
            const r = await fetch('/api/v1/auth/verify-hr', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ company_id: mobileState.activeCompanyId, password: pw })
            });
            const d = await r.json();
            if (r.ok && d.authenticated) {
                document.getElementById('mob-hr-tools-box').style.display = 'block';
                const b = document.getElementById('mob-hr-lock-badge');
                b.className = 'badge badge-ok';
                b.textContent = '🔓 HR Unlocked';
            } else {
                alert('ভুল HR পাসওয়ার্ড! (ডিফল্ট: HR@123)');
            }
        }

        async function inspectMobileEmbeddingsReport() {
            const out = document.getElementById('mob-hr-output');
            out.style.display = 'block';
            out.textContent = 'Loading 12-Column Embeddings Report & FAISS Index...';
            const r = await fetch('/api/v1/face/embeddings-report');
            const d = await r.json();
            out.innerHTML = `<b>🧬 Total Enrolled Profiles: ${d.total_records || 0}</b> | FAISS RAM Vectors: ${d.faiss_index ? d.faiss_index.ntotal_in_ram : 0}<br>` +
                (d.report || []).slice(0, 12).map(row => `• <b>${row.Username}</b> (${row.Image_Name}) — Dim: ${row.Number_of_Embeddings_Generated}, Pos: ${row.Positive_Count}, Neg: ${row.Negative_Count}`).join('<br>');
        }

        async function inspectMobileFakeAttendance() {
            const out = document.getElementById('mob-hr-output');
            out.style.display = 'block';
            out.textContent = 'Loading captured attendance verification images...';
            const r = await fetch('/api/v1/reports/verify-images');
            const d = await r.json();
            const items = d.items || [];
            out.innerHTML = `<b>🕵️ Captured Attendance Records: ${d.total_found || items.length}</b><br>` +
                items.slice(0, 15).map(it => `• <b>${it.username}</b> (${it.date}) — IN: ${it.in_time || '---'} ${it.image_url_in ? `<a href="${it.image_url_in}" target="_blank" style="color:#00f2fe;">[Photo]</a>` : ''}`).join('<br>');
        }

        async function runMobileHrSync(endpoint, title) {
            const out = document.getElementById('mob-hr-output');
            out.style.display = 'block';
            const r = await fetch(endpoint, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ company_id: mobileState.activeCompanyId })
            });
            const d = await r.json();
            out.innerHTML = `✔ <b>${title}:</b> ${d.message || JSON.stringify(d)}`;
        }

        async function deleteMobileUserFacePrompt() {
            const u = prompt('যে ইউজারের ফেস ডাটা ডিলিট করতে চান তার Username লিখুন (সব রিসেট করতে ALL লিখুন):');
            if (!u) return;
            const out = document.getElementById('mob-hr-output');
            out.style.display = 'block';
            if (u.trim().toUpperCase() === 'ALL') {
                const r = await fetch('/api/v1/face/reset', { method: 'POST' });
                const d = await r.json();
                out.textContent = `✔ All face encodings reset: ${JSON.stringify(d)}`;
            } else {
                const r = await fetch(`/api/v1/face/profiles/${encodeURIComponent(u.trim().toUpperCase())}`, { method: 'DELETE' });
                const d = await r.json();
                out.textContent = `✔ Deleted face data for ${u.toUpperCase()}: ${JSON.stringify(d)}`;
            }
        }

        async function changeMobileHrPassword() {
            const cur = document.getElementById('mob-hrcp-old').value.trim();
            const nw = document.getElementById('mob-hrcp-new').value.trim();
            const out = document.getElementById('mob-hr-output');
            out.style.display = 'block';
            const r = await fetch('/api/v1/auth/change-hr-password', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    company_id: mobileState.activeCompanyId,
                    current_password: cur,
                    new_password: nw,
                    confirm_password: nw
                })
            });
            const d = await r.json();
            out.textContent = d.message || d.error || JSON.stringify(d);
        }

        function fillCurrentGpsIntoPolicyForm() {
            if (mobileState.lat !== null && mobileState.lon !== null) {
                document.getElementById('pol-lat').value = mobileState.lat;
                document.getElementById('pol-lon').value = mobileState.lon;
            } else if (navigator.geolocation) {
                navigator.geolocation.getCurrentPosition(pos => {
                    mobileState.lat = pos.coords.latitude;
                    mobileState.lon = pos.coords.longitude;
                    document.getElementById('pol-lat').value = mobileState.lat;
                    document.getElementById('pol-lon').value = mobileState.lon;
                });
            }
        }

        async function saveMobilePolicySettings() {
            const st = document.getElementById('pol-save-status');
            const payload = {
                location_name: document.getElementById('pol-loc-name').value.trim(),
                target_latitude: parseFloat(document.getElementById('pol-lat').value),
                target_longitude: parseFloat(document.getElementById('pol-lon').value),
                allowed_radius_meters: parseFloat(document.getElementById('pol-radius').value) || 150,
                wifi_option_enabled: document.getElementById('pol-wifi-enabled').checked,
                allowed_wifi_ssids: document.getElementById('pol-wifi-ssids').value.trim()
            };
            const r = await fetch('/api/v1/mobile/policy', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload)
            });
            const d = await r.json();
            if (r.ok && d.success) {
                st.style.color = '#00e676';
                st.textContent = '✔ লোকেশন (বাধ্যতামূলক) ও Wi-Fi পলিসি সফলভাবে আপডেট হয়েছে!';
                await loadPolicyFromServer();
                await verifyWithBackend();
            } else {
                st.style.color = '#ff5252';
                st.textContent = '❌ ' + (d.error || 'সেভ ব্যর্থ');
            }
        }

        window.addEventListener('DOMContentLoaded', async () => {
            await loadPolicyFromServer();
            requestAndVerifyMobileLocation();
        });
    </script>
</body>
</html>"###
        .to_string()
}
