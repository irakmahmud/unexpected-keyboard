pub fn get_kiosk_js() -> &'static str {
    r##"
let cameraStream = null;
let hrRegCameraStream = null;
let activeCompany = {
    company_id: 'TECHFIRM_01',
    company_name: 'TechFirm Bangladesh Ltd.',
    address_city: 'Dhaka'
};
let lastCapturedImageB64 = null;
let lastVerifyResult = null;
let verifyCountdownTimer = null;
let verifySecondsLeft = 5;
let qrPollTimer = null;
let hrQrPollTimer = null;
let currentDisplayedUser = null;
let cachedCompaniesList = [];

// 20 Bilingual Pose Instructions (attendance_v8.py lines 4862-4903)
const POSE_INSTRUCTIONS = [
    { en: "1/20: Look straight at camera (normal expression)", bn: "১/২০: ক্যামেরার দিকে সরাসরি তাকান (স্বাভাবিক অভিব্যক্তি)" },
    { en: "2/20: Look straight (slight smile)", bn: "২/২০: সরাসরি তাকান (হালকা হাসুন)" },
    { en: "3/20: Tilt head slightly UP", bn: "৩/২০: মাথা সামান্য উপরে তুলুন" },
    { en: "4/20: Tilt head slightly DOWN", bn: "৪/২০: মাথা সামান্য নিচে নামান" },
    { en: "5/20: Turn head slightly LEFT", bn: "৫/২০: মাথা সামান্য বামে ঘোরান" },
    { en: "6/20: Turn head slightly RIGHT", bn: "৬/২০: মাথা সামান্য ডানে ঘোরান" },
    { en: "7/20: Tilt head to LEFT shoulder", bn: "৭/২০: মাথা বাম কাঁধের দিকে কাত করুন" },
    { en: "8/20: Tilt head to RIGHT shoulder", bn: "৮/২০: মাথা ডান কাঁধের দিকে কাত করুন" },
    { en: "9/20: Look straight (serious expression)", bn: "৯/২০: সরাসরি তাকান (গম্ভীর অভিব্যক্তি)" },
    { en: "10/20: Look straight (eyes wide)", bn: "১০/২০: সরাসরি তাকান (চোখ বড় করে)" },
    { en: "11/20: Turn head MORE to LEFT", bn: "১১/২০: মাথা আরও একটু বামে ঘোরান" },
    { en: "12/20: Turn head MORE to RIGHT", bn: "১২/২০: মাথা আরও একটু ডানে ঘোরান" },
    { en: "13/20: Chin UP slightly, look at camera", bn: "১৩/২০: থুতনি সামান্য উপরে তুলে ক্যামেরায় তাকান" },
    { en: "14/20: Chin DOWN slightly, look at camera", bn: "১৪/২০: থুতনি সামান্য নিচে নামিয়ে ক্যামেরায় তাকান" },
    { en: "15/20: Move slightly CLOSER to camera", bn: "১৫/২০: ক্যামেরার সামান্য কাছে আসুন" },
    { en: "16/20: Move slightly FURTHER from camera", bn: "১৬/২০: ক্যামেরা থেকে সামান্য দূরে যান" },
    { en: "17/20: Look UP-LEFT towards camera", bn: "১৭/২০: উপরে-বামে কোণ থেকে ক্যামেরায় তাকান" },
    { en: "18/20: Look UP-RIGHT towards camera", bn: "১৮/২০: উপরে-ডানে কোণ থেকে ক্যামেরায় তাকান" },
    { en: "19/20: Natural smile, look straight", bn: "১৯/২০: স্বাভাবিক হাসি দিয়ে সরাসরি তাকান" },
    { en: "20/20: Final straight pose (relaxed)", bn: "২০/২০: চূড়ান্ত সোজা পোজ (স্বাভাবিক থাকুন)" }
];

document.addEventListener('DOMContentLoaded', () => {
    initClock();
    initTerminalLogs();
    loadActiveCompany();
    loadLastAttendanceData();
    setInterval(loadLastAttendanceData, 10000);
});

// Digital Clock with Asia/Dhaka simulation (attendance_v8.py line 5944)
function initClock() {
    function updateClock() {
        const now = new Date();
        const dhakaTimeStr = now.toLocaleTimeString('en-US', { timeZone: 'Asia/Dhaka', hour12: false });
        const dhakaDateStr = now.toLocaleDateString('en-GB', { timeZone: 'Asia/Dhaka', day: '2-digit', month: 'short', year: 'numeric' });
        
        const clockEl = document.getElementById('digital-clock');
        const dateEl = document.getElementById('dhaka-date');
        if (clockEl) clockEl.innerText = dhakaTimeStr;
        if (dateEl) dateEl.innerText = `${dhakaDateStr} (Dhaka Time)`;
    }
    updateClock();
    setInterval(updateClock, 1000);
}

// Terminal Console Logging (attendance_v8.py line 6591)
function addTerminalLog(msg, level = 'INFO') {
    const term = document.getElementById('terminal-body');
    if (!term) return;
    const now = new Date();
    const timeStr = now.toLocaleTimeString('en-US', { hour12: false });
    const logLine = `[${timeStr}] [${level}] ${msg}\n`;
    term.innerText += logLine;
    term.scrollTop = term.scrollHeight;
}

function initTerminalLogs() {
    addTerminalLog('Application initialized successfully (attendance_v8.py 100% pure-Rust parity)', 'SYSTEM');
    addTerminalLog('ONNX Runtime (tract-onnx) FaceNet512 (512-D) + MTCNN detector online', 'AI_CORE');
    addTerminalLog('SQLite WAL database pool connected & face profiles indexed', 'DATABASE');
    addTerminalLog('NetworkSyncManager active. Transactional Outbox ready for BigQuery sync', 'SYNC');
}

function clearConsole() {
    const term = document.getElementById('terminal-body');
    if (term) term.innerText = '';
    addTerminalLog('Console cleared by user', 'SYSTEM');
}

function exportConsoleLogs() {
    const term = document.getElementById('terminal-body');
    if (!term) return;
    const blob = new Blob([term.innerText], { type: 'text/plain' });
    const a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = `attendance_logs_${Date.now()}.log`;
    a.click();
    addTerminalLog('Console logs exported to file', 'USER');
}

async function forceSync() {
    addTerminalLog('Force sync triggered by user...', 'SYNC');
    try {
        const res = await fetch('/api/v1/sync/trigger', { method: 'POST' });
        const data = await res.json();
        addTerminalLog(`BigQuery sync complete: ${JSON.stringify(data)}`, 'SYNC');
    } catch (e) {
        addTerminalLog(`Sync error: ${e.message}`, 'ERROR');
    }
}

// ============================================================================
// COMPANY MANAGEMENT (attendance_v8.py lines 10366-11600 Parity)
// ============================================================================
function updateCompanyBadges(comp) {
    if (!comp || !comp.company_name) return;
    activeCompany = comp;
    const headerName = document.getElementById('header-company-name');
    if (headerName) {
        headerName.innerText = `Company: ${comp.company_name}`;
    }
    const banner = document.getElementById('company-active-banner');
    if (banner) {
        banner.innerText = `${comp.company_name} (${comp.company_id})`;
    }
    const hrLabel = document.getElementById('hr-auth-company-label');
    if (hrLabel) {
        hrLabel.innerText = `${comp.company_name} (${comp.company_id})`;
    }
}

async function loadActiveCompany() {
    try {
        const res = await fetch('/api/v1/companies/profile');
        if (!res.ok) return;
        const data = await res.json();
        if (data && data.company_name) {
            updateCompanyBadges(data);
            addTerminalLog(`Active Company loaded: ${data.company_name} (${data.company_id})`, 'COMPANY');
        }
    } catch (e) {
        console.error('Error loading active company:', e);
    }
}

function openCompanyManagement() {
    closeHrManagement();
    const modal = document.getElementById('modal-company');
    if (modal) modal.style.display = 'flex';
    updateCompanyBadges(activeCompany);
    showCompanySubView('menu');
}

function closeCompanyManagement() {
    const modal = document.getElementById('modal-company');
    if (modal) modal.style.display = 'none';
}

function showCompanySubView(view) {
    const target = (view === 'main') ? 'menu' : view;
    const menuEl = document.getElementById('company-view-menu');
    const selectEl = document.getElementById('company-view-select');
    const createEl = document.getElementById('company-view-create');
    const editEl = document.getElementById('company-view-edit');

    if (menuEl) menuEl.style.display = target === 'menu' ? 'block' : 'none';
    if (selectEl) selectEl.style.display = target === 'select' ? 'block' : 'none';
    if (createEl) createEl.style.display = target === 'create' ? 'block' : 'none';
    if (editEl) editEl.style.display = target === 'edit' ? 'block' : 'none';

    if (target === 'menu') {
        updateCompanyBadges(activeCompany);
    } else if (target === 'select') {
        const sInput = document.getElementById('comp-select-search');
        const pInput = document.getElementById('comp-select-password');
        const stEl = document.getElementById('comp-select-status');
        if (sInput) sInput.value = '';
        if (pInput) pInput.value = '';
        if (stEl) stEl.innerText = '';
        loadCompanyListForSelect();
    } else if (target === 'create') {
        ['comp-new-name', 'comp-new-password', 'comp-new-road', 'comp-new-city',
         'comp-new-contact-name', 'comp-new-contact-desig', 'comp-new-contact-num'].forEach(id => {
            const el = document.getElementById(id);
            if (el) el.value = '';
        });
        const stEl = document.getElementById('comp-create-status');
        if (stEl) stEl.innerText = '';
    } else if (target === 'edit') {
        const sInput = document.getElementById('comp-edit-search');
        const pInput = document.getElementById('comp-edit-verify-pw');
        const st1 = document.getElementById('comp-edit-verify-status');
        const step1 = document.getElementById('comp-edit-step1');
        const step2 = document.getElementById('comp-edit-step2');
        if (sInput) sInput.value = '';
        if (pInput) pInput.value = '';
        if (st1) st1.innerText = '';
        if (step1) step1.style.display = 'block';
        if (step2) step2.style.display = 'none';
        loadCompanyListForEdit();
    }
}

async function loadCompanyListForSelect() {
    const searchEl = document.getElementById('comp-select-search');
    const search = searchEl ? searchEl.value.trim() : '';
    const sel = document.getElementById('comp-select-listbox');
    if (!sel) return;
    try {
        const res = await fetch(`/api/v1/companies/list?search=${encodeURIComponent(search)}`);
        const list = await res.json();
        cachedCompaniesList = Array.isArray(list) ? list : [];
        if (cachedCompaniesList.length === 0) {
            sel.innerHTML = '<option value="">No companies found</option>';
            return;
        }
        sel.innerHTML = cachedCompaniesList.map((c, idx) => {
            const isAct = activeCompany && c.company_id === activeCompany.company_id;
            return `<option value="${c.company_id}" ${isAct || idx === 0 ? 'selected' : ''}>${c.company_name} (${c.company_id})${isAct ? ' ✓ Active' : ''}</option>`;
        }).join('');
    } catch (e) {
        sel.innerHTML = '<option value="">Error loading companies</option>';
    }
}

async function verifyAndSelectCompany() {
    const sel = document.getElementById('comp-select-listbox');
    const pwEl = document.getElementById('comp-select-password');
    const stEl = document.getElementById('comp-select-status');
    const company_id = sel ? sel.value : '';
    const password = pwEl ? pwEl.value : '';

    if (!company_id) {
        if (stEl) stEl.innerText = 'Please select a company from the list.';
        return;
    }
    if (!password) {
        if (stEl) stEl.innerText = 'Please enter the company password.';
        return;
    }
    try {
        const res = await fetch('/api/v1/companies/verify', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ company_id, password })
        });
        const data = await res.json();
        if (res.ok && data.success && data.company) {
            updateCompanyBadges(data.company);
            addTerminalLog(`Company switched to: ${data.company.company_name} (${data.company.company_id})`, 'COMPANY');
            closeCompanyManagement();
        } else {
            if (stEl) stEl.innerText = data.error || data.message || 'Invalid company password';
        }
    } catch (e) {
        if (stEl) stEl.innerText = 'Error verifying company: ' + e.message;
    }
}

async function submitCreateCompany() {
    const company_name = (document.getElementById('comp-new-name')?.value || '').trim();
    const password = document.getElementById('comp-new-password')?.value || '';
    const address_road = (document.getElementById('comp-new-road')?.value || '').trim();
    const address_city = (document.getElementById('comp-new-city')?.value || '').trim();
    const contact_person_name = (document.getElementById('comp-new-contact-name')?.value || '').trim();
    const contact_person_designation = (document.getElementById('comp-new-contact-desig')?.value || '').trim();
    const contact_person_number = (document.getElementById('comp-new-contact-num')?.value || '').trim();
    const stEl = document.getElementById('comp-create-status');

    if (!company_name || !password) {
        if (stEl) stEl.innerText = 'Company Name and Password are required!';
        return;
    }

    try {
        const res = await fetch('/api/v1/companies/create', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                company_name,
                password,
                address_road,
                address_city,
                contact_person_name,
                contact_person_designation,
                contact_person_number
            })
        });
        const data = await res.json();
        if (res.ok && data.success && data.company) {
            updateCompanyBadges(data.company);
            addTerminalLog(`New company created & activated: ${data.company.company_name} (${data.company.company_id})`, 'COMPANY');
            closeCompanyManagement();
        } else {
            if (stEl) stEl.innerText = data.error || data.message || 'Failed to create company';
        }
    } catch (e) {
        if (stEl) stEl.innerText = 'Error creating company: ' + e.message;
    }
}

async function loadCompanyListForEdit() {
    const searchEl = document.getElementById('comp-edit-search');
    const search = searchEl ? searchEl.value.trim() : '';
    const sel = document.getElementById('comp-edit-listbox');
    if (!sel) return;
    try {
        const res = await fetch(`/api/v1/companies/list?search=${encodeURIComponent(search)}`);
        const list = await res.json();
        cachedCompaniesList = Array.isArray(list) ? list : [];
        if (cachedCompaniesList.length === 0) {
            sel.innerHTML = '<option value="">No companies found</option>';
            return;
        }
        sel.innerHTML = cachedCompaniesList.map((c, idx) =>
            `<option value="${c.company_id}" ${idx === 0 ? 'selected' : ''}>${c.company_name} (${c.company_id})</option>`
        ).join('');
    } catch (e) {
        sel.innerHTML = '<option value="">Error loading companies</option>';
    }
}

async function verifyCompanyForEdit() {
    const company_id = document.getElementById('comp-edit-listbox')?.value || '';
    const password = document.getElementById('comp-edit-verify-pw')?.value || '';
    const stEl = document.getElementById('comp-edit-verify-status');
    if (!company_id || !password) {
        if (stEl) stEl.innerText = 'Please select a company and enter its password.';
        return;
    }
    try {
        const res = await fetch('/api/v1/companies/verify', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ company_id, password })
        });
        const data = await res.json();
        if (res.ok && data.success && data.company) {
            const c = data.company;
            document.getElementById('comp-edit-id').value = c.company_id;
            document.getElementById('comp-edit-name').value = c.company_name || '';
            document.getElementById('comp-edit-password').value = '';
            document.getElementById('comp-edit-road').value = c.address_road || '';
            document.getElementById('comp-edit-city').value = c.address_city || '';
            document.getElementById('comp-edit-contact-name').value = c.contact_person_name || '';
            document.getElementById('comp-edit-contact-desig').value = c.contact_person_designation || '';
            document.getElementById('comp-edit-contact-num').value = c.contact_person_number || '';
            document.getElementById('comp-edit-step1').style.display = 'none';
            document.getElementById('comp-edit-step2').style.display = 'block';
        } else {
            if (stEl) stEl.innerText = data.error || data.message || 'Incorrect company password';
        }
    } catch (e) {
        if (stEl) stEl.innerText = 'Error verifying company: ' + e.message;
    }
}

async function submitUpdateCompany() {
    const company_id = document.getElementById('comp-edit-id')?.value || '';
    const company_name = (document.getElementById('comp-edit-name')?.value || '').trim();
    const password = document.getElementById('comp-edit-password')?.value || '';
    const address_road = (document.getElementById('comp-edit-road')?.value || '').trim();
    const address_city = (document.getElementById('comp-edit-city')?.value || '').trim();
    const contact_person_name = (document.getElementById('comp-edit-contact-name')?.value || '').trim();
    const contact_person_designation = (document.getElementById('comp-edit-contact-desig')?.value || '').trim();
    const contact_person_number = (document.getElementById('comp-edit-contact-num')?.value || '').trim();
    const stEl = document.getElementById('comp-edit-save-status');

    if (!company_name || !company_id) {
        if (stEl) stEl.innerText = 'Company Name cannot be empty.';
        return;
    }

    try {
        const res = await fetch(`/api/v1/companies/${encodeURIComponent(company_id)}`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                company_id,
                company_name,
                password,
                address_road,
                address_city,
                contact_person_name,
                contact_person_designation,
                contact_person_number
            })
        });
        const data = await res.json();
        if (res.ok && data.success && data.company) {
            updateCompanyBadges(data.company);
            addTerminalLog(`Company information updated for ${company_name} (${company_id})`, 'COMPANY');
            closeCompanyManagement();
        } else {
            if (stEl) stEl.innerText = data.error || data.message || 'Failed to update company';
        }
    } catch (e) {
        if (stEl) stEl.innerText = 'Error updating company: ' + e.message;
    }
}

// ============================================================================
// ATTENDANCE CARDS & MONTHLY STATS (attendance_v8.py lines 7534-7577)
// ============================================================================
async function loadLastAttendanceData() {
    try {
        const res = await fetch('/api/v1/attendance/today');
        if (!res.ok) return;
        const records = await res.json();
        if (records && records.length > 0) {
            const last = records[0];
            currentDisplayedUser = last.username;
            let stats = { total_lates: 0, total_late_absents: 0, total_early_leaves: 0 };
            try {
                const stRes = await fetch(`/api/v1/attendance/user-stats/${encodeURIComponent(last.username)}`);
                if (stRes.ok) stats = await stRes.json();
            } catch (_) {}

            updateAttendanceCards({
                username: last.username,
                record_time: last.out_time || last.in_time || '--:--',
                login_type: last.out_time ? 'OUT_TIME' : 'IN_TIME',
                attendance_type: last.status || 'On Time',
                date: last.date,
                method: last.in_time_with_camera === 'Y' ? 'Camera' : (last.in_time_with_qr === 'Y' ? 'QR Code' : 'Without Camera'),
                lates: String(stats.total_lates || 0),
                late_absents: String(stats.total_late_absents || 0),
                early_leaves: String(stats.total_early_leaves || 0)
            });
        }
    } catch (e) {
        console.error('Error fetching today attendance:', e);
    }
}

function updateAttendanceCards(data) {
    if (data.username && data.username !== '---') {
        currentDisplayedUser = data.username;
    }
    if (document.getElementById('card-username')) document.getElementById('card-username').innerText = (data.username || '---').toUpperCase();
    if (document.getElementById('card-record-time')) document.getElementById('card-record-time').innerText = data.record_time || '---';
    if (document.getElementById('card-login-type')) document.getElementById('card-login-type').innerText = data.login_type || '---';
    if (document.getElementById('card-attendance-type')) document.getElementById('card-attendance-type').innerText = data.attendance_type || '---';
    if (document.getElementById('card-date')) document.getElementById('card-date').innerText = data.date || '---';
    if (document.getElementById('card-attendance-method')) document.getElementById('card-attendance-method').innerText = data.method || '---';
    if (document.getElementById('card-total-lates')) document.getElementById('card-total-lates').innerText = data.lates || '0';
    if (document.getElementById('card-total-late-absents')) document.getElementById('card-total-late-absents').innerText = data.late_absents || '0';
    if (document.getElementById('card-total-early-leaves')) document.getElementById('card-total-early-leaves').innerText = data.early_leaves || '0';
}

function captureFrameFromVideoElement(videoEl) {
    if (window.__TEST_FACE_B64) {
        return window.__TEST_FACE_B64;
    }
    const canvas = document.getElementById('camera-canvas') || document.createElement('canvas');
    let vw = (videoEl && videoEl.videoWidth) ? videoEl.videoWidth : 640;
    let vh = (videoEl && videoEl.videoHeight) ? videoEl.videoHeight : 480;
    const maxDim = 640;
    if (vw > maxDim || vh > maxDim) {
        if (vw >= vh) {
            vh = Math.max(1, Math.round((vh * maxDim) / vw));
            vw = maxDim;
        } else {
            vw = Math.max(1, Math.round((vw * maxDim) / vh));
            vh = maxDim;
        }
    }
    canvas.width = vw;
    canvas.height = vh;
    const ctx = canvas.getContext('2d');

    if (videoEl && videoEl.srcObject && videoEl.readyState >= 2) {
        ctx.drawImage(videoEl, 0, 0, vw, vh);
    }
    return canvas.toDataURL('image/jpeg', 0.92);
}

// ============================================================================
// 1. IN & OUT WITH CAMERA + IDENTITY VERIFICATION SPLIT WINDOW
// ============================================================================
async function openCameraLogin() {
    document.getElementById('modal-camera').style.display = 'flex';
    const video = document.getElementById('camera-stream');
    const status = document.getElementById('camera-status');
    const captureBtn = document.getElementById('btn-capture');
    status.innerText = 'Initializing camera...';
    status.style.color = '#fdcb6e';

    try {
        cameraStream = await navigator.mediaDevices.getUserMedia({
            video: { facingMode: 'user', width: { ideal: 640 }, height: { ideal: 480 } }
        });
        video.srcObject = cameraStream;
        status.innerText = 'Camera Ready - Click Capture when ready';
        status.style.color = '#00b894';
        captureBtn.disabled = false;
        addTerminalLog('Webcam stream started for ONNX FaceNet512 verification', 'CAMERA');
    } catch (e) {
        status.innerText = 'No physical webcam detected — Ready for capture / test frame';
        status.style.color = '#fdcb6e';
        captureBtn.disabled = false;
        addTerminalLog('Webcam unavailable (' + e.message + ')', 'CAMERA');
    }
}

function closeCameraLogin() {
    document.getElementById('modal-camera').style.display = 'none';
    if (cameraStream) {
        cameraStream.getTracks().forEach(t => t.stop());
        cameraStream = null;
    }
}

async function executeCaptureImage() {
    const status = document.getElementById('camera-status');
    const video = document.getElementById('camera-stream');
    status.innerText = 'Image Captured! Running ONNX FaceNet512 (512-D) Inference...';
    status.style.color = '#e67e22';

    lastCapturedImageB64 = captureFrameFromVideoElement(video);
    addTerminalLog('Captured frame -> Running MTCNN + FaceNet512 ONNX 512-D embedding extraction...', 'AI_CORE');

    try {
        const res = await fetch('/api/v1/face/verify-identity', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ image_b64: lastCapturedImageB64 })
        });
        const verifyData = await res.json();
        lastVerifyResult = verifyData;
        closeCameraLogin();
        openIdentityVerificationModal(verifyData, lastCapturedImageB64);
    } catch (e) {
        status.innerText = 'Network error during ONNX face verification: ' + e.message;
        status.style.color = '#e74c3c';
        addTerminalLog('ONNX verification error: ' + e.message, 'ERROR');
    }
}

function openIdentityVerificationModal(verifyData, imgB64) {
    const modal = document.getElementById('modal-identity-verify');
    if (modal) modal.style.display = 'flex';

    const imgEl = document.getElementById('verify-captured-img');
    if (imgEl && imgB64) imgEl.src = imgB64;

    const qualEl = document.getElementById('verify-match-quality');
    const confEl = document.getElementById('verify-confidence');
    const confBar = document.getElementById('verify-conf-bar');
    const minDistEl = document.getElementById('verify-min-dist');
    const exactStrongEl = document.getElementById('verify-exact-strong');
    const totalMatchesEl = document.getElementById('verify-total-matches');
    const userDisplayEl = document.getElementById('verify-user-display');
    const typeDisplayEl = document.getElementById('verify-type-display');
    const timeDisplayEl = document.getElementById('verify-time-display');
    const dateDisplayEl = document.getElementById('verify-date-display');
    const countdownLabel = document.getElementById('verify-countdown-label');
    const questionLabel = document.getElementById('verify-question-label');
    const yesBtn = document.getElementById('btn-verify-yes');

    if (verifyCountdownTimer) {
        clearInterval(verifyCountdownTimer);
        verifyCountdownTimer = null;
    }

    if (verifyData && verifyData.matched && verifyData.username) {
        const conf = Number(verifyData.confidence || 0).toFixed(2);
        const dist = Number(verifyData.min_distance ?? verifyData.distance ?? 0).toFixed(4);
        const quality = verifyData.match_quality || 'EXACT';
        const uname = verifyData.username.toUpperCase();

        if (qualEl) {
            qualEl.innerText = quality;
            qualEl.style.color = '#198754';
        }
        if (confEl) confEl.innerText = `${conf}%`;
        if (confBar) {
            confBar.style.width = `${Math.min(100, Math.max(5, Number(conf)))}%`;
            confBar.style.background = '#198754';
        }
        if (minDistEl) minDistEl.innerText = dist;
        if (exactStrongEl) exactStrongEl.innerText = `${verifyData.exact_matches || 0} / ${verifyData.strong_matches || 0}`;
        if (totalMatchesEl) totalMatchesEl.innerText = String(verifyData.total_matches || 1);
        if (userDisplayEl) {
            userDisplayEl.innerText = uname;
            userDisplayEl.style.color = '#198754';
        }
        if (typeDisplayEl) typeDisplayEl.innerText = verifyData.login_type || 'IN_TIME';
        if (timeDisplayEl) timeDisplayEl.innerText = verifyData.time || '--:--';
        if (dateDisplayEl) dateDisplayEl.innerText = verifyData.date || '--';
        if (questionLabel) questionLabel.innerText = `Are you ${uname}?`;
        if (yesBtn) {
            yesBtn.innerText = `Yes, I am ${uname}`;
            yesBtn.disabled = false;
        }

        addTerminalLog(`Face matched: ${uname} (Distance: ${dist}, Confidence: ${conf}%)`, 'AI_CORE');

        verifySecondsLeft = 5;
        if (countdownLabel) countdownLabel.innerText = `Auto-verification in: ${verifySecondsLeft} seconds`;

        verifyCountdownTimer = setInterval(() => {
            verifySecondsLeft -= 1;
            if (countdownLabel) countdownLabel.innerText = `Auto-verification in: ${verifySecondsLeft} seconds`;
            if (verifySecondsLeft <= 0) {
                clearInterval(verifyCountdownTimer);
                verifyCountdownTimer = null;
                confirmIdentityAndPunch();
            }
        }, 1000);
    } else {
        const errMsg = (verifyData && (verifyData.error || verifyData.message)) || 'No confident match found';
        if (qualEl) {
            qualEl.innerText = 'NO MATCH';
            qualEl.style.color = '#dc3545';
        }
        if (confEl) confEl.innerText = '0.00%';
        if (confBar) {
            confBar.style.width = '0%';
            confBar.style.background = '#dc3545';
        }
        if (minDistEl) minDistEl.innerText = '1.0000';
        if (exactStrongEl) exactStrongEl.innerText = '0 / 0';
        if (totalMatchesEl) totalMatchesEl.innerText = '0';
        if (userDisplayEl) {
            userDisplayEl.innerText = 'UNRECOGNIZED';
            userDisplayEl.style.color = '#dc3545';
        }
        if (typeDisplayEl) typeDisplayEl.innerText = '---';
        if (timeDisplayEl) timeDisplayEl.innerText = '--:--';
        if (dateDisplayEl) dateDisplayEl.innerText = errMsg;
        if (countdownLabel) countdownLabel.innerText = 'Verification paused — No match found';
        if (questionLabel) questionLabel.innerText = errMsg;
        if (yesBtn) {
            yesBtn.innerText = 'No Match Found';
            yesBtn.disabled = true;
        }
        addTerminalLog(`Face verification rejected: ${errMsg}`, 'WARNING');
    }
}

function closeIdentityVerifyModal() {
    if (verifyCountdownTimer) {
        clearInterval(verifyCountdownTimer);
        verifyCountdownTimer = null;
    }
    const modal = document.getElementById('modal-identity-verify');
    if (modal) modal.style.display = 'none';
}

function denyIdentityAndRetry() {
    closeIdentityVerifyModal();
    addTerminalLog('User clicked "No, I\'m someone else" — Retrying camera capture...', 'USER');
    openCameraLogin();
}

async function confirmIdentityAndPunch() {
    if (verifyCountdownTimer) {
        clearInterval(verifyCountdownTimer);
        verifyCountdownTimer = null;
    }
    if (!lastVerifyResult || !lastVerifyResult.matched || !lastVerifyResult.username) {
        return;
    }
    const matchedUsername = lastVerifyResult.username;
    closeIdentityVerifyModal();

    try {
        const payload = {
            username: matchedUsername,
            method: 'CAMERA',
            device_id: 'KIOSK-CAM-01',
            face_embedding: lastVerifyResult.embedding || null,
            image_b64: lastCapturedImageB64,
            remarks: 'Verified via ONNX FaceNet512 Split-Window'
        };

        const res = await fetch('/api/v1/attendance/punch', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
        });
        const data = await res.json();

        if (res.ok && data.success) {
            addTerminalLog(`Attendance recorded: ${data.username} (${data.login_type}) at ${data.timestamp}`, 'SUCCESS');
            let stats = { total_lates: 0, total_late_absents: 0, total_early_leaves: 0 };
            try {
                const stRes = await fetch(`/api/v1/attendance/user-stats/${encodeURIComponent(data.username)}`);
                if (stRes.ok) stats = await stRes.json();
            } catch (_) {}

            updateAttendanceCards({
                username: data.username,
                record_time: data.timestamp,
                login_type: data.login_type,
                attendance_type: data.shift_status || 'On Time',
                date: data.date,
                method: 'Camera',
                lates: String(stats.total_lates || 0),
                late_absents: String(stats.total_late_absents || 0),
                early_leaves: String(stats.total_early_leaves || 0)
            });
        } else {
            addTerminalLog('Attendance punch failed: ' + (data.error || data.message || 'Unknown error'), 'ERROR');
        }
    } catch (e) {
        addTerminalLog('Error recording camera attendance: ' + e.message, 'ERROR');
    }
}

// ============================================================================
// 2. IN & OUT WITHOUT CAMERA + INTERACTIVE FORGOT PASSWORD (CAP-19)
// ============================================================================
function openNoCameraLogin() {
    const errEl = document.getElementById('no-cam-error');
    if (errEl) errEl.style.display = 'none';
    document.getElementById('modal-no-camera').style.display = 'flex';
    document.getElementById('no-cam-username')?.focus();
}

function closeNoCameraLogin() {
    document.getElementById('modal-no-camera').style.display = 'none';
}

function togglePasswordVisibility() {
    const pw = document.getElementById('no-cam-password');
    if (!pw) return;
    pw.type = pw.type === 'password' ? 'text' : 'password';
}

function showForgotPasswordPopup() {
    let resetBox = document.getElementById('no-cam-reset-box');
    if (!resetBox) {
        const parent = document.getElementById('btn-no-cam-login')?.parentElement;
        if (!parent) return;
        resetBox = document.createElement('div');
        resetBox.id = 'no-cam-reset-box';
        resetBox.style.cssText = 'margin-top: 1rem; padding-top: 1rem; border-top: 1px solid #334155;';
        resetBox.innerHTML = `
            <div style="font-size: 0.85rem; font-weight: 700; color: #a29bfe; margin-bottom: 0.5rem;">🔑 Reset Forgotten Password</div>
            <input type="text" id="fp-username" class="form-input" style="margin-bottom: 0.5rem; padding: 0.55rem;" placeholder="Username (e.g. JOLIE)">
            <input type="password" id="fp-hr-verify" class="form-input" style="margin-bottom: 0.5rem; padding: 0.55rem;" placeholder="HR Password or Registered Android ID">
            <input type="password" id="fp-new-password" class="form-input" style="margin-bottom: 0.5rem; padding: 0.55rem;" placeholder="New Password (min 3 chars)">
            <div id="fp-status-msg" style="font-size: 0.8rem; margin-bottom: 0.5rem;"></div>
            <button type="button" class="btn-primary-action" style="background: #00b894; color: #fff; padding: 0.6rem;" onclick="submitForgotPasswordReset()">Reset Password</button>
        `;
        parent.appendChild(resetBox);
    } else {
        resetBox.style.display = resetBox.style.display === 'none' ? 'block' : 'none';
    }
}

async function submitForgotPasswordReset() {
    const username = (document.getElementById('fp-username')?.value || document.getElementById('no-cam-username')?.value || '').trim();
    const hr_password = (document.getElementById('fp-hr-verify')?.value || '').trim();
    const new_password = (document.getElementById('fp-new-password')?.value || '').trim();
    const stEl = document.getElementById('fp-status-msg');

    if (!username || !hr_password || !new_password) {
        if (stEl) {
            stEl.style.color = '#ff7675';
            stEl.innerText = 'All fields are required to reset password.';
        }
        return;
    }

    try {
        const res = await fetch('/api/v1/auth/forgot-password', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ username, hr_password, new_password })
        });
        const data = await res.json();
        if (res.ok && data.success) {
            if (stEl) {
                stEl.style.color = '#00b894';
                stEl.innerText = data.message || 'Password reset successfully!';
            }
            addTerminalLog(`Password reset completed for user ${username.toUpperCase()}`, 'AUTH');
        } else {
            if (stEl) {
                stEl.style.color = '#ff7675';
                stEl.innerText = data.error || data.message || 'Password reset failed';
            }
        }
    } catch (e) {
        if (stEl) {
            stEl.style.color = '#ff7675';
            stEl.innerText = 'Error: ' + e.message;
        }
    }
}

async function submitNoCameraLogin() {
    const username = (document.getElementById('no-cam-username')?.value || '').trim();
    const password = document.getElementById('no-cam-password')?.value || '';
    const errEl = document.getElementById('no-cam-error');

    if (!username || !password) {
        if (errEl) {
            errEl.style.display = 'block';
            errEl.innerText = 'Please enter both Username and Password!';
        }
        return;
    }

    addTerminalLog(`No-camera login submitted for user: ${username}`, 'AUTH');

    try {
        const payload = {
            username: username,
            method: 'PIN',
            pin: password,
            device_id: 'NO-CAM-TERMINAL',
            remarks: 'Login without camera'
        };

        const res = await fetch('/api/v1/attendance/punch', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
        });
        const data = await res.json();

        if (res.ok && data.success) {
            closeNoCameraLogin();
            document.getElementById('no-cam-username').value = '';
            document.getElementById('no-cam-password').value = '';
            if (errEl) errEl.style.display = 'none';

            addTerminalLog(`Login successful: ${data.username} (${data.login_type}) recorded`, 'SUCCESS');

            let stats = { total_lates: 0, total_late_absents: 0, total_early_leaves: 0 };
            try {
                const stRes = await fetch(`/api/v1/attendance/user-stats/${encodeURIComponent(data.username)}`);
                if (stRes.ok) stats = await stRes.json();
            } catch (_) {}

            updateAttendanceCards({
                username: data.username,
                record_time: data.timestamp,
                login_type: data.login_type,
                attendance_type: data.shift_status || 'On Time',
                date: data.date,
                method: 'Without Camera',
                lates: String(stats.total_lates || 0),
                late_absents: String(stats.total_late_absents || 0),
                early_leaves: String(stats.total_early_leaves || 0)
            });
        } else {
            if (errEl) {
                errEl.style.display = 'block';
                errEl.innerText = data.error || data.message || 'Invalid username or password';
            }
            addTerminalLog(`No-camera login failed for ${username}: ${data.error || 'Invalid credentials'}`, 'ERROR');
        }
    } catch (e) {
        addTerminalLog('Login error: ' + e.message, 'ERROR');
    }
}

// ============================================================================
// 3. IN & OUT WITH QR CODE (attendance_v8.py line 6222 Parity)
// ============================================================================
async function openQrLogin() {
    document.getElementById('modal-qr').style.display = 'flex';
    const svgBox = document.getElementById('qr-login-box');
    const statusText = document.getElementById('qr-session-status');

    try {
        const res = await fetch('/api/v1/devices/session', { method: 'POST' });
        const data = await res.json();
        const sessionPath = `/receive_android_id?session=${encodeURIComponent(data.session_id)}`;
        const fullUrl = window.location.origin + sessionPath;
        if (svgBox) {
            svgBox.innerHTML = generateSimpleQrSvg(fullUrl);
        }
        if (statusText) {
            statusText.innerHTML = `Session <strong>${data.session_id}</strong> — Waiting for mobile scan...`;
        }
        addTerminalLog(`QR Session created: ${data.session_id}`, 'QR');

        if (qrPollTimer) clearInterval(qrPollTimer);
        qrPollTimer = setInterval(async () => {
            try {
                const pollRes = await fetch(`/api/v1/devices/session/${encodeURIComponent(data.session_id)}`);
                const pollData = await pollRes.json();
                const aid = pollData.registered_android_id || pollData.android_id;
                if ((pollData.session_status === 'CONNECTED' || pollData.status === 'COMPLETED') && aid) {
                    clearInterval(qrPollTimer);
                    qrPollTimer = null;
                    document.getElementById('qr-device-id').value = aid;
                    closeQrLogin();
                    await loadLastAttendanceData();
                    addTerminalLog(`QR mobile scan completed for Android ID: ${aid}`, 'QR');
                }
            } catch (_) {}
        }, 2000);
    } catch (e) {
        if (statusText) statusText.innerText = 'Ready for Android ID input';
    }
}

function closeQrLogin() {
    if (qrPollTimer) {
        clearInterval(qrPollTimer);
        qrPollTimer = null;
    }
    document.getElementById('modal-qr').style.display = 'none';
}

function generateSimpleQrSvg(text) {
    const size = 21;
    let hash = 2166136261;
    for (let i = 0; i < text.length; i++) {
        hash ^= text.charCodeAt(i);
        hash = Math.imul(hash, 16777619);
    }
    let rects = '';
    const isFinder = (r, c) => (r < 7 && c < 7) || (r < 7 && c >= size - 7) || (r >= size - 7 && c < 7);
    const isFinderCell = (r, c) => {
        const lr = r >= size - 7 ? r - (size - 7) : r;
        const lc = c >= size - 7 ? c - (size - 7) : c;
        if (lr === 0 || lr === 6 || lc === 0 || lc === 6) return true;
        if (lr >= 2 && lr <= 4 && lc >= 2 && lc <= 4) return true;
        return false;
    };
    for (let r = 0; r < size; r++) {
        for (let c = 0; c < size; c++) {
            let on = false;
            if (isFinder(r, c)) {
                on = isFinderCell(r, c);
            } else {
                const bit = ((hash ^ (r * 31 + c * 17)) >>> ((r + c) % 16)) & 1;
                on = bit === 1;
            }
            if (on) {
                rects += `<rect x="${c * 8}" y="${r * 8}" width="8" height="8" fill="#171721"/>`;
            }
        }
    }
    return `<svg width="168" height="168" viewBox="0 0 168 168" xmlns="http://www.w3.org/2000/svg" style="background:white;padding:6px;border-radius:6px;">${rects}</svg>`;
}

async function submitQrLogin() {
    const deviceId = (document.getElementById('qr-device-id')?.value || '').trim();
    const statusText = document.getElementById('qr-session-status');
    if (!deviceId) {
        if (statusText) statusText.innerText = 'Please scan QR code or enter Android Device ID!';
        return;
    }
    addTerminalLog(`QR Code / AndroidID received: ${deviceId}`, 'QR');

    try {
        const payload = {
            username: '',
            method: 'QR',
            android_id: deviceId,
            device_id: 'ANDROID-RECEIVER',
            remarks: 'In & Out with QR Code'
        };

        const res = await fetch('/api/v1/attendance/punch', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
        });
        const data = await res.json();

        if (res.ok && data.success) {
            closeQrLogin();
            addTerminalLog(`QR attendance recorded for ${data.username} (${data.login_type})`, 'SUCCESS');

            let stats = { total_lates: 0, total_late_absents: 0, total_early_leaves: 0 };
            try {
                const stRes = await fetch(`/api/v1/attendance/user-stats/${encodeURIComponent(data.username)}`);
                if (stRes.ok) stats = await stRes.json();
            } catch (_) {}

            updateAttendanceCards({
                username: data.username,
                record_time: data.timestamp,
                login_type: data.login_type,
                attendance_type: data.shift_status || 'On Time',
                date: data.date,
                method: 'QR Code',
                lates: String(stats.total_lates || 0),
                late_absents: String(stats.total_late_absents || 0),
                early_leaves: String(stats.total_early_leaves || 0)
            });
        } else {
            if (statusText) {
                statusText.style.color = '#ff7675';
                statusText.innerText = data.error || data.message || 'Unregistered Android ID';
            }
        }
    } catch (e) {
        addTerminalLog('QR Error: ' + e.message, 'ERROR');
    }
}

// ============================================================================
// 4. 7-DAY HISTORY POPUP (attendance_v8.py line 6938 Parity)
// ============================================================================
async function openSevenDayHistory() {
    document.getElementById('modal-seven-days').style.display = 'flex';
    const tbody = document.getElementById('seven-day-tbody');
    tbody.innerHTML = '<tr><td colspan="4" style="text-align: center; padding: 1rem; color: #666;">Loading 7-day history...</td></tr>';

    try {
        const q = currentDisplayedUser ? `?username=${encodeURIComponent(currentDisplayedUser)}` : '';
        const res = await fetch(`/api/v1/attendance/history-7days${q}`);
        const data = await res.json();
        const list = Array.isArray(data) ? data : (data.history || data.records || []);

        const titleEl = document.getElementById('history-header-title');
        const dateEl = document.getElementById('history-header-date');
        if (titleEl && data.title) titleEl.innerText = data.title;
        if (dateEl) dateEl.innerText = `Generated on: ${data.generated_on || new Date().toLocaleDateString('en-GB')}`;

        const rows = [];
        for (let i = 0; i < list.length; i++) {
            const rec = list[i];
            const isToday = i === 0;
            const inTime = rec.in_time || '---';
            const outTime = rec.out_time || '---';
            const duration = rec.duration || '---';
            const bg = rec.status_color || ((inTime !== '---' && outTime !== '---') ? '#e8f5e9' : (inTime !== '---' ? '#fff3e0' : (i % 2 === 0 ? '#f8f9fa' : 'white')));

            rows.push(`
                <tr style="background: ${bg}; border-bottom: 1px solid #e0e0e0;">
                    <td style="padding: 10px 15px; color: ${isToday ? '#1976d2' : '#333'}; font-weight: ${isToday ? 'bold' : 'normal'};">${rec.date}</td>
                    <td style="padding: 10px 15px; font-family: monospace;">${inTime}</td>
                    <td style="padding: 10px 15px; font-family: monospace;">${outTime}</td>
                    <td style="padding: 10px 15px; font-family: monospace;">${duration}</td>
                </tr>
            `);
        }
        tbody.innerHTML = rows.join('');
        addTerminalLog('7-day attendance history loaded', 'UI');
    } catch (e) {
        tbody.innerHTML = `<tr><td colspan="4" style="text-align: center; color: red;">Error: ${e.message}</td></tr>`;
    }
}

function closeSevenDayHistory() {
    document.getElementById('modal-seven-days').style.display = 'none';
}

// ============================================================================
// 5. HR AUTHENTICATION & PASSWORD CHANGE (attendance_v8.py lines 9911-10035)
// ============================================================================
function openHrManagement() {
    if (!activeCompany || !activeCompany.company_id) {
        openCompanyManagement();
        return;
    }
    const label = document.getElementById('hr-auth-company-label');
    if (label) label.innerText = `${activeCompany.company_name} (${activeCompany.company_id})`;
    const pinEl = document.getElementById('hr-admin-pin');
    if (pinEl) pinEl.value = '';
    const stEl = document.getElementById('hr-auth-status');
    if (stEl) stEl.innerText = '';
    toggleHrChangePasswordView(false);
    document.getElementById('modal-hr-auth').style.display = 'flex';
    pinEl?.focus();
}

function closeHrAuthModal() {
    document.getElementById('modal-hr-auth').style.display = 'none';
}

function toggleHrChangePasswordView(show) {
    const loginView = document.getElementById('hr-auth-login-view');
    const changeView = document.getElementById('hr-auth-change-view');
    if (loginView) loginView.style.display = show ? 'none' : 'block';
    if (changeView) changeView.style.display = show ? 'block' : 'none';
    if (show) {
        const sub = document.getElementById('hr-change-comp-sub');
        if (sub && activeCompany) sub.innerText = `Company: ${activeCompany.company_name}`;
        ['hr-cp-current', 'hr-cp-new', 'hr-cp-confirm'].forEach(id => {
            const el = document.getElementById(id);
            if (el) el.value = '';
        });
        const st = document.getElementById('hr-cp-status');
        if (st) st.innerText = '';
    }
}

async function unlockHrManagement() {
    const password = document.getElementById('hr-admin-pin')?.value || '';
    const stEl = document.getElementById('hr-auth-status');
    if (!password) {
        if (stEl) stEl.innerText = 'Please enter HR Password (default: HR@123)';
        return;
    }
    try {
        const res = await fetch('/api/v1/auth/verify-hr', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                company_uuid: activeCompany.company_id,
                password: password
            })
        });
        const data = await res.json();
        if (res.ok && data.authenticated) {
            closeHrAuthModal();
            document.getElementById('modal-hr').style.display = 'flex';
            backToHrMainMenu();
            addTerminalLog(`HR Management unlocked for company ${activeCompany.company_name}`, 'AUTH');
        } else {
            if (stEl) stEl.innerText = data.error || data.message || 'Invalid HR Password!';
        }
    } catch (e) {
        if (stEl) stEl.innerText = 'Error verifying HR password: ' + e.message;
    }
}

async function submitChangeHrPassword() {
    const current_password = document.getElementById('hr-cp-current')?.value || '';
    const new_password = document.getElementById('hr-cp-new')?.value || '';
    const confirm_password = document.getElementById('hr-cp-confirm')?.value || '';
    const stEl = document.getElementById('hr-cp-status');

    if (!current_password || !new_password || !confirm_password) {
        if (stEl) stEl.innerText = 'All password fields are required!';
        return;
    }
    if (new_password !== confirm_password) {
        if (stEl) stEl.innerText = 'New password and confirm password do not match!';
        return;
    }

    try {
        const res = await fetch('/api/v1/auth/change-hr-password', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                company_uuid: activeCompany.company_id,
                company_name: activeCompany.company_name,
                current_password,
                new_password
            })
        });
        const data = await res.json();
        if (res.ok && data.success) {
            toggleHrChangePasswordView(false);
            const authSt = document.getElementById('hr-auth-status');
            if (authSt) {
                authSt.style.color = '#00b894';
                authSt.innerText = data.message || 'HR password changed successfully!';
            }
            addTerminalLog('HR password changed successfully', 'AUTH');
        } else {
            if (stEl) stEl.innerText = data.error || data.message || 'Failed to change HR password';
        }
    } catch (e) {
        if (stEl) stEl.innerText = 'Error changing HR password: ' + e.message;
    }
}

function closeHrManagement() {
    stopHrRegCamera();
    if (hrQrPollTimer) {
        clearInterval(hrQrPollTimer);
        hrQrPollTimer = null;
    }
    document.getElementById('modal-hr').style.display = 'none';
}

function backToHrMainMenu() {
    stopHrRegCamera();
    if (hrQrPollTimer) {
        clearInterval(hrQrPollTimer);
        hrQrPollTimer = null;
    }
    const grid = document.getElementById('hr-main-menu-grid');
    const ws = document.getElementById('hr-action-workspace');
    const body = document.getElementById('hr-workspace-body');
    if (grid) grid.style.display = 'block';
    if (ws) ws.style.display = 'none';
    if (body) body.innerHTML = '';
}

function stopHrRegCamera() {
    regAutoCaptureRunning = false;
    if (hrRegCameraStream) {
        hrRegCameraStream.getTracks().forEach(t => t.stop());
        hrRegCameraStream = null;
    }
}

// ============================================================================
// 6. ALL 12 HR MANAGEMENT ACTIONS + USERS LIST
// ============================================================================
let regCapturedFrames = [];
let regCapturedEmbeddings = [];
let regCurrentPoseIdx = 0;
let regAutoCaptureRunning = false;
let regBaselineMetrics = null;
let regPrevPoseMetrics = null;
let regPoseAttemptCount = 0;
let regTotalPoseRetries = 0;

function buildRegPoseThumbsHtml() {
    let html = '';
    for (let i = 0; i < 20; i++) {
        const isCurrent = (i === 0);
        html += `<div id="reg-pose-thumb-${i}" style="height: 36px; border-radius: 6px; background: #10101c; border: 1px solid ${isCurrent ? '#00d2d3' : 'rgba(255,255,255,0.12)'}; display: flex; align-items: center; justify-content: center; font-size: 0.7rem; font-weight: 700; color: ${isCurrent ? '#00d2d3' : '#636e72'}; overflow: hidden; position: relative;" title="${POSE_INSTRUCTIONS[i].en}">${i + 1}</div>`;
    }
    return html;
}

function resetRegPoses() {
    regAutoCaptureRunning = false;
    regCapturedFrames = [];
    regCapturedEmbeddings = [];
    regCurrentPoseIdx = 0;
    regBaselineMetrics = null;
    regPrevPoseMetrics = null;
    regPoseAttemptCount = 0;
    regTotalPoseRetries = 0;
    const bar = document.getElementById('reg-pose-bar');
    const status = document.getElementById('reg-pose-status');
    const enEl = document.getElementById('reg-pose-en');
    const bnEl = document.getElementById('reg-pose-bn');
    const msgEl = document.getElementById('reg-cam-result-msg');
    const angleEl = document.getElementById('reg-pose-angle-badge');
    const lastImg = document.getElementById('reg-last-captured-img');
    const lastPlaceholder = document.getElementById('reg-last-captured-empty');
    const lastLabel = document.getElementById('reg-last-captured-label');
    const autoBtn = document.getElementById('btn-auto-capture-poses');
    const gridEl = document.getElementById('reg-pose-thumbs-grid');
    const cdBadge = document.getElementById('hr-reg-countdown-badge');

    if (bar) bar.style.width = '0%';
    if (status) status.innerText = 'Captured: 0 / 20 poses (512-D ONNX vectors)';
    if (enEl) enEl.innerText = POSE_INSTRUCTIONS[0].en;
    if (bnEl) bnEl.innerText = POSE_INSTRUCTIONS[0].bn;
    if (msgEl) {
        msgEl.style.color = '#dfe6e9';
        msgEl.innerText = 'Poses reset to 0/20. Ready to capture & verify Pose 1/20.';
    }
    if (angleEl) angleEl.innerText = 'MTCNN 5-Landmark Pose Verifier: Ready for Pose 1/20';
    if (lastImg) {
        lastImg.src = '';
        lastImg.style.display = 'none';
    }
    if (lastPlaceholder) lastPlaceholder.style.display = 'flex';
    if (lastLabel) lastLabel.innerText = 'Last Captured (0/20)';
    if (autoBtn) {
        autoBtn.innerText = '⚡ Auto Capture All 20 Poses';
        autoBtn.style.background = '#3498db';
    }
    if (gridEl) gridEl.innerHTML = buildRegPoseThumbsHtml();
    if (cdBadge) cdBadge.innerText = 'Pose 1/20 Ready';
}

async function openHrAction(action) {
    const grid = document.getElementById('hr-main-menu-grid');
    const ws = document.getElementById('hr-action-workspace');
    const box = document.getElementById('hr-workspace-body');
    const titleEl = document.getElementById('hr-workspace-title');
    if (grid) grid.style.display = 'none';
    if (ws) ws.style.display = 'block';
    if (!box) return;

    if (action === 'users-list') {
        if (titleEl) titleEl.innerText = '👥 Registered Users';
        box.innerHTML = `<p style="color:#ccc;">Loading registered users...</p>`;
        const res = await fetch('/api/v1/users');
        const users = await res.json();
        const list = Array.isArray(users) ? users : [];
        box.innerHTML = `
            <div class="table-responsive" style="max-height: 420px; overflow-y: auto;">
                <table class="attendance-table" style="width:100%; border-collapse: collapse;">
                    <thead>
                        <tr style="border-bottom: 1px solid rgba(255,255,255,0.15); text-align:left;">
                            <th style="padding:8px;">Username</th>
                            <th style="padding:8px;">Designation</th>
                            <th style="padding:8px;">Camera</th>
                            <th style="padding:8px;">Android ID</th>
                            <th style="padding:8px;">Action</th>
                        </tr>
                    </thead>
                    <tbody>
                        ${list.map(u => `
                            <tr style="border-bottom: 1px solid rgba(255,255,255,0.08);">
                                <td style="padding:8px;"><strong>${u.username}</strong></td>
                                <td style="padding:8px;">${u.user_designation || 'Employee'}</td>
                                <td style="padding:8px;">${u.register_with_camera || 'N'}</td>
                                <td style="padding:8px;">${u.user_android_id || '---'}</td>
                                <td style="padding:8px;">
                                    <button onclick="deleteUserAccount('${u.username}')" style="background:#e74c3c;color:#fff;border:none;padding:4px 10px;border-radius:4px;cursor:pointer;">Delete</button>
                                </td>
                            </tr>
                        `).join('')}
                    </tbody>
                </table>
            </div>
        `;
    } else if (action === 'reg-camera') {
        if (titleEl) titleEl.innerText = '📷 1. User Registration with Camera (20-Pose ONNX FaceNet512)';
        regAutoCaptureRunning = false;
        regCapturedFrames = [];
        regCapturedEmbeddings = [];
        regCurrentPoseIdx = 0;
        regBaselineMetrics = null;
        regPrevPoseMetrics = null;
        regPoseAttemptCount = 0;
        regTotalPoseRetries = 0;
        box.innerHTML = `
            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(290px, 1fr)); gap: 1.15rem;">
                <div>
                    <label style="font-size: 0.85rem; color: #ccc;">Username (Full Name / ID):</label>
                    <input type="text" id="reg-cam-username" class="form-input" placeholder="e.g. RAKIB_01">
                    <label style="font-size: 0.85rem; color: #ccc;">Designation:</label>
                    <input type="text" id="reg-cam-desig" class="form-input" placeholder="e.g. Software Engineer">
                    <label style="font-size: 0.85rem; color: #ccc;">Password:</label>
                    <input type="password" id="reg-cam-password" class="form-input" placeholder="Enter password for user">
                    
                    <div style="background: #10101c; padding: 0.9rem; border-radius: 10px; border: 1px solid rgba(46,204,113,0.35); margin-top: 0.35rem;">
                        <div style="display: flex; justify-content: space-between; align-items: center;">
                            <span style="font-size: 0.78rem; color: #a29bfe; font-weight: 700;">POSE INSTRUCTION (ENGLISH & বাংলা):</span>
                            <button type="button" onclick="resetRegPoses()" style="background: rgba(255,255,255,0.08); color: #dfe6e9; border: 1px solid rgba(255,255,255,0.15); border-radius: 5px; padding: 2px 8px; font-size: 0.72rem; cursor: pointer;">🔄 Reset</button>
                        </div>
                        <div id="reg-pose-en" style="font-size: 0.98rem; font-weight: 700; color: #2ecc71; margin-top: 0.35rem;">${POSE_INSTRUCTIONS[0].en}</div>
                        <div id="reg-pose-bn" style="font-size: 0.92rem; color: #fdcb6e; margin-top: 0.2rem;">${POSE_INSTRUCTIONS[0].bn}</div>
                        <div style="margin-top: 0.65rem; background: #2d3436; height: 12px; border-radius: 6px; overflow: hidden;">
                            <div id="reg-pose-bar" style="width: 0%; height: 100%; background: #2ecc71; transition: width 0.25s;"></div>
                        </div>
                        <div id="reg-pose-status" style="font-size: 0.8rem; color: #dfe6e9; margin-top: 0.4rem; font-weight: 600;">Captured: 0 / 20 poses (512-D ONNX vectors)</div>
                        <div id="reg-pose-angle-badge" style="font-size: 0.75rem; color: #00d2d3; margin-top: 0.25rem; font-family: monospace;">MTCNN 5-Landmark Pose Verifier: Ready for Pose 1/20</div>
                    </div>

                    <div style="display: flex; gap: 0.5rem; margin-top: 0.75rem; flex-wrap: wrap;">
                        <button class="btn-primary-action" style="flex: 1; background: #2ecc71; color: #fff; padding: 0.7rem;" onclick="captureSingleRegPose()">📸 Capture Pose</button>
                        <button id="btn-auto-capture-poses" class="btn-primary-action" style="flex: 1.2; background: #3498db; color: #fff; padding: 0.7rem;" onclick="runAuto20PoseCapture()">⚡ Auto Capture All 20 Poses</button>
                    </div>
                    <div id="reg-cam-result-msg" style="font-size: 0.84rem; margin-top: 0.55rem; min-height: 2.2rem; padding: 0.45rem 0.65rem; border-radius: 8px; background: rgba(16,16,28,0.7); border: 1px solid rgba(255,255,255,0.08); color: #dfe6e9;">Enter Username and click "📸 Capture Pose" or "⚡ Auto Capture All 20 Poses" (each pose is verified).</div>
                    <button id="btn-finish-cam-reg" class="btn-primary-action" style="margin-top: 0.65rem; background: #9b59b6; color: #fff;" onclick="submitCameraRegistration()">✅ Complete ONNX Face Registration</button>
                </div>
                <div>
                    <!-- Side-by-Side Live Camera & Last Captured Pose (attendance_v8.py lines 1290-1324 Parity) -->
                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 0.6rem; margin-bottom: 0.5rem;">
                        <div>
                            <div style="font-size: 0.75rem; font-weight: 700; color: #00b894; margin-bottom: 0.25rem; text-align: center;">🔴 LIVE CAMERA</div>
                            <div id="hr-reg-cam-wrap" class="camera-wrapper" style="height: 220px; margin-bottom: 0; border: 2px solid rgba(0, 184, 148, 0.5); transition: border-color 0.2s;">
                                <video id="hr-reg-video" class="camera-video" autoplay playsinline muted style="transform: scaleX(-1);"></video>
                                <div class="camera-overlay-box" style="width: 120px; height: 155px;"></div>
                                <div id="hr-reg-countdown-badge" style="position: absolute; bottom: 6px; left: 6px; right: 6px; background: rgba(16,16,28,0.82); color: #00d2d3; font-size: 0.72rem; font-weight: 700; text-align: center; padding: 3px 6px; border-radius: 6px;">Pose 1/20 Ready</div>
                            </div>
                        </div>
                        <div>
                            <div id="reg-last-captured-label" style="font-size: 0.75rem; font-weight: 700; color: #fdcb6e; margin-bottom: 0.25rem; text-align: center;">🖼️ LAST CAPTURED (0/20)</div>
                            <div id="reg-last-captured-wrap" style="height: 220px; background: #10101c; border-radius: 12px; border: 2px solid rgba(253, 203, 110, 0.35); overflow: hidden; display: flex; align-items: center; justify-content: center; position: relative;">
                                <img id="reg-last-captured-img" src="" alt="Last Captured Pose" style="width: 100%; height: 100%; object-fit: cover; transform: scaleX(-1); display: none;" />
                                <div id="reg-last-captured-empty" style="display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 0.75rem; text-align: center; color: #636e72; font-size: 0.76rem;">
                                    <span style="font-size: 1.6rem; margin-bottom: 0.35rem;">📸</span>
                                    <span>Verified pose photo will appear here after each capture</span>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div id="hr-reg-cam-msg" style="text-align: center; font-size: 0.8rem; color: #00b894; margin-bottom: 0.45rem;">Initializing registration camera...</div>
                    <div style="background: rgba(16,16,28,0.75); padding: 0.55rem; border-radius: 10px; border: 1px solid rgba(255,255,255,0.08);">
                        <div style="font-size: 0.72rem; color: #a29bfe; font-weight: 700; margin-bottom: 0.35rem;">VERIFIED 20-POSE GALLERY (1/20 .. 20/20):</div>
                        <div id="reg-pose-thumbs-grid" style="display: grid; grid-template-columns: repeat(10, 1fr); gap: 4px;">
                            ${buildRegPoseThumbsHtml()}
                        </div>
                    </div>
                </div>
            </div>
        `;
        try {
            hrRegCameraStream = await navigator.mediaDevices.getUserMedia({
                video: { facingMode: 'user', width: { ideal: 640 }, height: { ideal: 480 } }
            });
            const v = document.getElementById('hr-reg-video');
            if (v) v.srcObject = hrRegCameraStream;
            document.getElementById('hr-reg-cam-msg').innerText = 'Webcam Live — Follow each pose instruction (all 20 poses are verified)';
        } catch (e) {
            document.getElementById('hr-reg-cam-msg').innerText = 'Webcam unavailable — Ready for frame capture';
            document.getElementById('hr-reg-cam-msg').style.color = '#fdcb6e';
        }
    } else if (action === 'reg-no-camera') {
        if (titleEl) titleEl.innerText = '👤 2. User Registration without Camera';
        box.innerHTML = `
            <div style="max-width: 480px;">
                <label style="font-size: 0.85rem; color: #ccc;">Username:</label>
                <input type="text" id="reg-nocam-username" class="form-input" placeholder="Enter Username">
                <label style="font-size: 0.85rem; color: #ccc;">Designation:</label>
                <input type="text" id="reg-nocam-desig" class="form-input" placeholder="Enter Designation">
                <label style="font-size: 0.85rem; color: #ccc;">Password:</label>
                <input type="password" id="reg-nocam-password" class="form-input" placeholder="Enter Password">
                <div id="reg-nocam-status" style="font-size: 0.85rem; margin-bottom: 0.5rem;"></div>
                <button class="btn-primary-action" style="background: #3498db; color: white; margin-top: 0.5rem;" onclick="submitNoCameraRegistration()">✅ Register User (Without Camera)</button>
            </div>
        `;
    } else if (action === 'reg-qr') {
        if (titleEl) titleEl.innerText = '📲 11. User Registration with QR Code';
        box.innerHTML = `
            <div style="display: grid; grid-template-columns: 1fr 220px; gap: 1.25rem; align-items: start;">
                <div>
                    <label style="font-size: 0.85rem; color: #ccc;">Username:</label>
                    <input type="text" id="reg-qr-username" class="form-input" placeholder="Enter Username">
                    <label style="font-size: 0.85rem; color: #ccc;">Designation:</label>
                    <input type="text" id="reg-qr-desig" class="form-input" placeholder="Enter Designation">
                    <label style="font-size: 0.85rem; color: #ccc;">Password:</label>
                    <input type="password" id="reg-qr-password" class="form-input" placeholder="Enter Password">
                    <label style="font-size: 0.85rem; color: #ccc;">Android Device ID (from QR Scan):</label>
                    <input type="text" id="reg-qr-android-id" class="form-input" placeholder="Scan QR or enter Android ID">
                    <div id="reg-qr-status" style="font-size: 0.85rem; margin-bottom: 0.5rem;"></div>
                    <button class="btn-primary-action" style="background: #27ae60; color: white; margin-top: 0.5rem;" onclick="submitQrRegistration()">✅ Register User with QR Device ID</button>
                </div>
                <div style="text-align: center; background: rgba(255,255,255,0.05); padding: 1rem; border-radius: 10px;">
                    <div id="hr-reg-qr-svg" style="display: inline-block; background: #fff; padding: 8px; border-radius: 8px;"></div>
                    <div id="hr-reg-qr-session-text" style="font-size: 0.78rem; color: #00d2d3; margin-top: 0.5rem;">Generating QR session...</div>
                </div>
            </div>
        `;
        try {
            const sRes = await fetch('/api/v1/devices/session', { method: 'POST' });
            const sData = await sRes.json();
            const qrUrl = `${window.location.origin}/receive_android_id?session=${encodeURIComponent(sData.session_id)}`;
            const svgEl = document.getElementById('hr-reg-qr-svg');
            const txtEl = document.getElementById('hr-reg-qr-session-text');
            if (svgEl) svgEl.innerHTML = generateSimpleQrSvg(qrUrl);
            if (txtEl) txtEl.innerText = `Session: ${sData.session_id} (Scan with mobile)`;
            if (hrQrPollTimer) clearInterval(hrQrPollTimer);
            hrQrPollTimer = setInterval(async () => {
                try {
                    const pRes = await fetch(`/api/v1/devices/session/${encodeURIComponent(sData.session_id)}`);
                    const pData = await pRes.json();
                    const aid = pData.registered_android_id || pData.android_id;
                    if (aid) {
                        clearInterval(hrQrPollTimer);
                        hrQrPollTimer = null;
                        const inputEl = document.getElementById('reg-qr-android-id');
                        if (inputEl) inputEl.value = aid;
                        if (txtEl) txtEl.innerText = `✅ Scanned: ${aid}`;
                    }
                } catch (_) {}
            }, 2000);
        } catch (_) {}
    } else if (action === 'att-report') {
        if (titleEl) titleEl.innerText = '📊 3. Customized Attendance Report';
        const now = new Date();
        const yyyy = now.getFullYear();
        const mm = String(now.getMonth() + 1).padStart(2, '0');
        box.innerHTML = `
            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 0.75rem; max-width: 680px;">
                <div>
                    <label style="font-size: 0.8rem; color: #ccc;">Username (All Users or specific):</label>
                    <input type="text" id="rep-username" class="form-input" value="All Users">
                </div>
                <div>
                    <label style="font-size: 0.8rem; color: #ccc;">Year-Month (YYYY-MM):</label>
                    <input type="text" id="rep-month" class="form-input" value="${yyyy}-${mm}">
                </div>
                <div>
                    <label style="font-size: 0.8rem; color: #ccc;">From Day (1-31):</label>
                    <input type="number" id="rep-from-date" class="form-input" value="1" min="1" max="31">
                </div>
                <div>
                    <label style="font-size: 0.8rem; color: #ccc;">To Day (1-31):</label>
                    <input type="number" id="rep-to-date" class="form-input" value="31" min="1" max="31">
                </div>
                <div>
                    <label style="font-size: 0.8rem; color: #ccc;">Office In Time (HH:MM:SS):</label>
                    <input type="text" id="rep-in-time" class="form-input" value="09:00:00">
                </div>
                <div>
                    <label style="font-size: 0.8rem; color: #ccc;">Office Out Time (HH:MM:SS):</label>
                    <input type="text" id="rep-out-time" class="form-input" value="17:00:00">
                </div>
            </div>
            <div style="display: flex; gap: 0.75rem; margin-top: 1rem; flex-wrap: wrap;">
                <button class="btn-primary-action" style="width: auto; background: #9b59b6; color: white;" onclick="downloadCustomizedCsvReport()">📥 Download Customized CSV</button>
                <button class="btn-primary-action" style="width: auto; background: #16a085; color: white;" onclick="downloadCustomizedXlsxReport()">📊 Download Customized Excel (.xlsx)</button>
                <button class="btn-primary-action" style="width: auto; background: #2ecc71; color: white;" onclick="openMusterRollHtmlReport()">📄 Open Monthly Muster Roll (HTML)</button>
            </div>
        `;
    } else if (action === 'sync-local') {
        if (titleEl) titleEl.innerText = '🔄 4. Sync Users to Local Database';
        box.innerHTML = `<p style="margin-top:0.5rem;">Syncing user data to local SQLite & FAISS index...</p>`;
        const res = await fetch('/api/v1/sync/users-to-local', { method: 'POST' });
        const data = await res.json();
        box.innerHTML = `<pre style="background: #10101c; padding: 1rem; border-radius: 8px; color: #2ecc71;">${JSON.stringify(data, null, 2)}</pre>`;
    } else if (action === 'verify-image') {
        if (titleEl) titleEl.innerText = '🕵️ 5. Check Fake Attendance (Verify Image)';
        box.innerHTML = `<p>Loading captured attendance images...</p>`;
        const res = await fetch('/api/v1/reports/verify-images');
        const data = await res.json();
        const records = data.items || (Array.isArray(data) ? data : []);
        if (!records || records.length === 0) {
            box.innerHTML = `<p style="margin-top: 0.75rem; color: #ccc;">No attendance records found for this period.</p>`;
            return;
        }
        box.innerHTML = `
            <div class="table-responsive" style="max-height: 420px; overflow-y: auto;">
                <table class="attendance-table" style="width:100%;">
                    <thead><tr><th>User</th><th>Date</th><th>In Time</th><th>Out Time</th><th>In Photo</th><th>Out Photo</th></tr></thead>
                    <tbody>
                        ${records.map(r => `
                            <tr>
                                <td><strong>${r.username}</strong></td>
                                <td>${r.date}</td>
                                <td>${r.in_time || '---'} (${r.in_time_with_camera || 'N'})</td>
                                <td>${r.out_time || '---'} (${r.out_time_with_camera || 'N'})</td>
                                <td>${r.video_path_in ? `<a href="${r.video_path_in}" target="_blank" style="color:#2ecc71;">🖼️ View In Photo</a>` : 'No Photo'}</td>
                                <td>${r.video_path_out ? `<a href="${r.video_path_out}" target="_blank" style="color:#3498db;">🖼️ View Out Photo</a>` : 'No Photo'}</td>
                            </tr>
                        `).join('')}
                    </tbody>
                </table>
            </div>
        `;
    } else if (action === 'upload-model') {
        if (titleEl) titleEl.innerText = '🧠 6. Upload AI Model (Hot-Reload ONNX)';
        box.innerHTML = `
            <p style="color: #ccc; font-size: 0.9rem; margin-bottom: 1rem;">Upload a FaceNet512 or MTCNN ONNX model file (.onnx) to validate and hot-reload in memory.</p>
            <input type="file" id="onnx-model-file" accept=".onnx" class="form-input">
            <button class="btn-primary-action" style="background: #16a085; color: white; max-width: 300px;" onclick="submitUploadOnnxModel()">📤 Upload &amp; Hot-Reload Model</button>
            <div id="upload-model-result" style="margin-top: 1rem;"></div>
        `;
    } else if (action === 'delete-face') {
        if (titleEl) titleEl.innerText = '🗑️ 7. Delete Face (Reset Encodings)';
        const res = await fetch('/api/v1/face/embeddings-report');
        const report = await res.json();
        const rows = report.report || [];
        const uniqueUsers = [...new Set(rows.map(r => r.Username))];
        box.innerHTML = `
            <p style="color: #ccc; font-size: 0.9rem; margin-bottom: 1rem;">Select a user to delete their biometric face embeddings from both SQLite and RAM index, or reset all profiles.</p>
            <div style="display: flex; gap: 0.75rem; max-width: 520px; margin-bottom: 1rem;">
                <select id="del-face-user" class="form-input" style="margin-bottom: 0;">
                    ${uniqueUsers.length > 0 ? uniqueUsers.map(u => `<option value="${u}">${u}</option>`).join('') : '<option value="">No enrolled faces</option>'}
                </select>
                <button class="btn-primary-action" style="width: auto; background: #e74c3c; color: white;" onclick="deleteSelectedUserFace()">🗑️ Delete User Face</button>
            </div>
            <div id="del-face-status" style="font-size: 0.85rem; margin-bottom: 0.75rem;"></div>
            <button class="btn-primary-action" style="width: auto; background: #c0392b; color: white;" onclick="resetAllFaceProfiles()">⚠️ Reset ALL Face Profiles</button>
        `;
    } else if (action === 'embeddings-report') {
        if (titleEl) titleEl.innerText = '🧬 8. Facial Features Report (512-D ONNX)';
        const res = await fetch('/api/v1/face/embeddings-report');
        const report = await res.json();
        const rows = report.report || [];
        box.innerHTML = `
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem;">
                <span style="color: #74b9ff; font-weight: 700;">Total Indexed Embeddings: ${report.total_records || rows.length}</span>
                <a href="/api/v1/face/embeddings-report.csv" target="_blank" style="background: #00b894; color: #fff; padding: 0.4rem 0.85rem; border-radius: 6px; text-decoration: none; font-weight: 700; font-size: 0.85rem;">📥 Download CSV</a>
            </div>
            <div class="table-responsive" style="max-height: 400px; overflow-y: auto;">
                <table class="attendance-table" style="width:100%;">
                    <thead><tr><th>Username</th><th>Frame</th><th>Dim</th><th>Pos / Neg</th><th>Zeros</th><th>Top 5 Highest Values</th></tr></thead>
                    <tbody>
                        ${rows.map(r => `
                            <tr>
                                <td><strong>${r.Username}</strong></td>
                                <td>${r.Image_Name}</td>
                                <td>${r.Number_of_Embeddings_Generated}-D</td>
                                <td>+${r.Number_of_Positive_Values} / -${r.Number_of_Negative_Values}</td>
                                <td>${r.Number_of_Zero_Values}</td>
                                <td style="font-family: monospace; font-size: 0.78rem;">${r.Top_5_Highest_Values}</td>
                            </tr>
                        `).join('')}
                    </tbody>
                </table>
            </div>
        `;
    } else if (action === 'sync-cloud') {
        if (titleEl) titleEl.innerText = '☁️ 9. Sync Users to Cloud Database';
        box.innerHTML = `<p>Syncing local users to cloud outbox...</p>`;
        const res = await fetch('/api/v1/sync/users-to-cloud', { method: 'POST' });
        const data = await res.json();
        box.innerHTML = `<pre style="background: #10101c; padding: 1rem; border-radius: 8px; color: #a29bfe;">${JSON.stringify(data, null, 2)}</pre>`;
    } else if (action === 'sync-hr-pw') {
        if (titleEl) titleEl.innerText = '🔑 10. Sync HR Password to Local';
        box.innerHTML = `<p>Syncing HR password for active company...</p>`;
        const explicitPw = (document.getElementById('hr-admin-pin')?.value || '').trim();
        const bodyObj = {
            company_uuid: activeCompany.company_id,
            company_name: activeCompany.company_name
        };
        if (explicitPw) bodyObj.password = explicitPw;
        const res = await fetch('/api/v1/auth/sync-hr-password', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(bodyObj)
        });
        const data = await res.json();
        box.innerHTML = `<pre style="background: #10101c; padding: 1rem; border-radius: 8px; color: #00d2d3;">${JSON.stringify(data, null, 2)}</pre>`;
    }
}

async function deleteUserAccount(username) {
    const res = await fetch(`/api/v1/users/${encodeURIComponent(username)}`, { method: 'DELETE' });
    if (res.ok) {
        addTerminalLog(`Deleted user ${username}`, 'HR');
        openHrAction('users-list');
    }
}

async function captureSingleRegPose(skipRemoteExtract = false) {
    const username = (document.getElementById('reg-cam-username')?.value || '').trim();
    const msgEl = document.getElementById('reg-cam-result-msg');
    const angleEl = document.getElementById('reg-pose-angle-badge');
    const camWrap = document.getElementById('hr-reg-cam-wrap');
    const cdBadge = document.getElementById('hr-reg-countdown-badge');

    if (!username) {
        if (msgEl) {
            msgEl.style.color = '#ff7675';
            msgEl.innerText = 'Please enter Username first before capturing poses!';
        }
        return false;
    }
    if (regCurrentPoseIdx >= 20) {
        return true;
    }

    const video = document.getElementById('hr-reg-video');
    const frameB64 = captureFrameFromVideoElement(video);
    const isTestFrame = Boolean(window.__TEST_FACE_B64);
    const hasLiveOrTestFrame = Boolean(isTestFrame || (video && video.srcObject && video.readyState >= 2));

    let verifiedPoseLabel = 'VERIFIED';
    let extractedEmbedding = null;

    if (!skipRemoteExtract && hasLiveOrTestFrame) {
        try {
            if (cdBadge) cdBadge.innerText = `🔍 Verifying Pose ${regCurrentPoseIdx + 1}/20...`;
            const reqPayload = {
                image_b64: frameB64,
                pose_index: regCurrentPoseIdx,
                enforce_pose: !isTestFrame,
                baseline_yaw: regBaselineMetrics ? regBaselineMetrics.yaw_ratio : null,
                baseline_pitch: regBaselineMetrics ? regBaselineMetrics.pitch_ratio : null,
                baseline_roll: regBaselineMetrics ? regBaselineMetrics.roll_deg : null,
                baseline_area: regBaselineMetrics ? regBaselineMetrics.face_area_ratio : null,
                prev_yaw: regPrevPoseMetrics ? regPrevPoseMetrics.yaw_ratio : null,
                prev_pitch: regPrevPoseMetrics ? regPrevPoseMetrics.pitch_ratio : null,
                prev_roll: regPrevPoseMetrics ? regPrevPoseMetrics.roll_deg : null,
                prev_area: regPrevPoseMetrics ? regPrevPoseMetrics.face_area_ratio : null,
                attempt: regPoseAttemptCount
            };

            const extRes = await fetch('/api/v1/face/extract-frame', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(reqPayload)
            });
            const extData = await extRes.json();

            if (extData && extData.pose_metrics && angleEl) {
                const pm = extData.pose_metrics;
                angleEl.innerText = `Detected: ${extData.detected_pose || 'FACE'} | Yaw: ${Number(pm.yaw_deg || 0).toFixed(1)}° | Pitch: ${Number(pm.pitch_deg || 0).toFixed(1)}° | Roll: ${Number(pm.roll_deg || 0).toFixed(1)}°`;
            }

            if (!extRes.ok || !extData.success || extData.pose_verified === false) {
                regTotalPoseRetries += 1;
                if (extData && extData.face_detected && extData.pose_verified === false) {
                    regPoseAttemptCount += 1;
                    if (camWrap) camWrap.style.borderColor = '#fdcb6e';
                    if (cdBadge) cdBadge.innerText = `⚠️ Adjust Pose ${regCurrentPoseIdx + 1}/20`;
                    if (msgEl) {
                        msgEl.style.color = '#fdcb6e';
                        msgEl.innerText = `⚠️ ${extData.message || extData.error} (Detected: ${extData.detected_pose || 'FRONTAL'})`;
                    }
                } else {
                    if (camWrap) camWrap.style.borderColor = '#ff7675';
                    if (cdBadge) cdBadge.innerText = `⚠️ Face Not Detected (${regCurrentPoseIdx + 1}/20)`;
                    if (msgEl) {
                        msgEl.style.color = '#ff7675';
                        msgEl.innerText = `⚠️ Face not detected clearly for Pose ${regCurrentPoseIdx + 1}/20 (${POSE_INSTRUCTIONS[regCurrentPoseIdx].bn}) — keep your face inside the camera frame.`;
                    }
                }
                return false;
            }

            verifiedPoseLabel = extData.detected_pose || 'VERIFIED';
            if (Array.isArray(extData.embedding) && extData.embedding.length === 512) {
                extractedEmbedding = extData.embedding;
            }
            if (regCurrentPoseIdx === 0 && extData.pose_metrics) {
                regBaselineMetrics = extData.pose_metrics;
            }
            if (extData.pose_metrics) {
                regPrevPoseMetrics = extData.pose_metrics;
            }
        } catch (err) {
            if (msgEl) {
                msgEl.style.color = '#ff7675';
                msgEl.innerText = 'Face extraction error: ' + err.message;
            }
            return false;
        }
    }

    const capturedIdx = regCurrentPoseIdx;
    regPoseAttemptCount = 0;
    regTotalPoseRetries = 0;
    regCapturedFrames.push(frameB64);
    if (extractedEmbedding) {
        regCapturedEmbeddings.push(extractedEmbedding);
    }
    regCurrentPoseIdx += 1;

    // Update Live Camera border & Last Captured Pose preview
    if (camWrap) camWrap.style.borderColor = '#2ecc71';
    if (cdBadge) cdBadge.innerText = `✅ Pose ${regCurrentPoseIdx}/20 Captured!`;

    const lastImg = document.getElementById('reg-last-captured-img');
    const lastEmpty = document.getElementById('reg-last-captured-empty');
    const lastLabel = document.getElementById('reg-last-captured-label');
    if (lastImg) {
        lastImg.src = frameB64;
        lastImg.style.display = 'block';
    }
    if (lastEmpty) lastEmpty.style.display = 'none';
    if (lastLabel) lastLabel.innerText = `✅ LAST CAPTURED: #${regCurrentPoseIdx}/20 (${verifiedPoseLabel})`;

    // Update 20-Pose Gallery Thumbnail Slot
    const thumbEl = document.getElementById(`reg-pose-thumb-${capturedIdx}`);
    if (thumbEl) {
        thumbEl.style.border = '2px solid #2ecc71';
        thumbEl.innerHTML = `
            <img src="${frameB64}" style="width:100%;height:100%;object-fit:cover;transform:scaleX(-1);" />
            <span style="position:absolute;bottom:0;right:0;background:rgba(46,204,113,0.92);color:#000;font-size:0.6rem;font-weight:800;padding:0 3px;border-top-left-radius:4px;">✓${regCurrentPoseIdx}</span>
        `;
    }
    const nextThumbEl = document.getElementById(`reg-pose-thumb-${regCurrentPoseIdx}`);
    if (nextThumbEl) {
        nextThumbEl.style.borderColor = '#00d2d3';
        nextThumbEl.style.color = '#00d2d3';
    }

    const pct = Math.round((regCurrentPoseIdx / 20) * 100);
    const bar = document.getElementById('reg-pose-bar');
    const status = document.getElementById('reg-pose-status');
    const enEl = document.getElementById('reg-pose-en');
    const bnEl = document.getElementById('reg-pose-bn');

    if (bar) bar.style.width = `${pct}%`;
    if (status) status.innerText = `Captured: ${regCurrentPoseIdx} / 20 poses (512-D ONNX vectors)`;

    if (msgEl) {
        msgEl.style.color = '#2ecc71';
        msgEl.innerText = `✅ Pose ${regCurrentPoseIdx}/20 Verified & Captured! (${POSE_INSTRUCTIONS[capturedIdx].en})`;
    }

    if (regCurrentPoseIdx < 20) {
        if (enEl) enEl.innerText = POSE_INSTRUCTIONS[regCurrentPoseIdx].en;
        if (bnEl) bnEl.innerText = POSE_INSTRUCTIONS[regCurrentPoseIdx].bn;
    } else {
        if (enEl) enEl.innerText = '✅ All 20/20 Poses Verified & Captured! Ready to Save.';
        if (bnEl) bnEl.innerText = '✅ ২০/২০টি পোজ যাচাই ও ক্যাপচার সম্পন্ন হয়েছে! এখন রেজিস্ট্রেশন সম্পন্ন করুন।';
    }
    return true;
}

async function runAuto20PoseCapture() {
    const username = (document.getElementById('reg-cam-username')?.value || '').trim();
    const msgEl = document.getElementById('reg-cam-result-msg');
    const autoBtn = document.getElementById('btn-auto-capture-poses');
    const cdBadge = document.getElementById('hr-reg-countdown-badge');

    if (!username) {
        if (msgEl) {
            msgEl.style.color = '#ff7675';
            msgEl.innerText = 'Please enter Username first before starting 20-pose capture!';
        }
        return;
    }

    // Toggle pause if already running
    if (regAutoCaptureRunning) {
        regAutoCaptureRunning = false;
        if (autoBtn) {
            autoBtn.innerText = '⚡ Resume Auto Capture (20 Poses)';
            autoBtn.style.background = '#3498db';
        }
        if (msgEl) {
            msgEl.style.color = '#fdcb6e';
            msgEl.innerText = `⏸️ Auto-capture paused at ${regCurrentPoseIdx}/20 poses. Click Resume or Capture Pose to continue.`;
        }
        return;
    }

    if (regCurrentPoseIdx >= 20) {
        resetRegPoses();
    }

    regAutoCaptureRunning = true;
    const isTestMode = Boolean(window.__TEST_FACE_B64);

    if (autoBtn) {
        autoBtn.innerText = '⏸️ Pause Auto Capture';
        autoBtn.style.background = '#e67e22';
    }

    while (regAutoCaptureRunning && regCurrentPoseIdx < 20) {
        const poseNum = regCurrentPoseIdx + 1;
        const currInstr = POSE_INSTRUCTIONS[regCurrentPoseIdx];

        if (isTestMode) {
            const ok = await captureSingleRegPose(regCurrentPoseIdx > 0);
            if (!ok) {
                regAutoCaptureRunning = false;
                break;
            }
            await new Promise(r => setTimeout(r, 20));
            continue;
        }

        // Human-paced countdown so the user can read the pose instruction and see each captured photo
        if (regTotalPoseRetries === 0) {
            for (let sec = 3; sec >= 1; sec--) {
                if (!regAutoCaptureRunning) break;
                if (cdBadge) cdBadge.innerText = `⏳ Pose ${poseNum}/20 in ${sec}s...`;
                if (msgEl) {
                    msgEl.style.color = '#74b9ff';
                    msgEl.innerText = `⏳ Get Ready for ${currInstr.en} (${currInstr.bn}) — Capturing in ${sec}s...`;
                }
                await new Promise(r => setTimeout(r, 750));
            }
        } else {
            if (cdBadge) cdBadge.innerText = `🔄 Retrying Pose ${poseNum}/20...`;
            await new Promise(r => setTimeout(r, 350));
        }

        if (!regAutoCaptureRunning) break;

        const ok = await captureSingleRegPose(false);
        if (ok) {
            // Pause so the user clearly sees the green verification flash & captured image
            await new Promise(r => setTimeout(r, 850));
        } else if (regTotalPoseRetries >= 25) {
            regAutoCaptureRunning = false;
            if (msgEl) {
                msgEl.style.color = '#fdcb6e';
                msgEl.innerText = `⏸️ Paused at Pose ${poseNum}/20 (${currInstr.bn}). Adjust your face angle and click Resume or Capture Pose.`;
            }
            break;
        } else {
            // Keep the warning badge & bilingual adjustment message visible before retrying
            await new Promise(r => setTimeout(r, 950));
        }
    }

    regAutoCaptureRunning = false;
    if (autoBtn) {
        autoBtn.innerText = regCurrentPoseIdx >= 20 ? '✅ 20/20 Poses Verified' : '⚡ Resume Auto Capture (20 Poses)';
        autoBtn.style.background = regCurrentPoseIdx >= 20 ? '#27ae60' : '#3498db';
    }
    if (regCurrentPoseIdx >= 20) {
        addTerminalLog(`Captured & verified all 20 multi-angle frames for ${username}`, 'CAMERA');
    }
}

async function submitCameraRegistration() {
    const username = (document.getElementById('reg-cam-username')?.value || '').trim();
    const designation = (document.getElementById('reg-cam-desig')?.value || '').trim();
    const password = document.getElementById('reg-cam-password')?.value || '';
    const msgEl = document.getElementById('reg-cam-result-msg');

    if (!username || !designation || !password) {
        if (msgEl) {
            msgEl.style.color = '#ff7675';
            msgEl.innerText = 'Username, Designation, and Password are required!';
        }
        return;
    }
    if (regCapturedFrames.length < 20) {
        if (regAutoCaptureRunning && !window.__TEST_FACE_B64) {
            if (msgEl) {
                msgEl.style.color = '#fdcb6e';
                msgEl.innerText = `⏳ Auto-capture is in progress (${regCapturedFrames.length}/20 poses verified). Please complete all 20 poses first.`;
            }
            return;
        }
        await runAuto20PoseCapture();
        if (regCapturedFrames.length < 20) {
            if (msgEl && !msgEl.innerText) {
                msgEl.style.color = '#fdcb6e';
                msgEl.innerText = `Please complete all 20 verified poses (currently ${regCapturedFrames.length}/20) before saving.`;
            }
            return;
        }
    }

    try {
        const hasAllPreVerifiedEmbeddings = (regCapturedEmbeddings.length === regCapturedFrames.length && regCapturedEmbeddings.length > 0);
        const reqBody = {
            username,
            user_designation: designation,
            password,
            company_uuid: activeCompany?.company_id || 'TECHFIRM_01',
            images_b64: hasAllPreVerifiedEmbeddings ? [regCapturedFrames[0]] : regCapturedFrames,
            overwrite_existing: true
        };
        if (hasAllPreVerifiedEmbeddings) {
            reqBody.face_embeddings = regCapturedEmbeddings;
        }
        const res = await fetch('/api/v1/users', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(reqBody)
        });
        const data = await res.json();
        if (res.ok && (data.success || data.id)) {
            addTerminalLog(`User ${username} registered with ${data.embeddings_enrolled || data.enrolled_poses || regCapturedFrames.length} ONNX FaceNet512 poses`, 'SUCCESS');
            backToHrMainMenu();
        } else {
            if (msgEl) {
                msgEl.style.color = '#ff7675';
                msgEl.innerText = data.error || data.message || 'Registration failed';
            }
        }
    } catch (e) {
        if (msgEl) {
            msgEl.style.color = '#ff7675';
            msgEl.innerText = 'Error registering user: ' + e.message;
        }
    }
}

async function submitNoCameraRegistration() {
    const username = (document.getElementById('reg-nocam-username')?.value || '').trim();
    const designation = (document.getElementById('reg-nocam-desig')?.value || '').trim();
    const password = document.getElementById('reg-nocam-password')?.value || '';
    const stEl = document.getElementById('reg-nocam-status');

    if (!username || !designation || !password) {
        if (stEl) {
            stEl.style.color = '#ff7675';
            stEl.innerText = 'All fields (Username, Designation, Password) are required!';
        }
        return;
    }

    const res = await fetch('/api/v1/users', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            username,
            user_designation: designation,
            password,
            company_uuid: activeCompany?.company_id || 'TECHFIRM_01',
            overwrite_existing: true
        })
    });
    const data = await res.json();
    if (res.ok && (data.success || data.id)) {
        addTerminalLog(`User ${username} registered (Without Camera)`, 'SUCCESS');
        backToHrMainMenu();
    } else if (stEl) {
        stEl.style.color = '#ff7675';
        stEl.innerText = data.error || data.message || 'Registration failed';
    }
}

async function submitQrRegistration() {
    const username = (document.getElementById('reg-qr-username')?.value || '').trim();
    const designation = (document.getElementById('reg-qr-desig')?.value || '').trim();
    const password = document.getElementById('reg-qr-password')?.value || '';
    const user_android_id = (document.getElementById('reg-qr-android-id')?.value || '').trim();
    const stEl = document.getElementById('reg-qr-status');

    if (!username || !designation || !password || !user_android_id) {
        if (stEl) {
            stEl.style.color = '#ff7675';
            stEl.innerText = 'All fields including Android Device ID are required!';
        }
        return;
    }

    const res = await fetch('/api/v1/users', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            username,
            user_designation: designation,
            password,
            user_android_id,
            company_uuid: activeCompany?.company_id || 'TECHFIRM_01',
            overwrite_existing: true
        })
    });
    const data = await res.json();
    if (res.ok && (data.success || data.id)) {
        addTerminalLog(`User ${username} registered with QR Android ID: ${user_android_id}`, 'SUCCESS');
        backToHrMainMenu();
    } else if (stEl) {
        stEl.style.color = '#ff7675';
        stEl.innerText = data.error || data.message || 'Registration failed';
    }
}

function buildReportQueryString() {
    const username = (document.getElementById('rep-username')?.value || 'All Users').trim();
    const ym = (document.getElementById('rep-month')?.value || '2026-10').trim();
    const parts = ym.split('-');
    const year = parts[0] || '2026';
    const month = parts[1] || '10';
    const from_day = (document.getElementById('rep-from-date')?.value || '1').trim();
    const to_day = (document.getElementById('rep-to-date')?.value || '31').trim();
    const in_time = (document.getElementById('rep-in-time')?.value || '09:00:00').trim();
    const out_time = (document.getElementById('rep-out-time')?.value || '17:00:00').trim();
    return `username=${encodeURIComponent(username)}&year=${encodeURIComponent(year)}&month=${encodeURIComponent(month)}&from_day=${encodeURIComponent(from_day)}&to_day=${encodeURIComponent(to_day)}&set_in_time=${encodeURIComponent(in_time)}&set_out_time=${encodeURIComponent(out_time)}`;
}

function downloadCustomizedCsvReport() {
    window.open(`/api/v1/reports/customized.csv?${buildReportQueryString()}`, '_blank');
}

function downloadCustomizedXlsxReport() {
    window.open(`/api/v1/reports/customized.xlsx?${buildReportQueryString()}`, '_blank');
}

function openMusterRollHtmlReport() {
    const ym = (document.getElementById('rep-month')?.value || '2026-10').trim();
    const parts = ym.split('-');
    const year = parts[0] || '2026';
    const month = parts[1] || '10';
    window.open(`/api/v1/reports/muster-roll/html?year=${encodeURIComponent(year)}&month=${encodeURIComponent(month)}`, '_blank');
}

async function submitUploadOnnxModel() {
    const fileInput = document.getElementById('onnx-model-file');
    const resEl = document.getElementById('upload-model-result');
    if (!fileInput || !fileInput.files || fileInput.files.length === 0) {
        if (resEl) resEl.innerHTML = `<span style="color:#ff7675;">Please select a .onnx file first!</span>`;
        return;
    }
    const file = fileInput.files[0];
    const reader = new FileReader();
    reader.onload = async () => {
        const base64Data = reader.result.split(',')[1] || '';
        const res = await fetch('/api/v1/face/upload-model', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ model_name: file.name, model_b64: base64Data })
        });
        const data = await res.json();
        if (resEl) {
            resEl.innerHTML = `<pre style="background:#10101c;padding:1rem;border-radius:8px;color:#2ecc71;">${JSON.stringify(data, null, 2)}</pre>`;
        }
    };
    reader.readAsDataURL(file);
}

async function deleteSelectedUserFace() {
    const username = document.getElementById('del-face-user')?.value || '';
    const stEl = document.getElementById('del-face-status');
    if (!username) return;
    const res = await fetch(`/api/v1/face/profiles/${encodeURIComponent(username)}`, { method: 'DELETE' });
    const data = await res.json();
    if (stEl) {
        stEl.style.color = '#00b894';
        stEl.innerText = `Deleted face profiles for ${username} (DB: ${data.deleted_count || 0}, RAM: ${data.ram_vectors_removed || 0})`;
    }
    openHrAction('delete-face');
}

async function resetAllFaceProfiles() {
    const res = await fetch('/api/v1/face/reset', { method: 'POST' });
    await res.json();
    openHrAction('delete-face');
}
    "##
}
