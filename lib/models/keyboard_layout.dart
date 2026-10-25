import 'key_model.dart';

enum LayoutType {
  qwerty,
  symbols,
  functionKeys,
}

class KeyboardLayout {
  /// Top Terminal Bar: ESC, CTRL, ALT, and Arrow keys (Shift and ?123 are in the dedicated Action/Symbol Row directly above the Number Row)
  static List<KeyModel> get terminalBarKeys => const [
    KeyModel(primary: 'ESC', actionType: KeyActionType.escape, flex: 1.0, androidKeyCode: 111),
    KeyModel(primary: 'CTRL', actionType: KeyActionType.ctrl, flex: 1.1, androidKeyCode: 113),
    KeyModel(primary: 'ALT', actionType: KeyActionType.alt, flex: 1.0, androidKeyCode: 57),
    KeyModel(primary: '◀', actionType: KeyActionType.arrowLeft, flex: 0.85, androidKeyCode: 21),
    KeyModel(primary: '▼', actionType: KeyActionType.arrowDown, flex: 0.85, androidKeyCode: 20),
    KeyModel(primary: '▲', actionType: KeyActionType.arrowUp, flex: 0.85, androidKeyCode: 19),
    KeyModel(primary: '▶', actionType: KeyActionType.arrowRight, flex: 0.85, androidKeyCode: 22),
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

  /* ============================================================================
   * VERSION 1 SAVED ALIGNMENT (Tag: version-1 / v1.1.5)
   * Uncomment this block and comment out VERSION 2 below to revert to Version 1:
   *
   * static List<List<KeyModel>> get qwertyLayout => [
   *   // Row 0: 1 2 3 4 5 6 7 8 9 0
   *   numberRow,
   *   // First row (q to y): q w e r t y
   *   const [
   *     KeyModel(primary: 'q', topLeft: '1', topRight: '`', bottomLeft: '~', bottomRight: '\\', swipeUp: '1', flex: 1.0),
   *     KeyModel(primary: 'w', topLeft: '2', topRight: '|', bottomLeft: '<', bottomRight: '>', swipeUp: '2', flex: 1.0),
   *     KeyModel(primary: 'e', topLeft: '3', topRight: '€', bottomLeft: '{', bottomRight: '}', swipeUp: '3', flex: 1.0),
   *     KeyModel(primary: 'r', topLeft: '4', topRight: '₹', bottomLeft: '[', bottomRight: ']', swipeUp: '4', flex: 1.0),
   *     KeyModel(primary: 't', topLeft: '5', topRight: '%', bottomLeft: '(', bottomRight: ')', swipeUp: '5', flex: 1.0),
   *     KeyModel(primary: 'y', topLeft: '6', topRight: '^', bottomLeft: '=', bottomRight: '+', swipeUp: '6', flex: 1.0),
   *   ],
   *   // Second row (a to h): a s d f g h
   *   const [
   *     KeyModel(primary: 'a', topLeft: '@', topRight: '!', bottomLeft: '#', bottomRight: '\$', swipeUp: '@', flex: 1.0),
   *     KeyModel(primary: 's', topLeft: '#', topRight: '\$', bottomLeft: '%', bottomRight: '^', swipeUp: '#', flex: 1.0),
   *     KeyModel(primary: 'd', topLeft: '\$', topRight: '%', bottomLeft: '&', bottomRight: '*', swipeUp: '\$', flex: 1.0),
   *     KeyModel(primary: 'f', topLeft: '%', topRight: '^', bottomLeft: '_', bottomRight: '+', swipeUp: '%', flex: 1.0),
   *     KeyModel(primary: 'g', topLeft: '&', topRight: '*', bottomLeft: '-', bottomRight: '=', swipeUp: '&', flex: 1.0),
   *     KeyModel(primary: 'h', topLeft: '-', topRight: '_', bottomLeft: ':', bottomRight: ';', swipeUp: '-', flex: 1.0),
   *   ],
   *   // Third row (z to n): z x c v b n
   *   const [
   *     KeyModel(primary: 'z', topLeft: '*', topRight: '/', bottomLeft: '\\', bottomRight: '|', swipeUp: '*', flex: 1.0),
   *     KeyModel(primary: 'x', topLeft: '"', topRight: "'", bottomLeft: '`', bottomRight: '~', swipeUp: '"', flex: 1.0),
   *     KeyModel(primary: 'c', topLeft: "'", topRight: '`', bottomLeft: '©', bottomRight: '¢', swipeUp: "'", flex: 1.0),
   *     KeyModel(primary: 'v', topLeft: ':', topRight: ';', bottomLeft: '•', bottomRight: '°', swipeUp: ':', flex: 1.0),
   *     KeyModel(primary: 'b', topLeft: ';', topRight: ':', bottomLeft: '§', bottomRight: '¶', swipeUp: ';', flex: 1.0),
   *     KeyModel(primary: 'n', topLeft: '!', topRight: '?', bottomLeft: '¿', bottomRight: '¡', swipeUp: '!', flex: 1.0),
   *   ],
   *   // Fourth row (u i j k m o): u i j k m o
   *   const [
   *     KeyModel(primary: 'u', topLeft: '7', topRight: '&', bottomLeft: '-', bottomRight: '_', swipeUp: '7', flex: 1.0),
   *     KeyModel(primary: 'i', topLeft: '8', topRight: '*', bottomLeft: '/', bottomRight: '*', swipeUp: '8', flex: 1.0),
   *     KeyModel(primary: 'j', topLeft: '+', topRight: '=', bottomLeft: '"', bottomRight: "'", swipeUp: '+', flex: 1.0),
   *     KeyModel(primary: 'k', topLeft: '(', topRight: ')', bottomLeft: '<', bottomRight: '>', swipeUp: '(', flex: 1.0),
   *     KeyModel(primary: 'm', topLeft: '?', topRight: '!', bottomLeft: '«', bottomRight: '»', swipeUp: '?', flex: 1.0),
   *     KeyModel(primary: 'o', topLeft: '9', topRight: '(', bottomLeft: '[', bottomRight: ']', swipeUp: '9', flex: 1.0),
   *   ],
   *   // Fifth row: , . Space p l ⌫ ↵
   *   const [
   *     KeyModel(primary: ',', topLeft: '<', topRight: ';', bottomLeft: '[', bottomRight: '{', flex: 1.0),
   *     KeyModel(primary: '.', topLeft: ':', topRight: '>', bottomLeft: '}', bottomRight: ']', flex: 1.0),
   *     KeyModel(primary: ' ', actionType: KeyActionType.space, flex: 2.0),
   *     KeyModel(primary: 'p', topLeft: '0', topRight: ')', bottomLeft: '{', bottomRight: '}', swipeUp: '0', flex: 1.0),
   *     KeyModel(primary: 'l', topLeft: ')', topRight: ':', bottomLeft: '[', bottomRight: ']', swipeUp: ')', flex: 1.0),
   *     KeyModel(primary: '⌫', actionType: KeyActionType.backspace, flex: 1.0, androidKeyCode: 67),
   *     KeyModel(primary: '↵', actionType: KeyActionType.enter, flex: 1.0, androidKeyCode: 66),
   *   ],
   * ];
   * ============================================================================ */

  /// VERSION 2 ACTIVE ALIGNMENT:
  /// Row 0: Action & Symbol Row directly above Number Row (⇧, ?123, ?, /, ,, ., Space, ⌫, ↵)
  /// Row 1: Number Row (1..0)
  /// Rows 2..5: 5-letter sequential rows (a..e, f..j, k..o, p..t)
  /// Row 6 (Bottom Row): Only letters — u, v, w, x (flex 1.0) + compact y, z (flex 0.55)
  static List<List<KeyModel>> get qwertyLayout => [
    // Row 0 (Above Number Row): ⇧ ?123 ? / , . Space ⌫ ↵
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
