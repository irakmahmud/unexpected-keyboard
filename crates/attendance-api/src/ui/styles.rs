pub fn get_global_css() -> &'static str {
    r##"
:root {
    --bg-main: #171721;
    --bg-darker: #10101c;
    --bg-card: rgba(30, 30, 46, 0.7);
    --border-color: rgba(255, 255, 255, 0.08);
    --accent-purple: #6c5ce7;
    --accent-purple-light: #a29bfe;
    --accent-green: #00b894;
    --accent-teal: #00d2d3;
    --accent-blue: #3498db;
    --accent-yellow: #fdcb6e;
    --accent-red: #e74c3c;
    --text-primary: #ffffff;
    --text-secondary: #dfe6e9;
    --text-muted: #636e72;
}

* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

body {
    font-family: 'Segoe UI', -apple-system, BlinkMacSystemFont, Roboto, sans-serif;
    background: radial-gradient(circle at 20% 20%, #1e1e30 0%, #171721 60%, #10101c 100%);
    color: var(--text-primary);
    min-height: 100vh;
    display: flex;
    flex-direction: column;
}

/* Floating Header */
.app-header {
    background: rgba(23, 23, 33, 0.85);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border-bottom: 1px solid var(--border-color);
    padding: 1.25rem 2.5rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    position: sticky;
    top: 0;
    z-index: 100;
}

.brand-wrapper {
    display: flex;
    align-items: center;
    gap: 1.25rem;
}

.brand-logo {
    width: 52px;
    height: 52px;
    border-radius: 14px;
    background: linear-gradient(135deg, #6c5ce7, #00d2d3);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.75rem;
    box-shadow: 0 0 20px rgba(108, 92, 231, 0.4);
}

.brand-title-box {
    display: flex;
    flex-direction: column;
}

.brand-main-title {
    font-size: 1.6rem;
    font-weight: 800;
    letter-spacing: 0.05em;
    color: #ffffff;
    line-height: 1.1;
}

.brand-subtitle {
    font-size: 0.78rem;
    font-weight: 600;
    letter-spacing: 0.15em;
    color: var(--accent-purple-light);
}

.clock-wrapper {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
}

.clock-digital {
    font-family: 'Consolas', 'Courier New', monospace;
    font-size: 1.6rem;
    font-weight: 700;
    color: #ffffff;
    letter-spacing: 0.05em;
    text-shadow: 0 0 10px rgba(255, 255, 255, 0.2);
}

.clock-sub {
    font-size: 0.8rem;
    color: var(--accent-purple-light);
}

/* Main Layout: 2 Columns */
.main-layout {
    display: flex;
    flex: 1;
    gap: 1.75rem;
    padding: 1.75rem 2.5rem;
    max-width: 1600px;
    margin: 0 auto;
    width: 100%;
}

.left-panel {
    flex: 0 0 460px;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
}

.right-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    min-width: 0;
}

@media (max-width: 1024px) {
    .app-header {
        padding: 1rem;
    }
    .main-layout {
        flex-direction: column;
        padding: 1rem;
    }
    .left-panel {
        flex: none;
        width: 100%;
    }
}

/* Glass Card */
.glass-panel {
    background: rgba(28, 28, 42, 0.6);
    backdrop-filter: blur(10px);
    -webkit-backdrop-filter: blur(10px);
    border: 1px solid var(--border-color);
    border-radius: 16px;
    padding: 1.25rem;
    box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.37);
}

/* Section Header */
.section-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 1rem;
}

.accent-bar {
    width: 28px;
    height: 4px;
    border-radius: 2px;
}

.section-title {
    font-size: 0.85rem;
    font-weight: 700;
    letter-spacing: 0.1em;
    color: #ffffff;
}

/* Status Grid (4 Cards) */
.status-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 0.75rem;
}

.status-card {
    background: rgba(20, 20, 32, 0.7);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 12px;
    padding: 0.85rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
}

.status-card-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.status-card-icon {
    font-size: 1.1rem;
}

.status-card-title {
    font-size: 0.75rem;
    font-weight: 700;
    color: #ffffff;
}

.status-card-progress {
    height: 3px;
    background: #2d2d4a;
    border-radius: 2px;
    overflow: hidden;
}

.status-card-progress-bar {
    height: 100%;
    background: var(--accent-purple);
    width: 100%;
}

.status-card-state {
    font-size: 0.75rem;
    color: var(--accent-green);
    font-weight: 600;
}

/* Action Buttons (Python Tkinter Parity) */
.action-buttons-box {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
}

.btn-action-gradient {
    height: 68px;
    border: none;
    border-radius: 14px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: flex-start;
    padding: 0 1.5rem;
    gap: 1.25rem;
    font-size: 1.05rem;
    font-weight: 700;
    color: #171721;
    transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
    box-shadow: 0 4px 15px rgba(0, 0, 0, 0.2);
    text-align: left;
    width: 100%;
}

.btn-action-gradient:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 25px rgba(0, 0, 0, 0.35);
}

.btn-action-gradient:active {
    transform: translateY(1px);
}

.btn-action-gradient .btn-icon {
    font-size: 1.8rem;
}

.btn-camera {
    background: linear-gradient(135deg, #00b894, #00d2d3);
    color: #0c392c;
}

.btn-no-camera {
    background: linear-gradient(135deg, #fdcb6e, #ffeaa7);
    color: #4a3809;
}

.btn-qr {
    background: linear-gradient(135deg, #74b9ff, #81ecec);
    color: #0b3459;
}

.btn-hr-floating {
    margin-top: 0.5rem;
    height: 54px;
    border: 1px solid rgba(108, 92, 231, 0.4);
    border-radius: 14px;
    background: linear-gradient(135deg, rgba(108, 92, 231, 0.3), rgba(162, 155, 254, 0.2));
    color: #ffffff;
    font-size: 0.95rem;
    font-weight: 700;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    transition: all 0.2s ease;
    width: 100%;
}

.btn-hr-floating:hover {
    background: linear-gradient(135deg, rgba(108, 92, 231, 0.5), rgba(162, 155, 254, 0.35));
    border-color: var(--accent-purple-light);
    transform: translateY(-2px);
}

/* System Console (Terminal Parity) */
.console-box {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 280px;
}

.console-header-controls {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
}

.console-action-btns {
    display: flex;
    gap: 0.5rem;
}

.btn-tool {
    width: 32px;
    height: 32px;
    border-radius: 8px;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1rem;
    color: #ffffff;
    transition: all 0.2s ease;
}

.btn-tool:hover {
    transform: scale(1.08);
}

.btn-export { background: #2ecc71; }
.btn-sync { background: #3498db; }
.btn-clear { background: #e74c3c; }

.terminal-window {
    background: #10101c;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    flex: 1;
}

.terminal-titlebar {
    background: #1e1e2e;
    height: 32px;
    padding: 0 1rem;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.mac-dots {
    display: flex;
    gap: 6px;
}

.dot {
    width: 11px;
    height: 11px;
    border-radius: 50%;
}

.dot-red { background: #ff5f56; }
.dot-yellow { background: #ffbd2e; }
.dot-green { background: #27c93f; }

.terminal-title {
    font-family: 'Consolas', monospace;
    font-size: 0.8rem;
    color: var(--accent-purple-light);
}

.terminal-body {
    font-family: 'Consolas', monospace;
    font-size: 0.85rem;
    line-height: 1.5;
    color: #dfe6e9;
    padding: 1rem;
    overflow-y: auto;
    height: 200px;
    white-space: pre-wrap;
    background: #10101c;
}

/* Today's Attendance Section */
.attendance-box {
    display: flex;
    flex-direction: column;
    flex: 1;
}

.table-responsive {
    overflow-x: auto;
    border-radius: 10px;
    background: rgba(16, 16, 28, 0.6);
    border: 1px solid rgba(255, 255, 255, 0.05);
}

.attendance-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.88rem;
    text-align: left;
}

.attendance-table th {
    background: rgba(30, 30, 46, 0.8);
    padding: 0.75rem 1rem;
    color: var(--accent-purple-light);
    font-weight: 700;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.attendance-table td {
    padding: 0.75rem 1rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.attendance-table tr:hover td {
    background: rgba(255, 255, 255, 0.03);
}

.badge {
    display: inline-block;
    padding: 0.25rem 0.6rem;
    border-radius: 9999px;
    font-size: 0.72rem;
    font-weight: 700;
}

.badge-success { background: rgba(0, 184, 148, 0.2); color: #00b894; border: 1px solid rgba(0, 184, 148, 0.3); }
.badge-warning { background: rgba(253, 203, 110, 0.2); color: #fdcb6e; border: 1px solid rgba(253, 203, 110, 0.3); }
.badge-danger { background: rgba(231, 76, 60, 0.2); color: #e74c3c; border: 1px solid rgba(231, 76, 60, 0.3); }

/* Modal Dialogs */
.modal-overlay {
    position: fixed;
    top: 0; left: 0; right: 0; bottom: 0;
    background: rgba(10, 10, 18, 0.85);
    backdrop-filter: blur(8px);
    display: none;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 1rem;
}

.modal-dialog {
    background: #171721;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 18px;
    width: 100%;
    max-width: 580px;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.7);
    overflow: hidden;
    animation: modalSlide 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes modalSlide {
    from { opacity: 0; transform: translateY(20px) scale(0.96); }
    to { opacity: 1; transform: translateY(0) scale(1); }
}

.modal-header {
    background: #1e1e2e;
    padding: 1.25rem 1.5rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.modal-title {
    font-size: 1.15rem;
    font-weight: 700;
    color: #ffffff;
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.modal-close {
    background: none;
    border: none;
    color: #a29bfe;
    font-size: 1.5rem;
    cursor: pointer;
    line-height: 1;
}

.modal-body {
    padding: 1.5rem;
}

/* Camera Preview */
.camera-wrapper {
    position: relative;
    width: 100%;
    height: 320px;
    background: #000;
    border-radius: 12px;
    overflow: hidden;
    margin-bottom: 1rem;
    border: 2px solid rgba(108, 92, 231, 0.3);
}

.camera-video {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.camera-overlay-box {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 200px;
    height: 240px;
    border: 2px dashed rgba(0, 210, 211, 0.8);
    border-radius: 16px;
    box-shadow: 0 0 0 9999px rgba(0, 0, 0, 0.4);
    pointer-events: none;
}

/* Keypad */
.keypad-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.75rem;
    margin-top: 1rem;
}

.keypad-btn {
    height: 52px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: #1e1e2e;
    color: #ffffff;
    font-size: 1.25rem;
    font-weight: 700;
    border-radius: 10px;
    cursor: pointer;
    transition: all 0.15s ease;
}

.keypad-btn:hover {
    background: #2b2b40;
    border-color: var(--accent-purple);
}

.form-input {
    width: 100%;
    background: #10101c;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 10px;
    padding: 0.85rem 1rem;
    color: #ffffff;
    font-size: 1.1rem;
    outline: none;
    margin-bottom: 0.75rem;
}

.form-input:focus {
    border-color: var(--accent-purple);
    box-shadow: 0 0 0 3px rgba(108, 92, 231, 0.2);
}

.btn-primary-action {
    background: linear-gradient(135deg, #00b894, #00d2d3);
    color: #0c392c;
    border: none;
    border-radius: 10px;
    padding: 0.85rem 1.5rem;
    font-size: 1rem;
    font-weight: 700;
    cursor: pointer;
    width: 100%;
    transition: transform 0.15s ease;
}

.btn-primary-action:hover {
    transform: translateY(-2px);
}

/* 9 ATTENDANCE CARDS (Python Tkinter Parity) */
.attendance-cards-scroll-container {
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-height: 380px;
    overflow-y: auto;
    padding-right: 4px;
}

.attendance-card {
    background: #1e1e2e;
    border-radius: 8px;
    padding: 0.65rem 1rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-left: 4px solid #6c5ce7;
    transition: background 0.15s ease;
}

.attendance-card:hover {
    background: #252538;
}

.att-card-left {
    display: flex;
    align-items: center;
    gap: 0.6rem;
}

.att-card-icon {
    font-size: 1.15rem;
}

.att-card-label {
    font-size: 0.85rem;
    font-weight: 700;
    color: #dfe6e9;
}

.att-card-value {
    font-family: 'Consolas', monospace;
    font-size: 0.95rem;
    font-weight: 700;
    color: #ffffff;
}

/* Python Large Green Capture Button (attendance_v8.py line 11827) */
.btn-python-capture {
    width: 220px;
    height: 60px;
    font-family: 'Arial', sans-serif;
    font-size: 1.25rem;
    font-weight: bold;
    background-color: #4CAF50;
    color: white;
    border: none;
    border-radius: 8px;
    box-shadow: 0 4px 10px rgba(76, 175, 80, 0.4);
    cursor: pointer;
    transition: all 0.2s ease;
}

.btn-python-capture:hover {
    background-color: #45a049;
    transform: translateY(-2px);
    box-shadow: 0 6px 14px rgba(76, 175, 80, 0.5);
}

.btn-python-capture:active {
    transform: translateY(1px);
}

.hr-menu-btn {
    border: none;
    border-radius: 10px;
    padding: 0.9rem 1rem;
    font-size: 0.9rem;
    font-weight: 700;
    color: #ffffff;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 0.6rem;
    text-align: left;
    transition: all 0.2s ease;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
}

.hr-menu-btn:hover {
    transform: translateY(-2px);
    filter: brightness(1.1);
}
    "##
}
