import 'package:flutter/services.dart';

class ImeChannelService {
  static const MethodChannel _channel = MethodChannel('com.antigravity.keyboard/ime');

  static const double minKeyHeight = 34.0;
  static const double maxKeyHeight = 136.0;
  static const double defaultKeyHeight = 42.0;

  static const int minLongPressMs = 0;
  static const int maxLongPressMs = 3000;
  static const int defaultLongPressMs = 400;

  bool isShiftActive = false;
  bool isCapsLock = false;
  bool isCtrlActive = false;
  bool isAltActive = false;
  bool isMetaActive = false;
  bool hapticsEnabled = true;
  int longPressMs = defaultLongPressMs;

  /// Callbacks for in-app preview typing & dynamic config updates
  void Function(String text)? onLocalTextInput;
  void Function(String text)? onLocalReplaceLastText;
  void Function()? onLocalBackspace;
  void Function()? onLocalEnter;
  void Function(double keyHeight)? onHeightUpdatedFromNative;
  void Function(String themeName)? onThemeUpdatedFromNative;
  void Function(bool enabled)? onHapticsUpdatedFromNative;
  void Function(int longPressMs)? onLongPressUpdatedFromNative;
  void Function()? onOpenSettingsFromNative;
  void Function(bool isEnabled, bool isSelected)? onKeyboardStatusUpdatedFromNative;

  ImeChannelService() {
    _channel.setMethodCallHandler(_handleNativeCall);
  }

  Future<dynamic> _handleNativeCall(MethodCall call) async {
    if (call.method == 'onHeightChanged') {
      final raw = call.arguments;
      if (raw is num) {
        onHeightUpdatedFromNative?.call(clampKeyHeight(raw.toDouble()));
      }
    } else if (call.method == 'onKeyboardStatusChanged') {
      final args = call.arguments;
      if (args is Map) {
        final isEnabled = args['isEnabled'] == true;
        final isSelected = args['isSelected'] == true;
        onKeyboardStatusUpdatedFromNative?.call(isEnabled, isSelected);
      }
    } else if (call.method == 'onConfigChanged') {
      final args = call.arguments;
      if (args is Map) {
        final rawHeight = args['keyHeight'];
        if (rawHeight is num) {
          onHeightUpdatedFromNative?.call(clampKeyHeight(rawHeight.toDouble()));
        }
        final rawTheme = args['themeMode'];
        if (rawTheme is String) {
          onThemeUpdatedFromNative?.call(rawTheme);
        }
        final rawHaptics = args['hapticsEnabled'];
        if (rawHaptics is bool) {
          hapticsEnabled = rawHaptics;
          onHapticsUpdatedFromNative?.call(rawHaptics);
        }
        final rawLongPress = args['longPressMs'];
        if (rawLongPress is num) {
          longPressMs = clampLongPressMs(rawLongPress.round());
          onLongPressUpdatedFromNative?.call(longPressMs);
        }
        if (args['openSettings'] == true) {
          onOpenSettingsFromNative?.call();
        }
      }
    } else if (call.method == 'onOpenSettings') {
      onOpenSettingsFromNative?.call();
    }
    return true;
  }

  static double clampKeyHeight(double height) {
    if (height < minKeyHeight) return minKeyHeight;
    if (height > maxKeyHeight) return maxKeyHeight;
    return height;
  }

  static int clampLongPressMs(int ms) {
    if (ms < minLongPressMs) return minLongPressMs;
    if (ms > maxLongPressMs) return maxLongPressMs;
    return ms;
  }

  static int calculateTotalKeyboardHeightDp(
    double keyHeight, {
    bool showTerminalBar = true,
    bool showResizeBar = false,
  }) {
    final clamped = clampKeyHeight(keyHeight);
    // 7 rows (Row 0 Action/Symbol Row + Row 1 Number Row + 5 Letter Rows) with zero vertical gaps
    final rowsHeight = 7.0 * clamped;
    final modifierBarHeight = showTerminalBar ? (clamped * 0.85).clamp(36.0, 68.0) : 0.0;
    final resizeBarHeight = showResizeBar ? 72.0 : 0.0;
    return (rowsHeight + modifierBarHeight + resizeBarHeight + 4.0).round();
  }

  Future<double> getKeyboardHeight() async {
    try {
      final result = await _channel.invokeMethod<dynamic>('getKeyboardHeight');
      if (result is num) {
        return clampKeyHeight(result.toDouble());
      }
    } on PlatformException catch (e) {
      print('IME Channel getKeyboardHeight error: ${e.message}');
    } catch (_) {}
    return defaultKeyHeight;
  }

  Future<void> setKeyboardHeight(
    double keyHeight, {
    bool showTerminalBar = true,
    bool showResizeBar = false,
  }) async {
    final clamped = clampKeyHeight(keyHeight);
    final totalHeightDp = calculateTotalKeyboardHeightDp(
      clamped,
      showTerminalBar: showTerminalBar,
      showResizeBar: showResizeBar,
    );
    try {
      await _channel.invokeMethod('setKeyboardHeight', {
        'keyHeight': clamped,
        'totalHeightDp': totalHeightDp,
      });
    } on PlatformException catch (e) {
      print('IME Channel setKeyboardHeight error: ${e.message}');
    } catch (_) {}
  }

  Future<Map<String, dynamic>> getKeyboardConfig() async {
    try {
      final result = await _channel.invokeMethod<dynamic>('getKeyboardConfig');
      if (result is Map) {
        final map = Map<String, dynamic>.from(result);
        if (map['hapticsEnabled'] is bool) {
          hapticsEnabled = map['hapticsEnabled'] as bool;
        }
        if (map['longPressMs'] is num) {
          longPressMs = clampLongPressMs((map['longPressMs'] as num).round());
        }
        return map;
      }
    } on PlatformException catch (e) {
      print('IME Channel getKeyboardConfig error: ${e.message}');
    } catch (_) {}
    return {
      'keyHeight': defaultKeyHeight,
      'themeMode': 'amoled',
      'hapticsEnabled': true,
      'longPressMs': defaultLongPressMs,
      'openSettings': false,
    };
  }

  Future<void> setKeyboardConfig({
    String? themeMode,
    bool? haptics,
    int? longPressMs,
  }) async {
    if (haptics != null) {
      hapticsEnabled = haptics;
    }
    if (longPressMs != null) {
      this.longPressMs = clampLongPressMs(longPressMs);
    }
    try {
      await _channel.invokeMethod('setKeyboardConfig', {
        if (themeMode != null) 'themeMode': themeMode,
        if (haptics != null) 'hapticsEnabled': haptics,
        if (longPressMs != null) 'longPressMs': clampLongPressMs(longPressMs),
      });
    } on PlatformException catch (e) {
      print('IME Channel setKeyboardConfig error: ${e.message}');
    } catch (_) {}
  }

  Future<void> openSettings() async {
    try {
      await _channel.invokeMethod('openSettings');
    } on PlatformException catch (e) {
      print('IME Channel openSettings error: ${e.message}');
    } catch (_) {}
  }

  Future<bool> closeSettingsIfExternal() async {
    try {
      final res = await _channel.invokeMethod<dynamic>('closeSettingsIfExternal');
      return res == true;
    } on PlatformException catch (e) {
      print('IME Channel closeSettingsIfExternal error: ${e.message}');
    } catch (_) {}
    return false;
  }

  Future<Map<String, bool>> getKeyboardStatus() async {
    try {
      final res = await _channel.invokeMethod<dynamic>('getKeyboardStatus');
      if (res is Map) {
        return {
          'isEnabled': res['isEnabled'] == true,
          'isSelected': res['isSelected'] == true,
        };
      }
    } on PlatformException catch (e) {
      print('IME Channel getKeyboardStatus error: ${e.message}');
    } catch (_) {}
    return {'isEnabled': false, 'isSelected': false};
  }

  Future<void> enableKeyboardInSystemSettings() async {
    try {
      await _channel.invokeMethod('enableKeyboardInSystemSettings');
    } on PlatformException catch (e) {
      print('IME Channel enableKeyboardInSystemSettings error: ${e.message}');
    } catch (_) {}
  }

  Future<void> showInputMethodPicker() async {
    try {
      await _channel.invokeMethod('showInputMethodPicker');
    } on PlatformException catch (e) {
      print('IME Channel showInputMethodPicker error: ${e.message}');
    } catch (_) {}
  }

  int calculateMetaState() {
    int state = 0;
    if (isShiftActive || isCapsLock) state |= 1; // KeyEvent.META_SHIFT_ON
    if (isAltActive) state |= 2; // KeyEvent.META_ALT_ON
    if (isCtrlActive) state |= 4096; // KeyEvent.META_CTRL_ON
    if (isMetaActive) state |= 65536; // KeyEvent.META_META_ON
    return state;
  }

  Future<void> sendText(String text) async {
    // Notify local listener immediately (0ms latency for in-app preview)
    onLocalTextInput?.call(text);

    final bool useModifiers = isCtrlActive || isAltActive;
    final int metaState = useModifiers ? calculateMetaState() : 0;

    // Reset single-use modifiers synchronously BEFORE platform channel invoke
    if (isShiftActive && !isCapsLock) {
      isShiftActive = false;
    }
    isCtrlActive = false;
    isAltActive = false;
    isMetaActive = false;

    try {
      if (useModifiers) {
        await _channel.invokeMethod('sendTextWithModifiers', {
          'text': text,
          'metaState': metaState,
        });
      } else {
        await _channel.invokeMethod('commitText', {'text': text});
      }
    } on PlatformException catch (e) {
      print('IME Channel sendText error: ${e.message}');
    }
  }

  Future<void> replaceLastText(String text) async {
    onLocalReplaceLastText?.call(text);
    try {
      await _channel.invokeMethod('replaceLastText', {'text': text});
    } on PlatformException catch (e) {
      print('IME Channel replaceLastText error: ${e.message}');
    }
  }

  Future<void> sendKeyCode(int keyCode) async {
    if (keyCode == 67) {
      onLocalBackspace?.call();
    } else if (keyCode == 66) {
      onLocalEnter?.call();
    }

    final int metaState = calculateMetaState();
    if (isShiftActive && !isCapsLock) isShiftActive = false;
    isCtrlActive = false;
    isAltActive = false;
    isMetaActive = false;

    try {
      await _channel.invokeMethod('sendKeyEvent', {
        'keyCode': keyCode,
        'metaState': metaState,
      });
    } on PlatformException catch (e) {
      print('IME Channel sendKeyCode error: ${e.message}');
    }
  }

  Future<void> sendBackspace() async {
    onLocalBackspace?.call();

    try {
      await _channel.invokeMethod('deleteSurroundingText', {
        'beforeLength': 1,
        'afterLength': 0,
      });
    } on PlatformException catch (e) {
      print('IME Channel sendBackspace error: ${e.message}');
    }
  }

  Future<void> sendEnter() async {
    onLocalEnter?.call();

    try {
      await _channel.invokeMethod('sendEnterAction');
    } on PlatformException catch (e) {
      print('IME Channel sendEnter error: ${e.message}');
    }
  }

  Future<void> hideKeyboard() async {
    try {
      await _channel.invokeMethod('hideKeyboard');
    } on PlatformException catch (e) {
      print('IME Channel hideKeyboard error: ${e.message}');
    }
  }

  void triggerHaptic() {
    if (hapticsEnabled) {
      HapticFeedback.lightImpact();
    }
  }
}
