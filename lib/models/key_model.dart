import 'gesture_direction.dart';

enum KeyActionType {
  inputChar,
  backspace,
  enter,
  space,
  tab,
  escape,
  shift,
  ctrl,
  alt,
  meta,
  arrowLeft,
  arrowRight,
  arrowUp,
  arrowDown,
  switchLayout,
  openSettings,
}

class KeyModel {
  final String primary;
  final String? topLeft;
  final String? topRight;
  final String? bottomLeft;
  final String? bottomRight;
  final String? swipeUp;
  final String? swipeDown;
  final KeyActionType actionType;
  final double flex;
  final int? androidKeyCode;

  const KeyModel({
    required this.primary,
    this.topLeft,
    this.topRight,
    this.bottomLeft,
    this.bottomRight,
    this.swipeUp,
    this.swipeDown,
    this.actionType = KeyActionType.inputChar,
    this.flex = 1.0,
    this.androidKeyCode,
  });

  /// Resolves the character or string to send given a gesture direction
  String resolveOutput(GestureDirection direction, {bool isShifted = false}) {
    if (actionType != KeyActionType.inputChar) {
      return '';
    }

    String? candidate;
    switch (direction) {
      case GestureDirection.topLeft:
        candidate = topLeft ?? swipeUp;
        break;
      case GestureDirection.topRight:
        candidate = topRight ?? swipeUp;
        break;
      case GestureDirection.bottomLeft:
        candidate = bottomLeft ?? swipeDown;
        break;
      case GestureDirection.bottomRight:
        candidate = bottomRight ?? swipeDown;
        break;
      case GestureDirection.up:
        candidate = swipeUp ?? topLeft ?? topRight;
        break;
      case GestureDirection.down:
        candidate = swipeDown ?? bottomLeft ?? bottomRight;
        break;
      case GestureDirection.center:
      case GestureDirection.left:
      case GestureDirection.right:
      case GestureDirection.longPress:
        candidate = primary;
        break;
    }

    final String result = candidate ?? primary;
    if (isShifted && result.length == 1) {
      return result.toUpperCase();
    }
    return result;
  }
}
