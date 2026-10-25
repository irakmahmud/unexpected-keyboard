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

  static int getBanglishRank(KeyModel key) {
    switch (key.actionType) {
      case KeyActionType.space:
        return 1;
      case KeyActionType.backspace:
        return 3;
      case KeyActionType.shift:
        return 19;
      case KeyActionType.enter:
        return 21;
      case KeyActionType.inputChar:
        switch (key.primary.toLowerCase()) {
          case 'a': return 2;
          case 'e': return 4;
          case 'r': return 5;
          case 'o': return 6;
          case 'i': return 7;
          case 'h': return 8;
          case 'k': return 9;
          case 'n': return 10;
          case 't': return 11;
          case 's': return 12;
          case 'b': return 13;
          case 'm': return 14;
          case 'l': return 15;
          case 'd': return 16;
          case 'u': return 17;
          case 'c': return 18;
          case 'p': return 20;
          case 'g': return 22;
          case 'j': return 23;
          case 'y': return 24;
          case '.': return 25;
          case ',': return 26;
          case 'v': return 27;
          case '?': return 28;
          case 'f': return 29;
          case "'": return 30;
          case '!': return 31;
          case 'w': return 32;
          case '1': return 33;
          case '2': return 34;
          case 'z': return 35;
          case '0': return 36;
          case '-': return 37;
          case 'x': return 38;
          case '"': return 39;
          case '3': return 40;
          case '4': return 41;
          case '5': return 42;
          case 'q': return 43;
          case ')': return 44;
          case '(': return 45;
          case ':': return 46;
          case '6': return 47;
          case '7': return 48;
          case '/': return 49;
          case '8': return 50;
          case '9': return 51;
          case ';': return 52;
          case '@': return 53;
          case '&': return 54;
          case '%': return 55;
          default: return 0;
        }
      default:
        return 0;
    }
  }

  static int? getResearchColorValue(KeyModel key) {
    final rank = getBanglishRank(key);
    if (rank <= 0) return null;
    return researchColorPalette[(rank - 1) % researchColorPalette.length];
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

  /// RIGHT HAND BANGLISH LAYOUT (9x5 Ergonomic Grid based on 55 Banglish Mobile Keys Rank #1..#55):
  static List<List<KeyModel>> get rightHandBanglishLayout => const [
    // Row 1: [ #55 (%) | #54 (&) | #53 (@) | #52 (;) | #17 (u) ]
    [
      KeyModel(primary: '%', flex: 1.0),
      KeyModel(primary: '&', flex: 1.0),
      KeyModel(primary: '@', flex: 1.0),
      KeyModel(primary: ';', flex: 1.0),
      KeyModel(primary: 'u', topLeft: '7', topRight: '&', bottomLeft: '-', bottomRight: '_', swipeUp: '7', flex: 1.0),
    ],
    // Row 2: [ 50/51 (8/9) | 49 (/) | 31/32 (!/w) | 14 (m) | 10 (n) ]
    [
      KeyModel(primary: '8', topLeft: '9', flex: 0.5),
      KeyModel(primary: '9', flex: 0.5),
      KeyModel(primary: '/', topLeft: '?', topRight: '\\', bottomLeft: '|', flex: 1.0),
      KeyModel(primary: '!', topLeft: 'w', flex: 0.5),
      KeyModel(primary: 'w', topLeft: '!', flex: 0.5),
      KeyModel(primary: 'm', topLeft: '?', topRight: '!', bottomLeft: '«', bottomRight: '»', swipeUp: '?', flex: 1.0),
      KeyModel(primary: 'n', topLeft: '!', topRight: '?', bottomLeft: '¿', bottomRight: '¡', swipeUp: '!', flex: 1.0),
    ],
    // Row 3: [ 47/48 (6/7) | 28 (?) | 13 (b) | 9 (k) | 5 (r moved up!) ]
    [
      KeyModel(primary: '6', topLeft: '7', flex: 0.5),
      KeyModel(primary: '7', flex: 0.5),
      KeyModel(primary: '?', topLeft: '!', topRight: '¿', bottomLeft: '¡', bottomRight: '@', swipeUp: '!', flex: 1.0),
      KeyModel(primary: 'b', topLeft: ';', topRight: ':', bottomLeft: '§', bottomRight: '¶', swipeUp: ';', flex: 1.0),
      KeyModel(primary: 'k', topLeft: '(', topRight: ')', bottomLeft: '<', bottomRight: '>', swipeUp: '(', flex: 1.0),
      KeyModel(primary: 'r', topLeft: '4', topRight: '₹', bottomLeft: '[', bottomRight: ']', swipeUp: '4', flex: 1.0),
    ],
    // Row 4: [ 41/42 (4/5) | 24 (y) | 11 (t) | 2 (a) | 7 (i moved up!) ]
    [
      KeyModel(primary: '4', topLeft: '5', flex: 0.5),
      KeyModel(primary: '5', flex: 0.5),
      KeyModel(primary: 'y', topLeft: '6', topRight: '^', bottomLeft: '=', bottomRight: '+', swipeUp: '6', flex: 1.0),
      KeyModel(primary: 't', topLeft: '5', topRight: '%', bottomLeft: '(', bottomRight: ')', swipeUp: '5', flex: 1.0),
      KeyModel(primary: 'a', topLeft: '@', topRight: '!', bottomLeft: '#', bottomRight: '\$', swipeUp: '@', flex: 1.0),
      KeyModel(primary: 'i', topLeft: '8', topRight: '*', bottomLeft: '/', bottomRight: '*', swipeUp: '8', flex: 1.0),
    ],
    // Row 5: [ 34/40 (2/3) | 21 (↵) | 19 (⇧ in o's place!) | 1 (Space — wider!) | 3 (⌫ Cross right of Space!) ]
    [
      KeyModel(primary: '2', topLeft: '3', flex: 0.44),
      KeyModel(primary: '3', flex: 0.44),
      KeyModel(primary: '↵', actionType: KeyActionType.enter, flex: 0.88, androidKeyCode: 66),
      KeyModel(primary: '⇧', actionType: KeyActionType.shift, flex: 0.88),
      KeyModel(primary: ' ', actionType: KeyActionType.space, flex: 1.48),
      KeyModel(primary: '⌫', actionType: KeyActionType.backspace, flex: 0.88, androidKeyCode: 67),
    ],
    // Row 6: [ 36/33 (0/1) | 18 (c) | 12 (s) | 4 (e) | 8 (h) ]
    [
      KeyModel(primary: '0', topLeft: '1', flex: 0.5),
      KeyModel(primary: '1', flex: 0.5),
      KeyModel(primary: 'c', topLeft: "'", topRight: '`', bottomLeft: '©', bottomRight: '¢', swipeUp: "'", flex: 1.0),
      KeyModel(primary: 's', topLeft: '#', topRight: '\$', bottomLeft: '%', bottomRight: '^', swipeUp: '#', flex: 1.0),
      KeyModel(primary: 'e', topLeft: '3', topRight: '€', bottomLeft: '{', bottomRight: '}', swipeUp: '3', flex: 1.0),
      KeyModel(primary: 'h', topLeft: '-', topRight: '_', bottomLeft: ':', bottomRight: ';', swipeUp: '-', flex: 1.0),
    ],
    // Row 7: [ 37/38 (-/x) | 25 (.) | 16 (d) | 15 (l) | 6 (o below h!) ]
    [
      KeyModel(primary: '-', topLeft: 'x', flex: 0.5),
      KeyModel(primary: 'x', topLeft: '-', flex: 0.5),
      KeyModel(primary: '.', topLeft: '>', topRight: ':', bottomLeft: '(', bottomRight: ')', swipeUp: '>', flex: 1.0),
      KeyModel(primary: 'd', topLeft: '\$', topRight: '%', bottomLeft: '&', bottomRight: '*', swipeUp: '\$', flex: 1.0),
      KeyModel(primary: 'l', topLeft: ')', topRight: ':', bottomLeft: '[', bottomRight: ']', swipeUp: ')', flex: 1.0),
      KeyModel(primary: 'o', topLeft: '9', topRight: '(', bottomLeft: '[', bottomRight: ']', swipeUp: '9', flex: 1.0),
    ],
    // Row 8: [ 43/44 (q/)) | 30 (') | 22 (g) | 20 (p) | 23 (j) ]
    [
      KeyModel(primary: 'q', topLeft: ')', flex: 0.5),
      KeyModel(primary: ')', flex: 0.5),
      KeyModel(primary: "'", topLeft: '"', topRight: '`', bottomLeft: '~', flex: 1.0),
      KeyModel(primary: 'g', topLeft: '&', topRight: '*', bottomLeft: '-', bottomRight: '=', swipeUp: '&', flex: 1.0),
      KeyModel(primary: 'p', topLeft: '0', topRight: ')', bottomLeft: '{', bottomRight: '}', swipeUp: '0', flex: 1.0),
      KeyModel(primary: 'j', topLeft: '+', topRight: '=', bottomLeft: '"', bottomRight: "'", swipeUp: '+', flex: 1.0),
    ],
    // Row 9: [ 45/46 ((/:) | 35/39 (z/") | 29 (f) | 27 (v) | 26 (,) ]
    [
      KeyModel(primary: '(', topLeft: ':', flex: 0.5),
      KeyModel(primary: ':', flex: 0.5),
      KeyModel(primary: 'z', topLeft: '"', flex: 0.5),
      KeyModel(primary: '"', flex: 0.5),
      KeyModel(primary: 'f', topLeft: '%', topRight: '^', bottomLeft: '_', bottomRight: '+', swipeUp: '%', flex: 1.0),
      KeyModel(primary: 'v', topLeft: ':', topRight: ';', bottomLeft: '•', bottomRight: '°', swipeUp: ':', flex: 1.0),
      KeyModel(primary: ',', topLeft: '<', topRight: ';', bottomLeft: "'", bottomRight: '"', swipeUp: '<', flex: 1.0),
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
