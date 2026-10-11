import 'package:flutter/material.dart';
import '../models/gesture_direction.dart';
import '../models/key_model.dart';
import '../models/keyboard_layout.dart';
import '../services/ime_channel_service.dart';
import '../themes/keyboard_theme.dart';
import 'key_widget.dart';
import 'modifier_bar.dart';

class KeyboardView extends StatefulWidget {
  final ImeChannelService imeService;
  final KeyboardThemeData theme;
  final VoidCallback onOpenSettings;

  const KeyboardView({
    super.key,
    required this.imeService,
    required this.theme,
    required this.onOpenSettings,
  });

  @override
  State<KeyboardView> createState() => _KeyboardViewState();
}

class _KeyboardViewState extends State<KeyboardView> {
  LayoutType _currentLayout = LayoutType.qwerty;
  bool _showTerminalBar = true;

  void _handleKeyPress(KeyModel key, GestureDirection direction) {
    widget.imeService.triggerHaptic();

    switch (key.actionType) {
      case KeyActionType.inputChar:
        final text = key.resolveOutput(direction, isShifted: widget.imeService.isShiftActive);
        widget.imeService.sendText(text);
        setState(() {});
        break;

      case KeyActionType.backspace:
        widget.imeService.sendBackspace();
        break;

      case KeyActionType.enter:
        widget.imeService.sendEnter();
        break;

      case KeyActionType.space:
        widget.imeService.sendText(' ');
        break;

      case KeyActionType.tab:
        widget.imeService.sendKeyCode(61); // KEYCODE_TAB
        break;

      case KeyActionType.escape:
        widget.imeService.sendKeyCode(111); // KEYCODE_ESCAPE
        break;

      case KeyActionType.shift:
        setState(() {
          if (!widget.imeService.isShiftActive) {
            widget.imeService.isShiftActive = true;
            widget.imeService.isCapsLock = false;
          } else if (!widget.imeService.isCapsLock) {
            widget.imeService.isCapsLock = true;
          } else {
            widget.imeService.isShiftActive = false;
            widget.imeService.isCapsLock = false;
          }
        });
        break;

      case KeyActionType.ctrl:
        setState(() {
          widget.imeService.isCtrlActive = !widget.imeService.isCtrlActive;
        });
        break;

      case KeyActionType.alt:
        setState(() {
          widget.imeService.isAltActive = !widget.imeService.isAltActive;
        });
        break;

      case KeyActionType.meta:
        setState(() {
          widget.imeService.isMetaActive = !widget.imeService.isMetaActive;
        });
        break;

      case KeyActionType.arrowLeft:
        widget.imeService.sendKeyCode(21); // KEYCODE_DPAD_LEFT
        break;

      case KeyActionType.arrowRight:
        widget.imeService.sendKeyCode(22); // KEYCODE_DPAD_RIGHT
        break;

      case KeyActionType.arrowUp:
        widget.imeService.sendKeyCode(19); // KEYCODE_DPAD_UP
        break;

      case KeyActionType.arrowDown:
        widget.imeService.sendKeyCode(20); // KEYCODE_DPAD_DOWN
        break;

      case KeyActionType.switchLayout:
        setState(() {
          _currentLayout = (_currentLayout == LayoutType.qwerty)
              ? LayoutType.symbols
              : LayoutType.qwerty;
        });
        break;

      case KeyActionType.openSettings:
        widget.onOpenSettings();
        break;
    }
  }

  @override
  Widget build(BuildContext context) {
    final List<List<KeyModel>> rows = (_currentLayout == LayoutType.qwerty)
        ? KeyboardLayout.qwertyLayout
        : KeyboardLayout.symbolLayout;

    return Container(
      color: widget.theme.backgroundColor,
      child: SafeArea(
        top: false,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            // Top Terminal Modifier Bar (Termux / Hacker Keys)
            if (_showTerminalBar)
              ModifierBar(
                theme: widget.theme,
                onKeyPress: _handleKeyPress,
                isShiftActive: widget.imeService.isShiftActive,
                isCtrlActive: widget.imeService.isCtrlActive,
                isAltActive: widget.imeService.isAltActive,
              ),

            // Keyboard Character Rows
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 2.0, vertical: 2.0),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: rows.map((row) {
                  return Row(
                    children: row.map((key) {
                      return KeyWidget(
                        keyModel: key,
                        theme: widget.theme,
                        onKeyPress: _handleKeyPress,
                        isShiftActive: widget.imeService.isShiftActive,
                        isCtrlActive: widget.imeService.isCtrlActive,
                        isAltActive: widget.imeService.isAltActive,
                        height: 46.0,
                      );
                    }).toList(),
                  );
                }).toList(),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
