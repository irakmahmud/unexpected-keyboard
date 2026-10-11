import 'package:flutter/services.dart';

class ImeChannelService {
  static const MethodChannel _channel = MethodChannel('com.antigravity.keyboard/ime');

  bool isShiftActive = false;
  bool isCapsLock = false;
  bool isCtrlActive = false;
  bool isAltActive = false;
  bool isMetaActive = false;

  int calculateMetaState() {
    int state = 0;
    if (isShiftActive || isCapsLock) state |= 1; // KeyEvent.META_SHIFT_ON
    if (isAltActive) state |= 2; // KeyEvent.META_ALT_ON
    if (isCtrlActive) state |= 4096; // KeyEvent.META_CTRL_ON
    if (isMetaActive) state |= 65536; // KeyEvent.META_META_ON
    return state;
  }

  Future<void> sendText(String text) async {
    try {
      if (isCtrlActive || isAltActive) {
        // If modifier is held, send as key event with meta state
        await _channel.invokeMethod('sendTextWithModifiers', {
          'text': text,
          'metaState': calculateMetaState(),
        });
      } else {
        await _channel.invokeMethod('commitText', {'text': text});
      }

      // Reset single-use shift if not caps locked
      if (isShiftActive && !isCapsLock) {
        isShiftActive = false;
      }
      // Reset single-use ctrl / alt / meta after key press
      isCtrlActive = false;
      isAltActive = false;
      isMetaActive = false;
    } on PlatformException catch (e) {
      print('IME Channel sendText error: ${e.message}');
    }
  }

  Future<void> sendKeyCode(int keyCode) async {
    try {
      await _channel.invokeMethod('sendKeyEvent', {
        'keyCode': keyCode,
        'metaState': calculateMetaState(),
      });
      if (isShiftActive && !isCapsLock) isShiftActive = false;
      isCtrlActive = false;
      isAltActive = false;
      isMetaActive = false;
    } on PlatformException catch (e) {
      print('IME Channel sendKeyCode error: ${e.message}');
    }
  }

  Future<void> sendBackspace() async {
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

  Future<void> triggerHaptic() async {
    try {
      HapticFeedback.lightImpact();
    } catch (_) {}
  }
}
