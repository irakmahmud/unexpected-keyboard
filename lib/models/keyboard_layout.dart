import 'key_model.dart';

enum LayoutType {
  qwerty,
  symbols,
  functionKeys,
}

class KeyboardLayout {
  static List<KeyModel> get terminalBarKeys => const [
    KeyModel(primary: 'ESC', actionType: KeyActionType.escape, flex: 0.9, androidKeyCode: 111),
    KeyModel(primary: 'TAB', actionType: KeyActionType.tab, flex: 0.9, androidKeyCode: 61),
    KeyModel(primary: 'CTRL', actionType: KeyActionType.ctrl, flex: 1.0, androidKeyCode: 113),
    KeyModel(primary: 'ALT', actionType: KeyActionType.alt, flex: 1.0, androidKeyCode: 57),
    KeyModel(primary: 'META', actionType: KeyActionType.meta, flex: 0.9, androidKeyCode: 117),
    KeyModel(primary: '◀', actionType: KeyActionType.arrowLeft, flex: 0.8, androidKeyCode: 21),
    KeyModel(primary: '▼', actionType: KeyActionType.arrowDown, flex: 0.8, androidKeyCode: 20),
    KeyModel(primary: '▲', actionType: KeyActionType.arrowUp, flex: 0.8, androidKeyCode: 19),
    KeyModel(primary: '▶', actionType: KeyActionType.arrowRight, flex: 0.8, androidKeyCode: 22),
  ];

  static List<List<KeyModel>> get qwertyLayout => const [
    // Row 1: q w e r t y u i o p
    [
      KeyModel(primary: 'q', topLeft: '1', topRight: '`', bottomLeft: '~', bottomRight: '\\', swipeUp: '1'),
      KeyModel(primary: 'w', topLeft: '2', topRight: '|', bottomLeft: '<', bottomRight: '>', swipeUp: '2'),
      KeyModel(primary: 'e', topLeft: '3', topRight: '€', bottomLeft: '{', bottomRight: '}', swipeUp: '3'),
      KeyModel(primary: 'r', topLeft: '4', topRight: '₹', bottomLeft: '[', bottomRight: ']', swipeUp: '4'),
      KeyModel(primary: 't', topLeft: '5', topRight: '%', bottomLeft: '(', bottomRight: ')', swipeUp: '5'),
      KeyModel(primary: 'y', topLeft: '6', topRight: '^', bottomLeft: '=', bottomRight: '+', swipeUp: '6'),
      KeyModel(primary: 'u', topLeft: '7', topRight: '&', bottomLeft: '-', bottomRight: '_', swipeUp: '7'),
      KeyModel(primary: 'i', topLeft: '8', topRight: '*', bottomLeft: '/', bottomRight: '*', swipeUp: '8'),
      KeyModel(primary: 'o', topLeft: '9', topRight: '(', bottomLeft: '[', bottomRight: ']', swipeUp: '9'),
      KeyModel(primary: 'p', topLeft: '0', topRight: ')', bottomLeft: '{', bottomRight: '}', swipeUp: '0'),
    ],
    // Row 2: a s d f g h j k l
    [
      KeyModel(primary: 'a', topLeft: '@', topRight: '!', bottomLeft: '#', bottomRight: '\$', swipeUp: '@'),
      KeyModel(primary: 's', topLeft: '#', topRight: '\$', bottomLeft: '%', bottomRight: '^', swipeUp: '#'),
      KeyModel(primary: 'd', topLeft: '\$', topRight: '%', bottomLeft: '&', bottomRight: '*', swipeUp: '\$'),
      KeyModel(primary: 'f', topLeft: '%', topRight: '^', bottomLeft: '_', bottomRight: '+', swipeUp: '%'),
      KeyModel(primary: 'g', topLeft: '&', topRight: '*', bottomLeft: '-', bottomRight: '=', swipeUp: '&'),
      KeyModel(primary: 'h', topLeft: '-', topRight: '_', bottomLeft: ':', bottomRight: ';', swipeUp: '-'),
      KeyModel(primary: 'j', topLeft: '+', topRight: '=', bottomLeft: '"', bottomRight: "'", swipeUp: '+'),
      KeyModel(primary: 'k', topLeft: '(', topRight: ')', bottomLeft: '<', bottomRight: '>', swipeUp: '('),
      KeyModel(primary: 'l', topLeft: ')', topRight: ':', bottomLeft: '[', bottomRight: ']', swipeUp: ')'),
    ],
    // Row 3: Shift, z x c v b n m, Backspace
    [
      KeyModel(primary: '⇧', actionType: KeyActionType.shift, flex: 1.3),
      KeyModel(primary: 'z', topLeft: '*', topRight: '/', bottomLeft: '\\', bottomRight: '|', swipeUp: '*'),
      KeyModel(primary: 'x', topLeft: '"', topRight: "'", bottomLeft: '`', bottomRight: '~', swipeUp: '"'),
      KeyModel(primary: 'c', topLeft: "'", topRight: '`', bottomLeft: '©', bottomRight: '¢', swipeUp: "'"),
      KeyModel(primary: 'v', topLeft: ':', topRight: ';', bottomLeft: '•', bottomRight: '°', swipeUp: ':'),
      KeyModel(primary: 'b', topLeft: ';', topRight: ':', bottomLeft: '§', bottomRight: '¶', swipeUp: ';'),
      KeyModel(primary: 'n', topLeft: '!', topRight: '?', bottomLeft: '¿', bottomRight: '¡', swipeUp: '!'),
      KeyModel(primary: 'm', topLeft: '?', topRight: '!', bottomLeft: '«', bottomRight: '»', swipeUp: '?'),
      KeyModel(primary: '⌫', actionType: KeyActionType.backspace, flex: 1.3, androidKeyCode: 67),
    ],
    // Row 4: ?123, comma, Space, dot, Enter
    [
      KeyModel(primary: '?123', actionType: KeyActionType.switchLayout, flex: 1.2),
      KeyModel(primary: ',', topLeft: ';', topRight: '<', bottomLeft: '{', bottomRight: '[', flex: 1.0),
      KeyModel(primary: ' ', actionType: KeyActionType.space, flex: 4.0),
      KeyModel(primary: '.', topLeft: ':', topRight: '>', bottomLeft: '}', bottomRight: ']', flex: 1.0),
      KeyModel(primary: '↵', actionType: KeyActionType.enter, flex: 1.5, androidKeyCode: 66),
    ],
  ];

  static List<List<KeyModel>> get symbolLayout => const [
    [
      KeyModel(primary: '1', topLeft: '~', topRight: '`'),
      KeyModel(primary: '2', topLeft: '@', topRight: '#'),
      KeyModel(primary: '3', topLeft: '#', topRight: '\$'),
      KeyModel(primary: '4', topLeft: '\$', topRight: '€'),
      KeyModel(primary: '5', topLeft: '%', topRight: '‰'),
      KeyModel(primary: '6', topLeft: '^', topRight: '&'),
      KeyModel(primary: '7', topLeft: '&', topRight: '*'),
      KeyModel(primary: '8', topLeft: '*', topRight: '+'),
      KeyModel(primary: '9', topLeft: '(', topRight: '['),
      KeyModel(primary: '0', topLeft: ')', topRight: ']'),
    ],
    [
      KeyModel(primary: '+', topLeft: '±', topRight: '×'),
      KeyModel(primary: '-', topLeft: '÷', topRight: '—'),
      KeyModel(primary: '*', topLeft: '°', topRight: '•'),
      KeyModel(primary: '/', topLeft: '\\', topRight: '|'),
      KeyModel(primary: '=', topLeft: '≠', topRight: '≈'),
      KeyModel(primary: '<', topLeft: '≤', topRight: '«'),
      KeyModel(primary: '>', topLeft: '≥', topRight: '»'),
      KeyModel(primary: '{', topLeft: '[', topRight: '('),
      KeyModel(primary: '}', topLeft: ']', topRight: ')'),
      KeyModel(primary: '|', topLeft: '¦', topRight: '§'),
    ],
    [
      KeyModel(primary: 'FN', actionType: KeyActionType.switchLayout, flex: 1.3),
      KeyModel(primary: '_', topLeft: '—', topRight: '-'),
      KeyModel(primary: '\\', topLeft: '/', topRight: '|'),
      KeyModel(primary: '~', topLeft: '`', topRight: '^'),
      KeyModel(primary: '`', topLeft: '"', topRight: "'"),
      KeyModel(primary: ':', topLeft: ';', topRight: '.'),
      KeyModel(primary: ';', topLeft: ':', topRight: ','),
      KeyModel(primary: '"', topLeft: "'", topRight: '`'),
      KeyModel(primary: '⌫', actionType: KeyActionType.backspace, flex: 1.3, androidKeyCode: 67),
    ],
    [
      KeyModel(primary: 'ABC', actionType: KeyActionType.switchLayout, flex: 1.2),
      KeyModel(primary: ',', flex: 1.0),
      KeyModel(primary: ' ', actionType: KeyActionType.space, flex: 4.0),
      KeyModel(primary: '.', flex: 1.0),
      KeyModel(primary: '↵', actionType: KeyActionType.enter, flex: 1.5, androidKeyCode: 66),
    ],
  ];
}
