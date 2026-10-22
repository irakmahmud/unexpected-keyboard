import 'dart:async';
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
  OverlayEntry? _popupEntry;
  Timer? _hideTimer;

  @override
  void dispose() {
    _hideTimer?.cancel();
    _removePopupEntry();
    super.dispose();
  }

  String _resolvePopupLabel() {
    final key = widget.keyModel;
    switch (key.actionType) {
      case KeyActionType.inputChar:
        if (_activeDirection == GestureDirection.longPress && key.topLeft == '⚙️') {
          return '⚙️';
        }
        return key.resolveOutput(_activeDirection, isShifted: widget.isShiftActive);
      case KeyActionType.space:
        return '␣';
      case KeyActionType.backspace:
        return '⌫';
      case KeyActionType.enter:
        return '↵';
      case KeyActionType.shift:
        return '⇧';
      case KeyActionType.openSettings:
        return '⚙️';
      default:
        return key.primary;
    }
  }

  void _showOrUpdatePopup() {
    _hideTimer?.cancel();
    if (_popupEntry != null) {
      _popupEntry!.markNeedsBuild();
      return;
    }
    final overlay = Overlay.maybeOf(context);
    if (overlay == null) return;

    _popupEntry = OverlayEntry(
      builder: (ctx) {
        if (!mounted) return const SizedBox.shrink();
        final renderObj = context.findRenderObject();
        if (renderObj is! RenderBox || !renderObj.hasSize) {
          return const SizedBox.shrink();
        }
        final Offset topLeft = renderObj.localToGlobal(Offset.zero);
        final Size size = renderObj.size;
        final double screenWidth = MediaQuery.of(ctx).size.width;
        final String label = _resolvePopupLabel();

        const double popupWidth = 56.0;
        const double popupHeight = 52.0;
        final double left = (topLeft.dx + (size.width - popupWidth) / 2.0)
            .clamp(4.0, (screenWidth - popupWidth - 4.0).clamp(4.0, double.infinity));
        final double rawTop = topLeft.dy - popupHeight - 4.0;
        final double top = rawTop < 2.0 ? 2.0 : rawTop;

        return Positioned(
          left: left,
          top: top,
          width: popupWidth,
          height: popupHeight,
          child: IgnorePointer(
            child: Material(
              color: Colors.transparent,
              child: Container(
                alignment: Alignment.center,
                decoration: BoxDecoration(
                  color: widget.theme.accentColor,
                  borderRadius: BorderRadius.circular(8.0),
                  border: Border.all(color: Colors.white, width: 1.5),
                  boxShadow: [
                    BoxShadow(
                      color: Colors.black.withOpacity(0.55),
                      blurRadius: 6.0,
                      offset: const Offset(0, 3),
                    ),
                  ],
                ),
                child: Text(
                  label,
                  style: TextStyle(
                    color: widget.theme.backgroundColor,
                    fontSize: label.length > 2 ? 14.0 : 24.0,
                    fontWeight: FontWeight.w900,
                  ),
                ),
              ),
            ),
          ),
        );
      },
    );

    overlay.insert(_popupEntry!);
  }

  void _scheduleHidePopup() {
    _hideTimer?.cancel();
    _hideTimer = Timer(const Duration(milliseconds: 160), () {
      _removePopupEntry();
      if (mounted) {
        setState(() {
          _isPressed = false;
          _startOffset = null;
          _activeDirection = GestureDirection.center;
        });
      }
    });
  }

  void _removePopupEntry() {
    _popupEntry?.remove();
    _popupEntry = null;
  }

  void _handleTapDown(TapDownDetails details) {
    setState(() {
      _isPressed = true;
      _activeDirection = GestureDirection.center;
    });
    _showOrUpdatePopup();
  }

  void _handlePanStart(DragStartDetails details) {
    setState(() {
      _startOffset = details.localPosition;
      _isPressed = true;
      _activeDirection = GestureDirection.center;
    });
    _showOrUpdatePopup();
  }

  void _handlePanUpdate(DragUpdateDetails details) {
    if (_startOffset == null) return;
    final Offset delta = details.localPosition - _startOffset!;
    final GestureDirection dir = GestureDirection.fromDelta(delta, threshold: 16.0);
    if (dir != _activeDirection) {
      setState(() {
        _activeDirection = dir;
      });
      _showOrUpdatePopup();
    }
  }

  void _handlePanEnd(DragEndDetails details) {
    _showOrUpdatePopup();
    widget.onKeyPress(widget.keyModel, _activeDirection);
    _scheduleHidePopup();
  }

  void _handlePanCancel() {
    _scheduleHidePopup();
  }

  void _handleTap() {
    setState(() {
      _isPressed = true;
      _activeDirection = GestureDirection.center;
    });
    _showOrUpdatePopup();
    widget.onKeyPress(widget.keyModel, GestureDirection.center);
    _scheduleHidePopup();
  }

  void _handleLongPress() {
    setState(() {
      _isPressed = true;
      _activeDirection = GestureDirection.longPress;
    });
    _showOrUpdatePopup();
    widget.onKeyPress(widget.keyModel, GestureDirection.longPress);
    _scheduleHidePopup();
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

    // Zero gaps between keys: EdgeInsets.zero + BorderRadius.zero + subtle 0.5px border
    return Expanded(
      flex: (key.flex * 10).round(),
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTapDown: _handleTapDown,
        onTap: _handleTap,
        onLongPress: _handleLongPress,
        onPanStart: _handlePanStart,
        onPanUpdate: _handlePanUpdate,
        onPanEnd: _handlePanEnd,
        onPanCancel: _handlePanCancel,
        child: Container(
          height: widget.height,
          decoration: BoxDecoration(
            color: bgColor,
            border: Border.all(
              color: theme.modifierInactiveColor.withOpacity(0.85),
              width: 0.5,
            ),
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
                child: key.actionType == KeyActionType.openSettings
                    ? Icon(
                        Icons.settings,
                        size: 19.0,
                        color: theme.primaryTextColor,
                      )
                    : Text(
                        displayPrimary,
                        style: TextStyle(
                          fontSize: displayPrimary.length > 2 ? 11.5 : 18.0,
                          fontWeight: FontWeight.w600,
                          color: isActiveModifier ? Colors.white : theme.primaryTextColor,
                        ),
                      ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
