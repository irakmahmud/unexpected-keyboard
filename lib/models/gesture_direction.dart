import 'dart:math' as math;
import 'package:flutter/material.dart';

enum GestureDirection {
  center,
  topLeft,
  topRight,
  bottomLeft,
  bottomRight,
  up,
  down,
  left,
  right,
  longPress;

  /// Decodes a drag vector delta into a directional swipe.
  /// Inspired by Unexpected Keyboard's 4-corner gesture detection.
  static GestureDirection fromDelta(Offset delta, {double threshold = 18.0}) {
    final double dx = delta.dx;
    final double dy = delta.dy;
    final double distance = math.sqrt(dx * dx + dy * dy);

    if (distance < threshold) {
      return GestureDirection.center;
    }

    // Angle in degrees from -180 to 180 (0 is positive X / right, 90 is positive Y / down)
    final double angle = math.atan2(dy, dx) * 180 / math.pi;

    // Corner zones: 45 deg diagonals with ~45 degree tolerance
    // Top-Left: -180 to -90 (diagonal is -135) -> -160 to -110
    // Top-Right: -90 to 0 (diagonal is -45) -> -70 to -20
    // Bottom-Right: 0 to 90 (diagonal is 45) -> 20 to 70
    // Bottom-Left: 90 to 180 (diagonal is 135) -> 110 to 160

    if (angle >= -160 && angle <= -110) {
      return GestureDirection.topLeft;
    } else if (angle >= -70 && angle <= -20) {
      return GestureDirection.topRight;
    } else if (angle >= 20 && angle <= 70) {
      return GestureDirection.bottomRight;
    } else if (angle >= 110 && angle <= 160) {
      return GestureDirection.bottomLeft;
    } else if (angle > -110 && angle < -70) {
      return GestureDirection.up;
    } else if (angle > 70 && angle < 110) {
      return GestureDirection.down;
    } else if (angle > -20 && angle < 20) {
      return GestureDirection.right;
    } else {
      return GestureDirection.left;
    }
  }
}
