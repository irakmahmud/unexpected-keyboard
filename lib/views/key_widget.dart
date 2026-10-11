import 'package:flutter/material.dart';
import '../models/gesture_direction.dart';
import '../models/key_model.dart';
import '../themes/keyboard_theme.dart';

typedef KeyPressCallback = void Function(KeyModel key, GestureDirection direction);

class KeyWidget extends StatefulWidget {
  final KeyModel keyModel;
  final KeyboardThemeData theme;
  final KeyPressCallback onKeyPress;
  final bool isShiftActive;
  final bool isCtrlActive;
  final bool isAltActive;
  final double height;

  const KeyWidget({
    super.key,
    required this.keyModel,
    required this.theme,
    required this.onKeyPress,
    this.isShiftActive = false,
    this.isCtrlActive = false,
    this.isAltActive = false,
    this.height = 48.0,
  });

  @override
  State<KeyWidget> createState() => _KeyWidgetState();
}

class _KeyWidgetState extends State<KeyWidget> {
  Offset? _startOffset;
  GestureDirection _activeDirection = GestureDirection.center;
  bool _isPressed = false;

  void _handlePanStart(DragStartDetails details) {
    setState(() {
      _startOffset = details.localPosition;
      _isPressed = true;
      _activeDirection = GestureDirection.center;
    });
  }

  void _handlePanUpdate(DragUpdateDetails details) {
    if (_startOffset == null) return;
    final Offset delta = details.localPosition - _startOffset!;
    final GestureDirection dir = GestureDirection.fromDelta(delta, threshold: 16.0);
    if (dir != _activeDirection) {
      setState(() {
        _activeDirection = dir;
      });
    }
  }

  void _handlePanEnd(DragEndDetails details) {
    widget.onKeyPress(widget.keyModel, _activeDirection);
    setState(() {
      _isPressed = false;
      _startOffset = null;
      _activeDirection = GestureDirection.center;
    });
  }

  void _handlePanCancel() {
    setState(() {
      _isPressed = false;
      _startOffset = null;
      _activeDirection = GestureDirection.center;
    });
  }

  void _handleTap() {
    widget.onKeyPress(widget.keyModel, GestureDirection.center);
  }

  @override
  Widget build(BuildContext context) {
    final key = widget.keyModel;
    final theme = widget.theme;

    // Check if modifier key is in active state
    bool isActiveModifier = false;
    if (key.actionType == KeyActionType.shift && widget.isShiftActive) isActiveModifier = true;
    if (key.actionType == KeyActionType.ctrl && widget.isCtrlActive) isActiveModifier = true;
    if (key.actionType == KeyActionType.alt && widget.isAltActive) isActiveModifier = true;

    final Color bgColor = isActiveModifier
        ? theme.modifierActiveColor
        : (_isPressed ? theme.keyPressedColor : theme.keyBackgroundColor);

    final String displayPrimary = (widget.isShiftActive && key.primary.length == 1)
        ? key.primary.toUpperCase()
        : key.primary;

    return Expanded(
      flex: (key.flex * 10).round(),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 2.0, vertical: 3.0),
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: _handleTap,
          onPanStart: _handlePanStart,
          onPanUpdate: _handlePanUpdate,
          onPanEnd: _handlePanEnd,
          onPanCancel: _handlePanCancel,
          child: Container(
            height: widget.height,
            decoration: BoxDecoration(
              color: bgColor,
              borderRadius: BorderRadius.circular(theme.borderRadius),
              boxShadow: [
                BoxShadow(
                  color: Colors.black.withOpacity(0.25),
                  offset: const Offset(0, 1.5),
                  blurRadius: 1.0,
                ),
              ],
            ),
            child: Stack(
              children: [
                // Top-Left Corner Symbol
                if (key.topLeft != null)
                  Positioned(
                    top: 2,
                    left: 4,
                    child: Text(
                      key.topLeft!,
                      style: TextStyle(
                        fontSize: 9.5,
                        fontWeight: _activeDirection == GestureDirection.topLeft ? FontWeight.bold : FontWeight.w500,
                        color: _activeDirection == GestureDirection.topLeft ? theme.accentColor : theme.cornerTextColor,
                      ),
                    ),
                  ),

                // Top-Right Corner Symbol
                if (key.topRight != null)
                  Positioned(
                    top: 2,
                    right: 4,
                    child: Text(
                      key.topRight!,
                      style: TextStyle(
                        fontSize: 9.5,
                        fontWeight: _activeDirection == GestureDirection.topRight ? FontWeight.bold : FontWeight.w500,
                        color: _activeDirection == GestureDirection.topRight ? theme.accentColor : theme.cornerTextColor,
                      ),
                    ),
                  ),

                // Bottom-Left Corner Symbol
                if (key.bottomLeft != null)
                  Positioned(
                    bottom: 2,
                    left: 4,
                    child: Text(
                      key.bottomLeft!,
                      style: TextStyle(
                        fontSize: 9.5,
                        fontWeight: _activeDirection == GestureDirection.bottomLeft ? FontWeight.bold : FontWeight.w500,
                        color: _activeDirection == GestureDirection.bottomLeft ? theme.accentColor : theme.cornerTextColor,
                      ),
                    ),
                  ),

                // Bottom-Right Corner Symbol
                if (key.bottomRight != null)
                  Positioned(
                    bottom: 2,
                    right: 4,
                    child: Text(
                      key.bottomRight!,
                      style: TextStyle(
                        fontSize: 9.5,
                        fontWeight: _activeDirection == GestureDirection.bottomRight ? FontWeight.bold : FontWeight.w500,
                        color: _activeDirection == GestureDirection.bottomRight ? theme.accentColor : theme.cornerTextColor,
                      ),
                    ),
                  ),

                // Primary Center Character / Icon
                Center(
                  child: Text(
                    displayPrimary,
                    style: TextStyle(
                      fontSize: displayPrimary.length > 2 ? 11.5 : 17.5,
                      fontWeight: FontWeight.w600,
                      color: isActiveModifier ? Colors.white : theme.primaryTextColor,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
