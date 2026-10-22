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

    test('Layout structure matches 6-key rows in all 4 letter rows (q..y, a..h, z..n, u i j k m o), 5th bottom row (, . Space p l ⌫ ↵), and top bar (⇧ and ?123)', () {
      final qwerty = KeyboardLayout.qwertyLayout;
      expect(qwerty.length, 6); // row0 (numbers) + 4 letter rows + 5th bottom row

      // Row 0: Persistent number row (1..9, 0) directly beneath terminal modifier bar
      expect(qwerty[0].length, 10);
      expect(qwerty[0].map((k) => k.primary).toList(), ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0']);
      expect(qwerty[0].every((k) => k.flex == 1.0), true);

      // First row: 6 keys (q..y), equal width flex 1.0
      expect(qwerty[1].length, 6);
      expect(qwerty[1].map((k) => k.primary).toList(), ['q', 'w', 'e', 'r', 't', 'y']);
      expect(qwerty[1].every((k) => k.flex == 1.0), true);

      // Second row: 6 keys (a..h), equal width flex 1.0
      expect(qwerty[2].length, 6);
      expect(qwerty[2].map((k) => k.primary).toList(), ['a', 's', 'd', 'f', 'g', 'h']);
      expect(qwerty[2].every((k) => k.flex == 1.0), true);

      // Third row: 6 keys (z..n), all equal horizontal width (flex 1.0)
      expect(qwerty[3].length, 6);
      expect(qwerty[3].map((k) => k.primary).toList(), ['z', 'x', 'c', 'v', 'b', 'n']);
      expect(qwerty[3].every((k) => k.flex == 1.0), true);

      // Fourth row: 6 keys (u i j k m o), all equal horizontal width (flex 1.0)
      expect(qwerty[4].length, 6);
      expect(qwerty[4].map((k) => k.primary).toList(), ['u', 'i', 'j', 'k', 'm', 'o']);
      expect(qwerty[4].every((k) => k.flex == 1.0), true);

      // Fifth row (Bottom row unchanged): [ , | . | Space | p | l | ⌫ | ↵ ]
      expect(qwerty[5].map((k) => k.primary).toList(), [',', '.', ' ', 'p', 'l', '⌫', '↵']);
      expect(qwerty[5][5].actionType, KeyActionType.backspace);
      expect(qwerty[5][6].actionType, KeyActionType.enter);
      expect(qwerty[5].any((k) => k.actionType == KeyActionType.shift), false);
      expect(qwerty[5].any((k) => k.actionType == KeyActionType.switchLayout), false);
      expect(qwerty[5].any((k) => k.actionType == KeyActionType.openSettings), false);

      // Symbol layout keeps row0 as number row, 6 keys in rows 1-4, and ABC + ⌫ left of ↵ in row 5
      final symbols = KeyboardLayout.symbolLayout;
      expect(symbols.length, 6);
      expect(symbols[0].map((k) => k.primary).toList(), ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0']);
      expect(symbols[1].length, 6);
      expect(symbols[2].length, 6);
      expect(symbols[3].length, 6);
      expect(symbols[4].length, 6);
      expect(symbols[5].any((k) => k.primary == 'ABC' && k.actionType == KeyActionType.switchLayout), true);
      expect(symbols[5][5].actionType, KeyActionType.backspace);
      expect(symbols[5][6].actionType, KeyActionType.enter);

      // Top bar: TAB and META hidden and replaced by Shift (⇧) and ?123
      final terminalKeys = KeyboardLayout.terminalBarKeys;
      expect(terminalKeys.any((k) => k.primary == 'ESC'), true);
      expect(terminalKeys.any((k) => k.primary == '⇧' && k.actionType == KeyActionType.shift), true);
      expect(terminalKeys.any((k) => k.primary == 'CTRL'), true);
      expect(terminalKeys.any((k) => k.primary == 'ALT'), true);
      expect(terminalKeys.any((k) => k.primary == '?123' && k.actionType == KeyActionType.switchLayout), true);
      expect(terminalKeys.any((k) => k.primary == 'TAB'), false);
      expect(terminalKeys.any((k) => k.primary == 'META'), false);
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
