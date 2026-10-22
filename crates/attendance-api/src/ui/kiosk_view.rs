use crate::ui::scripts::get_kiosk_js;
use crate::ui::styles::get_global_css;

pub fn render_full_portal_html() -> String {
    let css = get_global_css();
    let js = get_kiosk_js();

    format!(
        r##"<!DOCTYPE html>
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
        <div style="display: flex; align-items: center; gap: 1.5rem;">
            <button onclick="openCompanyManagement()" style="background: rgba(108, 92, 231, 0.2); border: 1px solid rgba(162, 155, 254, 0.4); color: #fff; padding: 0.5rem 1rem; border-radius: 10px; cursor: pointer; font-weight: 700; font-size: 0.85rem; display: flex; align-items: center; gap: 0.5rem;">
                <span>🏢</span>
                <span id="header-company-name">Company: Loading...</span>
            </button>
            <div class="clock-wrapper">
                <div id="digital-clock" class="clock-digital">00:00:00</div>
                <div id="dhaka-date" class="clock-sub">Asia/Dhaka</div>
            </div>
        </div>
    </header>

    <!-- TWO-COLUMN MAIN LAYOUT (attendance_v8.py line 6060 Parity) -->
    <main class="main-layout">
        <!-- LEFT PANEL -->
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
                        <span class="btn-text">In &amp; Out with Camera</span>
                    </button>
                    <button class="btn-action-gradient btn-no-camera" onclick="openNoCameraLogin()">
                        <span class="btn-icon">🚪</span>
                        <span class="btn-text">In &amp; Out without Camera</span>
                    </button>
                    <button class="btn-action-gradient btn-qr" onclick="openQrLogin()">
                        <span class="btn-icon">📲</span>
                        <span class="btn-text">In &amp; Out with QR Code (Online)</span>
                    </button>
                </div>

                <!-- FLOATING HR BUTTON & COMPANY INFO BUTTON (attendance_v8.py lines 6315 & 10366) -->
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 0.65rem; margin-top: 0.75rem;">
                    <button class="btn-hr-floating" style="margin-top: 0;" onclick="openHrManagement()">
                        <span>👥</span>
                        <span>HR MANAGEMENT</span>
                    </button>
                    <button class="btn-hr-floating" style="margin-top: 0; background: linear-gradient(135deg, rgba(0, 184, 148, 0.3), rgba(0, 210, 211, 0.2)); border-color: rgba(0, 210, 211, 0.45);" onclick="openCompanyManagement()">
                        <span>🏢</span>
                        <span>COMPANY INFO.</span>
                    </button>
                </div>
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
                            <span class="status-card-title">AI Models (ONNX)</span>
                        </div>
                        <div class="status-card-progress"><div class="status-card-progress-bar"></div></div>
                        <span class="status-card-state" id="st-ai">FaceNet512 + MTCNN ✓</span>
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
                        <button class="btn-tool" style="background: #5E0037; width: auto; padding: 0 10px; font-size: 0.8rem; font-weight: 700; gap: 4px;" title="7 day's attendance" onclick="openSevenDayHistory()">📈 7 Days</button>
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

    <!-- ========================================================================= -->
    <!-- MODAL 1A: CAMERA PREVIEW WINDOW (attendance_v8.py lines 11782-11840)      -->
    <!-- ========================================================================= -->
    <div id="modal-camera" class="modal-overlay">
        <div class="modal-dialog" style="max-width: 660px;">
            <div class="modal-header">
                <div class="modal-title"><span>📷</span> <span>Camera Preview</span></div>
                <button class="modal-close" onclick="closeCameraLogin()">&times;</button>
            </div>
            <div class="modal-body" style="text-align: center;">
                <div class="camera-wrapper" style="height: 380px;">
                    <video id="camera-stream" class="camera-video" autoplay playsinline muted style="transform: scaleX(-1);"></video>
                    <canvas id="camera-canvas" width="640" height="480" style="display:none;"></canvas>
                    <div class="camera-overlay-box"></div>
                </div>
                <div id="camera-status" style="font-size: 0.95rem; font-weight: 600; color: #00b894; margin-bottom: 1.25rem;">
                    Camera Ready - Click Capture when ready
                </div>
                <button id="btn-capture" class="btn-python-capture" onclick="executeCaptureImage()">
                    CAPTURE
                </button>
            </div>
        </div>
    </div>

    <!-- ========================================================================= -->
    <!-- MODAL 1B: IDENTITY VERIFICATION SPLIT WINDOW (attendance_v8.py line 1644) -->
    <!-- ========================================================================= -->
    <div id="modal-identity-verify" class="modal-overlay">
        <div class="modal-dialog" style="max-width: 840px; background: #000000; border: 2px solid #333; padding: 16px;">
            <div style="display: grid; grid-template-columns: 3fr 2fr; gap: 16px;">
                <!-- LEFT PANEL: IDENTITY VERIFICATION & MATCH ANALYSIS -->
                <div style="background: #ffffff; color: #212529; padding: 22px; border-radius: 8px;">
                    <h3 style="font-size: 1.25rem; font-weight: 800; color: #212529; margin-bottom: 14px;">IDENTITY VERIFICATION</h3>
                    <div style="text-align: center; margin-bottom: 14px;">
                        <img id="verify-captured-img" src="" alt="Captured Face" style="width: 200px; height: 200px; object-fit: cover; border-radius: 6px; border: 3px solid #f0f0f0; box-shadow: 0 4px 10px rgba(0,0,0,0.12);" />
                    </div>

                    <!-- MATCH ANALYSIS CARD -->
                    <div style="border: 1px solid #dee2e6; border-radius: 6px; overflow: hidden; margin-bottom: 12px;">
                        <div style="background: #f1f8ff; padding: 8px 14px; font-weight: 700; color: #0d6efd; font-size: 0.85rem;">
                            MATCH ANALYSIS (FaceNet512 ONNX + MTCNN)
                        </div>
                        <div style="padding: 12px 14px; font-size: 0.85rem;">
                            <div style="display: flex; justify-content: space-between; margin-bottom: 4px;">
                                <strong>Match Quality:</strong>
                                <span id="verify-match-quality" style="font-weight: 800; color: #198754;">EXACT</span>
                            </div>
                            <div style="display: flex; justify-content: space-between; margin-bottom: 4px;">
                                <strong>Confidence Score:</strong>
                                <span id="verify-confidence">98.50%</span>
                            </div>
                            <div style="background: #e9ecef; height: 8px; border-radius: 4px; overflow: hidden; margin-bottom: 8px;">
                                <div id="verify-conf-bar" style="background: #198754; height: 100%; width: 98%;"></div>
                            </div>
                            <div style="display: flex; justify-content: space-between; color: #495057; margin-bottom: 2px;">
                                <span>Minimum Distance:</span> <span id="verify-min-dist">0.1200</span>
                            </div>
                            <div style="display: flex; justify-content: space-between; color: #495057; margin-bottom: 2px;">
                                <span>Exact / Strong Matches:</span> <span id="verify-exact-strong">1 / 0</span>
                            </div>
                            <div style="display: flex; justify-content: space-between; color: #495057;">
                                <span>Total Indexed Matches:</span> <span id="verify-total-matches">1</span>
                            </div>
                        </div>
                    </div>

                    <!-- ATTENDANCE RECORD CARD -->
                    <div style="border: 1px solid #dee2e6; border-radius: 6px; overflow: hidden;">
                        <div style="background: #f8f9fa; padding: 8px 14px; font-weight: 700; color: #212529; font-size: 0.85rem;">
                            ATTENDANCE RECORD
                        </div>
                        <div style="padding: 12px 14px;">
                            <div style="font-size: 0.72rem; color: #6c757d;">USER</div>
                            <div id="verify-user-display" style="font-size: 1.25rem; font-weight: 800; color: #212529; margin-bottom: 8px;">---</div>
                            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px; border-top: 1px solid #dee2e6; padding-top: 8px; font-size: 0.85rem;">
                                <div>
                                    <div style="font-size: 0.7rem; color: #6c757d;">TYPE</div>
                                    <strong id="verify-type-display">IN_TIME</strong>
                                </div>
                                <div>
                                    <div style="font-size: 0.7rem; color: #6c757d;">TIME</div>
                                    <strong id="verify-time-display">--:--</strong>
                                </div>
                                <div style="grid-column: span 2;">
                                    <div style="font-size: 0.7rem; color: #6c757d;">DATE</div>
                                    <strong id="verify-date-display">--</strong>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- RIGHT PANEL: CONFIRM IDENTITY & 5-SEC COUNTDOWN -->
                <div style="background: #ffffff; color: #212529; padding: 22px; border-radius: 8px; display: flex; flex-direction: column; justify-content: space-between;">
                    <div>
                        <h3 style="font-size: 1.25rem; font-weight: 800; color: #212529; margin-bottom: 16px;">CONFIRM IDENTITY</h3>
                        <div id="verify-countdown-label" style="font-size: 1rem; font-weight: 700; color: #0d6efd; text-align: center; padding: 12px; background: #f1f8ff; border-radius: 6px; margin-bottom: 20px;">
                            Auto-verification in: 5 seconds
                        </div>
                        <div id="verify-question-label" style="font-size: 1.2rem; font-weight: 800; text-align: center; color: #212529; margin-bottom: 24px;">
                            Are you ---?
                        </div>
                    </div>

                    <div style="display: flex; flex-direction: column; gap: 10px;">
                        <button id="btn-verify-yes" onclick="confirmIdentityAndPunch()" style="background: #198754; color: #fff; border: none; padding: 12px; border-radius: 6px; font-weight: 700; font-size: 0.95rem; cursor: pointer;">
                            Yes, I am ---
                        </button>
                        <button onclick="denyIdentityAndRetry()" style="background: #dc3545; color: #fff; border: none; padding: 12px; border-radius: 6px; font-weight: 700; font-size: 0.95rem; cursor: pointer;">
                            No, I'm someone else
                        </button>
                        <button onclick="confirmIdentityAndPunch()" style="background: #0d6efd; color: #fff; border: none; padding: 12px; border-radius: 6px; font-weight: 700; font-size: 0.95rem; cursor: pointer;">
                            Done
                        </button>
                        <button onclick="closeIdentityVerifyModal()" style="background: #6c757d; color: #fff; border: none; padding: 12px; border-radius: 6px; font-weight: 700; font-size: 0.95rem; cursor: pointer;">
                            Return to Main Window
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- ========================================================================= -->
    <!-- MODAL 2: IN & OUT WITHOUT CAMERA (attendance_v8.py lines 8295-8834)       -->
    <!-- ========================================================================= -->
    <div id="modal-no-camera" class="modal-overlay">
        <div class="modal-dialog" style="max-width: 440px; background: #1e293b;">
            <div class="modal-header" style="background: #1e293b; border-bottom: none; padding-bottom: 0;">
                <div class="modal-title" style="font-size: 1rem;"><span>🚪</span> <span>Login</span></div>
                <button class="modal-close" onclick="closeNoCameraLogin()">&times;</button>
            </div>
            <div class="modal-body" style="background: #1e293b; padding-top: 0.5rem;">
                <div style="text-align: center; margin-bottom: 1.25rem;">
                    <div style="width: 64px; height: 64px; border-radius: 50%; background: #2d3748; border: 2px solid #6c5ce7; margin: 0 auto 0.75rem auto; display: flex; align-items: center; justify-content: center; font-size: 1.6rem;">
                        👤
                    </div>
                    <h3 style="font-size: 1.3rem; font-weight: 700; color: #f1f5f9;">In &amp; Out without Camera</h3>
                    <p style="font-size: 0.85rem; color: #94a3b8;">Please enter your credentials to continue</p>
                </div>

                <div id="no-cam-error" style="display: none; background: rgba(231, 76, 60, 0.2); border: 1px solid #e74c3c; color: #ff7675; padding: 0.6rem; border-radius: 8px; margin-bottom: 1rem; font-size: 0.85rem; text-align: center;"></div>

                <!-- USERNAME -->
                <div style="margin-bottom: 1rem;">
                    <label style="font-size: 0.75rem; font-weight: 700; color: #94a3b8; display: block; margin-bottom: 0.25rem;">USERNAME</label>
                    <div style="background: #2d3748; border: 1px solid #4a5568; border-radius: 8px; display: flex; align-items: center; padding: 0 0.75rem;">
                        <span style="color: #a29bfe; margin-right: 0.5rem;">👤</span>
                        <input type="text" id="no-cam-username" placeholder="Enter Username (e.g. RAHIM)" style="background: transparent; border: none; padding: 0.75rem 0; color: #e2e8f0; width: 100%; outline: none; text-transform: uppercase;">
                    </div>
                </div>

                <!-- PASSWORD -->
                <div style="margin-bottom: 0.75rem;">
                    <label style="font-size: 0.75rem; font-weight: 700; color: #94a3b8; display: block; margin-bottom: 0.25rem;">PASSWORD</label>
                    <div style="background: #2d3748; border: 1px solid #4a5568; border-radius: 8px; display: flex; align-items: center; padding: 0 0.75rem;">
                        <span style="color: #a29bfe; margin-right: 0.5rem;">🔒</span>
                        <input type="password" id="no-cam-password" placeholder="Enter Password" style="background: transparent; border: none; padding: 0.75rem 0; color: #e2e8f0; width: 100%; outline: none;">
                    </div>
                </div>

                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.25rem; font-size: 0.82rem; color: #94a3b8;">
                    <label style="cursor: pointer; display: flex; align-items: center; gap: 6px;">
                        <input type="checkbox" id="no-cam-show-pw" onchange="togglePasswordVisibility()"> Show password
                    </label>
                    <a href="javascript:void(0)" onclick="showForgotPasswordPopup()" style="color: #a29bfe; text-decoration: none;">Forgot password?</a>
                </div>

                <!-- LOGIN BUTTON -->
                <button id="btn-no-cam-login" class="btn-primary-action" style="background: #6c5ce7; color: #fff; padding: 0.85rem; border-radius: 8px; font-weight: 700; font-size: 1rem;" onclick="submitNoCameraLogin()">
                    LOGIN
                </button>
            </div>
        </div>
    </div>

    <!-- ========================================================================= -->
    <!-- MODAL 3: QR CODE LOGIN (attendance_v8.py lines 5738 & 8853)               -->
    <!-- ========================================================================= -->
    <div id="modal-qr" class="modal-overlay">
        <div class="modal-dialog" style="max-width: 500px;">
            <div class="modal-header">
                <div class="modal-title"><span>📲</span> <span>QR Code Login (AndroidIDReceiver)</span></div>
                <button class="modal-close" onclick="closeQrLogin()">&times;</button>
            </div>
            <div class="modal-body" style="text-align: center;">
                <p style="color: var(--text-secondary); margin-bottom: 0.75rem; font-size: 0.9rem;">
                    স্মার্টফোন দিয়ে নিচের QR কোডটি স্ক্যান করুন অথবা সরাসরি নিচের বক্সে আপনার রেজিস্টার্ড Android Device ID দিন:
                </p>
                <div style="background: #ffffff; padding: 1rem; display: inline-block; border-radius: 12px; margin-bottom: 0.75rem;">
                    <div id="qr-login-box" style="width: 180px; height: 180px; display: flex; align-items: center; justify-content: center;"></div>
                </div>
                <div id="qr-session-status" style="font-size: 0.82rem; color: #00d2d3; margin-bottom: 0.85rem;">
                    Waiting for mobile scan...
                </div>
                <input type="text" id="qr-device-id" class="form-input" placeholder="Registered Android ID (যেমন: ANDR_DEV001)">
                <button class="btn-primary-action" style="background: linear-gradient(135deg, #74b9ff, #81ecec); color: #0b3459;" onclick="submitQrLogin()">
                    VERIFY QR / ANDROID ID
                </button>
            </div>
        </div>
    </div>

    <!-- ========================================================================= -->
    <!-- MODAL 4: 7-DAY ATTENDANCE HISTORY (attendance_v8.py line 6938 Parity)     -->
    <!-- ========================================================================= -->
    <div id="modal-seven-days" class="modal-overlay">
        <div class="modal-dialog" style="max-width: 680px; background: #b8b8f3;">
            <div style="background: #4a6fa5; height: 68px; display: flex; justify-content: space-between; align-items: center; padding: 0 1.5rem;">
                <span id="history-header-title" style="color: #fff; font-size: 1.15rem; font-weight: 700;">7 Day Attendance History</span>
                <span id="history-header-date" style="color: #fff; font-size: 0.85rem;">Generated on: --</span>
            </div>
            <div style="background: #f5f5f7; padding: 1.25rem;">
                <div style="background: #fff; border: 1px solid #e0e0e0; border-radius: 8px; overflow: hidden;">
                    <table style="width: 100%; border-collapse: collapse; text-align: left; font-size: 0.85rem; color: #222;">
                        <thead>
                            <tr style="background: #4a6fa5; color: #fff;">
                                <th style="padding: 10px 15px;">Date</th>
                                <th style="padding: 10px 15px;">In Time</th>
                                <th style="padding: 10px 15px;">Out Time</th>
                                <th style="padding: 10px 15px;">Duration</th>
                            </tr>
                        </thead>
                        <tbody id="seven-day-tbody">
                            <tr><td colspan="4" style="text-align: center; padding: 1rem; color: #666;">Loading...</td></tr>
                        </tbody>
                    </table>
                </div>
                <div style="margin-top: 1rem; text-align: right;">
                    <button style="background: #dc3545; color: #fff; border: none; padding: 8px 24px; font-weight: 700; border-radius: 6px; cursor: pointer;" onclick="closeSevenDayHistory()">Exit</button>
                </div>
            </div>
        </div>
    </div>

    <!-- ========================================================================= -->
    <!-- MODAL 5: COMPANY MANAGEMENT (attendance_v8.py lines 10366-11600 Parity)   -->
    <!-- ========================================================================= -->
    <div id="modal-company" class="modal-overlay">
        <div class="modal-dialog" style="max-width: 560px;">
            <div class="modal-header">
                <div class="modal-title"><span>🏢</span> <span>Company Management</span></div>
                <button class="modal-close" onclick="closeCompanyManagement()">&times;</button>
            </div>
            <div class="modal-body">
                <!-- SUB-VIEW 1: 3 MAIN BUTTONS (lines 10405-10445) -->
                <div id="company-view-menu" style="text-align: center;">
                    <div style="background: rgba(108, 92, 231, 0.15); border: 1px solid rgba(162, 155, 254, 0.3); padding: 0.85rem; border-radius: 10px; margin-bottom: 1.25rem;">
                        <div style="font-size: 0.78rem; color: #a29bfe;">CURRENT ACTIVE COMPANY</div>
                        <div id="company-active-banner" style="font-size: 1.15rem; font-weight: 800; color: #fff; margin-top: 4px;">---</div>
                    </div>
                    <div style="display: flex; flex-direction: column; gap: 0.85rem;">
                        <button class="btn-primary-action" style="background: #1e3a8a; color: #fff; padding: 1rem; font-size: 1rem;" onclick="showCompanySubView('select')">
                            📋 Select An Existing Company
                        </button>
                        <button class="btn-primary-action" style="background: #047857; color: #fff; padding: 1rem; font-size: 1rem;" onclick="showCompanySubView('create')">
                            ➕ Create a New Company
                        </button>
                        <button class="btn-primary-action" style="background: #b45309; color: #fff; padding: 1rem; font-size: 1rem;" onclick="showCompanySubView('edit')">
                            ✏️ Edit Existing Company
                        </button>
                    </div>
                </div>

                <!-- SUB-VIEW 2: SELECT EXISTING COMPANY (lines 10453-10865) -->
                <div id="company-view-select" style="display: none;">
                    <h4 style="margin-bottom: 0.75rem; color: #a29bfe;">Select An Existing Company</h4>
                    <div style="display: flex; gap: 0.5rem; margin-bottom: 0.75rem;">
                        <input type="text" id="comp-select-search" class="form-input" style="margin-bottom: 0;" placeholder="Search company name...">
                        <button onclick="loadCompanyListForSelect()" style="background: #3b82f6; color: #fff; border: none; padding: 0 1rem; border-radius: 8px; font-weight: 700; cursor: pointer;">Search</button>
                    </div>
                    <select id="comp-select-listbox" size="6" style="width: 100%; background: #10101c; color: #fff; border: 1px solid rgba(255,255,255,0.15); border-radius: 8px; padding: 0.5rem; margin-bottom: 0.85rem; font-size: 0.95rem;"></select>
                    <input type="password" id="comp-select-password" class="form-input" placeholder="Enter company password to verify &amp; select">
                    <div id="comp-select-status" style="color: #ff7675; font-size: 0.85rem; margin-bottom: 0.75rem;"></div>
                    <div style="display: flex; gap: 0.75rem;">
                        <button class="btn-primary-action" style="background: #28a745; color: #fff;" onclick="verifyAndSelectCompany()">Verify &amp; Select Company</button>
                        <button class="btn-primary-action" style="background: #6c757d; color: #fff;" onclick="showCompanySubView('menu')">Back</button>
                    </div>
                </div>

                <!-- SUB-VIEW 3: CREATE NEW COMPANY (lines 10867-11101 - ALL 7 FIELDS) -->
                <div id="company-view-create" style="display: none;">
                    <h4 style="margin-bottom: 0.75rem; color: #00b894;">Create New Company</h4>
                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 0.65rem;">
                        <div style="grid-column: span 2;">
                            <label style="font-size: 0.78rem; color: #dfe6e9;">Company Name *</label>
                            <input type="text" id="comp-new-name" class="form-input" style="margin-bottom: 0.25rem; padding: 0.6rem;" placeholder="e.g. TechFirm Bangladesh Ltd.">
                        </div>
                        <div style="grid-column: span 2;">
                            <label style="font-size: 0.78rem; color: #dfe6e9;">Password *</label>
                            <input type="password" id="comp-new-password" class="form-input" style="margin-bottom: 0.25rem; padding: 0.6rem;" placeholder="Company Password">
                        </div>
                        <div>
                            <label style="font-size: 0.78rem; color: #dfe6e9;">Address Road/House</label>
                            <input type="text" id="comp-new-road" class="form-input" style="margin-bottom: 0.25rem; padding: 0.6rem;" placeholder="Road / House">
                        </div>
                        <div>
                            <label style="font-size: 0.78rem; color: #dfe6e9;">Address City</label>
                            <input type="text" id="comp-new-city" class="form-input" style="margin-bottom: 0.25rem; padding: 0.6rem;" placeholder="City (e.g. Dhaka)">
                        </div>
                        <div>
                            <label style="font-size: 0.78rem; color: #dfe6e9;">Contact Person Name</label>
                            <input type="text" id="comp-new-contact-name" class="form-input" style="margin-bottom: 0.25rem; padding: 0.6rem;" placeholder="Full Name">
                        </div>
                        <div>
                            <label style="font-size: 0.78rem; color: #dfe6e9;">Contact Person Designation</label>
                            <input type="text" id="comp-new-contact-desig" class="form-input" style="margin-bottom: 0.25rem; padding: 0.6rem;" placeholder="Designation">
                        </div>
                        <div style="grid-column: span 2;">
                            <label style="font-size: 0.78rem; color: #dfe6e9;">Contact Person Number</label>
                            <input type="text" id="comp-new-contact-num" class="form-input" style="margin-bottom: 0.5rem; padding: 0.6rem;" placeholder="Phone Number">
                        </div>
                    </div>
                    <div id="comp-create-status" style="color: #ff7675; font-size: 0.85rem; margin-bottom: 0.75rem;"></div>
                    <div style="display: flex; gap: 0.75rem;">
                        <button class="btn-primary-action" style="background: #28a745; color: #fff;" onclick="submitCreateCompany()">Create Company</button>
                        <button class="btn-primary-action" style="background: #dc3545; color: #fff;" onclick="showCompanySubView('menu')">Cancel</button>
                    </div>
                </div>

                <!-- SUB-VIEW 4: EDIT EXISTING COMPANY (lines 11104-11525) -->
                <div id="company-view-edit" style="display: none;">
                    <h4 style="margin-bottom: 0.75rem; color: #fdcb6e;">Edit Existing Company</h4>
                    <!-- Step 1: Select & Verify Password -->
                    <div id="comp-edit-step1">
                        <div style="display: flex; gap: 0.5rem; margin-bottom: 0.65rem;">
                            <input type="text" id="comp-edit-search" class="form-input" style="margin-bottom: 0;" placeholder="Search company to edit...">
                            <button onclick="loadCompanyListForEdit()" style="background: #3b82f6; color: #fff; border: none; padding: 0 1rem; border-radius: 8px; font-weight: 700; cursor: pointer;">Search</button>
                        </div>
                        <select id="comp-edit-listbox" size="5" style="width: 100%; background: #10101c; color: #fff; border: 1px solid rgba(255,255,255,0.15); border-radius: 8px; padding: 0.5rem; margin-bottom: 0.75rem; font-size: 0.95rem;"></select>
                        <input type="password" id="comp-edit-verify-pw" class="form-input" placeholder="Enter company password to edit">
                        <div id="comp-edit-verify-status" style="color: #ff7675; font-size: 0.85rem; margin-bottom: 0.75rem;"></div>
                        <div style="display: flex; gap: 0.75rem;">
                            <button class="btn-primary-action" style="background: #e67e22; color: #fff;" onclick="verifyCompanyForEdit()">Verify &amp; Edit</button>
                            <button class="btn-primary-action" style="background: #6c757d; color: #fff;" onclick="showCompanySubView('menu')">Back</button>
                        </div>
                    </div>
                    <!-- Step 2: Edit 7 Fields -->
                    <div id="comp-edit-step2" style="display: none;">
                        <input type="hidden" id="comp-edit-id">
                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 0.65rem;">
                            <div style="grid-column: span 2;">
                                <label style="font-size: 0.78rem; color: #dfe6e9;">Company Name *</label>
                                <input type="text" id="comp-edit-name" class="form-input" style="margin-bottom: 0.25rem; padding: 0.6rem;">
                            </div>
                            <div style="grid-column: span 2;">
                                <label style="font-size: 0.78rem; color: #dfe6e9;">Password *</label>
                                <input type="password" id="comp-edit-password" class="form-input" style="margin-bottom: 0.25rem; padding: 0.6rem;">
                            </div>
                            <div>
                                <label style="font-size: 0.78rem; color: #dfe6e9;">Address Road/House</label>
                                <input type="text" id="comp-edit-road" class="form-input" style="margin-bottom: 0.25rem; padding: 0.6rem;">
                            </div>
                            <div>
                                <label style="font-size: 0.78rem; color: #dfe6e9;">Address City</label>
                                <input type="text" id="comp-edit-city" class="form-input" style="margin-bottom: 0.25rem; padding: 0.6rem;">
                            </div>
                            <div>
                                <label style="font-size: 0.78rem; color: #dfe6e9;">Contact Person Name</label>
                                <input type="text" id="comp-edit-contact-name" class="form-input" style="margin-bottom: 0.25rem; padding: 0.6rem;">
                            </div>
                            <div>
                                <label style="font-size: 0.78rem; color: #dfe6e9;">Contact Person Designation</label>
                                <input type="text" id="comp-edit-contact-desig" class="form-input" style="margin-bottom: 0.25rem; padding: 0.6rem;">
                            </div>
                            <div style="grid-column: span 2;">
                                <label style="font-size: 0.78rem; color: #dfe6e9;">Contact Person Number</label>
                                <input type="text" id="comp-edit-contact-num" class="form-input" style="margin-bottom: 0.5rem; padding: 0.6rem;">
                            </div>
                        </div>
                        <div id="comp-edit-save-status" style="color: #ff7675; font-size: 0.85rem; margin-bottom: 0.75rem;"></div>
                        <div style="display: flex; gap: 0.75rem;">
                            <button class="btn-primary-action" style="background: #28a745; color: #fff;" onclick="submitUpdateCompany()">Update Company</button>
                            <button class="btn-primary-action" style="background: #dc3545; color: #fff;" onclick="showCompanySubView('menu')">Cancel</button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- ========================================================================= -->
    <!-- MODAL 6: HR AUTHENTICATION & CHANGE PASSWORD (attendance_v8.py line 9819) -->
    <!-- ========================================================================= -->
    <div id="modal-hr-auth" class="modal-overlay">
        <div class="modal-dialog" style="max-width: 440px;">
            <div class="modal-header">
                <div class="modal-title"><span>🔐</span> <span>HR Authentication</span></div>
                <button class="modal-close" onclick="closeHrAuthModal()">&times;</button>
            </div>
            <div class="modal-body">
                <!-- LOGIN VIEW -->
                <div id="hr-auth-login-view">
                    <div style="background: rgba(59, 130, 246, 0.15); border-left: 4px solid #3b82f6; padding: 0.75rem 1rem; border-radius: 6px; margin-bottom: 1rem;">
                        <div style="font-size: 0.75rem; color: #94a3b8; font-weight: 700;">COMPANY:</div>
                        <div id="hr-auth-company-label" style="font-size: 1.05rem; font-weight: 800; color: #60a5fa;">Smart Attendance HQ</div>
                    </div>
                    <label style="font-size: 0.85rem; font-weight: 700; display: block; margin-bottom: 0.4rem;">Enter HR Password:</label>
                    <input type="password" id="hr-admin-pin" class="form-input" placeholder="Default: HR@123 (or 1234)">
                    <div id="hr-auth-status" style="color: #ff7675; font-size: 0.85rem; margin-bottom: 0.75rem; min-height: 1.2rem;"></div>
                    <div style="display: grid; grid-template-columns: 1fr 1.2fr 1fr; gap: 0.5rem;">
                        <button onclick="unlockHrManagement()" style="background: #007bff; color: #fff; border: none; padding: 0.75rem; border-radius: 8px; font-weight: 700; cursor: pointer;">Login</button>
                        <button onclick="toggleHrChangePasswordView(true)" style="background: #ffc107; color: #000; border: none; padding: 0.75rem; border-radius: 8px; font-weight: 700; cursor: pointer;">Change Password</button>
                        <button onclick="closeHrAuthModal()" style="background: #6c757d; color: #fff; border: none; padding: 0.75rem; border-radius: 8px; font-weight: 700; cursor: pointer;">Cancel</button>
                    </div>
                </div>

                <!-- CHANGE HR PASSWORD VIEW (attendance_v8.py lines 10000-10143) -->
                <div id="hr-auth-change-view" style="display: none;">
                    <h4 style="margin-bottom: 0.5rem; color: #ffc107;">Change HR Password</h4>
                    <p id="hr-change-comp-sub" style="font-size: 0.82rem; color: #60a5fa; margin-bottom: 0.85rem;"></p>
                    <label style="font-size: 0.78rem; font-weight: 700;">Current Password:</label>
                    <input type="password" id="hr-cp-current" class="form-input" style="padding: 0.65rem;" placeholder="Current HR Password">
                    <label style="font-size: 0.78rem; font-weight: 700;">New Password:</label>
                    <input type="password" id="hr-cp-new" class="form-input" style="padding: 0.65rem;" placeholder="New HR Password">
                    <label style="font-size: 0.78rem; font-weight: 700;">Confirm New Password:</label>
                    <input type="password" id="hr-cp-confirm" class="form-input" style="padding: 0.65rem;" placeholder="Confirm New Password">
                    <div id="hr-cp-status" style="color: #ff7675; font-size: 0.85rem; margin-bottom: 0.75rem;"></div>
                    <div style="display: flex; gap: 0.75rem;">
                        <button class="btn-primary-action" style="background: #28a745; color: #fff;" onclick="submitChangeHrPassword()">OK</button>
                        <button class="btn-primary-action" style="background: #dc3545; color: #fff;" onclick="toggleHrChangePasswordView(false)">Cancel</button>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- ========================================================================= -->
    <!-- MODAL 7: HR MANAGEMENT PORTAL — ALL 12 PYTHON BUTTONS (line 10281 Parity) -->
    <!-- ========================================================================= -->
    <div id="modal-hr" class="modal-overlay">
        <div class="modal-dialog" style="max-width: 880px; max-height: 92vh; display: flex; flex-direction: column;">
            <div class="modal-header">
                <div class="modal-title"><span>👔</span> <span>HR Management (attendance_v8.py Complete Parity)</span></div>
                <button class="modal-close" onclick="closeHrManagement()">&times;</button>
            </div>
            <div class="modal-body" style="overflow-y: auto; flex: 1;">
                <!-- 12 HR MANAGEMENT BUTTONS GRID (lines 10308-10321) -->
                <div id="hr-main-menu-grid">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.85rem;">
                        <span style="font-size: 0.85rem; color: #a29bfe; font-weight: 700;">Select an HR Management Action (All 12 Python Functions):</span>
                        <button onclick="openHrAction('users-list')" style="background: rgba(0, 184, 148, 0.2); border: 1px solid #00b894; color: #00b894; padding: 4px 12px; border-radius: 6px; font-weight: 700; cursor: pointer; font-size: 0.8rem;">👥 View Registered Users</button>
                    </div>
                    <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 0.65rem;">
                        <button class="hr-menu-btn" onclick="openHrAction('reg-camera')">📷 1. Register with Camera (20-Pose ONNX)</button>
                        <button class="hr-menu-btn" onclick="openHrAction('reg-no-camera')">👤 2. Register without Camera</button>
                        <button class="hr-menu-btn" onclick="openHrAction('att-report')">📊 3. Attendance Report (Customized)</button>
                        <button class="hr-menu-btn" onclick="openHrAction('sync-local')">🔄 4. Sync Users to Local Database</button>
                        <button class="hr-menu-btn" onclick="openHrAction('verify-image')">🕵️ 5. Check Fake Attendance (Verify Image)</button>
                        <button class="hr-menu-btn" onclick="openHrAction('upload-model')">🧠 6. Upload AI Model (if Failure)</button>
                        <button class="hr-menu-btn" onclick="openHrAction('delete-face')">🗑️ 7. Delete Face (Reset Encodings)</button>
                        <button class="hr-menu-btn" onclick="openHrAction('embeddings-report')">🧬 8. Generate Facial Features Report</button>
                        <button class="hr-menu-btn" onclick="openHrAction('sync-cloud')">☁️ 9. Sync Users to Cloud Database</button>
                        <button class="hr-menu-btn" onclick="openHrAction('sync-hr-pw')">🔑 10. Sync HR Password to Local</button>
                        <button class="hr-menu-btn" onclick="openHrAction('reg-qr')">📲 11. Register New User With QR Code</button>
                        <button class="hr-menu-btn" style="background: linear-gradient(135deg, #00b894, #00d2d3); color: #0b2e25;" onclick="closeHrManagement(); openCompanyManagement();">🏢 12. Company Info. (Select / Create / Edit)</button>
                    </div>
                </div>

                <!-- HR ACTION WORKSPACE (Shown when any of the buttons is clicked) -->
                <div id="hr-action-workspace" style="display: none;">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; padding-bottom: 0.5rem; border-bottom: 1px solid rgba(255,255,255,0.1);">
                        <h4 id="hr-workspace-title" style="color: #00d2d3; font-size: 1.1rem;">Action</h4>
                        <button onclick="backToHrMainMenu()" style="background: #4b5563; color: #fff; border: none; padding: 6px 14px; border-radius: 6px; font-weight: 700; cursor: pointer;">← Back to HR Menu</button>
                    </div>
                    <div id="hr-workspace-body"></div>
                </div>
            </div>
        </div>
    </div>

    <script>{}</script>
</body>
</html>"##,
        css, js
    )
}
