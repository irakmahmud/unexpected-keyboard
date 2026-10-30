import 'key_model.dart';

enum LayoutType {
  qwerty,
  symbols,
  functionKeys,
}

class KeyboardLayout {
  /// Available Layout Options shown in the Layout Picker Popup when clicking the Layout Icon Button (⊞)
  static const Map<String, String> layoutModeOptions = {
    'qwerty': '1. QWERTY Type',
    'abcd': '2. ABCD Serially',
    'left_hand': '3. Left Hand',
    'right_hand': '4. Right Hand Banglish',
    'both_hand': '5. Both Hand',
  };

  /// Scientifically Tested 12-Color Palette from User's Research:
  /// #1 Spacebar: Blue (নীল বোতাম — Journal of Marketing #1, 12.7% CTR)
  /// #2 'a': Grey (ধূসর বোতাম — Journal of Marketing #2, 9.4% CTR)
  /// #3..#12: 2022 Button-Press Preference Study Top 10:
  ///   1. উজ্জ্বল সবুজ (Bright Green)
  ///   2. গাঢ় উজ্জ্বল হলুদ (Deep Bright Yellow)
  ///   3. উজ্জ্বল সবুজাভ নীল (Bright Teal-Blue)
  ///   4. কালো (Pure Black)
  ///   5. সাদা (Crisp White)
  ///   6. হালকা নীল (Light Blue)
  ///   7. তীব্র নীল (Intense Royal Blue)
  ///   8. গাঢ় লাল (Dark Red)
  ///   9. হালকা ধূসর (Light Grey)
  ///  10. তীব্র হলুদাভ কমলা (Intense Yellowish Orange)
  static const List<int> researchColorPalette = [
    0xFF1565C0, // #1: নীল বোতাম (Blue — Journal of Marketing #1)
    0xFF607D8B, // #2: ধূসর বোতাম (Grey — Journal of Marketing #2)
    0xFF00C853, // #3 (Study #1): উজ্জ্বল সবুজ (Bright Green)
    0xFFFBC02D, // #4 (Study #2): গাঢ় উজ্জ্বল হলুদ (Deep Bright Yellow)
    0xFF00ACC1, // #5 (Study #3): উজ্জ্বল সবুজাভ নীল (Bright Teal-Blue)
    0xFF141414, // #6 (Study #4): কালো (Black)
    0xFFF5F5F5, // #7 (Study #5): সাদা (White)
    0xFF29B6F6, // #8 (Study #6): হালকা নীল (Light Blue)
    0xFF1A237E, // #9 (Study #7): তীব্র নীল (Intense Blue)
    0xFFC62828, // #10 (Study #8): গাঢ় লাল (Dark Red)
    0xFF90A4AE, // #11 (Study #9): হালকা ধূসর (Light Grey)
    0xFFFF6D00, // #12 (Study #10): তীব্র হলুদাভ কমলা (Intense Yellowish Orange)
  ];

  /// Official 150-Key Final Rank Table (1..110 Verified Text Output Keys + 111..150 Action/Gesture Keys):
  static int getBanglishRank(KeyModel key) {
    switch (key.actionType) {
      case KeyActionType.space:
        return 1; // KEY_063_SPACE
      case KeyActionType.enter:
        return 31; // KEY_064_ENTER
      case KeyActionType.backspace:
        return 111; // KEY_111_BACKSPACE
      case KeyActionType.shift:
        return 112; // KEY_112_SHIFT_MODIFIER
      case KeyActionType.switchLayout:
        return key.primary == 'ABC' ? 116 : 114; // KEY_116_ABC_RETURN / KEY_114_SYM_LAYER_123
      case KeyActionType.arrowLeft:
        return 123; // KEY_123_CURSOR_LEFT
      case KeyActionType.arrowRight:
        return 124; // KEY_124_CURSOR_RIGHT
      case KeyActionType.tab:
        return 130; // KEY_130_TAB_INDENT
      case KeyActionType.openSettings:
        return 134; // KEY_134_KB_SETTINGS
      case KeyActionType.inputChar:
        switch (key.primary) {
          case 'a': return 2;   // KEY_001_A_LOWER
          case 'e': return 3;   // KEY_005_E_LOWER
          case 'o': return 4;   // KEY_015_O_LOWER
          case 'i': return 5;   // KEY_009_I_LOWER
          case 't': return 6;   // KEY_020_T_LOWER
          case 'r': return 7;   // KEY_018_R_LOWER
          case 'n': return 8;   // KEY_014_N_LOWER
          case 'h': return 9;   // KEY_008_H_LOWER
          case 's': return 10;  // KEY_019_S_LOWER
          case 'l': return 11;  // KEY_012_L_LOWER
          case 'k': return 12;  // KEY_011_K_LOWER
          case 'd': return 13;  // KEY_004_D_LOWER
          case 'm': return 14;  // KEY_013_M_LOWER
          case 'u': return 15;  // KEY_021_U_LOWER
          case 'b': return 16;  // KEY_002_B_LOWER
          case 'c': return 17;  // KEY_003_C_LOWER
          case '.': return 18;  // KEY_065_PUNCT_01
          case 'p': return 19;  // KEY_016_P_LOWER
          case 'g': return 20;  // KEY_007_G_LOWER
          case 'y': return 21;  // KEY_025_Y_LOWER
          case 'j': return 22;  // KEY_010_J_LOWER
          case 'v': return 23;  // KEY_022_V_LOWER
          case 'f': return 24;  // KEY_006_F_LOWER
          case 'w': return 25;  // KEY_023_W_LOWER
          case ',': return 26;  // KEY_066_PUNCT_02
          case 'A': return 27;  // KEY_027_A_UPPER
          case '?': return 28;  // KEY_067_PUNCT_03
          case 'T': return 29;  // KEY_046_T_UPPER
          case 'S': return 30;  // KEY_045_S_UPPER
          case 'E': return 32;  // KEY_031_E_UPPER
          case 'I': return 33;  // KEY_035_I_UPPER
          case 'B': return 34;  // KEY_028_B_UPPER
          case '0': return 35;  // KEY_053_DIGIT_0
          case '1': return 36;  // KEY_054_DIGIT_1
          case 'K': return 37;  // KEY_037_K_UPPER
          case '!': return 38;  // KEY_068_PUNCT_04
          case '।': return 39;  // KEY_097_REGSYM_01
          case 'M': return 40;  // KEY_039_M_UPPER
          case 'D': return 41;  // KEY_030_D_UPPER
          case 'z': return 42;  // KEY_026_Z_LOWER
          case 'P': return 43;  // KEY_042_P_UPPER
          case 'x': return 44;  // KEY_024_X_LOWER
          case '2': return 45;  // KEY_055_DIGIT_2
          case 'O': return 46;  // KEY_041_O_UPPER
          case 'N': return 47;  // KEY_040_N_UPPER
          case "'": return 48;  // KEY_069_PUNCT_05
          case 'C': return 49;  // KEY_029_C_UPPER
          case 'V': return 50;  // KEY_048_V_UPPER
          case 'R': return 51;  // KEY_044_R_UPPER
          case 'H': return 52;  // KEY_034_H_UPPER
          case '-': return 53;  // KEY_071_PUNCT_07
          case 'G': return 54;  // KEY_033_G_UPPER
          case 'q': return 55;  // KEY_017_Q_LOWER
          case 'J': return 56;  // KEY_036_J_UPPER
          case '5': return 57;  // KEY_058_DIGIT_5
          case '3': return 58;  // KEY_056_DIGIT_3
          case 'F': return 59;  // KEY_032_F_UPPER
          case 'L': return 60;  // KEY_038_L_UPPER
          case '4': return 61;  // KEY_057_DIGIT_4
          case 'W': return 62;  // KEY_049_W_UPPER
          case '6': return 63;  // KEY_059_DIGIT_6
          case 'U': return 64;  // KEY_047_U_UPPER
          case '7': return 65;  // KEY_060_DIGIT_7
          case '9': return 66;  // KEY_062_DIGIT_9
          case ')': return 67;  // KEY_073_PUNCT_09
          case '"': return 68;  // KEY_070_PUNCT_06
          case '8': return 69;  // KEY_061_DIGIT_8
          case '&': return 70;  // KEY_079_EXTSYM_01
          case ':': return 71;  // KEY_074_PUNCT_10
          case '(': return 72;  // KEY_072_PUNCT_08
          case '/': return 73;  // KEY_076_PUNCT_12
          case '%': return 74;  // KEY_080_EXTSYM_02
          case 'Y': return 75;  // KEY_051_Y_UPPER
          case '*': return 76;  // KEY_083_EXTSYM_05
          case '+': return 77;  // KEY_081_EXTSYM_03
          case 'Z': return 78;  // KEY_052_Z_UPPER
          case '…': return 79;  // KEY_107_REGSYM_11
          case 'Q': return 80;  // KEY_043_Q_UPPER
          case 'X': return 81;  // KEY_050_X_UPPER
          case '#': return 82;  // KEY_078_PUNCT_14
          case '=': return 83;  // KEY_082_EXTSYM_04
          case ';': return 84;  // KEY_075_PUNCT_11
          case '—': return 85;  // KEY_109_REGSYM_13
          case '>': return 86;  // KEY_092_EXTSYM_14
          case ']': return 87;  // KEY_094_EXTSYM_16
          case '[': return 88;  // KEY_093_EXTSYM_15
          case '~': return 89;  // KEY_087_EXTSYM_09
          case '|': return 90;  // KEY_090_EXTSYM_12
          case '<': return 91;  // KEY_091_EXTSYM_13
          case '•': return 92;  // KEY_110_REGSYM_14
          case '°': return 93;  // KEY_103_REGSYM_07
          case '^': return 94;  // KEY_089_EXTSYM_11
          case '\$': return 95; // KEY_086_EXTSYM_08
          case '×': return 96;  // KEY_105_REGSYM_09
          case '\\': return 97; // KEY_085_EXTSYM_07
          case '–': return 98;  // KEY_108_REGSYM_12
          case '@': return 99;  // KEY_077_PUNCT_13
          case '}': return 100; // KEY_096_EXTSYM_18
          case '{': return 101; // KEY_095_EXTSYM_17
          case '÷': return 102; // KEY_106_REGSYM_10
          case '_': return 103; // KEY_084_EXTSYM_06
          case '`': return 104; // KEY_088_EXTSYM_10
          case '৳': return 105; // KEY_098_REGSYM_02
          case '₹': return 106; // KEY_099_REGSYM_03
          case '€': return 107; // KEY_100_REGSYM_04
          case '£': return 108; // KEY_101_REGSYM_05
          case '¥': return 109; // KEY_102_REGSYM_06
          case '±': return 110; // KEY_104_REGSYM_08
          default: return 0;
        }
      default:
        return 0;
    }
  }

  /// 4 Distinct Category Colors for Right Hand Banglish Layout:
  /// 1. Small Letters (a..z)        -> Blue (0xFF1565C0)
  /// 2. Capital Letters (A..Z)      -> Green (0xFF2E7D32)
  /// 3. Number Row (123: 1..9, 0)   -> Teal (0xFF00838F)
  /// 4. Non-Letter Keys (Punct/Act) -> Slate Grey (0xFF455A64)
  static const int smallLetterColor = 0xFF1565C0;
  static const int capitalLetterColor = 0xFF2E7D32;
  static const int numberRowColor = 0xFF00838F;
  static const int nonLetterKeyColor = 0xFF455A64;

  static bool isSmallLetterKey(KeyModel key) {
    if (key.actionType != KeyActionType.inputChar || key.primary.length != 1) {
      return false;
    }
    final int code = key.primary.codeUnitAt(0);
    return code >= 0x61 && code <= 0x7A; // 'a'..'z'
  }

  static bool isCapitalLetterKey(KeyModel key) {
    if (key.actionType != KeyActionType.inputChar || key.primary.length != 1) {
      return false;
    }
    final int code = key.primary.codeUnitAt(0);
    return code >= 0x41 && code <= 0x5A; // 'A'..'Z'
  }

  static bool isDigitKey(KeyModel key) {
    if (key.actionType != KeyActionType.inputChar || key.primary.length != 1) {
      return false;
    }
    final int code = key.primary.codeUnitAt(0);
    return code >= 0x30 && code <= 0x39; // '0'..'9'
  }

  static int? getResearchColorValue(KeyModel key) {
    if (isSmallLetterKey(key)) {
      return smallLetterColor;
    }
    if (isCapitalLetterKey(key)) {
      return capitalLetterColor;
    }
    if (isDigitKey(key)) {
      return numberRowColor;
    }
    return nonLetterKeyColor;
  }

  /// Top Terminal Bar: ESC, CTRL, ALT, Arrow keys, and Layout Icon Button (⊞)
  static List<KeyModel> get terminalBarKeys => const [
    KeyModel(primary: 'ESC', actionType: KeyActionType.escape, flex: 1.0, androidKeyCode: 111),
    KeyModel(primary: 'CTRL', actionType: KeyActionType.ctrl, flex: 1.15, androidKeyCode: 113),
    KeyModel(primary: 'ALT', actionType: KeyActionType.alt, flex: 1.0, androidKeyCode: 57),
    KeyModel(primary: '◀', actionType: KeyActionType.arrowLeft, flex: 0.85, androidKeyCode: 21),
    KeyModel(primary: '▼', actionType: KeyActionType.arrowDown, flex: 0.85, androidKeyCode: 20),
    KeyModel(primary: '▲', actionType: KeyActionType.arrowUp, flex: 0.85, androidKeyCode: 19),
    KeyModel(primary: '▶', actionType: KeyActionType.arrowRight, flex: 0.85, androidKeyCode: 22),
    KeyModel(primary: '⊞', actionType: KeyActionType.switchVersion, flex: 0.9),
  ];

  /// Row 0 (Directly above Number Row): Shift (⇧), ?123, ?, /, ,, ., Space, Backspace (Cross ⌫), Enter (↵)
  static List<KeyModel> get actionSymbolRow => const [
    KeyModel(primary: '⇧', actionType: KeyActionType.shift, flex: 1.0),
    KeyModel(primary: '?123', actionType: KeyActionType.switchLayout, flex: 1.0),
    KeyModel(primary: '?', topLeft: '!', topRight: '¿', bottomLeft: '¡', bottomRight: '@', swipeUp: '!', flex: 0.85),
    KeyModel(primary: '/', topLeft: '\\', topRight: '|', bottomLeft: '-', bottomRight: '_', swipeUp: '\\', flex: 0.85),
    KeyModel(primary: ',', topLeft: '<', topRight: ';', bottomLeft: "'", bottomRight: '"', swipeUp: '<', flex: 0.85),
    KeyModel(primary: '.', topLeft: '>', topRight: ':', bottomLeft: '(', bottomRight: ')', swipeUp: '>', flex: 0.85),
    KeyModel(primary: ' ', actionType: KeyActionType.space, flex: 1.5),
    KeyModel(primary: '⌫', actionType: KeyActionType.backspace, flex: 1.0, androidKeyCode: 67),
    KeyModel(primary: '↵', actionType: KeyActionType.enter, flex: 1.0, androidKeyCode: 66),
  ];

  static List<KeyModel> get symbolActionRow => const [
    KeyModel(primary: '⇧', actionType: KeyActionType.shift, flex: 1.0),
    KeyModel(primary: 'ABC', actionType: KeyActionType.switchLayout, flex: 1.0),
    KeyModel(primary: '?', topLeft: '!', topRight: '¿', bottomLeft: '¡', bottomRight: '@', swipeUp: '!', flex: 0.85),
    KeyModel(primary: '/', topLeft: '\\', topRight: '|', bottomLeft: '-', bottomRight: '_', swipeUp: '\\', flex: 0.85),
    KeyModel(primary: ',', topLeft: '<', topRight: ';', bottomLeft: "'", bottomRight: '"', swipeUp: '<', flex: 0.85),
    KeyModel(primary: '.', topLeft: '>', topRight: ':', bottomLeft: '(', bottomRight: ')', swipeUp: '>', flex: 0.85),
    KeyModel(primary: ' ', actionType: KeyActionType.space, flex: 1.5),
    KeyModel(primary: '⌫', actionType: KeyActionType.backspace, flex: 1.0, androidKeyCode: 67),
    KeyModel(primary: '↵', actionType: KeyActionType.enter, flex: 1.0, androidKeyCode: 66),
  ];

  /// Row 1: Dedicated Number Row positioned directly below Action/Symbol Row
  static List<KeyModel> get numberRow => const [
    KeyModel(primary: '1', topLeft: '!', topRight: '~', bottomLeft: '`', bottomRight: '¡', swipeUp: '!', flex: 1.0),
    KeyModel(primary: '2', topLeft: '@', topRight: '#', bottomLeft: '<', bottomRight: '>', swipeUp: '@', flex: 1.0),
    KeyModel(primary: '3', topLeft: '#', topRight: '\$', bottomLeft: '{', bottomRight: '}', swipeUp: '#', flex: 1.0),
    KeyModel(primary: '4', topLeft: '\$', topRight: '€', bottomLeft: '[', bottomRight: ']', swipeUp: '\$', flex: 1.0),
    KeyModel(primary: '5', topLeft: '%', topRight: '‰', bottomLeft: '(', bottomRight: ')', swipeUp: '%', flex: 1.0),
    KeyModel(primary: '6', topLeft: '^', topRight: '&', bottomLeft: '=', bottomRight: '+', swipeUp: '^', flex: 1.0),
    KeyModel(primary: '7', topLeft: '&', topRight: '*', bottomLeft: '-', bottomRight: '_', swipeUp: '&', flex: 1.0),
    KeyModel(primary: '8', topLeft: '*', topRight: '+', bottomLeft: '/', bottomRight: '\\', swipeUp: '*', flex: 1.0),
    KeyModel(primary: '9', topLeft: '(', topRight: '[', bottomLeft: '{', bottomRight: '«', swipeUp: '(', flex: 1.0),
    KeyModel(primary: '0', topLeft: ')', topRight: ']', bottomLeft: '}', bottomRight: '»', swipeUp: ')', flex: 1.0),
  ];

  /// VERSION 1 LAYER ALIGNMENT (Activated by clicking the V2/V1 Layer button):
  /// Row 0: Action & Symbol Row directly above Number Row
  /// Row 1: Number Row (1..0)
  /// Rows 2..6: Version 1 Letter Alignment (q..y, a..h, z..n, u i j k m o, and , . Space p l ⌫ ↵)
  static List<List<KeyModel>> get version1Layout => [
    actionSymbolRow,
    numberRow,
    // First row (q to y): q w e r t y
    const [
      KeyModel(primary: 'q', topLeft: '1', topRight: '`', bottomLeft: '~', bottomRight: '\\', swipeUp: '1', flex: 1.0),
      KeyModel(primary: 'w', topLeft: '2', topRight: '|', bottomLeft: '<', bottomRight: '>', swipeUp: '2', flex: 1.0),
      KeyModel(primary: 'e', topLeft: '3', topRight: '€', bottomLeft: '{', bottomRight: '}', swipeUp: '3', flex: 1.0),
      KeyModel(primary: 'r', topLeft: '4', topRight: '₹', bottomLeft: '[', bottomRight: ']', swipeUp: '4', flex: 1.0),
      KeyModel(primary: 't', topLeft: '5', topRight: '%', bottomLeft: '(', bottomRight: ')', swipeUp: '5', flex: 1.0),
      KeyModel(primary: 'y', topLeft: '6', topRight: '^', bottomLeft: '=', bottomRight: '+', swipeUp: '6', flex: 1.0),
    ],
    // Second row (a to h): a s d f g h
    const [
      KeyModel(primary: 'a', topLeft: '@', topRight: '!', bottomLeft: '#', bottomRight: '\$', swipeUp: '@', flex: 1.0),
      KeyModel(primary: 's', topLeft: '#', topRight: '\$', bottomLeft: '%', bottomRight: '^', swipeUp: '#', flex: 1.0),
      KeyModel(primary: 'd', topLeft: '\$', topRight: '%', bottomLeft: '&', bottomRight: '*', swipeUp: '\$', flex: 1.0),
      KeyModel(primary: 'f', topLeft: '%', topRight: '^', bottomLeft: '_', bottomRight: '+', swipeUp: '%', flex: 1.0),
      KeyModel(primary: 'g', topLeft: '&', topRight: '*', bottomLeft: '-', bottomRight: '=', swipeUp: '&', flex: 1.0),
      KeyModel(primary: 'h', topLeft: '-', topRight: '_', bottomLeft: ':', bottomRight: ';', swipeUp: '-', flex: 1.0),
    ],
    // Third row (z to n): z x c v b n
    const [
      KeyModel(primary: 'z', topLeft: '*', topRight: '/', bottomLeft: '\\', bottomRight: '|', swipeUp: '*', flex: 1.0),
      KeyModel(primary: 'x', topLeft: '"', topRight: "'", bottomLeft: '`', bottomRight: '~', swipeUp: '"', flex: 1.0),
      KeyModel(primary: 'c', topLeft: "'", topRight: '`', bottomLeft: '©', bottomRight: '¢', swipeUp: "'", flex: 1.0),
      KeyModel(primary: 'v', topLeft: ':', topRight: ';', bottomLeft: '•', bottomRight: '°', swipeUp: ':', flex: 1.0),
      KeyModel(primary: 'b', topLeft: ';', topRight: ':', bottomLeft: '§', bottomRight: '¶', swipeUp: ';', flex: 1.0),
      KeyModel(primary: 'n', topLeft: '!', topRight: '?', bottomLeft: '¿', bottomRight: '¡', swipeUp: '!', flex: 1.0),
    ],
    // Fourth row (u i j k m o): u i j k m o
    const [
      KeyModel(primary: 'u', topLeft: '7', topRight: '&', bottomLeft: '-', bottomRight: '_', swipeUp: '7', flex: 1.0),
      KeyModel(primary: 'i', topLeft: '8', topRight: '*', bottomLeft: '/', bottomRight: '*', swipeUp: '8', flex: 1.0),
      KeyModel(primary: 'j', topLeft: '+', topRight: '=', bottomLeft: '"', bottomRight: "'", swipeUp: '+', flex: 1.0),
      KeyModel(primary: 'k', topLeft: '(', topRight: ')', bottomLeft: '<', bottomRight: '>', swipeUp: '(', flex: 1.0),
      KeyModel(primary: 'm', topLeft: '?', topRight: '!', bottomLeft: '«', bottomRight: '»', swipeUp: '?', flex: 1.0),
      KeyModel(primary: 'o', topLeft: '9', topRight: '(', bottomLeft: '[', bottomRight: ']', swipeUp: '9', flex: 1.0),
    ],
    // Fifth row: , . Space p l ⌫ ↵
    const [
      KeyModel(primary: ',', topLeft: '<', topRight: ';', bottomLeft: '[', bottomRight: '{', flex: 1.0),
      KeyModel(primary: '.', topLeft: ':', topRight: '>', bottomLeft: '}', bottomRight: ']', flex: 1.0),
      KeyModel(primary: ' ', actionType: KeyActionType.space, flex: 2.0),
      KeyModel(primary: 'p', topLeft: '0', topRight: ')', bottomLeft: '{', bottomRight: '}', swipeUp: '0', flex: 1.0),
      KeyModel(primary: 'l', topLeft: ')', topRight: ':', bottomLeft: '[', bottomRight: ']', swipeUp: ')', flex: 1.0),
      KeyModel(primary: '⌫', actionType: KeyActionType.backspace, flex: 1.0, androidKeyCode: 67),
      KeyModel(primary: '↵', actionType: KeyActionType.enter, flex: 1.0, androidKeyCode: 66),
    ],
  ];

  /// VERSION 2 DEFAULT ALIGNMENT:
  /// Row 0: Action & Symbol Row directly above Number Row (⇧, ?123, V2, ?, /, ,, ., Space, ⌫, ↵)
  /// Row 1: Number Row (1..0)
  /// Rows 2..5: 5-letter sequential rows (a..e, f..j, k..o, p..t)
  /// Row 6 (Bottom Row): Only letters — u, v, w, x (flex 1.0) + compact y, z (flex 0.55)
  static List<List<KeyModel>> get version2Layout => [
    // Row 0 (Above Number Row): ⇧ ?123 V2 ? / , . Space ⌫ ↵
    actionSymbolRow,
    // Row 1 (Number Row): 1 2 3 4 5 6 7 8 9 0
    numberRow,
    // Letter Row 1 (5 keys, a to e, equal width flex 1.0): a b c d e
    const [
      KeyModel(primary: 'a', topLeft: '1', topRight: '`', bottomLeft: '~', bottomRight: '\\', swipeUp: '1', flex: 1.0),
      KeyModel(primary: 'b', topLeft: '2', topRight: '|', bottomLeft: '<', bottomRight: '>', swipeUp: '2', flex: 1.0),
      KeyModel(primary: 'c', topLeft: '3', topRight: '€', bottomLeft: '{', bottomRight: '}', swipeUp: '3', flex: 1.0),
      KeyModel(primary: 'd', topLeft: '4', topRight: '₹', bottomLeft: '[', bottomRight: ']', swipeUp: '4', flex: 1.0),
      KeyModel(primary: 'e', topLeft: '5', topRight: '%', bottomLeft: '(', bottomRight: ')', swipeUp: '5', flex: 1.0),
    ],
    // Letter Row 2 (5 keys, f to j, equal width flex 1.0): f g h i j
    const [
      KeyModel(primary: 'f', topLeft: '6', topRight: '^', bottomLeft: '=', bottomRight: '+', swipeUp: '6', flex: 1.0),
      KeyModel(primary: 'g', topLeft: '@', topRight: '!', bottomLeft: '#', bottomRight: '\$', swipeUp: '@', flex: 1.0),
      KeyModel(primary: 'h', topLeft: '#', topRight: '\$', bottomLeft: '%', bottomRight: '^', swipeUp: '#', flex: 1.0),
      KeyModel(primary: 'i', topLeft: '\$', topRight: '%', bottomLeft: '&', bottomRight: '*', swipeUp: '\$', flex: 1.0),
      KeyModel(primary: 'j', topLeft: '%', topRight: '^', bottomLeft: '_', bottomRight: '+', swipeUp: '%', flex: 1.0),
    ],
    // Letter Row 3 (5 keys, k to o, equal width flex 1.0): k l m n o
    const [
      KeyModel(primary: 'k', topLeft: '&', topRight: '*', bottomLeft: '-', bottomRight: '=', swipeUp: '&', flex: 1.0),
      KeyModel(primary: 'l', topLeft: '-', topRight: '_', bottomLeft: ':', bottomRight: ';', swipeUp: '-', flex: 1.0),
      KeyModel(primary: 'm', topLeft: '*', topRight: '/', bottomLeft: '\\', bottomRight: '|', swipeUp: '*', flex: 1.0),
      KeyModel(primary: 'n', topLeft: '"', topRight: "'", bottomLeft: '`', bottomRight: '~', swipeUp: '"', flex: 1.0),
      KeyModel(primary: 'o', topLeft: "'", topRight: '`', bottomLeft: '©', bottomRight: '¢', swipeUp: "'", flex: 1.0),
    ],
    // Letter Row 4 (5 keys, p to t, equal width flex 1.0): p q r s t
    const [
      KeyModel(primary: 'p', topLeft: ':', topRight: ';', bottomLeft: '•', bottomRight: '°', swipeUp: ':', flex: 1.0),
      KeyModel(primary: 'q', topLeft: ';', topRight: ':', bottomLeft: '§', bottomRight: '¶', swipeUp: ';', flex: 1.0),
      KeyModel(primary: 'r', topLeft: '!', topRight: '?', bottomLeft: '¿', bottomRight: '¡', swipeUp: '!', flex: 1.0),
      KeyModel(primary: 's', topLeft: '7', topRight: '&', bottomLeft: '-', bottomRight: '_', swipeUp: '7', flex: 1.0),
      KeyModel(primary: 't', topLeft: '8', topRight: '*', bottomLeft: '/', bottomRight: '*', swipeUp: '8', flex: 1.0),
    ],
    // Letter Row 5 (Bottom Row — ONLY letters: u, v, w, x full size flex 1.0 + compact y, z flex 0.55)
    const [
      KeyModel(primary: 'u', topLeft: '+', topRight: '=', bottomLeft: '"', bottomRight: "'", swipeUp: '+', flex: 1.0),
      KeyModel(primary: 'v', topLeft: '(', topRight: ')', bottomLeft: '<', bottomRight: '>', swipeUp: '(', flex: 1.0),
      KeyModel(primary: 'w', topLeft: '?', topRight: '!', bottomLeft: '«', bottomRight: '»', swipeUp: '?', flex: 1.0),
      KeyModel(primary: 'x', topLeft: '9', topRight: '(', bottomLeft: '[', bottomRight: ']', swipeUp: '9', flex: 1.0),
      KeyModel(primary: 'y', topLeft: '0', topRight: ')', bottomLeft: '{', bottomRight: '}', swipeUp: '0', flex: 0.55),
      KeyModel(primary: 'z', topLeft: ')', topRight: ':', bottomLeft: '[', bottomRight: ']', swipeUp: ')', flex: 0.55),
    ],
  ];

  /// RIGHT HAND BANGLISH LAYOUT (Number Row below ESC/CTRL + 9x5 Ergonomic Grid from Official Final Rank Table & Spreadsheet):
  /// Row 0 (Directly below ESC/CTRL): Number Row [ 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 0 ]
  /// Row 1 (Sheet Row 4): [ #55/#54 (q/G) | #53/#52 (-/H) | #112 (⇧) | #111 (⌫) | #17 (c) ]
  /// Row 2 (Sheet Row 5): [ #50/#51 (V/R) | #49/#50 (C/V) | #31/#32 (↵/E) | #14 (m) | #10 (s) ]
  /// Row 3 (Sheet Row 6): [ #47/#48 (N/') | #28 (?) | #13 (d) | #9 (h) | #3 (e) ]
  /// Row 4 (Sheet Row 7): [ #41/#42 (D/z) | #24 (f) | #11 (l) | #2 (a) | #5 (i) ]
  /// Row 5 (Sheet Row 8): [ #39/#40 (।/M) | #21 (y) | #6 (t) | #1 (Space) | #7 (r) ]
  /// Row 6 (Sheet Row 9): [ #56/#59 (J/F — in 0/1's gap!) | #18 (.) | #12 (k) | #4 (o) | #8 (n) ]
  /// Row 7 (Sheet Row 10): [ #37/#38 (K/!) | #25 (w) | #16 (b) | #15 (u) | #19 (p) ]
  /// Row 8 (Sheet Row 11): [ #43/#44 (P/x) | #30 (S) | #22 (j) | #20 (g) | #23 (v) ]
  /// Row 9 (Sheet Row 12): [ #60/#46 (L/O — L in 2's gap!) | #33/#34 (I/B) | #29 (T) | #27 (A) | #26 (,) ]
  static List<List<KeyModel>> get rightHandBanglishLayout => [
    // Row 0 (Directly below ESC CTRL bar): Dedicated Number Row (1..9, 0)
    numberRow,
    // Row 1 (Sheet Row 4): [ 55/54 (q/G) | 53/52 (-/H) | 112 (⇧) | 111 (⌫) | 17 (c) ]
    const [
      KeyModel(primary: 'q', topLeft: '1', topRight: '`', bottomLeft: '~', bottomRight: '\\', swipeUp: '1', flex: 0.44),
      KeyModel(primary: 'G', topLeft: 'g', flex: 0.44),
      KeyModel(primary: '-', topLeft: '_', topRight: '+', bottomLeft: '=', flex: 0.515),
      KeyModel(primary: 'H', topLeft: 'h', flex: 0.515),
      KeyModel(primary: '⇧', actionType: KeyActionType.shift, flex: 1.03),
      KeyModel(primary: '⌫', actionType: KeyActionType.backspace, flex: 1.03, androidKeyCode: 67),
      KeyModel(primary: 'c', topLeft: "'", topRight: '`', bottomLeft: '©', bottomRight: '¢', swipeUp: "'", flex: 1.03),
    ],
    // Row 2 (Sheet Row 5): [ 50/51 (V/R) | 49/50 (C/V) | 31/32 (↵/E) | 14 (m) | 10 (s) ]
    const [
      KeyModel(primary: 'V', topLeft: 'v', flex: 0.44),
      KeyModel(primary: 'R', topLeft: 'r', flex: 0.44),
      KeyModel(primary: 'C', topLeft: 'c', flex: 0.515),
      KeyModel(primary: 'V', topLeft: 'v', flex: 0.515),
      KeyModel(primary: '↵', actionType: KeyActionType.enter, flex: 0.515, androidKeyCode: 66),
      KeyModel(primary: 'E', topLeft: '3', flex: 0.515),
      KeyModel(primary: 'm', topLeft: '?', topRight: '!', bottomLeft: '«', bottomRight: '»', swipeUp: '?', flex: 1.03),
      KeyModel(primary: 's', topLeft: '#', topRight: '\$', bottomLeft: '%', bottomRight: '^', swipeUp: '#', flex: 1.03),
    ],
    // Row 3 (Sheet Row 6): [ 47/48 (N/') | 28 (?) | 13 (d) | 9 (h) | 3 (e) ]
    const [
      KeyModel(primary: 'N', topLeft: 'n', flex: 0.44),
      KeyModel(primary: "'", topLeft: '"', topRight: '`', bottomLeft: '~', flex: 0.44),
      KeyModel(primary: '?', topLeft: '!', topRight: '¿', bottomLeft: '¡', bottomRight: '@', swipeUp: '!', flex: 1.03),
      KeyModel(primary: 'd', topLeft: '\$', topRight: '%', bottomLeft: '&', bottomRight: '*', swipeUp: '\$', flex: 1.03),
      KeyModel(primary: 'h', topLeft: '-', topRight: '_', bottomLeft: ':', bottomRight: ';', swipeUp: '-', flex: 1.03),
      KeyModel(primary: 'e', topLeft: '3', topRight: '€', bottomLeft: '{', bottomRight: '}', swipeUp: '3', flex: 1.03),
    ],
    // Row 4 (Sheet Row 7): [ 41/42 (D/z) | 24 (f) | 11 (l) | 2 (a) | 5 (i) ]
    const [
      KeyModel(primary: 'D', topLeft: 'd', flex: 0.44),
      KeyModel(primary: 'z', topLeft: '*', topRight: '/', bottomLeft: '\\', bottomRight: '|', swipeUp: '*', flex: 0.44),
      KeyModel(primary: 'f', topLeft: '%', topRight: '^', bottomLeft: '_', bottomRight: '+', swipeUp: '%', flex: 1.03),
      KeyModel(primary: 'l', topLeft: ')', topRight: ':', bottomLeft: '[', bottomRight: ']', swipeUp: ')', flex: 1.03),
      KeyModel(primary: 'a', topLeft: '@', topRight: '!', bottomLeft: '#', bottomRight: '\$', swipeUp: '@', flex: 1.03),
      KeyModel(primary: 'i', topLeft: '8', topRight: '*', bottomLeft: '/', bottomRight: '*', swipeUp: '8', flex: 1.03),
    ],
    // Row 5 (Sheet Row 8): [ 39/40 (।/M) | 21 (y) | 6 (t) | 1 (Space) | 7 (r) ]
    const [
      KeyModel(primary: '।', topLeft: '.', flex: 0.44),
      KeyModel(primary: 'M', topLeft: 'm', flex: 0.44),
      KeyModel(primary: 'y', topLeft: '6', topRight: '^', bottomLeft: '=', bottomRight: '+', swipeUp: '6', flex: 1.03),
      KeyModel(primary: 't', topLeft: '5', topRight: '%', bottomLeft: '(', bottomRight: ')', swipeUp: '5', flex: 1.03),
      KeyModel(primary: ' ', actionType: KeyActionType.space, flex: 1.03),
      KeyModel(primary: 'r', topLeft: '4', topRight: '₹', bottomLeft: '[', bottomRight: ']', swipeUp: '4', flex: 1.03),
    ],
    // Row 6 (Sheet Row 9): [ 56/59 (J/F — in 35/36 0/1's gap!) | 18 (.) | 12 (k) | 4 (o) | 8 (n) ]
    const [
      KeyModel(primary: 'J', topLeft: 'j', flex: 0.44),
      KeyModel(primary: 'F', topLeft: 'f', flex: 0.44),
      KeyModel(primary: '.', topLeft: '?', topRight: '!', bottomLeft: ',', bottomRight: "'", swipeUp: '?', flex: 1.03),
      KeyModel(primary: 'k', topLeft: '(', topRight: ')', bottomLeft: '<', bottomRight: '>', swipeUp: '(', flex: 1.03),
      KeyModel(primary: 'o', topLeft: '9', topRight: '(', bottomLeft: '[', bottomRight: ']', swipeUp: '9', flex: 1.03),
      KeyModel(primary: 'n', topLeft: '!', topRight: '?', bottomLeft: '¿', bottomRight: '¡', swipeUp: '!', flex: 1.03),
    ],
    // Row 7 (Sheet Row 10): [ 37/38 (K/!) | 25 (w) | 16 (b) | 15 (u) | 19 (p) ]
    const [
      KeyModel(primary: 'K', topLeft: 'k', flex: 0.44),
      KeyModel(primary: '!', topLeft: '?', flex: 0.44),
      KeyModel(primary: 'w', topLeft: '2', topRight: '|', bottomLeft: '<', bottomRight: '>', swipeUp: '2', flex: 1.03),
      KeyModel(primary: 'b', topLeft: ';', topRight: ':', bottomLeft: '§', bottomRight: '¶', swipeUp: ';', flex: 1.03),
      KeyModel(primary: 'u', topLeft: '7', topRight: '&', bottomLeft: '-', bottomRight: '_', swipeUp: '7', flex: 1.03),
      KeyModel(primary: 'p', topLeft: '0', topRight: ')', bottomLeft: '{', bottomRight: '}', swipeUp: '0', flex: 1.03),
    ],
    // Row 8 (Sheet Row 11): [ 43/44 (P/x) | 30 (S) | 22 (j) | 20 (g) | 23 (v) ]
    const [
      KeyModel(primary: 'P', topLeft: 'p', flex: 0.44),
      KeyModel(primary: 'x', topLeft: '"', topRight: "'", bottomLeft: '`', bottomRight: '~', swipeUp: '"', flex: 0.44),
      KeyModel(primary: 'S', topLeft: 's', flex: 1.03),
      KeyModel(primary: 'j', topLeft: '+', topRight: '=', bottomLeft: '"', bottomRight: "'", swipeUp: '+', flex: 1.03),
      KeyModel(primary: 'g', topLeft: '&', topRight: '*', bottomLeft: '-', bottomRight: '=', swipeUp: '&', flex: 1.03),
      KeyModel(primary: 'v', topLeft: ':', topRight: ';', bottomLeft: '•', bottomRight: '°', swipeUp: ':', flex: 1.03),
    ],
    // Row 9 (Sheet Row 12): [ 60/46 (L/O — L in 45 2's gap!) | 33/34 (I/B) | 29 (T) | 27 (A) | 26 (,) ]
    const [
      KeyModel(primary: 'L', topLeft: 'l', flex: 0.44),
      KeyModel(primary: 'O', topLeft: 'o', flex: 0.44),
      KeyModel(primary: 'I', topLeft: 'i', flex: 0.515),
      KeyModel(primary: 'B', topLeft: 'b', flex: 0.515),
      KeyModel(primary: 'T', topLeft: 't', flex: 1.03),
      KeyModel(primary: 'A', topLeft: 'a', flex: 1.03),
      KeyModel(primary: ',', topLeft: '<', topRight: ';', bottomLeft: "'", bottomRight: '"', swipeUp: '<', flex: 1.03),
    ],
  ];

  /// Default layout is Version 2 (as requested: "Maane version 2 is default")
  static List<List<KeyModel>> get qwertyLayout => version2Layout;

  static List<List<KeyModel>> get symbolLayout => [
    // Row 0 (Above Number Row): ⇧ ABC ? / , . Space ⌫ ↵
    symbolActionRow,
    // Row 1 (Number Row): 1 2 3 4 5 6 7 8 9 0
    numberRow,
    const [
      KeyModel(primary: '!', topLeft: '~', topRight: '`', flex: 1.0),
      KeyModel(primary: '@', topLeft: '#', topRight: '\$', flex: 1.0),
      KeyModel(primary: '#', topLeft: '\$', topRight: '€', flex: 1.0),
      KeyModel(primary: '\$', topLeft: '€', topRight: '₹', flex: 1.0),
      KeyModel(primary: '%', topLeft: '‰', topRight: '^', flex: 1.0),
    ],
    const [
      KeyModel(primary: '^', topLeft: '&', topRight: '*', flex: 1.0),
      KeyModel(primary: '&', topLeft: '*', topRight: '(', flex: 1.0),
      KeyModel(primary: '*', topLeft: '(', topRight: ')', flex: 1.0),
      KeyModel(primary: '(', topLeft: '[', topRight: '{', flex: 1.0),
      KeyModel(primary: ')', topLeft: ']', topRight: '}', flex: 1.0),
    ],
    const [
      KeyModel(primary: '+', topLeft: '±', topRight: '×', flex: 1.0),
      KeyModel(primary: '-', topLeft: '÷', topRight: '—', flex: 1.0),
      KeyModel(primary: '=', topLeft: '≠', topRight: '≈', flex: 1.0),
      KeyModel(primary: '_', topLeft: '—', topRight: '-', flex: 1.0),
      KeyModel(primary: '|', topLeft: '¦', topRight: '§', flex: 1.0),
    ],
    const [
      KeyModel(primary: '\\', topLeft: '/', topRight: '|', flex: 1.0),
      KeyModel(primary: '~', topLeft: '`', topRight: '^', flex: 1.0),
      KeyModel(primary: '`', topLeft: '"', topRight: "'", flex: 1.0),
      KeyModel(primary: '{', topLeft: '[', topRight: '(', flex: 1.0),
      KeyModel(primary: '}', topLeft: ']', topRight: ')', flex: 1.0),
    ],
    const [
      KeyModel(primary: '[', topLeft: '{', topRight: '(', flex: 1.0),
      KeyModel(primary: ']', topLeft: '}', topRight: ')', flex: 1.0),
      KeyModel(primary: '<', topLeft: '≤', topRight: '«', flex: 1.0),
      KeyModel(primary: '>', topLeft: '≥', topRight: '»', flex: 1.0),
      KeyModel(primary: ';', topLeft: ':', topRight: ',', flex: 0.55),
      KeyModel(primary: ':', topLeft: ';', topRight: '.', flex: 0.55),
    ],
  ];
}
