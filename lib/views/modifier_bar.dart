import 'package:flutter/material.dart';
import '../models/gesture_direction.dart';
import '../models/key_model.dart';
import '../models/keyboard_layout.dart';
import '../themes/keyboard_theme.dart';
import 'key_widget.dart';

class ModifierBar extends StatelessWidget {
  final KeyboardThemeData theme;
  final KeyPressCallback onKeyPress;
  final bool isShiftActive;
  final bool isCtrlActive;
  final bool isAltActive;
  final bool isSymbolLayout;
  final bool isVersion1Layout;
  final double keyHeight;
  final bool isResizeBarOpen;
  final VoidCallback? onToggleResizeBar;
  final ValueChanged<double>? onVerticalResizeDrag;
  final VoidCallback? onSwitchKeyboard;
  final VoidCallback? onOpenSettings;

  const ModifierBar({
    super.key,
    required this.theme,
    required this.onKeyPress,
    required this.isShiftActive,
    required this.isCtrlActive,
    required this.isAltActive,
    this.isSymbolLayout = false,
    this.isVersion1Layout = false,
    this.keyHeight = 46.0,
    this.isResizeBarOpen = false,
    this.onToggleResizeBar,
    this.onVerticalResizeDrag,
    this.onSwitchKeyboard,
    this.onOpenSettings,
  });

  @override
  Widget build(BuildContext context) {
    final keys = KeyboardLayout.terminalBarKeys;
    final barButtonHeight = (keyHeight * 0.85).clamp(36.0, 68.0);

    return Container(
      height: barButtonHeight,
      padding: EdgeInsets.zero,
      color: theme.backgroundColor,
      child: Row(
        children: [
          ...keys.map((key) {
            final bool isLayoutSwitch = key.actionType == KeyActionType.switchLayout;
            final bool isVersionSwitch = key.actionType == KeyActionType.switchVersion;
            final KeyModel effectiveKey = (isLayoutSwitch && isSymbolLayout)
                ? KeyModel(
                    primary: 'ABC',
                    actionType: KeyActionType.switchLayout,
                    flex: key.flex,
                  )
                : (isVersionSwitch
                    ? KeyModel(
                        primary: isVersion1Layout ? 'V1' : 'V2',
                        actionType: KeyActionType.switchVersion,
                        flex: key.flex,
                      )
                    : key);

            bool isActive = false;
            if (effectiveKey.actionType == KeyActionType.shift && isShiftActive) isActive = true;
            if (effectiveKey.actionType == KeyActionType.ctrl && isCtrlActive) isActive = true;
            if (effectiveKey.actionType == KeyActionType.alt && isAltActive) isActive = true;
            if (isLayoutSwitch && isSymbolLayout) isActive = true;
            if (isVersionSwitch && isVersion1Layout) isActive = true;

            return Expanded(
              flex: (effectiveKey.flex * 10).round(),
              child: Listener(
                behavior: HitTestBehavior.opaque,
                onPointerDown: (_) => onKeyPress(effectiveKey, GestureDirection.center),
                child: Container(
                  height: barButtonHeight,
                  alignment: Alignment.center,
                  decoration: BoxDecoration(
                    color: isActive ? theme.modifierActiveColor : theme.keyBackgroundColor,
                    border: Border.all(
                      color: isActive ? theme.accentColor : theme.modifierInactiveColor.withOpacity(0.85),
                      width: isActive ? 1.2 : 0.5,
                    ),
                  ),
                  child: Text(
                    effectiveKey.primary,
                    maxLines: 1,
                    softWrap: false,
                    textAlign: TextAlign.center,
                    style: TextStyle(
                      fontFamily: 'sans-serif',
                      fontSize: effectiveKey.primary.length > 1
                          ? (barButtonHeight * 0.34).clamp(11.5, 20.0)
                          : (barButtonHeight * 0.46).clamp(15.0, 26.0),
                      fontWeight: isActive ? FontWeight.bold : FontWeight.w600,
                      color: isActive ? Colors.white : theme.primaryTextColor,
                      height: 1.0,
                    ),
                  ),
                ),
              ),
            );
          }),
          if (onSwitchKeyboard != null)
            Listener(
              behavior: HitTestBehavior.opaque,
              onPointerDown: (_) => onSwitchKeyboard!(),
              child: Container(
                width: 34.0,
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
                  Icons.language,
                  size: 17.0,
                  color: theme.accentColor,
                ),
              ),
            ),
          if (onToggleResizeBar != null)
            GestureDetector(
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
            Listener(
              behavior: HitTestBehavior.opaque,
              onPointerDown: (_) => onOpenSettings!(),
              child: Container(
                width: 34.0,
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
