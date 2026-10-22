import 'dart:async';
import 'dart:ui' show FontVariation;
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
                    fontVariations: const [FontVariation('wght', 100.0)],
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

    // 3. If backspace or arrow key is held down, repeat smoothly
    if (_isRepeatableAction) {
      _repeatDelayTimer = Timer(const Duration(milliseconds: 350), () {
        _repeatTimer = Timer.periodic(const Duration(milliseconds: 45), (_) {
          if (mounted && _isPressed) {
            widget.onKeyPress(widget.keyModel, GestureDirection.center);
          }
        });
      });
    } else if (widget.keyModel.actionType == KeyActionType.space && widget.onSpaceLongPress != null) {
      // Holding Spacebar opens Android system keyboard switcher (like Ridmik / Gboard)
      _repeatDelayTimer = Timer(const Duration(milliseconds: 450), () {
        if (mounted && _isPressed) {
          widget.onSpaceLongPress?.call();
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

    final Color bgColor = isActiveModifier
        ? theme.modifierActiveColor
        : (_isPressed ? theme.keyPressedColor : theme.keyBackgroundColor);

    final String displayPrimary = (widget.isShiftActive && key.primary.length == 1)
        ? key.primary.toUpperCase()
        : key.primary;

    // Zero gaps between keys + raw Listener for 0.0ms touch-down latency + RepaintBoundary
    return Expanded(
      flex: (key.flex * 10).round(),
      child: RepaintBoundary(
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
              color: theme.modifierInactiveColor.withOpacity(0.85),
              width: 0.5,
            ),
          ),
          child: Stack(
            alignment: Alignment.center,
            fit: StackFit.expand,
            children: [
              // Primary Center Character / Icon (5x font size 90.0 scaled via FittedBox & strictly middle-aligned)
              Positioned.fill(
                child: Center(
                  child: Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 2.0, vertical: 1.0),
                    child: _buildMiddleAlignedPrimaryContent(
                      key: key,
                      displayPrimary: displayPrimary,
                      theme: theme,
                      isActiveModifier: isActiveModifier,
                    ),
                  ),
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
                        fontVariations: const [FontVariation('wght', 100.0)],
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
                        fontVariations: const [FontVariation('wght', 100.0)],
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
                        fontVariations: const [FontVariation('wght', 100.0)],
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
                        fontVariations: const [FontVariation('wght', 100.0)],
                        color: _activeDirection == GestureDirection.bottomRight ? theme.accentColor : theme.cornerTextColor,
                      ),
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

  Widget _buildMiddleAlignedPrimaryContent({
    required KeyModel key,
    required String displayPrimary,
    required KeyboardThemeData theme,
    required bool isActiveModifier,
  }) {
    final Color fgColor = isActiveModifier ? Colors.white : theme.primaryTextColor;

    if (key.actionType == KeyActionType.openSettings) {
      return FittedBox(
        fit: BoxFit.contain,
        alignment: Alignment.center,
        child: Icon(
          Icons.settings_outlined,
          size: 48.0,
          color: fgColor,
        ),
      );
    }

    if (key.actionType == KeyActionType.space) {
      return FittedBox(
        fit: BoxFit.contain,
        alignment: Alignment.center,
        child: Text(
          '␣',
          textAlign: TextAlign.center,
          textHeightBehavior: const TextHeightBehavior(
            applyHeightToFirstAscent: false,
            applyHeightToLastDescent: false,
          ),
          style: TextStyle(
            fontFamily: 'sans-serif-thin',
            fontSize: 90.0, // 5x size increase (18.0 * 5 = 90.0) with ultra-thin line stroke
            height: 1.0,
            fontWeight: FontWeight.w100,
            fontVariations: const [FontVariation('wght', 100.0)],
            color: fgColor,
          ),
        ),
      );
    }

    // 5x letter size increase (18.0 * 5 = 90.0) with ultra-thin hairline stroke (FontWeight.w100 / sans-serif-thin)
    final bool isMultiCharLabel = displayPrimary.length > 1;
    final double targetFontSize = isMultiCharLabel ? 57.5 : 90.0;
    final bool isLowPunctuation = displayPrimary == ',' || displayPrimary == '.';

    Widget textWidget = Text(
      displayPrimary,
      textAlign: TextAlign.center,
      textHeightBehavior: const TextHeightBehavior(
        applyHeightToFirstAscent: false,
        applyHeightToLastDescent: false,
      ),
      style: TextStyle(
        fontFamily: isMultiCharLabel ? 'sans-serif' : 'sans-serif-thin',
        fontSize: targetFontSize,
        height: 1.0,
        fontWeight: isMultiCharLabel ? FontWeight.w400 : FontWeight.w100,
        fontVariations: isMultiCharLabel ? null : const [FontVariation('wght', 100.0)],
        color: fgColor,
      ),
    );

    if (isLowPunctuation) {
      // Shift comma and dot upward so the glyph sits in the exact visual middle of the button
      textWidget = Transform.translate(
        offset: const Offset(0, -18.0),
        child: textWidget,
      );
    }

    return FittedBox(
      fit: BoxFit.contain,
      alignment: Alignment.center,
      child: textWidget,
    );
  }
}
