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

    test('Layout structure contains persistent number row at row0, 9 equal-width keys in rows 1-3, and Shift/Settings in row 4', () {
      final qwerty = KeyboardLayout.qwertyLayout;
      expect(qwerty.length, 5); // row0 (numbers) + 4 main rows

      // Row 0: Persistent number row (1..9, 0) directly beneath terminal modifier bar
      expect(qwerty[0].length, 10);
      expect(qwerty[0].map((k) => k.primary).toList(), ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0']);
      expect(qwerty[0].every((k) => k.flex == 1.0), true);

      // Row 1: 9 keys (q..o), p moved to Row 3
      expect(qwerty[1].length, 9);
      expect(qwerty[1].map((k) => k.primary).toList(), ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o']);
      expect(qwerty[1].every((k) => k.flex == 1.0), true);

      // Row 2: 9 keys (a..l), unchanged
      expect(qwerty[2].length, 9);
      expect(qwerty[2].map((k) => k.primary).toList(), ['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l']);
      expect(qwerty[2].every((k) => k.flex == 1.0), true);

      // Row 3: 9 keys (z x c v b n m p ⌫), all equal horizontal width (flex 1.0)
      expect(qwerty[3].length, 9);
      expect(qwerty[3].map((k) => k.primary).toList(), ['z', 'x', 'c', 'v', 'b', 'n', 'm', 'p', '⌫']);
      expect(qwerty[3].every((k) => k.flex == 1.0), true);

      // Row 4: Shift (⇧) moved to left of ?123, Settings (⚙️) key present, comma has ⚙️ corner, Space width reduced
      expect(qwerty[4][0].primary, '⇧');
      expect(qwerty[4][0].actionType, KeyActionType.shift);
      expect(qwerty[4][1].primary, '?123');
      expect(qwerty[4][2].primary, '⚙️');
      expect(qwerty[4][2].actionType, KeyActionType.openSettings);
      expect(qwerty[4][3].primary, ',');
      expect(qwerty[4][3].topLeft, '⚙️');
      final spaceKey = qwerty[4].firstWhere((k) => k.actionType == KeyActionType.space);
      expect(spaceKey.flex < 4.0, true);

      // Symbol layout also keeps row0 as number row and Row 4 contains Settings (⚙️) key
      final symbols = KeyboardLayout.symbolLayout;
      expect(symbols.length, 5);
      expect(symbols[0].map((k) => k.primary).toList(), ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0']);
      expect(symbols[4].any((k) => k.actionType == KeyActionType.openSettings), true);

      final terminalKeys = KeyboardLayout.terminalBarKeys;
      expect(terminalKeys.any((k) => k.primary == 'ESC'), true);
      expect(terminalKeys.any((k) => k.primary == 'CTRL'), true);
      expect(terminalKeys.any((k) => k.primary == 'ALT'), true);
      expect(terminalKeys.any((k) => k.primary == 'TAB'), true);
    });

    test('Keyboard height clamping and double-height calculation work accurately', () {
      expect(ImeChannelService.maxKeyHeight, 136.0);
      expect(ImeChannelService.clampKeyHeight(20.0), ImeChannelService.minKeyHeight);
      expect(ImeChannelService.clampKeyHeight(200.0), ImeChannelService.maxKeyHeight);
      expect(ImeChannelService.clampKeyHeight(120.0), 120.0);

      final compactTotal = ImeChannelService.calculateTotalKeyboardHeightDp(38.0);
      final normalTotal = ImeChannelService.calculateTotalKeyboardHeightDp(46.0);
      final tallTotal = ImeChannelService.calculateTotalKeyboardHeightDp(68.0);
      final doubleTotal = ImeChannelService.calculateTotalKeyboardHeightDp(136.0);

      expect(compactTotal < normalTotal, true);
      expect(normalTotal < tallTotal, true);
      expect(tallTotal < doubleTotal, true);
    });
  });
}
