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
  final VoidCallback? onOpenSettings;

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
    this.onOpenSettings,
  });

  @override
  Widget build(BuildContext context) {
    final keys = KeyboardLayout.terminalBarKeys;
    final barButtonHeight = (keyHeight * 0.72).clamp(28.0, 68.0);

    return Container(
      height: barButtonHeight,
      padding: EdgeInsets.zero,
      color: theme.backgroundColor,
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
            GestureDetector(
              behavior: HitTestBehavior.opaque,
              onTap: onToggleResizeBar,
              onVerticalDragUpdate: onVerticalResizeDrag != null
                  ? (details) => onVerticalResizeDrag!(-details.delta.dy * 0.35)
                  : null,
              child: Container(
                width: 36.0,
                height: barButtonHeight,
                decoration: BoxDecoration(
                  color: isResizeBarOpen ? theme.modifierActiveColor : theme.keyBackgroundColor,
                  border: Border.all(
                    color: isResizeBarOpen ? theme.accentColor : theme.modifierInactiveColor.withOpacity(0.85),
                    width: 0.5,
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
          if (onOpenSettings != null)
            GestureDetector(
              behavior: HitTestBehavior.opaque,
              onTap: onOpenSettings,
              child: Container(
                width: 36.0,
                height: barButtonHeight,
                decoration: BoxDecoration(
                  color: theme.keyBackgroundColor,
                  border: Border.all(
                    color: theme.modifierInactiveColor.withOpacity(0.85),
                    width: 0.5,
                  ),
                ),
                alignment: Alignment.center,
                child: Icon(
                  Icons.settings,
                  size: 17.0,
                  color: theme.primaryTextColor,
                ),
              ),
            ),
        ],
      ),
    );
  }
}
