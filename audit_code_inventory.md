# 📋 Local Keyboard Source Code Inventory Audit

- **Date & Time:** 2026-10-11T17:42:00Z
- **Repository Path:** `/home/opc/keyboard`
- **Total Local Source Code Lines:** 1,323 lines (11 Dart files + 2 Kotlin files + 2 Test files)
- **Status:** Fully Present and Active Locally on Oracle VM

---

## 1. Local Code Hierarchy
- `lib/main.dart` - Entry point
- `lib/app.dart` - Testing harness & settings screen
- `lib/models/gesture_direction.dart` - 4-corner swipe vector math
- `lib/models/key_model.dart` - Key data structures
- `lib/models/keyboard_layout.dart` - QWERTY & terminal keys
- `lib/services/ime_channel_service.dart` - Native IME platform channel
- `lib/themes/keyboard_theme.dart` - AMOLED, Hacker Green, Cyberpunk
- `lib/views/key_widget.dart` - Gesture key widget
- `lib/views/modifier_bar.dart` - ESC, TAB, CTRL, ALT, Arrow keys
- `lib/views/keyboard_view.dart` - Keyboard view layout
- `lib/views/settings_screen.dart` - Settings UI
- `android/.../FlutterInputMethodService.kt` - Android InputMethodService
- `android/.../MainActivity.kt` - App launcher
- `dist/apks/unexpected-keyboard-release.apk` - 49MB compiled release APK
