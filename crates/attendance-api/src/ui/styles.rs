pub fn get_global_css() -> &'static str {
    r##"
:root {
    --bg-main: #070913;
    --bg-darker: #04060c;
    --bg-card: rgba(12, 18, 38, 0.72);
    --border-color: rgba(0, 242, 254, 0.22);
    --accent-purple: #6c5ce7;
    --accent-purple-light: #a29bfe;
    --accent-green: #00f5d4;
    --accent-teal: #00f2fe;
    --accent-blue: #3a86ff;
    --accent-yellow: #ffd166;
    --accent-red: #ff2a6d;
    --text-primary: #f8fafc;
    --text-secondary: #cbd5e1;
    --text-muted: #64748b;
    --neon-cyan: #00f2fe;
    --neon-violet: #7b2cbf;
    --neon-emerald: #00f5d4;
}

/* Classic V8 Fallback Variables when user toggles "Classic Design" */
body.classic-v8-mode {
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
    background: radial-gradient(circle at 20% 20%, #1e1e30 0%, #171721 60%, #10101c 100%) !important;
}

* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

body {
    font-family: 'Segoe UI', 'Inter', -apple-system, BlinkMacSystemFont, Roboto, sans-serif;
    background:
        radial-gradient(circle at 15% 15%, rgba(0, 242, 254, 0.14) 0%, transparent 42%),
        radial-gradient(circle at 85% 25%, rgba(123, 44, 191, 0.18) 0%, transparent 48%),
        radial-gradient(circle at 50% 85%, rgba(0, 245, 212, 0.10) 0%, transparent 50%),
        linear-gradient(135deg, #04060e 0%, #090d1e 50%, #050711 100%);
    color: var(--text-primary);
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    overflow-x: hidden;
    position: relative;
    perspective: 1400px;
}

/* ==========================================================================
   3D INTERACTIVE CANVAS & HOLOGRAPHIC CYBER FLOOR GRID
   ========================================================================== */
#cyber-3d-canvas {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    pointer-events: none;
    z-index: 0;
    opacity: 0.88;
    transition: opacity 0.4s ease;
}

.cyber-3d-floor {
    position: fixed;
    bottom: -32vh;
    left: -25vw;
    width: 150vw;
    height: 68vh;
    background-image:
        linear-gradient(to right, rgba(0, 242, 254, 0.14) 1px, transparent 1px),
        linear-gradient(to bottom, rgba(108, 92, 231, 0.16) 1px, transparent 1px);
    background-size: 56px 56px;
    transform: perspective(550px) rotateX(72deg);
    transform-origin: center top;
    mask-image: radial-gradient(ellipse at 50% 0%, rgba(0,0,0,0.85) 10%, transparent 75%);
    -webkit-mask-image: radial-gradient(ellipse at 50% 0%, rgba(0,0,0,0.85) 10%, transparent 75%);
    pointer-events: none;
    z-index: 0;
    animation: cyberFloorScroll 14s linear infinite;
}

@keyframes cyberFloorScroll {
    0% { background-position: 0 0; }
    100% { background-position: 0 56px; }
}

body.classic-v8-mode #cyber-3d-canvas,
body.classic-v8-mode .cyber-3d-floor,
body.classic-v8-mode .futuristic-hud-banner {
    display: none !important;
}

/* ==========================================================================
   FLOATING 3D FUTURISTIC HEADER & HOLOGRAPHIC BRANDING
   ========================================================================== */
.app-header {
    background: linear-gradient(90deg, rgba(8, 12, 26, 0.88) 0%, rgba(16, 22, 46, 0.85) 50%, rgba(8, 12, 26, 0.88) 100%);
    backdrop-filter: blur(20px) saturate(180%);
    -webkit-backdrop-filter: blur(20px) saturate(180%);
    border-bottom: 1px solid rgba(0, 242, 254, 0.32);
    box-shadow: 0 10px 35px rgba(0, 0, 0, 0.65), 0 1px 20px rgba(0, 242, 254, 0.15);
    padding: 1.1rem 2.5rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    position: sticky;
    top: 0;
    z-index: 100;
    transform-style: preserve-3d;
}

.app-header::after {
    content: '';
    position: absolute;
    bottom: -2px;
    left: 0;
    width: 100%;
    height: 2px;
    background: linear-gradient(90deg, transparent 0%, #00f2fe 25%, #6c5ce7 50%, #00f5d4 75%, transparent 100%);
    background-size: 200% 100%;
    animation: headerLaserFlow 5s linear infinite;
}

@keyframes headerLaserFlow {
    0% { background-position: 0% 50%; }
    100% { background-position: 200% 50%; }
}

body.classic-v8-mode .app-header {
    background: rgba(23, 23, 33, 0.85);
    border-bottom: 1px solid var(--border-color);
    box-shadow: none;
}
body.classic-v8-mode .app-header::after {
    display: none;
}

.brand-wrapper {
    display: flex;
    align-items: center;
    gap: 1.25rem;
    transform-style: preserve-3d;
}

.brand-logo {
    width: 56px;
    height: 56px;
    border-radius: 16px;
    background: linear-gradient(135deg, #00f2fe 0%, #6c5ce7 55%, #ff2a6d 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.85rem;
    position: relative;
    box-shadow:
        0 0 25px rgba(0, 242, 254, 0.55),
        0 8px 20px rgba(108, 92, 231, 0.45),
        inset 0 2px 4px rgba(255, 255, 255, 0.65);
    animation: emblemFloat3D 4.5s ease-in-out infinite;
    transform-style: preserve-3d;
}

.brand-logo::before {
    content: '';
    position: absolute;
    inset: -5px;
    border-radius: 20px;
    border: 1.5px dashed rgba(0, 242, 254, 0.6);
    animation: spinOrbit 12s linear infinite;
}

@keyframes emblemFloat3D {
    0%, 100% { transform: translateY(0px) rotateY(0deg) rotateX(0deg); }
    50% { transform: translateY(-4px) rotateY(14deg) rotateX(8deg); }
}

@keyframes spinOrbit {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
}

body.classic-v8-mode .brand-logo {
    width: 52px;
    height: 52px;
    border-radius: 14px;
    background: linear-gradient(135deg, #6c5ce7, #00d2d3);
    box-shadow: 0 0 20px rgba(108, 92, 231, 0.4);
    animation: none;
}
body.classic-v8-mode .brand-logo::before {
    display: none;
}

.brand-title-box {
    display: flex;
    flex-direction: column;
}

.brand-main-title {
    font-size: 1.65rem;
    font-weight: 900;
    letter-spacing: 0.08em;
    background: linear-gradient(90deg, #ffffff 0%, #00f2fe 55%, #a29bfe 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    line-height: 1.1;
    filter: drop-shadow(0 2px 10px rgba(0, 242, 254, 0.35));
}

body.classic-v8-mode .brand-main-title {
    background: none;
    -webkit-text-fill-color: #ffffff;
    color: #ffffff;
    filter: none;
}

.brand-subtitle {
    font-size: 0.78rem;
    font-weight: 700;
    letter-spacing: 0.22em;
    color: var(--neon-cyan);
    text-shadow: 0 0 8px rgba(0, 242, 254, 0.45);
}

body.classic-v8-mode .brand-subtitle {
    color: var(--accent-purple-light);
    text-shadow: none;
}

/* Theme Switcher Button (3D Futuristic <-> Classic V8 Rollback) */
.btn-cyber-theme-switch {
    background: linear-gradient(135deg, rgba(0, 242, 254, 0.18), rgba(108, 92, 231, 0.28));
    border: 1px solid rgba(0, 242, 254, 0.55);
    color: #ffffff;
    padding: 0.52rem 1rem;
    border-radius: 12px;
    cursor: pointer;
    font-weight: 800;
    font-size: 0.8rem;
    letter-spacing: 0.05em;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    box-shadow: 0 0 18px rgba(0, 242, 254, 0.25), inset 0 1px 0 rgba(255, 255, 255, 0.25);
    transition: all 0.25s cubic-bezier(0.22, 1, 0.36, 1);
}

.btn-cyber-theme-switch:hover {
    transform: translateY(-2px) scale(1.03);
    box-shadow: 0 0 26px rgba(0, 242, 254, 0.55);
    border-color: #00f5d4;
}

.clock-wrapper {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    background: rgba(6, 10, 24, 0.75);
    border: 1px solid rgba(0, 242, 254, 0.28);
    padding: 0.4rem 0.95rem;
    border-radius: 12px;
    box-shadow: inset 0 0 14px rgba(0, 242, 254, 0.12), 0 4px 14px rgba(0, 0, 0, 0.4);
}

body.classic-v8-mode .clock-wrapper {
    background: none;
    border: none;
    padding: 0;
    box-shadow: none;
}

.clock-digital {
    font-family: 'Consolas', 'Courier New', monospace;
    font-size: 1.55rem;
    font-weight: 800;
    color: #00f2fe;
    letter-spacing: 0.06em;
    text-shadow: 0 0 12px rgba(0, 242, 254, 0.65);
}

body.classic-v8-mode .clock-digital {
    color: #ffffff;
    text-shadow: 0 0 10px rgba(255, 255, 255, 0.2);
}

.clock-sub {
    font-size: 0.76rem;
    font-weight: 600;
    color: var(--accent-purple-light);
    letter-spacing: 0.06em;
}

/* ==========================================================================
   3D HOLOGRAPHIC HUD BANNER
   ========================================================================== */
.futuristic-hud-banner {
    max-width: 1600px;
    width: calc(100% - 5rem);
    margin: 1rem auto 0 auto;
    padding: 0.65rem 1.35rem;
    border-radius: 14px;
    background: linear-gradient(90deg, rgba(0, 242, 254, 0.10) 0%, rgba(108, 92, 231, 0.16) 50%, rgba(0, 245, 212, 0.10) 100%);
    border: 1px solid rgba(0, 242, 254, 0.30);
    box-shadow: 0 8px 25px rgba(0, 0, 0, 0.45), inset 0 0 20px rgba(0, 242, 254, 0.08);
    display: flex;
    justify-content: space-between;
    align-items: center;
    position: relative;
    z-index: 2;
    backdrop-filter: blur(12px);
}

.hud-badge-group {
    display: flex;
    align-items: center;
    gap: 0.85rem;
    flex-wrap: wrap;
}

.hud-pill {
    font-family: 'Consolas', monospace;
    font-size: 0.74rem;
    font-weight: 700;
    padding: 0.25rem 0.65rem;
    border-radius: 999px;
    letter-spacing: 0.06em;
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
}

.hud-pill-cyan {
    background: rgba(0, 242, 254, 0.14);
    color: #00f2fe;
    border: 1px solid rgba(0, 242, 254, 0.4);
    box-shadow: 0 0 12px rgba(0, 242, 254, 0.25);
}

.hud-pill-green {
    background: rgba(0, 245, 212, 0.14);
    color: #00f5d4;
    border: 1px solid rgba(0, 245, 212, 0.4);
}

.hud-pill-purple {
    background: rgba(162, 155, 254, 0.14);
    color: #c4b5fd;
    border: 1px solid rgba(162, 155, 254, 0.4);
}

/* ==========================================================================
   MAIN LAYOUT & 3D PERSPECTIVE GLASS PANELS
   ========================================================================== */
.main-layout {
    display: flex;
    flex: 1;
    gap: 1.75rem;
    padding: 1.35rem 2.5rem 2rem 2.5rem;
    max-width: 1600px;
    margin: 0 auto;
    width: 100%;
    position: relative;
    z-index: 2;
    transform-style: preserve-3d;
}

.left-panel {
    flex: 0 0 465px;
    display: flex;
    flex-direction: column;
    gap: 1.35rem;
    transform-style: preserve-3d;
}

.right-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 1.35rem;
    min-width: 0;
    transform-style: preserve-3d;
}

@media (max-width: 1024px) {
    .app-header {
        padding: 1rem;
        flex-wrap: wrap;
        gap: 0.75rem;
    }
    .futuristic-hud-banner {
        width: calc(100% - 2rem);
        flex-direction: column;
        gap: 0.5rem;
        align-items: flex-start;
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

/* 3D Holographic Glass Card */
.glass-panel {
    background:
        radial-gradient(550px circle at var(--mouse-x, 50%) var(--mouse-y, 20%), rgba(0, 242, 254, 0.11), transparent 45%),
        linear-gradient(145deg, rgba(16, 23, 46, 0.80) 0%, rgba(9, 13, 28, 0.86) 100%);
    backdrop-filter: blur(18px) saturate(170%);
    -webkit-backdrop-filter: blur(18px) saturate(170%);
    border: 1px solid rgba(0, 242, 254, 0.24);
    border-radius: 20px;
    padding: 1.35rem;
    position: relative;
    transform-style: preserve-3d;
    transform: perspective(1200px) rotateX(var(--rx, 0deg)) rotateY(var(--ry, 0deg)) translateZ(0px);
    transition: transform 0.18s ease-out, border-color 0.3s ease, box-shadow 0.3s ease;
    box-shadow:
        0 20px 45px -10px rgba(0, 0, 0, 0.75),
        0 0 25px rgba(0, 242, 254, 0.08),
        inset 0 1px 1px rgba(255, 255, 255, 0.18),
        inset 0 -2px 12px rgba(108, 92, 231, 0.12);
}

.glass-panel:hover {
    border-color: rgba(0, 242, 254, 0.52);
    box-shadow:
        0 26px 55px -12px rgba(0, 0, 0, 0.85),
        0 0 35px rgba(0, 242, 254, 0.18),
        inset 0 1px 2px rgba(255, 255, 255, 0.28);
}

/* Sci-Fi HUD Corner Brackets on Glass Panels */
.glass-panel::before {
    content: '';
    position: absolute;
    top: -1px;
    left: -1px;
    width: 22px;
    height: 22px;
    border-top: 2.5px solid #00f2fe;
    border-left: 2.5px solid #00f2fe;
    border-top-left-radius: 20px;
    pointer-events: none;
    filter: drop-shadow(0 0 6px #00f2fe);
}

.glass-panel::after {
    content: '';
    position: absolute;
    bottom: -1px;
    right: -1px;
    width: 22px;
    height: 22px;
    border-bottom: 2.5px solid #00f5d4;
    border-right: 2.5px solid #00f5d4;
    border-bottom-right-radius: 20px;
    pointer-events: none;
    filter: drop-shadow(0 0 6px #00f5d4);
}

body.classic-v8-mode .glass-panel {
    background: rgba(28, 28, 42, 0.6);
    border: 1px solid var(--border-color);
    border-radius: 16px;
    padding: 1.25rem;
    box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.37);
    transform: none !important;
}
body.classic-v8-mode .glass-panel::before,
body.classic-v8-mode .glass-panel::after {
    display: none;
}

/* Section Header */
.section-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 1.05rem;
}

.accent-bar {
    width: 30px;
    height: 5px;
    border-radius: 4px;
    box-shadow: 0 0 12px currentColor;
}

.section-title {
    font-size: 0.88rem;
    font-weight: 800;
    letter-spacing: 0.14em;
    color: #ffffff;
    text-shadow: 0 0 10px rgba(0, 242, 254, 0.35);
}

/* ==========================================================================
   3D STATUS GRID (4 TELEMETRY PODS)
   ========================================================================== */
.status-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 0.85rem;
    transform-style: preserve-3d;
}

.status-card {
    background: linear-gradient(145deg, rgba(14, 20, 42, 0.88), rgba(8, 11, 24, 0.92));
    border: 1px solid rgba(0, 242, 254, 0.18);
    border-radius: 14px;
    padding: 0.95rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    position: relative;
    overflow: hidden;
    transition: all 0.25s cubic-bezier(0.22, 1, 0.36, 1);
    box-shadow:
        0 10px 24px rgba(0, 0, 0, 0.5),
        inset 0 1px 0 rgba(255, 255, 255, 0.1);
}

.status-card:hover {
    transform: translateY(-5px) translateZ(14px) scale(1.02);
    border-color: rgba(0, 242, 254, 0.6);
    box-shadow:
        0 16px 32px rgba(0, 0, 0, 0.65),
        0 0 20px rgba(0, 242, 254, 0.25);
}

body.classic-v8-mode .status-card {
    background: rgba(20, 20, 32, 0.7);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 12px;
    padding: 0.85rem;
    box-shadow: none;
    transform: none !important;
}

.status-card-header {
    display: flex;
    align-items: center;
    gap: 0.55rem;
}

.status-card-icon {
    font-size: 1.2rem;
    filter: drop-shadow(0 0 8px rgba(0, 242, 254, 0.5));
}

.status-card-title {
    font-size: 0.78rem;
    font-weight: 800;
    color: #f1f5f9;
    letter-spacing: 0.03em;
}

.status-card-progress {
    height: 4px;
    background: rgba(45, 45, 74, 0.7);
    border-radius: 4px;
    overflow: hidden;
}

.status-card-progress-bar {
    height: 100%;
    background: linear-gradient(90deg, #00f2fe, #6c5ce7, #00f5d4);
    background-size: 200% 100%;
    width: 100%;
    box-shadow: 0 0 10px #00f2fe;
    animation: cyberBarFlow 3s linear infinite;
}

@keyframes cyberBarFlow {
    0% { background-position: 0% 50%; }
    100% { background-position: 200% 50%; }
}

.status-card-state {
    font-size: 0.76rem;
    color: var(--accent-green);
    font-weight: 700;
    text-shadow: 0 0 8px rgba(0, 245, 212, 0.4);
}

/* ==========================================================================
   3D EXTRUDED ACTION BUTTONS (BIOMETRIC KIOSK TRIGGERS)
   ========================================================================== */
.action-buttons-box {
    display: flex;
    flex-direction: column;
    gap: 0.95rem;
    transform-style: preserve-3d;
}

.btn-action-gradient {
    height: 72px;
    border: 1px solid rgba(255, 255, 255, 0.32);
    border-radius: 16px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: flex-start;
    padding: 0 1.5rem;
    gap: 1.25rem;
    font-size: 1.06rem;
    font-weight: 800;
    letter-spacing: 0.02em;
    color: #06131f;
    position: relative;
    overflow: hidden;
    transition: all 0.25s cubic-bezier(0.22, 1, 0.36, 1);
    text-align: left;
    width: 100%;
    transform-style: preserve-3d;
}

.btn-action-gradient::after {
    content: '';
    position: absolute;
    top: 0;
    left: -120%;
    width: 65%;
    height: 100%;
    background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.45), transparent);
    transform: skewX(-25deg);
    transition: left 0.55s ease;
    pointer-events: none;
}

.btn-action-gradient:hover::after {
    left: 140%;
}

.btn-action-gradient:hover {
    transform: perspective(700px) translateY(-4px) translateZ(14px) rotateX(4deg);
}

.btn-action-gradient:active {
    transform: perspective(700px) translateY(2px) translateZ(4px);
}

.btn-action-gradient .btn-icon {
    font-size: 1.85rem;
    width: 46px;
    height: 46px;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.28);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2), inset 0 1px 1px rgba(255, 255, 255, 0.6);
    transform: translateZ(16px);
}

.btn-camera {
    background: linear-gradient(135deg, #00f5d4 0%, #00bbf9 55%, #00f2fe 100%);
    color: #042426;
    box-shadow:
        0 6px 0 #007f8c,
        0 14px 28px rgba(0, 242, 254, 0.32),
        inset 0 1px 1px rgba(255, 255, 255, 0.65);
}

.btn-no-camera {
    background: linear-gradient(135deg, #ffd166 0%, #fcb045 55%, #ffeaa7 100%);
    color: #3b2504;
    box-shadow:
        0 6px 0 #b37400,
        0 14px 28px rgba(253, 203, 110, 0.30),
        inset 0 1px 1px rgba(255, 255, 255, 0.65);
}

.btn-qr {
    background: linear-gradient(135deg, #72efdd 0%, #4ea8de 50%, #a29bfe 100%);
    color: #061c36;
    box-shadow:
        0 6px 0 #2b5876,
        0 14px 28px rgba(116, 185, 255, 0.32),
        inset 0 1px 1px rgba(255, 255, 255, 0.65);
}

body.classic-v8-mode .btn-action-gradient {
    height: 68px;
    border: none;
    border-radius: 14px;
    box-shadow: 0 4px 15px rgba(0, 0, 0, 0.2);
}
body.classic-v8-mode .btn-action-gradient .btn-icon {
    width: auto;
    height: auto;
    background: none;
    box-shadow: none;
}

.btn-hr-floating {
    margin-top: 0.5rem;
    height: 56px;
    border: 1px solid rgba(0, 242, 254, 0.42);
    border-radius: 14px;
    background: linear-gradient(135deg, rgba(108, 92, 231, 0.42), rgba(0, 242, 254, 0.22));
    color: #ffffff;
    font-size: 0.95rem;
    font-weight: 800;
    letter-spacing: 0.04em;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    transition: all 0.25s cubic-bezier(0.22, 1, 0.36, 1);
    width: 100%;
    box-shadow:
        0 5px 0 rgba(54, 41, 138, 0.85),
        0 12px 24px rgba(0, 0, 0, 0.45),
        inset 0 1px 1px rgba(255, 255, 255, 0.3);
}

.btn-hr-floating:hover {
    background: linear-gradient(135deg, rgba(108, 92, 231, 0.65), rgba(0, 242, 254, 0.38));
    border-color: #00f2fe;
    transform: translateY(-3px) scale(1.01);
    box-shadow:
        0 7px 0 rgba(54, 41, 138, 0.9),
        0 16px 32px rgba(0, 242, 254, 0.28);
}

/* ==========================================================================
   3D SYSTEM CONSOLE (HOLOGRAPHIC TELEMETRY TERMINAL)
   ========================================================================== */
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
    margin-bottom: 0.85rem;
}

.console-action-btns {
    display: flex;
    gap: 0.55rem;
}

.btn-tool {
    width: 34px;
    height: 34px;
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1rem;
    color: #ffffff;
    transition: all 0.2s ease;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.35);
}

.btn-tool:hover {
    transform: translateY(-2px) scale(1.08);
}

.btn-export { background: linear-gradient(135deg, #00b894, #00f5d4); color: #05261f; }
.btn-sync { background: linear-gradient(135deg, #0984e3, #00f2fe); color: #041f36; }
.btn-clear { background: linear-gradient(135deg, #d63031, #ff2a6d); }

.terminal-window {
    background: rgba(5, 8, 19, 0.92);
    border: 1px solid rgba(0, 242, 254, 0.25);
    border-radius: 14px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    flex: 1;
    box-shadow: inset 0 0 30px rgba(0, 242, 254, 0.06), 0 10px 25px rgba(0, 0, 0, 0.5);
    position: relative;
}

/* Subtle CRT Holographic Scanlines inside Terminal */
.terminal-window::after {
    content: '';
    position: absolute;
    inset: 0;
    background: repeating-linear-gradient(
        to bottom,
        transparent 0px,
        transparent 2px,
        rgba(0, 242, 254, 0.025) 3px
    );
    pointer-events: none;
}

.terminal-titlebar {
    background: linear-gradient(90deg, #0f162e, #182246);
    height: 34px;
    padding: 0 1rem;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    border-bottom: 1px solid rgba(0, 242, 254, 0.18);
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

.dot-red { background: #ff5f56; box-shadow: 0 0 8px #ff5f56; }
.dot-yellow { background: #ffbd2e; box-shadow: 0 0 8px #ffbd2e; }
.dot-green { background: #27c93f; box-shadow: 0 0 8px #27c93f; }

.terminal-title {
    font-family: 'Consolas', monospace;
    font-size: 0.8rem;
    font-weight: 700;
    color: #00f2fe;
    letter-spacing: 0.06em;
}

.terminal-body {
    font-family: 'Consolas', monospace;
    font-size: 0.85rem;
    line-height: 1.55;
    color: #e2e8f0;
    padding: 1rem;
    overflow-y: auto;
    height: 200px;
    white-space: pre-wrap;
    background: transparent;
}

/* ==========================================================================
   TODAY'S ATTENDANCE SECTION & 9 3D HOLOGRAPHIC CARDS
   ========================================================================== */
.attendance-box {
    display: flex;
    flex-direction: column;
    flex: 1;
}

.table-responsive {
    overflow-x: auto;
    border-radius: 12px;
    background: rgba(10, 15, 32, 0.75);
    border: 1px solid rgba(0, 242, 254, 0.18);
}

.attendance-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.88rem;
    text-align: left;
}

.attendance-table th {
    background: rgba(18, 26, 52, 0.9);
    padding: 0.75rem 1rem;
    color: #00f2fe;
    font-weight: 700;
    border-bottom: 1px solid rgba(0, 242, 254, 0.2);
}

.attendance-table td {
    padding: 0.75rem 1rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.attendance-table tr:hover td {
    background: rgba(0, 242, 254, 0.06);
}

.badge {
    display: inline-block;
    padding: 0.25rem 0.65rem;
    border-radius: 9999px;
    font-size: 0.72rem;
    font-weight: 700;
}

.badge-success { background: rgba(0, 245, 212, 0.18); color: #00f5d4; border: 1px solid rgba(0, 245, 212, 0.4); }
.badge-warning { background: rgba(253, 203, 110, 0.2); color: #fdcb6e; border: 1px solid rgba(253, 203, 110, 0.3); }
.badge-danger { background: rgba(255, 42, 109, 0.2); color: #ff2a6d; border: 1px solid rgba(255, 42, 109, 0.35); }

/* 9 ATTENDANCE CARDS (3D Floating Telemetry Strips) */
.attendance-cards-scroll-container {
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-height: 390px;
    overflow-y: auto;
    padding-right: 6px;
    transform-style: preserve-3d;
}

.attendance-card {
    background: linear-gradient(90deg, rgba(16, 24, 50, 0.88) 0%, rgba(10, 15, 32, 0.78) 100%);
    border: 1px solid rgba(0, 242, 254, 0.14);
    border-radius: 12px;
    padding: 0.72rem 1.15rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-left: 4px solid #00f2fe;
    transition: all 0.22s cubic-bezier(0.22, 1, 0.36, 1);
    box-shadow: 0 6px 16px rgba(0, 0, 0, 0.38);
}

.attendance-card:hover {
    background: linear-gradient(90deg, rgba(24, 36, 74, 0.95) 0%, rgba(14, 22, 48, 0.9) 100%);
    transform: perspective(800px) translateX(8px) translateZ(10px);
    border-color: rgba(0, 242, 254, 0.45);
    box-shadow: 0 10px 24px rgba(0, 0, 0, 0.55), 0 0 16px rgba(0, 242, 254, 0.18);
}

body.classic-v8-mode .attendance-card {
    background: #1e1e2e;
    border-radius: 8px;
    border-top: none;
    border-right: none;
    border-bottom: none;
    box-shadow: none;
    transform: none !important;
}

.att-card-left {
    display: flex;
    align-items: center;
    gap: 0.65rem;
}

.att-card-icon {
    font-size: 1.2rem;
    filter: drop-shadow(0 0 6px rgba(0, 242, 254, 0.4));
}

.att-card-label {
    font-size: 0.86rem;
    font-weight: 700;
    color: #e2e8f0;
}

.att-card-value {
    font-family: 'Consolas', monospace;
    font-size: 0.98rem;
    font-weight: 800;
    color: #00f2fe;
    text-shadow: 0 0 8px rgba(0, 242, 254, 0.35);
}

body.classic-v8-mode .att-card-value {
    color: #ffffff;
    text-shadow: none;
}

/* ==========================================================================
   3D HOLOGRAPHIC MODAL DIALOGS & LASER BIOMETRIC SCANNER
   ========================================================================== */
.modal-overlay {
    position: fixed;
    top: 0; left: 0; right: 0; bottom: 0;
    background: rgba(4, 7, 17, 0.86);
    backdrop-filter: blur(12px);
    display: none;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 1rem;
    perspective: 1200px;
}

.modal-dialog {
    background: linear-gradient(155deg, #0d1328 0%, #080c1a 100%);
    border: 1px solid rgba(0, 242, 254, 0.35);
    border-radius: 22px;
    width: 100%;
    max-width: 580px;
    box-shadow:
        0 30px 70px -12px rgba(0, 0, 0, 0.85),
        0 0 40px rgba(0, 242, 254, 0.2),
        inset 0 1px 1px rgba(255, 255, 255, 0.2);
    overflow: hidden;
    animation: modalSlide3D 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes modalSlide3D {
    from { opacity: 0; transform: translateY(28px) rotateX(10deg) scale(0.94); }
    to { opacity: 1; transform: translateY(0) rotateX(0deg) scale(1); }
}

.modal-header {
    background: linear-gradient(90deg, rgba(16, 24, 52, 0.95), rgba(26, 20, 62, 0.95));
    padding: 1.25rem 1.5rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid rgba(0, 242, 254, 0.25);
}

.modal-title {
    font-size: 1.15rem;
    font-weight: 800;
    color: #ffffff;
    display: flex;
    align-items: center;
    gap: 0.55rem;
    text-shadow: 0 0 10px rgba(0, 242, 254, 0.4);
}

.modal-close {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(0, 242, 254, 0.3);
    width: 34px;
    height: 34px;
    border-radius: 10px;
    color: #00f2fe;
    font-size: 1.4rem;
    cursor: pointer;
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
}

.modal-close:hover {
    background: rgba(255, 42, 109, 0.25);
    border-color: #ff2a6d;
    color: #ffffff;
    transform: rotate(90deg);
}

.modal-body {
    padding: 1.5rem;
}

/* Camera Preview with 3D Biometric Laser Scanner */
.camera-wrapper {
    position: relative;
    width: 100%;
    height: 320px;
    background: #000;
    border-radius: 16px;
    overflow: hidden;
    margin-bottom: 1rem;
    border: 2px solid rgba(0, 242, 254, 0.5);
    box-shadow: 0 0 30px rgba(0, 242, 254, 0.22);
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
    width: 210px;
    height: 250px;
    border: 2px dashed rgba(0, 242, 254, 0.9);
    border-radius: 20px;
    box-shadow: 0 0 0 9999px rgba(4, 8, 20, 0.48), inset 0 0 25px rgba(0, 242, 254, 0.25);
    pointer-events: none;
    overflow: hidden;
}

.camera-overlay-box::after {
    content: '';
    position: absolute;
    left: 0;
    width: 100%;
    height: 3px;
    background: linear-gradient(90deg, transparent, #00f5d4, #00f2fe, transparent);
    box-shadow: 0 0 14px #00f2fe;
    animation: biometricLaserScan 2.6s ease-in-out infinite;
}

@keyframes biometricLaserScan {
    0%, 100% { top: 6%; opacity: 0.85; }
    50% { top: 92%; opacity: 1; }
}

/* Keypad & Form Inputs */
.keypad-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.75rem;
    margin-top: 1rem;
}

.keypad-btn {
    height: 52px;
    border: 1px solid rgba(0, 242, 254, 0.2);
    background: #111832;
    color: #ffffff;
    font-size: 1.25rem;
    font-weight: 700;
    border-radius: 10px;
    cursor: pointer;
    transition: all 0.15s ease;
}

.keypad-btn:hover {
    background: #1c2752;
    border-color: #00f2fe;
}

.form-input {
    width: 100%;
    background: rgba(8, 12, 26, 0.9);
    border: 1px solid rgba(0, 242, 254, 0.25);
    border-radius: 12px;
    padding: 0.85rem 1rem;
    color: #ffffff;
    font-size: 1.05rem;
    outline: none;
    margin-bottom: 0.75rem;
    transition: all 0.2s ease;
}

.form-input:focus {
    border-color: #00f2fe;
    box-shadow: 0 0 0 3px rgba(0, 242, 254, 0.2);
}

.btn-primary-action {
    background: linear-gradient(135deg, #00f5d4, #00f2fe);
    color: #052629;
    border: none;
    border-radius: 12px;
    padding: 0.85rem 1.5rem;
    font-size: 1rem;
    font-weight: 800;
    cursor: pointer;
    width: 100%;
    transition: all 0.2s ease;
    box-shadow: 0 6px 18px rgba(0, 242, 254, 0.28);
}

.btn-primary-action:hover {
    transform: translateY(-2px);
    box-shadow: 0 10px 25px rgba(0, 242, 254, 0.42);
}

/* Python Large Green Capture Button (attendance_v8.py line 11827) */
.btn-python-capture {
    width: 230px;
    height: 62px;
    font-family: 'Segoe UI', 'Arial', sans-serif;
    font-size: 1.25rem;
    font-weight: 800;
    letter-spacing: 0.06em;
    background: linear-gradient(135deg, #00f5d4 0%, #10b981 100%);
    color: #04261c;
    border: none;
    border-radius: 14px;
    box-shadow: 0 5px 0 #047857, 0 10px 25px rgba(16, 185, 129, 0.45);
    cursor: pointer;
    transition: all 0.2s ease;
}

.btn-python-capture:hover {
    transform: translateY(-3px);
    box-shadow: 0 8px 0 #047857, 0 14px 30px rgba(0, 245, 212, 0.6);
}

.btn-python-capture:active {
    transform: translateY(1px);
}

.hr-menu-btn {
    border: 1px solid rgba(0, 242, 254, 0.24);
    border-radius: 12px;
    padding: 0.95rem 1.1rem;
    font-size: 0.9rem;
    font-weight: 700;
    background: linear-gradient(135deg, rgba(22, 32, 68, 0.9), rgba(14, 20, 44, 0.95));
    color: #ffffff;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 0.65rem;
    text-align: left;
    transition: all 0.22s ease;
    box-shadow: 0 6px 16px rgba(0, 0, 0, 0.35);
}

.hr-menu-btn:hover {
    transform: translateY(-3px) scale(1.01);
    border-color: #00f2fe;
    box-shadow: 0 10px 24px rgba(0, 242, 254, 0.22);
    filter: brightness(1.12);
}
    "##
}
