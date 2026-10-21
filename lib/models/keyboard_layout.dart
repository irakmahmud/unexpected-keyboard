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
    // Row 1 (9 keys, equal width flex 1.0): q w e r t y u i o
    [
      KeyModel(primary: 'q', topLeft: '1', topRight: '`', bottomLeft: '~', bottomRight: '\\', swipeUp: '1', flex: 1.0),
      KeyModel(primary: 'w', topLeft: '2', topRight: '|', bottomLeft: '<', bottomRight: '>', swipeUp: '2', flex: 1.0),
      KeyModel(primary: 'e', topLeft: '3', topRight: '€', bottomLeft: '{', bottomRight: '}', swipeUp: '3', flex: 1.0),
      KeyModel(primary: 'r', topLeft: '4', topRight: '₹', bottomLeft: '[', bottomRight: ']', swipeUp: '4', flex: 1.0),
      KeyModel(primary: 't', topLeft: '5', topRight: '%', bottomLeft: '(', bottomRight: ')', swipeUp: '5', flex: 1.0),
      KeyModel(primary: 'y', topLeft: '6', topRight: '^', bottomLeft: '=', bottomRight: '+', swipeUp: '6', flex: 1.0),
      KeyModel(primary: 'u', topLeft: '7', topRight: '&', bottomLeft: '-', bottomRight: '_', swipeUp: '7', flex: 1.0),
      KeyModel(primary: 'i', topLeft: '8', topRight: '*', bottomLeft: '/', bottomRight: '*', swipeUp: '8', flex: 1.0),
      KeyModel(primary: 'o', topLeft: '9', topRight: '(', bottomLeft: '[', bottomRight: ']', swipeUp: '9', flex: 1.0),
    ],
    // Row 2 (9 keys, equal width flex 1.0): a s d f g h j k l
    [
      KeyModel(primary: 'a', topLeft: '@', topRight: '!', bottomLeft: '#', bottomRight: '\$', swipeUp: '@', flex: 1.0),
      KeyModel(primary: 's', topLeft: '#', topRight: '\$', bottomLeft: '%', bottomRight: '^', swipeUp: '#', flex: 1.0),
      KeyModel(primary: 'd', topLeft: '\$', topRight: '%', bottomLeft: '&', bottomRight: '*', swipeUp: '\$', flex: 1.0),
      KeyModel(primary: 'f', topLeft: '%', topRight: '^', bottomLeft: '_', bottomRight: '+', swipeUp: '%', flex: 1.0),
      KeyModel(primary: 'g', topLeft: '&', topRight: '*', bottomLeft: '-', bottomRight: '=', swipeUp: '&', flex: 1.0),
      KeyModel(primary: 'h', topLeft: '-', topRight: '_', bottomLeft: ':', bottomRight: ';', swipeUp: '-', flex: 1.0),
      KeyModel(primary: 'j', topLeft: '+', topRight: '=', bottomLeft: '"', bottomRight: "'", swipeUp: '+', flex: 1.0),
      KeyModel(primary: 'k', topLeft: '(', topRight: ')', bottomLeft: '<', bottomRight: '>', swipeUp: '(', flex: 1.0),
      KeyModel(primary: 'l', topLeft: ')', topRight: ':', bottomLeft: '[', bottomRight: ']', swipeUp: ')', flex: 1.0),
    ],
    // Row 3 (9 keys, equal width flex 1.0): z x c v b n m p ⌫
    [
      KeyModel(primary: 'z', topLeft: '*', topRight: '/', bottomLeft: '\\', bottomRight: '|', swipeUp: '*', flex: 1.0),
      KeyModel(primary: 'x', topLeft: '"', topRight: "'", bottomLeft: '`', bottomRight: '~', swipeUp: '"', flex: 1.0),
      KeyModel(primary: 'c', topLeft: "'", topRight: '`', bottomLeft: '©', bottomRight: '¢', swipeUp: "'", flex: 1.0),
      KeyModel(primary: 'v', topLeft: ':', topRight: ';', bottomLeft: '•', bottomRight: '°', swipeUp: ':', flex: 1.0),
      KeyModel(primary: 'b', topLeft: ';', topRight: ':', bottomLeft: '§', bottomRight: '¶', swipeUp: ';', flex: 1.0),
      KeyModel(primary: 'n', topLeft: '!', topRight: '?', bottomLeft: '¿', bottomRight: '¡', swipeUp: '!', flex: 1.0),
      KeyModel(primary: 'm', topLeft: '?', topRight: '!', bottomLeft: '«', bottomRight: '»', swipeUp: '?', flex: 1.0),
      KeyModel(primary: 'p', topLeft: '0', topRight: ')', bottomLeft: '{', bottomRight: '}', swipeUp: '0', flex: 1.0),
      KeyModel(primary: '⌫', actionType: KeyActionType.backspace, flex: 1.0, androidKeyCode: 67),
    ],
    // Row 4: Shift (left of ?123), ?123, Settings (⚙️), comma (with ⚙️ on long-press/topLeft), Space, dot, Enter
    [
      KeyModel(primary: '⇧', actionType: KeyActionType.shift, flex: 1.1),
      KeyModel(primary: '?123', actionType: KeyActionType.switchLayout, flex: 1.1),
      KeyModel(primary: '⚙️', actionType: KeyActionType.openSettings, flex: 0.9),
      KeyModel(primary: ',', topLeft: '⚙️', topRight: '<', bottomLeft: ';', bottomRight: '[', flex: 0.9),
      KeyModel(primary: ' ', actionType: KeyActionType.space, flex: 2.7),
      KeyModel(primary: '.', topLeft: ':', topRight: '>', bottomLeft: '}', bottomRight: ']', flex: 0.9),
      KeyModel(primary: '↵', actionType: KeyActionType.enter, flex: 1.4, androidKeyCode: 66),
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
      KeyModel(primary: '⚙️', actionType: KeyActionType.openSettings, flex: 1.0),
      KeyModel(primary: ',', topLeft: '⚙️', flex: 1.0),
      KeyModel(primary: ' ', actionType: KeyActionType.space, flex: 3.3),
      KeyModel(primary: '.', flex: 1.0),
      KeyModel(primary: '↵', actionType: KeyActionType.enter, flex: 1.5, androidKeyCode: 66),
    ],
  ];
}
