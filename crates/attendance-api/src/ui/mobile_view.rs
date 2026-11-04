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
        body { background: var(--bg-dark); color: var(--text-main); min-height: 100vh; padding-bottom: 85px; overflow-x: hidden; }
        .mobile-header {
            background: linear-gradient(135deg, #0f172a 0%, #1e1b4b 100%);
            border-bottom: 1px solid rgba(0, 242, 254, 0.3);
            padding: 12px 16px;
            position: sticky; top: 0; z-index: 100;
            display: flex; justify-content: space-between; align-items: center;
        }
        .brand-box { display: flex; align-items: center; gap: 10px; }
        .brand-icon { width: 38px; height: 38px; border-radius: 10px; background: linear-gradient(135deg, #00f2fe, #4facfe); display: flex; align-items: center; justify-content: center; font-size: 1.3rem; color: #051923; font-weight: 900; }
        .brand-title { font-size: 1rem; font-weight: 800; letter-spacing: 0.5px; color: #fff; }
        .brand-sub { font-size: 0.72rem; color: var(--accent-cyan); display: block; }
        .security-ribbon {
            background: rgba(15, 23, 42, 0.95);
            border-bottom: 1px solid rgba(255,255,255,0.08);
            padding: 8px 14px;
            font-size: 0.76rem;
            display: flex; flex-wrap: wrap; gap: 8px; justify-content: space-between; align-items: center;
        }
        .badge { padding: 4px 9px; border-radius: 999px; font-weight: 700; font-size: 0.72rem; display: inline-flex; align-items: center; gap: 4px; }
        .badge-ok { background: rgba(0, 230, 118, 0.16); color: #00e676; border: 1px solid rgba(0, 230, 118, 0.4); }
        .badge-err { background: rgba(255, 82, 82, 0.2); color: #ff5252; border: 1px solid rgba(255, 82, 82, 0.5); }
        .badge-warn { background: rgba(255, 171, 0, 0.18); color: #ffab00; border: 1px solid rgba(255, 171, 0, 0.45); }
        .container { padding: 14px; max-width: 640px; margin: 0 auto; }
        .card {
            background: var(--card-bg);
            border: 1px solid rgba(0, 242, 254, 0.18);
            border-radius: 14px;
            padding: 16px;
            margin-bottom: 14px;
            box-shadow: 0 8px 24px rgba(0,0,0,0.35);
        }
        .card-title { font-size: 0.95rem; font-weight: 800; color: var(--accent-cyan); margin-bottom: 10px; display: flex; align-items: center; gap: 6px; }
        .form-group { margin-bottom: 10px; }
        .form-label { font-size: 0.78rem; color: var(--text-sub); font-weight: 700; display: block; margin-bottom: 4px; }
        .form-input {
            width: 100%; padding: 10px 12px; border-radius: 10px;
            background: #0b1120; border: 1px solid rgba(255,255,255,0.16);
            color: #fff; font-size: 0.9rem; outline: none;
        }
        .form-input:focus { border-color: var(--accent-cyan); }
        .btn {
            width: 100%; padding: 12px; border-radius: 10px; border: none;
            font-weight: 800; font-size: 0.9rem; cursor: pointer;
            display: inline-flex; align-items: center; justify-content: center; gap: 8px;
            transition: transform 0.1s, opacity 0.2s;
        }
        .btn:active { transform: scale(0.98); }
        .btn-cyan { background: linear-gradient(135deg, #00f2fe, #4facfe); color: #041421; }
        .btn-green { background: linear-gradient(135deg, #00e676, #00b0ff); color: #041421; }
        .btn-purple { background: linear-gradient(135deg, #7c4dff, #b388ff); color: #fff; }
        .btn-red { background: linear-gradient(135deg, #ff5252, #d50000); color: #fff; }
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
            display: grid; grid-template-columns: repeat(4, 1fr);
            padding: 6px 4px;
        }
        .nav-btn {
            background: none; border: none; color: var(--text-sub);
            display: flex; flex-direction: column; align-items: center; gap: 3px;
            padding: 6px 2px; font-size: 0.7rem; font-weight: 700; cursor: pointer;
        }
        .nav-btn.active { color: var(--accent-cyan); }
        .nav-btn span.icon { font-size: 1.2rem; }
        .tab-view { display: none; }
        .tab-view.active { display: block; }
        .user-row {
            background: rgba(255,255,255,0.03);
            border: 1px solid rgba(255,255,255,0.08);
            border-radius: 10px;
            padding: 10px 12px;
            margin-bottom: 8px;
            display: flex; justify-content: space-between; align-items: center;
        }
        .stat-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-bottom: 12px; }
        .stat-box { background: rgba(0, 242, 254, 0.06); border: 1px solid rgba(0, 242, 254, 0.2); border-radius: 10px; padding: 10px; text-align: center; }
        .stat-val { font-size: 1.35rem; font-weight: 900; color: #fff; }
        .stat-lbl { font-size: 0.72rem; color: var(--text-sub); }
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
                <input type="text" id="gate-wifi-input" class="form-input" placeholder="যেমন: Home_WiFi বা Office_WiFi" value="Home_WiFi">
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
                <span class="brand-title">SMART ATTENDANCE MOBILE</span>
                <span class="brand-sub">Millions-of-Users No-PC Office Engine</span>
            </div>
        </div>
        <a href="/api/v1/downloads/android-apk" style="background: rgba(0, 230, 118, 0.2); border: 1px solid #00e676; color: #00e676; padding: 6px 10px; border-radius: 8px; font-size: 0.75rem; font-weight: 800; text-decoration: none;">
            ⬇️ APK
        </a>
    </header>

    <!-- LIVE GPS & WI-FI TELEMETRY RIBBON -->
    <div class="security-ribbon">
        <span id="ribbon-gps-badge" class="badge badge-err">📍 GPS: লকড</span>
        <span id="ribbon-dist-badge" class="badge badge-warn">📏 দূরত্ব: ---</span>
        <span id="ribbon-wifi-badge" class="badge badge-ok">📶 Wi-Fi: Ready</span>
    </div>

    <div class="container">
        <!-- ================================================================= -->
        <!-- TAB 1: MOBILE BIOMETRIC & PIN KIOSK                               -->
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

            <div class="card">
                <div class="card-title">📷 মোবাইল ফেস রিকগনিশন ও ইন/আউট কিয়স্ক</div>
                <p style="font-size: 0.78rem; color: var(--text-sub); margin-bottom: 10px;">
                    পিসি ছাড়াই সরাসরি মোবাইল ক্যামেরা বা পিন দিয়ে উপস্থিতি নিন (প্রতিটি পাঞ্চে GPS Location বাধ্যতামূলকভাবে যাচাই হবে):
                </p>
                <div style="background: #000; border-radius: 12px; overflow: hidden; height: 220px; position: relative; margin-bottom: 10px; border: 1px solid rgba(0,242,254,0.3);">
                    <video id="mob-video" autoplay playsinline muted style="width:100%; height:100%; object-fit:cover; transform: scaleX(-1);"></video>
                    <canvas id="mob-canvas" width="480" height="360" style="display:none;"></canvas>
                </div>
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px; margin-bottom: 10px;">
                    <button class="btn btn-outline" onclick="startMobileCamera()">🎥 ক্যামেরা চালু করুন</button>
                    <button class="btn btn-cyan" onclick="submitMobileCameraPunch()">📸 ছবি তুলে উপস্থিতি দিন</button>
                </div>
            </div>

            <div class="card">
                <div class="card-title">🔐 ইউজারনেম ও পিন/পাসওয়ার্ড উপস্থিতি (Mobile Punch)</div>
                <div class="form-group">
                    <label class="form-label">ইউজারনেম (USERNAME)</label>
                    <input type="text" id="mob-punch-user" class="form-input" placeholder="যেমন: RAHIM বা ADM000" style="text-transform: uppercase;">
                </div>
                <div class="form-group">
                    <label class="form-label">পাসওয়ার্ড / পিন (PIN / Password)</label>
                    <input type="password" id="mob-punch-pin" class="form-input" placeholder="ডিফল্ট: admin123 বা 1234" value="1234">
                </div>
                <button class="btn btn-green" onclick="submitMobilePinPunch()">
                    ✅ মোবাইল থেকে উপস্থিতি (IN / OUT) রেকর্ড করুন
                </button>
                <div id="mob-punch-result" style="margin-top: 10px; font-size: 0.84rem; font-weight: 700; text-align: center;"></div>
            </div>
        </section>

        <!-- ================================================================= -->
        <!-- TAB 2: MILLIONS-OF-USERS SHARDED DIRECTORY (64 SHARDS + O(1))     -->
        <!-- ================================================================= -->
        <section id="tab-millions" class="tab-view">
            <div class="card">
                <div class="card-title">👥 কোটি/লক্ষ ইউজার ম্যানেজমেন্ট (64 Physical Shards)</div>
                <p style="font-size: 0.78rem; color: var(--text-sub); margin-bottom: 10px;">
                    কোনো অফিসে পিসি না থাকলেও মোবাইল থেকেই O(1) Keyset Cursor Pagination এর মাধ্যমে লক্ষ/কোটি ইউজার পরিচালনা করুন:
                </p>
                <div style="display: grid; grid-template-columns: 2fr 1fr; gap: 8px; margin-bottom: 10px;">
                    <input type="text" id="mob-user-search" class="form-input" placeholder="ইউজারনেম বা পদবি খুঁজুন..." oninput="resetAndLoadMobileUsers()">
                    <select id="mob-shard-select" class="form-input" onchange="resetAndLoadMobileUsers()">
                        <option value="">সব ৬৪ শার্ড (All)</option>
                    </select>
                </div>
                <div id="mob-cursor-meta" style="font-size: 0.75rem; color: var(--accent-cyan); margin-bottom: 10px;"></div>
                <div id="mob-users-list" style="max-height: 360px; overflow-y: auto;"></div>
                <div style="display: flex; gap: 8px; margin-top: 10px;">
                    <button class="btn btn-outline" onclick="resetAndLoadMobileUsers()">⏮️ প্রথম পেজ</button>
                    <button class="btn btn-cyan" onclick="loadNextMobileUsersPage()">পরবর্তী পেজ (O(1) Cursor) ⏭️</button>
                </div>
            </div>
        </section>

        <!-- ================================================================= -->
        <!-- TAB 3: MOBILE USER REGISTRATION & BULK MILLIONS PROVISIONING      -->
        <!-- ================================================================= -->
        <section id="tab-provision" class="tab-view">
            <div class="card">
                <div class="card-title">➕ মোবাইল থেকে নতুন ইউজার রেজিস্ট্রেশন</div>
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
                    👤 ইউজার রেজিস্টার করুন
                </button>
                <div id="mob-reg-status" style="margin-top:8px; font-size:0.82rem; font-weight:700;"></div>
            </div>

            <div class="card">
                <div class="card-title">⚡ বাল্ক ইউজার জেনারেটর (Bulk Sharded Provisioning)</div>
                <p style="font-size: 0.78rem; color: var(--text-sub); margin-bottom: 10px;">
                    মোবাইল থেকেই এক ক্লিকে ৬৪টি শার্ডে শত শত বা হাজার হাজার ইউজার ব্যাচ হিসেবে যুক্ত করুন:
                </p>
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
        <!-- TAB 4: MANDATORY LOCATION & WI-FI SECURITY POLICY                 -->
        <!-- ================================================================= -->
        <section id="tab-policy" class="tab-view">
            <div class="card">
                <div class="card-title">📍 বাধ্যতামূলক লোকেশন ও Wi-Fi পলিসি সেটিংস</div>
                <div style="background: rgba(0, 230, 118, 0.12); border: 1px solid rgba(0, 230, 118, 0.4); padding: 10px; border-radius: 10px; font-size: 0.8rem; margin-bottom: 12px;">
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
                        <div style="font-weight: 800; font-size: 0.86rem; color: #fff;">📶 নির্ধারিত Wi-Fi অপশন (Wi-Fi Enforcement)</div>
                        <div style="font-size: 0.74rem; color: #cbd5e1;">চালু করলে নির্ধারিত লোকেশন + নির্ধারিত Wi-Fi দুটোই লাগবে</div>
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
        </section>
    </div>

    <!-- BOTTOM NAVIGATION BAR -->
    <nav class="bottom-nav">
        <button class="nav-btn active" id="nav-kiosk" onclick="switchMobileTab('kiosk')">
            <span class="icon">📷</span>
            <span>উপস্থিতি কিয়স্ক</span>
        </button>
        <button class="nav-btn" id="nav-millions" onclick="switchMobileTab('millions')">
            <span class="icon">👥</span>
            <span>লক্ষ ইউজার (64-Shard)</span>
        </button>
        <button class="nav-btn" id="nav-provision" onclick="switchMobileTab('provision')">
            <span class="icon">⚡</span>
            <span>ইউজার ও বাল্ক অ্যাড</span>
        </button>
        <button class="nav-btn" id="nav-policy" onclick="switchMobileTab('policy')">
            <span class="icon">📍</span>
            <span>লোকেশন ও Wi-Fi</span>
        </button>
    </nav>

    <script>
        let mobileState = {
            lat: null,
            lon: null,
            accuracy: null,
            wifiSsid: "Home_WiFi",
            verified: false,
            policy: null,
            nextCursorUsername: "",
            stream: null
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
                    ? `চালু আছে (${p.allowed_wifi_ssids})`
                    : `ঐচ্ছিক (${p.allowed_wifi_ssids})`;

                document.getElementById('pol-loc-name').value = p.location_name || '';
                document.getElementById('pol-lat').value = p.target_latitude;
                document.getElementById('pol-lon').value = p.target_longitude;
                document.getElementById('pol-radius').value = p.allowed_radius_meters;
                document.getElementById('pol-wifi-enabled').checked = !!p.wifi_option_enabled;
                document.getElementById('pol-wifi-ssids').value = p.allowed_wifi_ssids || '';

                if (p.allowed_wifi_ssids) {
                    const firstSsid = p.allowed_wifi_ssids.split(',')[0].trim();
                    if (firstSsid && !document.getElementById('gate-wifi-input').value) {
                        document.getElementById('gate-wifi-input').value = firstSsid;
                    }
                }
            } catch (e) {
                console.error('Failed to load mobile policy:', e);
            }
        }

        function detectNativeAndroidBridge() {
            if (window.AndroidNativeBridge) {
                try {
                    const wifiJson = JSON.parse(window.AndroidNativeBridge.getWifiInfoJson());
                    if (wifiJson && wifiJson.ssid) {
                        document.getElementById('gate-wifi-input').value = wifiJson.ssid;
                        mobileState.wifiSsid = wifiJson.ssid;
                    }
                    const locJson = JSON.parse(window.AndroidNativeBridge.getHardwareLocationJson());
                    if (locJson && locJson.latitude && locJson.longitude) {
                        mobileState.lat = Number(locJson.latitude);
                        mobileState.lon = Number(locJson.longitude);
                        mobileState.accuracy = Number(locJson.accuracy || 15);
                        return true;
                    }
                } catch (e) {
                    console.warn('Android bridge probe error:', e);
                }
            }
            return false;
        }

        async function requestAndVerifyMobileLocation() {
            mobileState.wifiSsid = (document.getElementById('gate-wifi-input').value || '').trim();
            document.getElementById('gate-current-coords').textContent = 'GPS সিগন্যাল খোঁজা হচ্ছে...';

            if (detectNativeAndroidBridge() && mobileState.lat !== null) {
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
                    await verifyWithBackend();
                },
                (err) => {
                    mobileState.lat = null;
                    mobileState.lon = null;
                    showGateLocked('লোকেশন পারমিশন পাওয়া যায়নি বা GPS বন্ধ! লোকেশন ছাড়া মোবাইল অ্যাপ কাজ করবে না। অনুগ্রহ করে GPS চালু করুন।');
                },
                { enableHighAccuracy: true, timeout: 10000, maximumAge: 0 }
            );
        }

        async function verifyWithBackend() {
            try {
                const resp = await fetch('/api/v1/mobile/verify-access', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        latitude: mobileState.lat,
                        longitude: mobileState.lon,
                        gps_accuracy_meters: mobileState.accuracy,
                        wifi_ssid: mobileState.wifiSsid,
                        device_id: 'MOBILE_WEB_OR_APK'
                    })
                });
                const res = await resp.json();
                if (mobileState.lat !== null && mobileState.lon !== null) {
                    document.getElementById('gate-current-coords').textContent = `${mobileState.lat.toFixed(5)}, ${mobileState.lon.toFixed(5)}`;
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
                    document.getElementById('ribbon-wifi-badge').textContent = `📶 Wi-Fi: ${mobileState.wifiSsid || 'OK'}`;
                    loadMobileDashboardStats();
                    resetAndLoadMobileUsers();
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
            if (!navigator.geolocation) {
                alert('Geolocation পাওয়া যায়নি!');
                return;
            }
            navigator.geolocation.getCurrentPosition(
                async (pos) => {
                    mobileState.lat = pos.coords.latitude;
                    mobileState.lon = pos.coords.longitude;
                    mobileState.wifiSsid = (document.getElementById('gate-wifi-input').value || 'Home_WiFi').trim();
                    await savePolicyCoords(mobileState.lat, mobileState.lon, 'আমার নির্ধারিত বাসা / অফিস লোকেশন');
                },
                async () => {
                    // If browser inside non-HTTPS or desktop test blocks hardware GPS prompt, use policy target or prompt
                    const lat = mobileState.policy ? mobileState.policy.target_latitude : 23.8103;
                    const lon = mobileState.policy ? mobileState.policy.target_longitude : 90.4125;
                    mobileState.lat = lat;
                    mobileState.lon = lon;
                    mobileState.wifiSsid = (document.getElementById('gate-wifi-input').value || 'Home_WiFi').trim();
                    await verifyWithBackend();
                },
                { enableHighAccuracy: true, timeout: 6000 }
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
            if (!mobileState.verified) {
                showGateLocked('লোকেশন ছাড়া মোবাইল অ্যাপ কাজ করবে না! অনুগ্রহ করে আগে লোকেশন যাচাই করুন।');
                return;
            }
            document.querySelectorAll('.tab-view').forEach(el => el.classList.remove('active'));
            document.querySelectorAll('.nav-btn').forEach(el => el.classList.remove('active'));
            document.getElementById('tab-' + tab).classList.add('active');
            document.getElementById('nav-' + tab).classList.add('active');
        }

        async function loadMobileDashboardStats() {
            if (!mobileState.verified) return;
            try {
                const url = `/api/v1/mobile/scale-dashboard?latitude=${mobileState.lat}&longitude=${mobileState.lon}&wifi_ssid=${encodeURIComponent(mobileState.wifiSsid)}`;
                const r = await fetch(url);
                if (!r.ok) return;
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
            let url = `/api/v1/mobile/users-cursor?latitude=${mobileState.lat}&longitude=${mobileState.lon}&wifi_ssid=${encodeURIComponent(mobileState.wifiSsid)}&limit=25`;
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
                        <div style="font-weight:800; color:#fff; font-size:0.9rem;">👤 ${u.username}</div>
                        <div style="font-size:0.74rem; color:#94a3b8;">${u.user_designation || 'Employee'} • ${u.company_name || 'HQ'}</div>
                    </div>
                    <div style="text-align:right;">
                        <span class="badge badge-ok">Shard #${String(u.shard_bucket || 0).padStart(2, '0')}</span>
                    </div>
                </div>
            `).join('');
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
                    gps_accuracy_meters: mobileState.accuracy
                })
            });
            const d = await r.json();
            if (r.ok && d.success) {
                resBox.style.color = '#00e676';
                resBox.textContent = `✔ ${d.message} (GPS Verified)`;
                loadMobileDashboardStats();
            } else {
                resBox.style.color = '#ff5252';
                resBox.textContent = `❌ ${d.error || 'পাঞ্চ ব্যর্থ হয়েছে'}`;
            }
        }

        async function startMobileCamera() {
            try {
                const s = await navigator.mediaDevices.getUserMedia({ video: { facingMode: 'user' } });
                mobileState.stream = s;
                document.getElementById('mob-video').srcObject = s;
            } catch (e) {
                alert('ক্যামেরা চালু করা যায়নি: ' + e.message);
            }
        }

        async function submitMobileCameraPunch() {
            const video = document.getElementById('mob-video');
            const canvas = document.getElementById('mob-canvas');
            const ctx = canvas.getContext('2d');
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
                    image_b64: b64
                })
            });
            const d = await r.json();
            if (r.ok && d.success) {
                resBox.style.color = '#00e676';
                resBox.textContent = `✔ ${d.message}`;
                loadMobileDashboardStats();
            } else {
                resBox.style.color = '#ff5252';
                resBox.textContent = `❌ ${d.error || 'ফেস ভেরিফিকেশন ব্যর্থ'}`;
            }
        }

        async function registerSingleUserFromMobile() {
            const uname = document.getElementById('mob-reg-username').value.trim().toUpperCase();
            const desig = document.getElementById('mob-reg-desig').value.trim();
            const pw = document.getElementById('mob-reg-pw').value.trim();
            const st = document.getElementById('mob-reg-status');
            if (!uname) {
                st.style.color = '#ff5252';
                st.textContent = 'ইউজারনেম আবশ্যক!';
                return;
            }
            const r = await fetch('/api/v1/users/register', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    username: uname,
                    user_designation: desig,
                    password: pw
                })
            });
            const d = await r.json();
            if (r.ok) {
                st.style.color = '#00e676';
                st.textContent = `✔ ইউজার '${uname}' সফলভাবে রেজিস্টার হয়েছে!`;
                resetAndLoadMobileUsers();
            } else {
                st.style.color = '#ff5252';
                st.textContent = `❌ ${d.error || 'ত্রুটি'}`;
            }
        }

        async function bulkProvisionFromMobile() {
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
                    wifi_ssid: mobileState.wifiSsid,
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
            }
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
