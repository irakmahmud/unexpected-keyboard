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
4. **Cloud-Native CI/CD (No Local Build Required)**:
   - Automated GitHub Actions workflow (`.github/workflows/build_apk.yml`) compiles debug and release APKs on GitHub runners with Java 17 and Flutter SDK.
   - APK artifacts are immediately available for download from the Actions tab.
5. **Multiple High-Contrast Themes**:
   - AMOLED Pure Dark (OLED battery saving)
   - Terminal Hacker Green (Matrix style)
   - Cyberpunk Neon
   - Material Dark

---

## 🚀 GitHub Actions Cloud Build Instructions

This project is configured to **build exclusively on GitHub Actions cloud runners**. You do not need to install Android SDK, NDK, or Flutter locally.

### Steps to Build and Download APK:
1. Push this repository to GitHub:
   ```bash
   git remote add origin https://github.com/<your-username>/unexpected-flutter-keyboard.git
   git branch -M main
   git push -u origin main
   ```
2. Navigate to your GitHub repository in your browser.
3. Click on the **Actions** tab.
4. The workflow **Build Unexpected Flutter Keyboard APK** will automatically trigger.
5. Once complete, click on the workflow run. Under **Artifacts**, download:
   - `unexpected-keyboard-release-apk` (Optimized release build)
   - `unexpected-keyboard-debug-apk` (Debug build for testing)

---

## 📁 Project Architecture

```
keyboard/
├── .github/workflows/
│   └── build_apk.yml              # GitHub Actions CI/CD APK build pipeline
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
