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
  final double? keyHeight;
  final ValueChanged<double>? onKeyHeightChanged;

  const KeyboardView({
    super.key,
    required this.imeService,
    required this.theme,
    required this.onOpenSettings,
    this.keyHeight,
    this.onKeyHeightChanged,
  });

  @override
  State<KeyboardView> createState() => _KeyboardViewState();
}

class _KeyboardViewState extends State<KeyboardView> {
  LayoutType _currentLayout = LayoutType.qwerty;
  final bool _showTerminalBar = true;
  bool _isResizeBarOpen = false;
  late double _localKeyHeight;

  double get _effectiveKeyHeight =>
      ImeChannelService.clampKeyHeight(widget.keyHeight ?? _localKeyHeight);

  @override
  void initState() {
    super.initState();
    _localKeyHeight = widget.keyHeight ?? ImeChannelService.defaultKeyHeight;
    if (widget.keyHeight == null) {
      _loadSavedHeight();
    }
  }

  Future<void> _loadSavedHeight() async {
    final saved = await widget.imeService.getKeyboardHeight();
    if (mounted) {
      setState(() {
        _localKeyHeight = saved;
      });
    }
  }

  void _updateKeyHeight(double newHeight) {
    final clamped = ImeChannelService.clampKeyHeight(newHeight);
    setState(() {
      _localKeyHeight = clamped;
    });
    widget.onKeyHeightChanged?.call(clamped);
    widget.imeService.setKeyboardHeight(
      clamped,
      showTerminalBar: _showTerminalBar,
      showResizeBar: _isResizeBarOpen,
    );
  }

  void _toggleResizeBar() {
    widget.imeService.triggerHaptic();
    setState(() {
      _isResizeBarOpen = !_isResizeBarOpen;
    });
    widget.imeService.setKeyboardHeight(
      _effectiveKeyHeight,
      showTerminalBar: _showTerminalBar,
      showResizeBar: _isResizeBarOpen,
    );
  }

  void _handleKeyPress(KeyModel key, GestureDirection direction) {
    widget.imeService.triggerHaptic();

    switch (key.actionType) {
      case KeyActionType.inputChar:
        if (direction == GestureDirection.longPress && key.topLeft == '⚙️') {
          widget.onOpenSettings();
          return;
        }
        final text = key.resolveOutput(direction, isShifted: widget.imeService.isShiftActive);
        if (text == '⚙️') {
          widget.onOpenSettings();
          return;
        }
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

  Widget _buildInlineResizeBar() {
    final currentH = _effectiveKeyHeight;
    final totalDp = ImeChannelService.calculateTotalKeyboardHeightDp(
      currentH,
      showTerminalBar: _showTerminalBar,
      showResizeBar: false,
    );

    return Container(
      height: 40.0,
      padding: const EdgeInsets.symmetric(horizontal: 8.0),
      decoration: BoxDecoration(
        color: widget.theme.keyBackgroundColor.withOpacity(0.95),
        border: Border(
          bottom: BorderSide(color: widget.theme.accentColor.withOpacity(0.4), width: 1.0),
        ),
      ),
      child: Row(
        children: [
          Icon(Icons.height, size: 16, color: widget.theme.accentColor),
          const SizedBox(width: 4),
          Text(
            '${currentH.round()}dp (${totalDp}dp)',
            style: TextStyle(
              color: widget.theme.primaryTextColor,
              fontSize: 11.5,
              fontWeight: FontWeight.bold,
            ),
          ),
          const SizedBox(width: 6),
          InkWell(
            onTap: () => _updateKeyHeight(currentH - 2.0),
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(
                color: widget.theme.backgroundColor,
                borderRadius: BorderRadius.circular(4),
              ),
              child: Text('−', style: TextStyle(color: widget.theme.primaryTextColor, fontWeight: FontWeight.bold)),
            ),
          ),
          Expanded(
            child: SliderTheme(
              data: SliderTheme.of(context).copyWith(
                trackHeight: 3.0,
                thumbShape: const RoundSliderThumbShape(enabledThumbRadius: 7.0),
                overlayShape: const RoundSliderOverlayShape(overlayRadius: 12.0),
              ),
              child: Slider(
                value: currentH,
                min: ImeChannelService.minKeyHeight,
                max: ImeChannelService.maxKeyHeight,
                activeColor: widget.theme.accentColor,
                inactiveColor: widget.theme.modifierInactiveColor,
                onChanged: _updateKeyHeight,
              ),
            ),
          ),
          InkWell(
            onTap: () => _updateKeyHeight(currentH + 2.0),
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(
                color: widget.theme.backgroundColor,
                borderRadius: BorderRadius.circular(4),
              ),
              child: Text('+', style: TextStyle(color: widget.theme.primaryTextColor, fontWeight: FontWeight.bold)),
            ),
          ),
          const SizedBox(width: 6),
          InkWell(
            onTap: () => _updateKeyHeight(ImeChannelService.defaultKeyHeight),
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 4),
              child: Icon(Icons.restart_alt, size: 17, color: widget.theme.cornerTextColor),
            ),
          ),
          InkWell(
            onTap: _toggleResizeBar,
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 4),
              child: Icon(Icons.check_circle, size: 18, color: widget.theme.accentColor),
            ),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final List<List<KeyModel>> rows = (_currentLayout == LayoutType.qwerty)
        ? KeyboardLayout.qwertyLayout
        : KeyboardLayout.symbolLayout;
    final currentKeyHeight = _effectiveKeyHeight;

    return Container(
      color: widget.theme.backgroundColor,
      child: SafeArea(
        top: false,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            if (_isResizeBarOpen) _buildInlineResizeBar(),

            // Top Terminal Modifier Bar (Termux / Hacker Keys + ↕ Resize Button + ⚙️ Settings Button)
            if (_showTerminalBar)
              ModifierBar(
                theme: widget.theme,
                onKeyPress: _handleKeyPress,
                isShiftActive: widget.imeService.isShiftActive,
                isCtrlActive: widget.imeService.isCtrlActive,
                isAltActive: widget.imeService.isAltActive,
                keyHeight: currentKeyHeight,
                isResizeBarOpen: _isResizeBarOpen,
                onToggleResizeBar: _toggleResizeBar,
                onOpenSettings: () {
                  widget.imeService.triggerHaptic();
                  widget.onOpenSettings();
                },
                onVerticalResizeDrag: (deltaDp) {
                  _updateKeyHeight(currentKeyHeight + deltaDp);
                },
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
                        height: currentKeyHeight,
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
