import 'package:flutter_test/flutter_test.dart';
import 'package:unexpected_flutter_keyboard/models/gesture_direction.dart';
import 'package:unexpected_flutter_keyboard/models/key_model.dart';
import 'package:unexpected_flutter_keyboard/models/keyboard_layout.dart';
import 'package:unexpected_flutter_keyboard/services/ime_channel_service.dart';

void main() {
  group('KeyModel and KeyboardLayout Tests', () {
    test('Resolves primary character on center tap', () {
      const key = KeyModel(
        primary: 'q',
        topLeft: '1',
        topRight: '`',
        bottomLeft: '~',
        bottomRight: '\\',
      );

      expect(key.resolveOutput(GestureDirection.center), 'q');
    });

    test('Resolves corner characters correctly on directional swipe', () {
      const key = KeyModel(
        primary: 'q',
        topLeft: '1',
        topRight: '`',
        bottomLeft: '~',
        bottomRight: '\\',
      );

      expect(key.resolveOutput(GestureDirection.topLeft), '1');
      expect(key.resolveOutput(GestureDirection.topRight), '`');
      expect(key.resolveOutput(GestureDirection.bottomLeft), '~');
      expect(key.resolveOutput(GestureDirection.bottomRight), '\\');
    });

    test('Shift state capitalizes primary letters', () {
      const key = KeyModel(primary: 'a', topLeft: '@');
      expect(key.resolveOutput(GestureDirection.center, isShifted: true), 'A');
    });

    test('Layout structure contains expected rows and keys', () {
      final qwerty = KeyboardLayout.qwertyLayout;
      expect(qwerty.length, 4); // 4 main rows
      expect(qwerty[0].length, 10); // Row 1 has 10 keys (q..p)

      final terminalKeys = KeyboardLayout.terminalBarKeys;
      expect(terminalKeys.any((k) => k.primary == 'ESC'), true);
      expect(terminalKeys.any((k) => k.primary == 'CTRL'), true);
      expect(terminalKeys.any((k) => k.primary == 'ALT'), true);
      expect(terminalKeys.any((k) => k.primary == 'TAB'), true);
    });

    test('Keyboard height clamping and total height calculation work accurately', () {
      expect(ImeChannelService.clampKeyHeight(20.0), ImeChannelService.minKeyHeight);
      expect(ImeChannelService.clampKeyHeight(90.0), ImeChannelService.maxKeyHeight);
      expect(ImeChannelService.clampKeyHeight(52.0), 52.0);

      final compactTotal = ImeChannelService.calculateTotalKeyboardHeightDp(38.0);
      final normalTotal = ImeChannelService.calculateTotalKeyboardHeightDp(46.0);
      final tallTotal = ImeChannelService.calculateTotalKeyboardHeightDp(58.0);

      expect(compactTotal < normalTotal, true);
      expect(normalTotal < tallTotal, true);
    });
  });
}
