import 'package:flutter_test/flutter_test.dart';
import 'package:unexpected_flutter_keyboard/models/gesture_direction.dart';

void main() {
  group('GestureDirection Decoding Tests', () {
    test('Zero or minor offset returns center', () {
      expect(GestureDirection.fromDelta(Offset.zero), GestureDirection.center);
      expect(GestureDirection.fromDelta(const Offset(3.0, 3.0)), GestureDirection.center);
      expect(GestureDirection.fromDelta(const Offset(-4.0, 2.0)), GestureDirection.center);
    });

    test('Diagonal Top-Left swipe detected', () {
      // Negative dx, negative dy
      final dir = GestureDirection.fromDelta(const Offset(-25.0, -25.0));
      expect(dir, GestureDirection.topLeft);
    });

    test('Diagonal Top-Right swipe detected', () {
      // Positive dx, negative dy
      final dir = GestureDirection.fromDelta(const Offset(25.0, -25.0));
      expect(dir, GestureDirection.topRight);
    });

    test('Diagonal Bottom-Left swipe detected', () {
      // Negative dx, positive dy
      final dir = GestureDirection.fromDelta(const Offset(-25.0, 25.0));
      expect(dir, GestureDirection.bottomLeft);
    });

    test('Diagonal Bottom-Right swipe detected', () {
      // Positive dx, positive dy
      final dir = GestureDirection.fromDelta(const Offset(25.0, 25.0));
      expect(dir, GestureDirection.bottomRight);
    });

    test('Vertical Up and Down swipes detected', () {
      final up = GestureDirection.fromDelta(const Offset(0.0, -30.0));
      expect(up, GestureDirection.up);

      final down = GestureDirection.fromDelta(const Offset(0.0, 30.0));
      expect(down, GestureDirection.down);
    });
  });
}
