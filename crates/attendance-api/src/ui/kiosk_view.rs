use crate::ui::scripts::get_kiosk_js;
use crate::ui::styles::get_global_css;

pub fn render_full_portal_html() -> String {
    let css = get_global_css();
    let js = get_kiosk_js();

    format!(
        r#"<!DOCTYPE html>
<html lang="bn">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Face Recognition System — Attendance Management System</title>
    <style>{}</style>
</head>
<body>
    <!-- FLOATING MODERN HEADER (attendance_v8.py line 6538 Parity) -->
    <header class="app-header">
        <div class="brand-wrapper">
            <div class="brand-logo">⚡</div>
            <div class="brand-title-box">
                <span class="brand-main-title">ATTENDANCE</span>
                <span class="brand-subtitle">MANAGEMENT SYSTEM</span>
            </div>
        </div>
        <div class="clock-wrapper">
            <div id="digital-clock" class="clock-digital">00:00:00</div>
            <div id="dhaka-date" class="clock-sub">Asia/Dhaka</div>
        </div>
    </header>

    <!-- TWO-COLUMN MAIN LAYOUT (attendance_v8.py line 6060 Parity) -->
    <main class="main-layout">
        <!-- LEFT PANEL (width 550px) -->
        <aside class="left-panel">
            <!-- 1. ACTIONS PANEL (attendance_v8.py line 6183) -->
            <section class="glass-panel">
                <div class="section-header">
                    <div class="accent-bar" style="background: #00b894;"></div>
                    <span class="section-title">ACTIONS</span>
                </div>
                <div class="action-buttons-box">
                    <button class="btn-action-gradient btn-camera" onclick="openCameraLogin()">
                        <span class="btn-icon">📷</span>
                        <span class="btn-text">In & Out with Camera</span>
                    </button>
                    <button class="btn-action-gradient btn-no-camera" onclick="openNoCameraLogin()">
                        <span class="btn-icon">🚪</span>
                        <span class="btn-text">In & Out without Camera</span>
                    </button>
                    <button class="btn-action-gradient btn-qr" onclick="openQrLogin()">
                        <span class="btn-icon">📲</span>
                        <span class="btn-text">In & Out with QR Code (Online)</span>
                    </button>
                </div>

                <!-- FLOATING HR BUTTON (attendance_v8.py line 6076) -->
                <button class="btn-hr-floating" onclick="openHrManagement()">
                    <span>👔</span>
                    <span>HR MANAGEMENT & ADMIN</span>
                </button>
            </section>

            <!-- 2. SYSTEM STATUS DASHBOARD (attendance_v8.py line 6092) -->
            <section class="glass-panel">
                <div class="section-header">
                    <div class="accent-bar" style="background: #6c5ce7;"></div>
                    <span class="section-title">SYSTEM STATUS</span>
                </div>
                <div class="status-grid">
                    <div class="status-card">
                        <div class="status-card-header">
                            <span class="status-card-icon">💻</span>
                            <span class="status-card-title">System Status</span>
                        </div>
                        <div class="status-card-progress"><div class="status-card-progress-bar"></div></div>
                        <span class="status-card-state" id="st-system">Loaded ✓</span>
                    </div>
                    <div class="status-card">
                        <div class="status-card-header">
                            <span class="status-card-icon">🌐</span>
                            <span class="status-card-title">Network</span>
                        </div>
                        <div class="status-card-progress"><div class="status-card-progress-bar"></div></div>
                        <span class="status-card-state" id="st-network">Network Ready ✓</span>
                    </div>
                    <div class="status-card">
                        <div class="status-card-header">
                            <span class="status-card-icon">🧠</span>
                            <span class="status-card-title">AI Models</span>
                        </div>
                        <div class="status-card-progress"><div class="status-card-progress-bar"></div></div>
                        <span class="status-card-state" id="st-ai">FaceNet512 Ready ✓</span>
                    </div>
                    <div class="status-card">
                        <div class="status-card-header">
                            <span class="status-card-icon">🗃️</span>
                            <span class="status-card-title">Database</span>
                        </div>
                        <div class="status-card-progress"><div class="status-card-progress-bar"></div></div>
                        <span class="status-card-state" id="st-db">Database Ready ✓</span>
                    </div>
                </div>
            </section>
        </aside>

        <!-- RIGHT PANEL (attendance_v8.py line 6065) -->
        <section class="right-panel">
            <!-- UPPER SECTION: SYSTEM CONSOLE (attendance_v8.py line 6591) -->
            <div class="glass-panel console-box">
                <div class="console-header-controls">
                    <div class="section-header" style="margin-bottom: 0;">
                        <div class="accent-bar" style="background: #6c5ce7;"></div>
                        <span class="section-title">SYSTEM CONSOLE</span>
                    </div>
                    <div class="console-action-btns">
                        <button class="btn-tool btn-export" title="Export Logs" onclick="exportConsoleLogs()">🡇</button>
                        <button class="btn-tool btn-sync" title="Force Sync" onclick="forceSync()">🔄</button>
                        <button class="btn-tool btn-clear" title="Clear Console" onclick="clearConsole()">🗑️</button>
                    </div>
                </div>

                <div class="terminal-window">
                    <div class="terminal-titlebar">
                        <div class="mac-dots">
                            <span class="dot dot-red"></span>
                            <span class="dot dot-yellow"></span>
                            <span class="dot dot-green"></span>
                        </div>
                        <span class="terminal-title">system_logs</span>
                    </div>
                    <div id="terminal-body" class="terminal-body"></div>
                </div>
            </div>

            <!-- LOWER SECTION: TODAY'S ATTENDANCE CARDS (attendance_v8.py line 6705 - EXACT 9 CARDS) -->
            <div class="glass-panel attendance-box">
                <div class="console-header-controls">
                    <div class="section-header" style="margin-bottom: 0;">
                        <div class="accent-bar" style="background: #00b894;"></div>
                        <span class="section-title">TODAY'S ATTENDANCE</span>
                    </div>
                    <div class="console-action-btns">
                        <!-- 7-Day History Button (attendance_v8.py line 6741) -->
                        <button class="btn-tool" style="background: #5E0037;" title="7 day's attendance" onclick="openSevenDayHistory()">📈</button>
                    </div>
                </div>

                <!-- 9 ATTENDANCE INFO CARDS (attendance_v8.py lines 6800-6850 Parity) -->
                <div class="attendance-cards-scroll-container">
                    <div class="attendance-card" style="border-left-color: #6c5ce7;">
                        <div class="att-card-left">
                            <span class="att-card-icon">👤</span>
                            <span class="att-card-label">User Name</span>
                        </div>
                        <div class="att-card-value" id="card-username">---</div>
                    </div>

                    <div class="attendance-card" style="border-left-color: #fdcb6e;">
                        <div class="att-card-left">
                            <span class="att-card-icon">⏱️</span>
                            <span class="att-card-label">Record Time</span>
                        </div>
                        <div class="att-card-value" id="card-record-time">---</div>
                    </div>

                    <div class="attendance-card" style="border-left-color: #00b894;">
                        <div class="att-card-left">
                            <span class="att-card-icon">🔐</span>
                            <span class="att-card-label">In/Out</span>
                        </div>
                        <div class="att-card-value" id="card-login-type">---</div>
                    </div>

                    <div class="attendance-card" style="border-left-color: #dfe6e9;">
                        <div class="att-card-left">
                            <span class="att-card-icon">📋</span>
                            <span class="att-card-label">Attendance Status</span>
                        </div>
                        <div class="att-card-value" id="card-attendance-type">---</div>
                    </div>

                    <div class="attendance-card" style="border-left-color: #e17055;">
                        <div class="att-card-left">
                            <span class="att-card-icon">📅</span>
                            <span class="att-card-label">Today's Date</span>
                        </div>
                        <div class="att-card-value" id="card-date">---</div>
                    </div>

                    <div class="attendance-card" style="border-left-color: #a29bfe;">
                        <div class="att-card-left">
                            <span class="att-card-icon">📱</span>
                            <span class="att-card-label">Camera Status</span>
                        </div>
                        <div class="att-card-value" id="card-attendance-method">---</div>
                    </div>

                    <div class="attendance-card" style="border-left-color: #ff7675;">
                        <div class="att-card-left">
                            <span class="att-card-icon">⏰</span>
                            <span class="att-card-label">Total Lates</span>
                        </div>
                        <div class="att-card-value" id="card-total-lates">0</div>
                    </div>

                    <div class="attendance-card" style="border-left-color: #d63031;">
                        <div class="att-card-left">
                            <span class="att-card-icon">🚫</span>
                            <span class="att-card-label">Total Late Absents</span>
                        </div>
                        <div class="att-card-value" id="card-total-late-absents">0</div>
                    </div>

                    <div class="attendance-card" style="border-left-color: #e84393;">
                        <div class="att-card-left">
                            <span class="att-card-icon">🏃</span>
                            <span class="att-card-label">Total Early Leaves</span>
                        </div>
                        <div class="att-card-value" id="card-total-early-leaves">0</div>
                    </div>
                </div>
            </div>
        </section>
    </main>

    <!-- MODAL 1: CAMERA PREVIEW WINDOW (attendance_v8.py lines 11782-11840 Parity) -->
    <div id="modal-camera" class="modal-overlay">
        <div class="modal-dialog" style="max-width: 660px;">
            <div class="modal-header">
                <div class="modal-title"><span>📷</span> <span>Camera Preview</span></div>
                <button class="modal-close" onclick="closeCameraLogin()">&times;</button>
            </div>
            <div class="modal-body" style="text-align: center;">
                <div class="camera-wrapper" style="height: 380px;">
                    <video id="camera-stream" class="camera-video" autoplay playsinline muted></video>
                    <canvas id="camera-canvas" style="display:none;"></canvas>
                    <div class="camera-overlay-box"></div>
                </div>
                <div id="camera-status" style="font-size: 0.95rem; font-weight: 600; color: #00b894; margin-bottom: 1.25rem;">
                    Camera Ready - Click Capture when ready
                </div>
                <!-- Large Green Capture Button (attendance_v8.py line 11827) -->
                <button id="btn-capture" class="btn-python-capture" onclick="executeCaptureImage()">
                    CAPTURE
                </button>
            </div>
        </div>
    </div>

    <!-- MODAL 2: IN & OUT WITHOUT CAMERA (attendance_v8.py lines 8295-8450 Parity) -->
    <div id="modal-no-camera" class="modal-overlay">
        <div class="modal-dialog" style="max-width: 440px; background: #1e293b;">
            <div class="modal-header" style="background: #1e293b; border-bottom: none; padding-bottom: 0;">
                <div class="modal-title" style="font-size: 1rem;"><span>🚪</span> <span>Login</span></div>
                <button class="modal-close" onclick="closeNoCameraLogin()">&times;</button>
            </div>
            <div class="modal-body" style="background: #1e293b; padding-top: 0.5rem;">
                <div style="text-align: center; margin-bottom: 1.25rem;">
                    <div style="width: 60px; height: 60px; border-radius: 50%; background: #2d3748; margin: 0 auto 0.75rem auto; display: flex; align-items: center; justify-content: center; font-size: 1.5rem;">
                        👤
                    </div>
                    <h3 style="font-size: 1.25rem; font-weight: 700; color: #f1f5f9;">In & Out without Camera</h3>
                    <p style="font-size: 0.85rem; color: #94a3b8;">Please enter your credentials to continue</p>
                </div>

                <!-- USERNAME -->
                <div style="margin-bottom: 1rem;">
                    <label style="font-size: 0.75rem; font-weight: 700; color: #94a3b8; display: block; margin-bottom: 0.25rem;">USERNAME</label>
                    <div style="background: #2d3748; border: 1px solid #4a5568; border-radius: 8px; display: flex; align-items: center; padding: 0 0.75rem;">
                        <span style="color: #a29bfe; margin-right: 0.5rem;">👤</span>
                        <input type="text" id="no-cam-username" class="python-input" placeholder="EMP001" style="background: transparent; border: none; padding: 0.75rem 0; color: #e2e8f0; width: 100%; outline: none;">
                    </div>
                </div>

                <!-- PASSWORD -->
                <div style="margin-bottom: 1.25rem;">
                    <label style="font-size: 0.75rem; font-weight: 700; color: #94a3b8; display: block; margin-bottom: 0.25rem;">PASSWORD</label>
                    <div style="background: #2d3748; border: 1px solid #4a5568; border-radius: 8px; display: flex; align-items: center; padding: 0 0.75rem;">
                        <span style="color: #a29bfe; margin-right: 0.5rem;">🔒</span>
                        <input type="password" id="no-cam-password" class="python-input" placeholder="••••" style="background: transparent; border: none; padding: 0.75rem 0; color: #e2e8f0; width: 100%; outline: none;">
                        <button type="button" onclick="togglePasswordVisibility()" style="background: none; border: none; color: #94a3b8; cursor: pointer;">👁️</button>
                    </div>
                </div>

                <!-- LOGIN BUTTON -->
                <button class="btn-primary-action" style="background: #3b82f6; color: #fff; padding: 0.85rem; border-radius: 8px; font-weight: 700; font-size: 1rem;" onclick="submitNoCameraLogin()">
                    LOGIN
                </button>
            </div>
        </div>
    </div>

    <!-- MODAL 3: QR CODE LOGIN (attendance_v8.py line 6222 Parity) -->
    <div id="modal-qr" class="modal-overlay">
        <div class="modal-dialog" style="max-width: 440px;">
            <div class="modal-header">
                <div class="modal-title"><span>📲</span> <span>In & Out with QR Code (Online)</span></div>
                <button class="modal-close" onclick="closeQrLogin()">&times;</button>
            </div>
            <div class="modal-body" style="text-align: center;">
                <p style="color: var(--text-secondary); margin-bottom: 1rem;">
                    AndroidIDReceiver অথবা QR কোড পাঞ্চ
                </p>
                <div style="background: #fff; padding: 1.5rem; display: inline-block; border-radius: 12px; margin-bottom: 1rem;">
                    <div style="font-size: 4rem;">📲</div>
                </div>
                <input type="text" id="qr-device-id" class="form-input" placeholder="Android ID (যেমন: ANDROID_A001)">
                <button class="btn-primary-action" style="background: linear-gradient(135deg, #74b9ff, #81ecec); color: #0b3459;" onclick="submitQrLogin()">
                    VERIFY QR / ANDROID ID
                </button>
            </div>
        </div>
    </div>

    <!-- MODAL 4: 7-DAY ATTENDANCE HISTORY (attendance_v8.py line 6938 Parity) -->
    <div id="modal-seven-days" class="modal-overlay">
        <div class="modal-dialog" style="max-width: 650px; background: #b8b8f3;">
            <div style="background: #4a6fa5; height: 70px; display: flex; justify-content: space-between; align-items: center; padding: 0 1.5rem;">
                <span id="history-header-title" style="color: #fff; font-size: 1.15rem; font-weight: 700;">7 Day Attendance History</span>
                <span id="history-header-date" style="color: #fff; font-size: 0.85rem;">Generated on: --</span>
            </div>
            <div style="background: #f5f5f7; padding: 1.5rem;">
                <div style="background: #fff; border: 1px solid #e0e0e0; border-radius: 8px; overflow: hidden;">
                    <table style="width: 100%; border-collapse: collapse; text-align: left; font-size: 0.85rem;">
                        <thead>
                            <tr style="background: #4a6fa5; color: #fff;">
                                <th style="padding: 10px 15px;">Date</th>
                                <th style="padding: 10px 15px;">In Time</th>
                                <th style="padding: 10px 15px;">Out Time</th>
                                <th style="padding: 10px 15px;">Duration</th>
                            </tr>
                        </thead>
                        <tbody id="seven-day-tbody">
                            <tr><td colspan="4" style="text-align: center; padding: 1rem; color: #666;">লোডিং...</td></tr>
                        </tbody>
                    </table>
                </div>
                <div style="margin-top: 1rem; text-align: right;">
                    <button style="background: #dc3545; color: #fff; border: none; padding: 8px 24px; font-weight: 700; border-radius: 6px; cursor: pointer;" onclick="closeSevenDayHistory()">Exit</button>
                </div>
            </div>
        </div>
    </div>

    <!-- MODAL 5: HR MANAGEMENT & ADMIN PORTAL -->
    <div id="modal-hr" class="modal-overlay">
        <div class="modal-dialog" style="max-width: 720px;">
            <div class="modal-header">
                <div class="modal-title"><span>👔</span> <span>HR Management & Admin Portal</span></div>
                <button class="modal-close" onclick="closeHrManagement()">&times;</button>
            </div>
            <div class="modal-body">
                <div id="hr-lock-screen">
                    <p style="color: var(--text-secondary); margin-bottom: 1rem;">
                        এইচআর প্যানেলে প্রবেশের জন্য অ্যাডমিন পিন কোড দিন:
                    </p>
                    <input type="password" id="hr-admin-pin" class="form-input" placeholder="ডিফল্ট পিন: 1234">
                    <button class="btn-primary-action" onclick="unlockHrManagement()">লগইন করুন</button>
                </div>
                <div id="hr-dashboard" style="display: none;">
                    <div style="display: flex; gap: 0.5rem; margin-bottom: 1rem; border-bottom: 1px solid var(--border-color); padding-bottom: 0.5rem;">
                        <button class="badge badge-success" onclick="loadHrTab('emp')">👥 কর্মকর্তা তালিকা (Employees)</button>
                        <button class="badge badge-warning" onclick="loadHrTab('rep')">📄 মাস্টার রোল রিপোর্ট (Muster Roll)</button>
                        <button class="badge badge-danger" onclick="loadHrTab('bq')">☁️ বিগকোয়েরি আউটবক্স (BigQuery)</button>
                    </div>
                    <div id="hr-content-area" style="max-height: 350px; overflow-y: auto;">
                        লোডিং...
                    </div>
                </div>
            </div>
        </div>
    </div>

    <script>{}</script>
</body>
</html>"#,
        css, js
    )
}
