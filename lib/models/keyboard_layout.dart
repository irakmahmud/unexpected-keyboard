import 'key_model.dart';

enum LayoutType {
  qwerty,
  symbols,
  functionKeys,
}

class KeyboardLayout {
  /// Top bar: TAB and META hidden and replaced by Shift (⇧) and Layout Switch (?123)
  static List<KeyModel> get terminalBarKeys => const [
    KeyModel(primary: 'ESC', actionType: KeyActionType.escape, flex: 0.9, androidKeyCode: 111),
    KeyModel(primary: '⇧', actionType: KeyActionType.shift, flex: 1.0),
    KeyModel(primary: 'CTRL', actionType: KeyActionType.ctrl, flex: 1.0, androidKeyCode: 113),
    KeyModel(primary: 'ALT', actionType: KeyActionType.alt, flex: 1.0, androidKeyCode: 57),
    KeyModel(primary: '?123', actionType: KeyActionType.switchLayout, flex: 1.1),
    KeyModel(primary: '◀', actionType: KeyActionType.arrowLeft, flex: 0.8, androidKeyCode: 21),
    KeyModel(primary: '▼', actionType: KeyActionType.arrowDown, flex: 0.8, androidKeyCode: 20),
    KeyModel(primary: '▲', actionType: KeyActionType.arrowUp, flex: 0.8, androidKeyCode: 19),
    KeyModel(primary: '▶', actionType: KeyActionType.arrowRight, flex: 0.8, androidKeyCode: 22),
  ];

  /// Row 0: Dedicated Number Row always positioned directly below top modifier bar
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

  /// VERSION 2 ACTIVE ALIGNMENT: Sequential Alphabetical (a, b, c, d ... y, z)
  static List<List<KeyModel>> get qwertyLayout => [
    // Row 0 (Always Number Row right below top bar): 1 2 3 4 5 6 7 8 9 0
    numberRow,
    // First row (6 keys, a to f, equal width flex 1.0): a b c d e f
    const [
      KeyModel(primary: 'a', topLeft: '1', topRight: '`', bottomLeft: '~', bottomRight: '\\', swipeUp: '1', flex: 1.0),
      KeyModel(primary: 'b', topLeft: '2', topRight: '|', bottomLeft: '<', bottomRight: '>', swipeUp: '2', flex: 1.0),
      KeyModel(primary: 'c', topLeft: '3', topRight: '€', bottomLeft: '{', bottomRight: '}', swipeUp: '3', flex: 1.0),
      KeyModel(primary: 'd', topLeft: '4', topRight: '₹', bottomLeft: '[', bottomRight: ']', swipeUp: '4', flex: 1.0),
      KeyModel(primary: 'e', topLeft: '5', topRight: '%', bottomLeft: '(', bottomRight: ')', swipeUp: '5', flex: 1.0),
      KeyModel(primary: 'f', topLeft: '6', topRight: '^', bottomLeft: '=', bottomRight: '+', swipeUp: '6', flex: 1.0),
    ],
    // Second row (6 keys, g to l, equal width flex 1.0): g h i j k l
    const [
      KeyModel(primary: 'g', topLeft: '@', topRight: '!', bottomLeft: '#', bottomRight: '\$', swipeUp: '@', flex: 1.0),
      KeyModel(primary: 'h', topLeft: '#', topRight: '\$', bottomLeft: '%', bottomRight: '^', swipeUp: '#', flex: 1.0),
      KeyModel(primary: 'i', topLeft: '\$', topRight: '%', bottomLeft: '&', bottomRight: '*', swipeUp: '\$', flex: 1.0),
      KeyModel(primary: 'j', topLeft: '%', topRight: '^', bottomLeft: '_', bottomRight: '+', swipeUp: '%', flex: 1.0),
      KeyModel(primary: 'k', topLeft: '&', topRight: '*', bottomLeft: '-', bottomRight: '=', swipeUp: '&', flex: 1.0),
      KeyModel(primary: 'l', topLeft: '-', topRight: '_', bottomLeft: ':', bottomRight: ';', swipeUp: '-', flex: 1.0),
    ],
    // Third row (6 keys, m to r, equal width flex 1.0): m n o p q r
    const [
      KeyModel(primary: 'm', topLeft: '*', topRight: '/', bottomLeft: '\\', bottomRight: '|', swipeUp: '*', flex: 1.0),
      KeyModel(primary: 'n', topLeft: '"', topRight: "'", bottomLeft: '`', bottomRight: '~', swipeUp: '"', flex: 1.0),
      KeyModel(primary: 'o', topLeft: "'", topRight: '`', bottomLeft: '©', bottomRight: '¢', swipeUp: "'", flex: 1.0),
      KeyModel(primary: 'p', topLeft: ':', topRight: ';', bottomLeft: '•', bottomRight: '°', swipeUp: ':', flex: 1.0),
      KeyModel(primary: 'q', topLeft: ';', topRight: ':', bottomLeft: '§', bottomRight: '¶', swipeUp: ';', flex: 1.0),
      KeyModel(primary: 'r', topLeft: '!', topRight: '?', bottomLeft: '¿', bottomRight: '¡', swipeUp: '!', flex: 1.0),
    ],
    // Fourth row (6 keys, s to x, equal width flex 1.0): s t u v w x
    const [
      KeyModel(primary: 's', topLeft: '7', topRight: '&', bottomLeft: '-', bottomRight: '_', swipeUp: '7', flex: 1.0),
      KeyModel(primary: 't', topLeft: '8', topRight: '*', bottomLeft: '/', bottomRight: '*', swipeUp: '8', flex: 1.0),
      KeyModel(primary: 'u', topLeft: '+', topRight: '=', bottomLeft: '"', bottomRight: "'", swipeUp: '+', flex: 1.0),
      KeyModel(primary: 'v', topLeft: '(', topRight: ')', bottomLeft: '<', bottomRight: '>', swipeUp: '(', flex: 1.0),
      KeyModel(primary: 'w', topLeft: '?', topRight: '!', bottomLeft: '«', bottomRight: '»', swipeUp: '?', flex: 1.0),
      KeyModel(primary: 'x', topLeft: '9', topRight: '(', bottomLeft: '[', bottomRight: ']', swipeUp: '9', flex: 1.0),
    ],
    // Fifth row (Bottom row with last 2 letters y, z: comma, dot, space, y, z, backspace, enter)
    const [
      KeyModel(primary: ',', topLeft: '<', topRight: ';', bottomLeft: '[', bottomRight: '{', flex: 1.0),
      KeyModel(primary: '.', topLeft: ':', topRight: '>', bottomLeft: '}', bottomRight: ']', flex: 1.0),
      KeyModel(primary: ' ', actionType: KeyActionType.space, flex: 2.0),
      KeyModel(primary: 'y', topLeft: '0', topRight: ')', bottomLeft: '{', bottomRight: '}', swipeUp: '0', flex: 1.0),
      KeyModel(primary: 'z', topLeft: ')', topRight: ':', bottomLeft: '[', bottomRight: ']', swipeUp: ')', flex: 1.0),
      KeyModel(primary: '⌫', actionType: KeyActionType.backspace, flex: 1.0, androidKeyCode: 67),
      KeyModel(primary: '↵', actionType: KeyActionType.enter, flex: 1.0, androidKeyCode: 66),
    ],
  ];

  static List<List<KeyModel>> get symbolLayout => [
    // Row 0 (Always Number Row right below top bar): 1 2 3 4 5 6 7 8 9 0
    numberRow,
    const [
      KeyModel(primary: '!', topLeft: '~', topRight: '`', flex: 1.0),
      KeyModel(primary: '@', topLeft: '#', topRight: '\$', flex: 1.0),
      KeyModel(primary: '#', topLeft: '\$', topRight: '€', flex: 1.0),
      KeyModel(primary: '\$', topLeft: '€', topRight: '₹', flex: 1.0),
      KeyModel(primary: '%', topLeft: '‰', topRight: '^', flex: 1.0),
      KeyModel(primary: '^', topLeft: '&', topRight: '*', flex: 1.0),
    ],
    const [
      KeyModel(primary: '+', topLeft: '±', topRight: '×', flex: 1.0),
      KeyModel(primary: '-', topLeft: '÷', topRight: '—', flex: 1.0),
      KeyModel(primary: '=', topLeft: '≠', topRight: '≈', flex: 1.0),
      KeyModel(primary: '/', topLeft: '\\', topRight: '|', flex: 1.0),
      KeyModel(primary: '<', topLeft: '≤', topRight: '«', flex: 1.0),
      KeyModel(primary: '>', topLeft: '≥', topRight: '»', flex: 1.0),
    ],
    const [
      KeyModel(primary: '|', topLeft: '¦', topRight: '§', flex: 1.0),
      KeyModel(primary: '_', topLeft: '—', topRight: '-', flex: 1.0),
      KeyModel(primary: '\\', topLeft: '/', topRight: '|', flex: 1.0),
      KeyModel(primary: '~', topLeft: '`', topRight: '^', flex: 1.0),
      KeyModel(primary: '`', topLeft: '"', topRight: "'", flex: 1.0),
      KeyModel(primary: ':', topLeft: ';', topRight: '.', flex: 1.0),
    ],
    const [
      KeyModel(primary: '&', topLeft: '*', topRight: '(', flex: 1.0),
      KeyModel(primary: '*', topLeft: '(', topRight: ')', flex: 1.0),
      KeyModel(primary: '{', topLeft: '[', topRight: '(', flex: 1.0),
      KeyModel(primary: '}', topLeft: ']', topRight: ')', flex: 1.0),
      KeyModel(primary: ';', topLeft: ':', topRight: ',', flex: 1.0),
      KeyModel(primary: '"', topLeft: "'", topRight: '`', flex: 1.0),
    ],
    const [
      KeyModel(primary: 'ABC', actionType: KeyActionType.switchLayout, flex: 1.0),
      KeyModel(primary: ',', topLeft: '<', flex: 1.0),
      KeyModel(primary: '.', topLeft: '>', flex: 1.0),
      KeyModel(primary: ' ', actionType: KeyActionType.space, flex: 2.0),
      KeyModel(primary: 'ABC', actionType: KeyActionType.switchLayout, flex: 1.0),
      KeyModel(primary: '⌫', actionType: KeyActionType.backspace, flex: 1.0, androidKeyCode: 67),
      KeyModel(primary: '↵', actionType: KeyActionType.enter, flex: 1.0, androidKeyCode: 66),
    ],
  ];
}
