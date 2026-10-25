import 'dart:async';
import 'package:flutter/material.dart';
import '../models/gesture_direction.dart';
import '../models/key_model.dart';
import '../themes/keyboard_theme.dart';

typedef KeyPressCallback = void Function(KeyModel key, GestureDirection direction);
typedef CornerSwipeReplaceCallback = void Function(String replacementText);

class KeyWidget extends StatefulWidget {
  final KeyModel keyModel;
  final KeyboardThemeData theme;
  final KeyPressCallback onKeyPress;
  final CornerSwipeReplaceCallback? onCornerSwipeReplace;
  final VoidCallback? onSpaceLongPress;
  final bool isShiftActive;
  final bool isCtrlActive;
  final bool isAltActive;
  final double height;
  final int longPressMs;

  const KeyWidget({
    super.key,
    required this.keyModel,
    required this.theme,
    required this.onKeyPress,
    this.onCornerSwipeReplace,
    this.onSpaceLongPress,
    this.isShiftActive = false,
    this.isCtrlActive = false,
    this.isAltActive = false,
    this.height = 48.0,
    this.longPressMs = 400,
  });

  @override
  State<KeyWidget> createState() => _KeyWidgetState();
}

class _KeyWidgetState extends State<KeyWidget> {
  int? _activePointerId;
  Offset? _startOffset;
  GestureDirection _activeDirection = GestureDirection.center;
  bool _isPressed = false;
  bool _wasShiftedOnDown = false;
  OverlayEntry? _popupEntry;
  Timer? _hideTimer;
  Timer? _repeatDelayTimer;
  Timer? _repeatTimer;

  bool get _isRepeatableAction {
    final t = widget.keyModel.actionType;
    return t == KeyActionType.backspace ||
        t == KeyActionType.arrowLeft ||
        t == KeyActionType.arrowRight ||
        t == KeyActionType.arrowUp ||
        t == KeyActionType.arrowDown;
  }

  @override
  void dispose() {
    _cancelRepeatTimers();
    _hideTimer?.cancel();
    _removePopupEntry();
    super.dispose();
  }

  void _cancelRepeatTimers() {
    _repeatDelayTimer?.cancel();
    _repeatDelayTimer = null;
    _repeatTimer?.cancel();
    _repeatTimer = null;
  }

  String _resolvePopupLabel() {
    final key = widget.keyModel;
    switch (key.actionType) {
      case KeyActionType.inputChar:
        return key.resolveOutput(_activeDirection, isShifted: _wasShiftedOnDown);
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
    if (widget.keyModel.actionType != KeyActionType.inputChar) {
      return;
    }
    _hideTimer?.cancel();
    // Safety auto-hide if pointer leaves IME window bounds without PointerUp
    _hideTimer = Timer(const Duration(milliseconds: 800), () {
      _removePopupEntry();
      if (mounted && _isPressed) {
        setState(() {
          _isPressed = false;
          _activeDirection = GestureDirection.center;
        });
      }
    });
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
                    fontFamily: 'sans-serif-thin',
                    color: widget.theme.backgroundColor,
                    fontSize: label.length > 2 ? 16.0 : 30.0,
                    fontWeight: FontWeight.w100,
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
    _hideTimer = Timer(const Duration(milliseconds: 25), () {
      _removePopupEntry();
    });
  }

  void _removePopupEntry() {
    _popupEntry?.remove();
    _popupEntry = null;
  }

  void _handlePointerDown(PointerDownEvent event) {
    _cancelRepeatTimers();
    _hideTimer?.cancel();
    _activePointerId = event.pointer;
    _startOffset = event.localPosition;
    _activeDirection = GestureDirection.center;
    _wasShiftedOnDown = widget.isShiftActive;

    // 1. Fire keypress IMMEDIATELY at t=0.0ms on pointer touch-down (zero GestureArena delay)
    widget.onKeyPress(widget.keyModel, GestureDirection.center);

    // 2. Update visual highlight & popup immediately after dispatch
    setState(() {
      _isPressed = true;
    });
    _showOrUpdatePopup();

    // 3. Customizable Long Press (0ms..3000ms) for repeatable keys, Spacebar switcher, and secondary topLeft symbols
    final int lpMs = widget.longPressMs.clamp(0, 3000);
    if (_isRepeatableAction) {
      final int initialDelay = lpMs <= 0 ? 35 : lpMs;
      _repeatDelayTimer = Timer(Duration(milliseconds: initialDelay), () {
        _repeatTimer = Timer.periodic(const Duration(milliseconds: 45), (_) {
          if (mounted && _isPressed) {
            widget.onKeyPress(widget.keyModel, GestureDirection.center);
          }
        });
      });
    } else if (widget.keyModel.actionType == KeyActionType.space && widget.onSpaceLongPress != null) {
      final int spaceDelay = lpMs <= 0 ? 80 : lpMs;
      _repeatDelayTimer = Timer(Duration(milliseconds: spaceDelay), () {
        if (mounted && _isPressed) {
          widget.onSpaceLongPress?.call();
        }
      });
    } else if (widget.keyModel.actionType == KeyActionType.inputChar &&
        lpMs > 0 &&
        widget.keyModel.topLeft != null) {
      _repeatDelayTimer = Timer(Duration(milliseconds: lpMs), () {
        if (mounted && _isPressed && _activeDirection == GestureDirection.center) {
          final String lpOut = widget.keyModel.topLeft!;
          setState(() {
            _activeDirection = GestureDirection.topLeft;
          });
          _showOrUpdatePopup();
          if (lpOut != '⚙️') {
            widget.onCornerSwipeReplace?.call(lpOut);
          }
        }
      });
    }
  }

  void _handlePointerMove(PointerMoveEvent event) {
    if (event.pointer != _activePointerId || _startOffset == null) return;
    if (widget.keyModel.actionType != KeyActionType.inputChar) return;

    final Offset delta = event.localPosition - _startOffset!;
    final GestureDirection dir = GestureDirection.fromDelta(delta, threshold: 14.0);
    if (dir != _activeDirection) {
      if (dir != GestureDirection.center) {
        _repeatDelayTimer?.cancel();
        _repeatDelayTimer = null;
      }
      final String prevOutput = widget.keyModel.resolveOutput(
        _activeDirection,
        isShifted: _wasShiftedOnDown,
      );
      final String newOutput = widget.keyModel.resolveOutput(
        dir,
        isShifted: _wasShiftedOnDown,
      );

      setState(() {
        _activeDirection = dir;
      });
      _showOrUpdatePopup();

      // Atomically replace the center character with the corner character (0ms live update)
      if (newOutput != prevOutput && newOutput != '⚙️') {
        widget.onCornerSwipeReplace?.call(newOutput);
      }
    }
  }

  void _handlePointerUp(PointerUpEvent event) {
    if (_activePointerId != null && event.pointer != _activePointerId) return;
    _cancelRepeatTimers();
    _activePointerId = null;
    _startOffset = null;

    if (mounted) {
      setState(() {
        _isPressed = false;
        _activeDirection = GestureDirection.center;
      });
    }
    _scheduleHidePopup();
  }

  void _handlePointerCancel(PointerCancelEvent event) {
    if (_activePointerId != null && event.pointer != _activePointerId) return;
    _cancelRepeatTimers();
    _activePointerId = null;
    _startOffset = null;

    if (mounted) {
      setState(() {
        _isPressed = false;
        _activeDirection = GestureDirection.center;
      });
    }
    _scheduleHidePopup();
  }

  @override
  Widget build(BuildContext context) {
    final key = widget.keyModel;
    final theme = widget.theme;

    // Check if modifier or layout switch key is in active state
    bool isActiveModifier = false;
    if (key.actionType == KeyActionType.shift && widget.isShiftActive) isActiveModifier = true;
    if (key.actionType == KeyActionType.ctrl && widget.isCtrlActive) isActiveModifier = true;
    if (key.actionType == KeyActionType.alt && widget.isAltActive) isActiveModifier = true;
    if (key.actionType == KeyActionType.switchLayout && key.primary == 'ABC') isActiveModifier = true;
    if (key.actionType == KeyActionType.switchVersion && key.primary == 'V1') isActiveModifier = true;

    final Color bgColor = isActiveModifier
        ? theme.modifierActiveColor
        : (_isPressed ? theme.keyPressedColor : theme.keyBackgroundColor);

    final String displayPrimary = (widget.isShiftActive && key.primary.length == 1)
        ? key.primary.toUpperCase()
        : key.primary;

    // Zero gaps between keys + raw Listener for 0.0ms touch-down latency (no RepaintBoundary/FittedBox texture stall)
    return Expanded(
      flex: (key.flex * 100).round(),
      child: Listener(
        behavior: HitTestBehavior.opaque,
        onPointerDown: _handlePointerDown,
        onPointerMove: _handlePointerMove,
        onPointerUp: _handlePointerUp,
        onPointerCancel: _handlePointerCancel,
        child: Container(
          height: widget.height,
          decoration: BoxDecoration(
            color: bgColor,
            border: Border.all(
              color: isActiveModifier ? theme.accentColor : theme.modifierInactiveColor.withOpacity(0.85),
              width: isActiveModifier ? 1.0 : 0.5,
            ),
          ),
          child: Stack(
            alignment: Alignment.center,
            fit: StackFit.expand,
            children: [
              // Primary Center Character / Icon (Direct fast Text layout without FittedBox or FontVariation overhead)
              Center(
                child: _buildMiddleAlignedPrimaryContent(
                  key: key,
                  displayPrimary: displayPrimary,
                  theme: theme,
                  isActiveModifier: isActiveModifier,
                ),
              ),

              // Top-Left Corner Symbol
              if (key.topLeft != null)
                Positioned(
                  top: 1,
                  left: 3,
                  child: IgnorePointer(
                    child: Text(
                      key.topLeft!,
                      style: TextStyle(
                        fontFamily: 'sans-serif-thin',
                        fontSize: 9.0,
                        height: 1.0,
                        fontWeight: FontWeight.w100,
                        color: _activeDirection == GestureDirection.topLeft ? theme.accentColor : theme.cornerTextColor,
                      ),
                    ),
                  ),
                ),

              // Top-Right Corner Symbol (or 🌐 Keyboard Switcher hint on Spacebar)
              if (key.topRight != null)
                Positioned(
                  top: 1,
                  right: 3,
                  child: IgnorePointer(
                    child: Text(
                      key.topRight!,
                      style: TextStyle(
                        fontFamily: 'sans-serif-thin',
                        fontSize: 9.0,
                        height: 1.0,
                        fontWeight: FontWeight.w100,
                        color: _activeDirection == GestureDirection.topRight ? theme.accentColor : theme.cornerTextColor,
                      ),
                    ),
                  ),
                )
              else if (key.actionType == KeyActionType.space)
                Positioned(
                  top: 2,
                  right: 4,
                  child: IgnorePointer(
                    child: Icon(
                      Icons.language,
                      size: 10.0,
                      color: theme.cornerTextColor,
                    ),
                  ),
                ),

              // Bottom-Left Corner Symbol
              if (key.bottomLeft != null)
                Positioned(
                  bottom: 1,
                  left: 3,
                  child: IgnorePointer(
                    child: Text(
                      key.bottomLeft!,
                      style: TextStyle(
                        fontFamily: 'sans-serif-thin',
                        fontSize: 9.0,
                        height: 1.0,
                        fontWeight: FontWeight.w100,
                        color: _activeDirection == GestureDirection.bottomLeft ? theme.accentColor : theme.cornerTextColor,
                      ),
                    ),
                  ),
                ),

              // Bottom-Right Corner Symbol
              if (key.bottomRight != null)
                Positioned(
                  bottom: 1,
                  right: 3,
                  child: IgnorePointer(
                    child: Text(
                      key.bottomRight!,
                      style: TextStyle(
                        fontFamily: 'sans-serif-thin',
                        fontSize: 9.0,
                        height: 1.0,
                        fontWeight: FontWeight.w100,
                        color: _activeDirection == GestureDirection.bottomRight ? theme.accentColor : theme.cornerTextColor,
                      ),
                    ),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildMiddleAlignedPrimaryContent({
    required KeyModel key,
    required String displayPrimary,
    required KeyboardThemeData theme,
    required bool isActiveModifier,
  }) {
    final Color fgColor = isActiveModifier ? Colors.white : theme.primaryTextColor;

    if (key.actionType == KeyActionType.openSettings) {
      return Icon(
        Icons.settings_outlined,
        size: (widget.height * 0.52).clamp(16.0, 36.0),
        color: fgColor,
      );
    }

    if (key.actionType == KeyActionType.space) {
      return Text(
        '␣',
        textAlign: TextAlign.center,
        textHeightBehavior: const TextHeightBehavior(
          applyHeightToFirstAscent: false,
          applyHeightToLastDescent: false,
        ),
        style: TextStyle(
          fontFamily: 'sans-serif-thin',
          fontSize: (widget.height * 0.68).clamp(22.0, 82.0),
          height: 1.0,
          fontWeight: FontWeight.w100,
          color: fgColor,
        ),
      );
    }

    final bool isMultiCharLabel = displayPrimary.length > 1;
    final double computedFontSize = isMultiCharLabel
        ? (widget.height * 0.34).clamp(11.0, 26.0)
        : (widget.height * 0.72).clamp(24.0, 88.0);
    final bool isLowPunctuation = displayPrimary == ',' || displayPrimary == '.';

    Widget textWidget = Text(
      displayPrimary,
      textAlign: TextAlign.center,
      maxLines: 1,
      softWrap: false,
      textHeightBehavior: const TextHeightBehavior(
        applyHeightToFirstAscent: false,
        applyHeightToLastDescent: false,
      ),
      style: TextStyle(
        fontFamily: isMultiCharLabel ? 'sans-serif' : 'sans-serif-thin',
        fontSize: computedFontSize,
        height: 1.0,
        fontWeight: isMultiCharLabel ? FontWeight.w600 : FontWeight.w100,
        color: fgColor,
      ),
    );

    if (isLowPunctuation) {
      textWidget = Transform.translate(
        offset: Offset(0, -widget.height * 0.16),
        child: textWidget,
      );
    }

    return textWidget;
  }
}
