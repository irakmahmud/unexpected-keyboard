# 📋 Audit Report: Mandatory Mobile Download Link Delivery Protocol

- **Date & Time:** 2026-10-12T06:14:30Z
- **User Directive:** "Always give me apk downloadable link from mobile after building finish in circleci"
- **Mandatory Policy:** Codified in AGENTS.md as Rule 16
- **Status:** ACTIVE & ENFORCED

---

## 1. Verified Live Mobile Download Endpoint
- **Public Tunnel URL:**  
  `https://apart-simulations-portion-melbourne.trycloudflare.com/static/unexpected-keyboard.apk`
- **Underlying File Target:**  
  `/home/opc/keyboard/dist/apks/unexpected-keyboard-release.apk`
- **HTTP Header Verification:**
  - HTTP/2 200 OK
  - Content-Type: `application/vnd.android.package-archive`
  - Content-Length: 51,282,311 bytes (~49 MB)
  - Content-Disposition: Direct APK stream

---

## 2. Operational Standard (Rule 16)
Whenever a build finishes on CircleCI:
1. Fetch latest release APK to `/home/opc/keyboard/dist/apks/unexpected-keyboard-release.apk`.
2. Symlink at `/home/opc/Cloud_VM_Live_Preview/static/unexpected-keyboard.apk` automatically updates.
3. Every agent response must explicitly feature the direct, clickable, mobile-accessible link.
