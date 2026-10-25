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

    test('Layout structure matches ABCD Serially default 7-row layout and supports Layout Icon Popup Menu options', () {
      final qwerty = KeyboardLayout.qwertyLayout;
      expect(qwerty.length, 7); // Row 0 (actionSymbolRow) + Row 1 (numberRow) + 5 letter rows

      // Row 0: Dedicated Action & Symbol Row directly above Number Row (9 keys)
      expect(qwerty[0].map((k) => k.primary).toList(), ['⇧', '?123', '?', '/', ',', '.', ' ', '⌫', '↵']);
      expect(qwerty[0][0].actionType, KeyActionType.shift);
      expect(qwerty[0][1].actionType, KeyActionType.switchLayout);
      expect(qwerty[0][6].actionType, KeyActionType.space);
      expect(qwerty[0][7].actionType, KeyActionType.backspace);
      expect(qwerty[0][8].actionType, KeyActionType.enter);

      // Row 1: Persistent number row (1..9, 0) directly beneath Action/Symbol Row
      expect(qwerty[1].length, 10);
      expect(qwerty[1].map((k) => k.primary).toList(), ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0']);
      expect(qwerty[1].every((k) => k.flex == 1.0), true);

      // 2. ABCD Serially (Default) Letter Row 1: 5 keys (a..e), equal width flex 1.0
      expect(qwerty[2].length, 5);
      expect(qwerty[2].map((k) => k.primary).toList(), ['a', 'b', 'c', 'd', 'e']);
      expect(qwerty[2].every((k) => k.flex == 1.0), true);

      // 2. ABCD Serially (Default) Letter Row 2: 5 keys (f..j), equal width flex 1.0
      expect(qwerty[3].length, 5);
      expect(qwerty[3].map((k) => k.primary).toList(), ['f', 'g', 'h', 'i', 'j']);
      expect(qwerty[3].every((k) => k.flex == 1.0), true);

      // 2. ABCD Serially (Default) Letter Row 3: 5 keys (k..o), equal width flex 1.0
      expect(qwerty[4].length, 5);
      expect(qwerty[4].map((k) => k.primary).toList(), ['k', 'l', 'm', 'n', 'o']);
      expect(qwerty[4].every((k) => k.flex == 1.0), true);

      // 2. ABCD Serially (Default) Letter Row 4: 5 keys (p..t), equal width flex 1.0
      expect(qwerty[5].length, 5);
      expect(qwerty[5].map((k) => k.primary).toList(), ['p', 'q', 'r', 's', 't']);
      expect(qwerty[5].every((k) => k.flex == 1.0), true);

      // 2. ABCD Serially (Default) Letter Row 5 (Bottom Row — ONLY letters: u, v, w, x flex 1.0 + compact y, z flex 0.55)
      expect(qwerty[6].length, 6);
      expect(qwerty[6].map((k) => k.primary).toList(), ['u', 'v', 'w', 'x', 'y', 'z']);
      expect(qwerty[6].every((k) => k.actionType == KeyActionType.inputChar), true);
      expect(qwerty[6].sublist(0, 4).every((k) => k.flex == 1.0), true);
      expect(qwerty[6][4].flex, 0.55);
      expect(qwerty[6][5].flex, 0.55);

      // 1. QWERTY Type Layout (selected via Layout Icon Button ⊞ Popup Menu):
      final v1 = KeyboardLayout.version1Layout;
      expect(v1.length, 7);
      expect(v1[2].map((k) => k.primary).toList(), ['q', 'w', 'e', 'r', 't', 'y']);
      expect(v1[3].map((k) => k.primary).toList(), ['a', 's', 'd', 'f', 'g', 'h']);
      expect(v1[4].map((k) => k.primary).toList(), ['z', 'x', 'c', 'v', 'b', 'n']);
      expect(v1[5].map((k) => k.primary).toList(), ['u', 'i', 'j', 'k', 'm', 'o']);
      expect(v1[6].map((k) => k.primary).toList(), [',', '.', ' ', 'p', 'l', '⌫', '↵']);

      // Layout Picker Popup Menu options:
      expect(KeyboardLayout.layoutModeOptions.values.toList(), [
        '1. QWERTY Type',
        '2. ABCD Serially',
        '3. Left Hand',
        '4. Right Hand Banglish',
        '5. Both Hand',
      ]);

      // 4. Right Hand Banglish Layout (9x5 ergonomic grid with 55 Banglish keys #1..#55):
      final rhb = KeyboardLayout.rightHandBanglishLayout;
      expect(rhb.length, 9);
      expect(rhb[0].map((k) => k.primary).toList(), ['%', '&', '@', ';', 'u']);
      expect(rhb[1].map((k) => k.primary).toList(), ['8', '9', '/', '!', 'w', 'm', 'n']);
      expect(rhb[2].map((k) => k.primary).toList(), ['6', '7', '?', 'b', 'k', 'r']);
      expect(rhb[3].map((k) => k.primary).toList(), ['4', '5', 'y', 't', 'a', 'i']);
      expect(rhb[4].map((k) => k.primary).toList(), ['2', '3', '↵', '⇧', '⌫', ' ']);
      expect(rhb[4][3].actionType, KeyActionType.shift);
      expect(rhb[4][4].actionType, KeyActionType.backspace); // Cross left of Space
      expect(rhb[4][5].actionType, KeyActionType.space); // Space at most right
      expect(rhb[4][5].flex > rhb[4][4].flex, true); // Space button occupies more horizontal width
      expect(rhb[5].map((k) => k.primary).toList(), ['0', '1', 'c', 's', 'e', 'h']);
      expect(rhb[6].map((k) => k.primary).toList(), ['-', 'x', '.', 'd', 'l', 'o']);
      expect(rhb[7].map((k) => k.primary).toList(), ['q', ')', "'", 'g', 'p', 'j']);
      expect(rhb[8].map((k) => k.primary).toList(), ['(', ':', 'z', '"', 'f', 'v', ',']);

      // Verify leftmost 2 columns take the exact same horizontal width as 2, 3 (0.44 each) across Rows 2..9:
      for (int r = 1; r < 9; r++) {
        expect(rhb[r][0].flex, 0.44);
        expect(rhb[r][1].flex, 0.44);
      }

      // Research Color Mapping (#1 Spacebar -> Blue, #2 'a' -> Grey, Cross (⌫) -> Red 0xFFD32F2F, #4..#12 -> Study Colors):
      expect(KeyboardLayout.getResearchColorValue(const KeyModel(primary: ' ', actionType: KeyActionType.space)), 0xFF1565C0);
      expect(KeyboardLayout.getResearchColorValue(const KeyModel(primary: 'a')), 0xFF607D8B);
      expect(KeyboardLayout.getResearchColorValue(const KeyModel(primary: '⌫', actionType: KeyActionType.backspace)), 0xFFD32F2F);
      expect(KeyboardLayout.getResearchColorValue(const KeyModel(primary: 'e')), 0xFFFBC02D);
      expect(KeyboardLayout.getResearchColorValue(const KeyModel(primary: 'r')), 0xFF00ACC1);
      expect(KeyboardLayout.getResearchColorValue(const KeyModel(primary: 'o')), 0xFF141414);
      expect(KeyboardLayout.getResearchColorValue(const KeyModel(primary: 'i')), 0xFFF5F5F5);
      expect(KeyboardLayout.getResearchColorValue(const KeyModel(primary: 'h')), 0xFF29B6F6);
      expect(KeyboardLayout.getResearchColorValue(const KeyModel(primary: 'k')), 0xFF1A237E);
      expect(KeyboardLayout.getResearchColorValue(const KeyModel(primary: 'n')), 0xFFC62828);
      expect(KeyboardLayout.getResearchColorValue(const KeyModel(primary: 't')), 0xFF90A4AE);
      expect(KeyboardLayout.getResearchColorValue(const KeyModel(primary: 's')), 0xFFFF6D00);

      // Symbol layout has 7 rows: Row 0 (symbolActionRow), Row 1 (numberRow), and 5 symbol rows
      final symbols = KeyboardLayout.symbolLayout;
      expect(symbols.length, 7);
      expect(symbols[0].any((k) => k.primary == 'ABC' && k.actionType == KeyActionType.switchLayout), true);
      expect(symbols[0][7].actionType, KeyActionType.backspace);
      expect(symbols[0][8].actionType, KeyActionType.enter);
      expect(symbols[1].map((k) => k.primary).toList(), ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0']);
      expect(symbols[2].length, 5);
      expect(symbols[3].length, 5);
      expect(symbols[4].length, 5);
      expect(symbols[5].length, 5);
      expect(symbols[6].length, 6);

      // Top terminal bar: ESC, CTRL, ALT, Arrow keys, and single Layout Icon Button (⊞)
      final terminalKeys = KeyboardLayout.terminalBarKeys;
      expect(terminalKeys.any((k) => k.primary == 'ESC'), true);
      expect(terminalKeys.any((k) => k.primary == 'CTRL'), true);
      expect(terminalKeys.any((k) => k.primary == 'ALT'), true);
      expect(terminalKeys.any((k) => k.primary == '⊞' && k.actionType == KeyActionType.switchVersion), true);
      expect(terminalKeys.any((k) => k.primary == 'V2'), false);
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

    test('Long press delay clamping supports 0ms to 3000ms user customization', () {
      expect(ImeChannelService.minLongPressMs, 0);
      expect(ImeChannelService.maxLongPressMs, 3000);
      expect(ImeChannelService.clampLongPressMs(-100), 0);
      expect(ImeChannelService.clampLongPressMs(0), 0);
      expect(ImeChannelService.clampLongPressMs(400), 400);
      expect(ImeChannelService.clampLongPressMs(1500), 1500);
      expect(ImeChannelService.clampLongPressMs(3000), 3000);
      expect(ImeChannelService.clampLongPressMs(5000), 3000);
    });
  });
}
