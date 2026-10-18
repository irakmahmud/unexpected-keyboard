# Unexpected Flutter Keyboard ⌨️

A private, gesture-driven Android keyboard (IME) built with Flutter and Kotlin, directly inspired by **Unexpected Keyboard**. Designed for coders, power users, and privacy enthusiasts who need fast terminal keys and symbols without switching layers.

---

## 🌟 Key Features

1. **4-Corner Gesture Typing (Unexpected Keyboard Style)**:
   - Each key provides 5 characters:
     - Center tap: Primary lowercase character
     - Swipe Top-Left: Number / primary symbol
     - Swipe Top-Right: Punctuation / alternative symbol
     - Swipe Bottom-Left: Bracket / math symbol
     - Swipe Bottom-Right: Special symbol
2. **Terminal Modifier Keys (Termux / SSH Friendly)**:
   - Dedicated modifier row with **ESC**, **TAB**, **CTRL**, **ALT**, **META**, and cursor navigation arrows (**◀**, **▼**, **▲**, **▶**).
   - Allows seamless terminal shortcuts like `Ctrl+C`, `Ctrl+Z`, `Ctrl+D`, `Alt+F`, etc.
3. **100% Offline & Zero Network Permissions**:
   - `android.permission.INTERNET` is completely absent from the Android Manifest.
   - Zero telemetry, zero analytics, zero network calls. Total privacy guaranteed.
4. **Cloud-Native CI/CD (Zero Local Flutter Build Architecture)**:
   - **Oracle VM as Authoring Node**: The Oracle VM acts strictly as a lightweight source code editing and git orchestration node without running any Flutter or Android Gradle builds locally.
   - **CircleCI Remote Cloud Fleet**: Automated CircleCI workflow (`.circleci/config.yml`) runs tests and builds debug and release APKs as well as Google Play App Bundles (AAB) on high-speed cloud Docker runners (`ghcr.io/cirruslabs/flutter:stable`).
   - **GitHub Actions Cloud Runner**: Dual CI/CD pipeline (`.github/workflows/build_apk.yml`) building APKs and AAB bundles on GitHub Ubuntu runners.
5. **Multiple High-Contrast Themes**:
   - AMOLED Pure Dark (OLED battery saving)
   - Terminal Hacker Green (Matrix style)
   - Cyberpunk Neon
   - Material Dark
6. **Live Adjustable Keyboard Height & Key Size**:
   - Tap or drag the `↕` resize button on the top modifier bar anytime while typing to adjust height live (`34 dp` – `68 dp` per row).
   - Also configurable in Settings with a precision slider and 4 quick presets (`Compact`, `Normal`, `Tall`, `Extra Tall`), persisted natively via Android `SharedPreferences`.

---

## 🚀 Zero Local Flutter Build: Oracle VM + CircleCI Remote Build Architecture

This project strictly adheres to **Zero Local Flutter Build**. Local builds on the Oracle VM are disallowed to preserve system memory and CPU for live agent services. All APK and AAB compilations occur remotely in the cloud.

### Remote Build Engines:
1. **CircleCI (`.circleci/config.yml`)**:
   - Runs on Docker container `ghcr.io/cirruslabs/flutter:stable`
   - Automatic dependency caching (`pub-cache` & `gradle-cache`)
   - Two-stage workflow: `validate-and-test` -> `build-remote-artifacts`
   - Artifacts uploaded directly to CircleCI Artifacts:
     - `unexpected-keyboard-debug.apk` (Debug APK for testing)
     - `unexpected-keyboard-release.apk` (Optimized release APK)
     - `unexpected-keyboard-release.aab` (Google Play App Bundle)
2. **GitHub Actions (`.github/workflows/build_apk.yml`)**:
   - Dual fallback pipeline on GitHub runners building both APK and AAB artifacts

### Steps to Trigger Remote Cloud Build:
1. Ensure changes are committed:
   ```bash
   git add .
   git commit -m "feat: your changes"
   ```
2. Push to GitHub:
   ```bash
   git push origin main
   ```
3. CircleCI and GitHub Actions will automatically detect the push and compile the APKs and AAB remotely.
4. Download generated APKs and AAB directly from CircleCI Artifacts or GitHub Actions Artifacts tab.

---

## 📁 Project Architecture

```
keyboard/
├── .circleci/
│   └── config.yml                 # CircleCI remote APK build pipeline
├── .github/workflows/
│   └── build_apk.yml              # GitHub Actions CI/CD APK build pipeline
├── scripts/
│   ├── guard_zero_local_build.sh  # Enforces zero local build policy on VM
│   └── trigger_remote_build.sh    # Remote cloud build trigger helper
├── android/
│   ├── app/
│   │   ├── src/main/
│   │   │   ├── AndroidManifest.xml # IME service declaration (NO internet permission)
│   │   │   ├── kotlin/com/antigravity/keyboard/
│   │   │   │   ├── MainActivity.kt               # App & settings launcher
│   │   │   │   └── FlutterInputMethodService.kt   # Android InputMethodService
│   │   │   └── res/xml/method.xml                # IME subtype configuration
│   │   └── build.gradle
│   └── build.gradle
├── lib/
│   ├── main.dart                  # Application entry point
│   ├── app.dart                   # Testing harness & settings navigation
│   ├── models/
│   │   ├── gesture_direction.dart # 4-corner swipe vector math & angle decoder
│   │   ├── key_model.dart         # Key layout model with corner symbols
│   │   └── keyboard_layout.dart   # QWERTY, Symbols & Terminal bar configurations
│   ├── services/
│   │   └── ime_channel_service.dart # Platform channel bridge to Android IME
│   ├── themes/
│   │   └── keyboard_theme.dart    # AMOLED, Terminal Green, Cyberpunk themes
│   └── views/
│       ├── key_widget.dart        # Gesture-aware key button with corner badges
│       ├── modifier_bar.dart      # ESC/TAB/CTRL/ALT/Arrow keys row
│       ├── keyboard_view.dart     # Responsive layout builder
│       └── settings_screen.dart   # Theme & behavior settings
├── test/
│   ├── gesture_decoder_test.dart  # Unit tests for corner gesture math
│   └── key_layout_test.dart       # Unit tests for key mappings & symbols
├── pubspec.yaml                   # Flutter dependency specifications
└── README.md
```
<!-- auto-sync live verified -->
