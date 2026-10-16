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
  final double keyHeight;
  final bool isResizeBarOpen;
  final VoidCallback? onToggleResizeBar;
  final ValueChanged<double>? onVerticalResizeDrag;

  const ModifierBar({
    super.key,
    required this.theme,
    required this.onKeyPress,
    required this.isShiftActive,
    required this.isCtrlActive,
    required this.isAltActive,
    this.keyHeight = 46.0,
    this.isResizeBarOpen = false,
    this.onToggleResizeBar,
    this.onVerticalResizeDrag,
  });

  @override
  Widget build(BuildContext context) {
    final keys = KeyboardLayout.terminalBarKeys;
    final barButtonHeight = (keyHeight * 0.72).clamp(28.0, 46.0);

    return Container(
      height: barButtonHeight + 6.0,
      padding: const EdgeInsets.symmetric(horizontal: 4.0),
      color: theme.backgroundColor.withOpacity(0.95),
      child: Row(
        children: [
          ...keys.map((key) {
            return KeyWidget(
              keyModel: key,
              theme: theme,
              onKeyPress: onKeyPress,
              isShiftActive: isShiftActive,
              isCtrlActive: isCtrlActive,
              isAltActive: isAltActive,
              height: barButtonHeight,
            );
          }),
          if (onToggleResizeBar != null)
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 2.0, vertical: 3.0),
              child: GestureDetector(
                behavior: HitTestBehavior.opaque,
                onTap: onToggleResizeBar,
                onVerticalDragUpdate: onVerticalResizeDrag != null
                    ? (details) => onVerticalResizeDrag!(-details.delta.dy * 0.35)
                    : null,
                child: Container(
                  width: 34.0,
                  height: barButtonHeight,
                  decoration: BoxDecoration(
                    color: isResizeBarOpen ? theme.modifierActiveColor : theme.keyBackgroundColor,
                    borderRadius: BorderRadius.circular(theme.borderRadius),
                    border: Border.all(
                      color: isResizeBarOpen ? theme.accentColor : Colors.transparent,
                      width: 1.0,
                    ),
                  ),
                  alignment: Alignment.center,
                  child: Icon(
                    Icons.unfold_more,
                    size: 18.0,
                    color: isResizeBarOpen ? theme.accentColor : theme.primaryTextColor,
                  ),
                ),
              ),
            ),
        ],
      ),
    );
  }
}
