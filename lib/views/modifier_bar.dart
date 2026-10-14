import 'package:flutter/material.dart';
import '../models/keyboard_layout.dart';
import '../themes/keyboard_theme.dart';
import 'key_widget.dart';

class ModifierBar extends StatelessWidget {
  final KeyboardThemeData theme;
  final KeyPressCallback onKeyPress;
  final bool isShiftActive;
  final bool isCtrlActive;
  final bool isAltActive;

  const ModifierBar({
    super.key,
    required this.theme,
    required this.onKeyPress,
    required this.isShiftActive,
    required this.isCtrlActive,
    required this.isAltActive,
  });

  @override
  Widget build(BuildContext context) {
    final keys = KeyboardLayout.terminalBarKeys;

    return Container(
      height: 38.0,
      padding: const EdgeInsets.symmetric(horizontal: 4.0),
      color: theme.backgroundColor.withOpacity(0.95),
      child: Row(
        children: keys.map((key) {
          return KeyWidget(
            keyModel: key,
            theme: theme,
            onKeyPress: onKeyPress,
            isShiftActive: isShiftActive,
            isCtrlActive: isCtrlActive,
            isAltActive: isAltActive,
            height: 32.0,
          );
        }).toList(),
      ),
    );
  }
}
