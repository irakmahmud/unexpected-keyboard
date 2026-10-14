# 📋 Technical Audit: CircleCI APK Collection Mechanism & Iteration Lifecycle

- **Date & Time:** 2026-10-12T05:11:45Z
- **User Query:** "Circle ci theke tumi kivabe apk collect korla? Aar, eti next edit e kivabe kaaj korbe?"
- **Workspace:** `/home/opc/keyboard`

---

## 1. How the APK Was Collected from CircleCI
1. **Remote Cloud Execution:** CircleCI runner executed the remote build step inside container image `ghcr.io/cirruslabs/flutter:stable`.
2. **Artifact Publishing:** CircleCI stored artifacts in `keyboard-apks/` and generated SHA-256 integrity hash (`7a5b44b01d637975de9b2b4030e39721c4acd358b707cdb4838cd55d369efb7f`).
3. **Safety Gate Verification:** The local script `scripts/fetch_remote_apk.sh` validated that Oracle VM had > 5 GB free disk space (actual: 10 GB).
4. **Secure Download & Integrity Probe:** Downloaded via direct artifact URL with `curl -fsSL`, immediately verified SHA-256 hash match, and persisted to `dist/apks/unexpected-keyboard-release.apk`.

---

## 2. How Next Edits Will Work (Iteration Lifecycle)
1. **Local Authoring:** Edit Dart/Kotlin code in `/home/opc/keyboard/lib/` or `/home/opc/keyboard/android/`.
2. **Dispatch to Cloud:** Trigger git push via `./scripts/trigger_remote_build.sh` (or `git push origin main`).
3. **Remote Execution:** CircleCI runner checks out code, runs automated test suite, and compiles new APK/AAB in the cloud (0% VM CPU/RAM load).
4. **Fetch Updated APK:** Run `./scripts/fetch_remote_apk.sh fetch <NEW_URL> <NEW_SHA256>` to update local artifact with new build.
