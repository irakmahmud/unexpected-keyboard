import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
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
  final int? longPressMs;
  final ValueChanged<int>? onLongPressMsChanged;

  const KeyboardView({
    super.key,
    required this.imeService,
    required this.theme,
    required this.onOpenSettings,
    this.keyHeight,
    this.onKeyHeightChanged,
    this.longPressMs,
    this.onLongPressMsChanged,
  });

  @override
  State<KeyboardView> createState() => _KeyboardViewState();
}

class _KeyboardViewState extends State<KeyboardView> {
  LayoutType _currentLayout = LayoutType.qwerty;
  String _currentLayoutMode = 'abcd'; // Default 'abcd' => 2. ABCD Serially
  bool _researchColorsEnabled = true;
  bool _isLayoutMenuOpen = false;
  final bool _showTerminalBar = true;
  bool _isResizeBarOpen = false;
  late double _localKeyHeight;
  late int _localLongPressMs;

  void _forceImmediateFrame() {
    SchedulerBinding.instance.ensureVisualUpdate();
    SchedulerBinding.instance.scheduleForcedFrame();
  }

  double get _effectiveKeyHeight =>
      ImeChannelService.clampKeyHeight(widget.keyHeight ?? _localKeyHeight);

  int get _effectiveLongPressMs =>
      ImeChannelService.clampLongPressMs(widget.longPressMs ?? _localLongPressMs);

  @override
  void initState() {
    super.initState();
    _localKeyHeight = widget.keyHeight ?? ImeChannelService.defaultKeyHeight;
    _localLongPressMs = widget.longPressMs ?? widget.imeService.longPressMs;
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

  void _updateLongPressMs(int newMs) {
    final clamped = ImeChannelService.clampLongPressMs(newMs);
    setState(() {
      _localLongPressMs = clamped;
    });
    widget.onLongPressMsChanged?.call(clamped);
    widget.imeService.setKeyboardConfig(longPressMs: clamped);
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

  void _handleCornerSwipeReplace(String replacementText) {
    widget.imeService.replaceLastText(replacementText);
    widget.imeService.triggerHaptic();
  }

  void _handleKeyPress(KeyModel key, GestureDirection direction) {
    final bool hadSingleUseModifier =
        (widget.imeService.isShiftActive && !widget.imeService.isCapsLock) ||
            widget.imeService.isCtrlActive ||
            widget.imeService.isAltActive ||
            widget.imeService.isMetaActive;

    switch (key.actionType) {
      case KeyActionType.inputChar:
        if (direction == GestureDirection.longPress && key.topLeft == '⚙️') {
          widget.imeService.triggerHaptic();
          widget.onOpenSettings();
          return;
        }
        final text = key.resolveOutput(direction, isShifted: widget.imeService.isShiftActive);
        if (text == '⚙️') {
          widget.imeService.triggerHaptic();
          widget.onOpenSettings();
          return;
        }
        // Dispatch text FIRST for 0.0ms latency, then haptic
        widget.imeService.sendText(text);
        widget.imeService.triggerHaptic();
        if (hadSingleUseModifier) {
          setState(() {});
          _forceImmediateFrame();
        }
        break;

      case KeyActionType.backspace:
        widget.imeService.sendBackspace();
        widget.imeService.triggerHaptic();
        break;

      case KeyActionType.enter:
        widget.imeService.sendEnter();
        widget.imeService.triggerHaptic();
        break;

      case KeyActionType.space:
        widget.imeService.sendText(' ');
        widget.imeService.triggerHaptic();
        if (hadSingleUseModifier) {
          setState(() {});
          _forceImmediateFrame();
        }
        break;

      case KeyActionType.tab:
        widget.imeService.sendKeyCode(61); // KEYCODE_TAB
        widget.imeService.triggerHaptic();
        if (hadSingleUseModifier) {
          setState(() {});
          _forceImmediateFrame();
        }
        break;

      case KeyActionType.escape:
        widget.imeService.sendKeyCode(111); // KEYCODE_ESCAPE
        widget.imeService.triggerHaptic();
        if (hadSingleUseModifier) {
          setState(() {});
          _forceImmediateFrame();
        }
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
        _forceImmediateFrame();
        widget.imeService.triggerHaptic();
        break;

      case KeyActionType.ctrl:
        setState(() {
          widget.imeService.isCtrlActive = !widget.imeService.isCtrlActive;
        });
        _forceImmediateFrame();
        widget.imeService.triggerHaptic();
        break;

      case KeyActionType.alt:
        setState(() {
          widget.imeService.isAltActive = !widget.imeService.isAltActive;
        });
        _forceImmediateFrame();
        widget.imeService.triggerHaptic();
        break;

      case KeyActionType.meta:
        setState(() {
          widget.imeService.isMetaActive = !widget.imeService.isMetaActive;
        });
        _forceImmediateFrame();
        widget.imeService.triggerHaptic();
        break;

      case KeyActionType.arrowLeft:
        widget.imeService.sendKeyCode(21); // KEYCODE_DPAD_LEFT
        widget.imeService.triggerHaptic();
        if (hadSingleUseModifier) {
          setState(() {});
          _forceImmediateFrame();
        }
        break;

      case KeyActionType.arrowRight:
        widget.imeService.sendKeyCode(22); // KEYCODE_DPAD_RIGHT
        widget.imeService.triggerHaptic();
        if (hadSingleUseModifier) {
          setState(() {});
          _forceImmediateFrame();
        }
        break;

      case KeyActionType.arrowUp:
        widget.imeService.sendKeyCode(19); // KEYCODE_DPAD_UP
        widget.imeService.triggerHaptic();
        if (hadSingleUseModifier) {
          setState(() {});
          _forceImmediateFrame();
        }
        break;

      case KeyActionType.arrowDown:
        widget.imeService.sendKeyCode(20); // KEYCODE_DPAD_DOWN
        widget.imeService.triggerHaptic();
        if (hadSingleUseModifier) {
          setState(() {});
          _forceImmediateFrame();
        }
        break;

      case KeyActionType.switchLayout:
        setState(() {
          _currentLayout = (_currentLayout == LayoutType.qwerty)
              ? LayoutType.symbols
              : LayoutType.qwerty;
        });
        _forceImmediateFrame();
        widget.imeService.triggerHaptic();
        break;

      case KeyActionType.switchVersion:
        setState(() {
          _isLayoutMenuOpen = !_isLayoutMenuOpen;
        });
        _forceImmediateFrame();
        widget.imeService.triggerHaptic();
        break;

      case KeyActionType.openSettings:
        widget.imeService.triggerHaptic();
        widget.onOpenSettings();
        break;
    }
  }

  Widget _buildInlineResizeBar() {
    final currentH = _effectiveKeyHeight;
    final currentLp = _effectiveLongPressMs;
    final totalDp = ImeChannelService.calculateTotalKeyboardHeightDp(
      currentH,
      showTerminalBar: _showTerminalBar,
      showResizeBar: false,
    );

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8.0, vertical: 2.0),
      decoration: BoxDecoration(
        color: widget.theme.keyBackgroundColor.withOpacity(0.95),
        border: Border(
          bottom: BorderSide(color: widget.theme.accentColor.withOpacity(0.4), width: 1.0),
        ),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          SizedBox(
            height: 34.0,
            child: Row(
              children: [
                Icon(Icons.height, size: 15, color: widget.theme.accentColor),
                const SizedBox(width: 4),
                Text(
                  '${currentH.round()}dp (${totalDp}dp)',
                  style: TextStyle(
                    color: widget.theme.primaryTextColor,
                    fontSize: 11.0,
                    fontWeight: FontWeight.bold,
                  ),
                ),
                const SizedBox(width: 6),
                InkWell(
                  onTap: () => _updateKeyHeight(currentH - 2.0),
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
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
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
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
                    padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 3),
                    child: Icon(Icons.restart_alt, size: 17, color: widget.theme.cornerTextColor),
                  ),
                ),
                InkWell(
                  onTap: _toggleResizeBar,
                  child: Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 3),
                    child: Icon(Icons.check_circle, size: 18, color: widget.theme.accentColor),
                  ),
                ),
              ],
            ),
          ),
          SizedBox(
            height: 34.0,
            child: Row(
              children: [
                Icon(Icons.timer_outlined, size: 15, color: widget.theme.accentColor),
                const SizedBox(width: 4),
                Text(
                  'Hold: ${currentLp}ms',
                  style: TextStyle(
                    color: widget.theme.primaryTextColor,
                    fontSize: 11.0,
                    fontWeight: FontWeight.bold,
                  ),
                ),
                const SizedBox(width: 6),
                InkWell(
                  onTap: () => _updateLongPressMs(currentLp - 50),
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
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
                      value: currentLp.toDouble(),
                      min: ImeChannelService.minLongPressMs.toDouble(),
                      max: ImeChannelService.maxLongPressMs.toDouble(),
                      activeColor: widget.theme.accentColor,
                      inactiveColor: widget.theme.modifierInactiveColor,
                      onChanged: (val) => _updateLongPressMs(val.round()),
                    ),
                  ),
                ),
                InkWell(
                  onTap: () => _updateLongPressMs(currentLp + 50),
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                    decoration: BoxDecoration(
                      color: widget.theme.backgroundColor,
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Text('+', style: TextStyle(color: widget.theme.primaryTextColor, fontWeight: FontWeight.bold)),
                  ),
                ),
                const SizedBox(width: 6),
                InkWell(
                  onTap: () => _updateLongPressMs(ImeChannelService.defaultLongPressMs),
                  child: Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 3),
                    child: Icon(Icons.restart_alt, size: 17, color: widget.theme.cornerTextColor),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildLayoutPickerPopup() {
    final entries = KeyboardLayout.layoutModeOptions.entries.toList();
    return Positioned.fill(
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: () {
          widget.imeService.triggerHaptic();
          setState(() {
            _isLayoutMenuOpen = false;
          });
        },
        child: Container(
          color: Colors.black.withOpacity(0.55),
          alignment: Alignment.center,
          child: GestureDetector(
            onTap: () {},
            child: Container(
              width: 260.0,
              padding: const EdgeInsets.all(10.0),
              decoration: BoxDecoration(
                color: widget.theme.keyBackgroundColor,
                borderRadius: BorderRadius.circular(12.0),
                border: Border.all(color: widget.theme.accentColor, width: 1.5),
              ),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text(
                        'Select Keyboard Layout',
                        style: TextStyle(
                          color: widget.theme.accentColor,
                          fontSize: 13.0,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      InkWell(
                        onTap: () {
                          widget.imeService.triggerHaptic();
                          setState(() {
                            _isLayoutMenuOpen = false;
                          });
                        },
                        child: Padding(
                          padding: const EdgeInsets.symmetric(horizontal: 6.0, vertical: 2.0),
                          child: Text(
                            '✕',
                            style: TextStyle(
                              color: widget.theme.primaryTextColor,
                              fontSize: 14.0,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 6.0),
                  ...entries.map((entry) {
                    final bool isSelected = _currentLayoutMode == entry.key;
                    return Padding(
                      padding: const EdgeInsets.only(bottom: 4.0),
                      child: InkWell(
                        onTap: () {
                          widget.imeService.triggerHaptic();
                          setState(() {
                            _currentLayoutMode = entry.key;
                            _currentLayout = LayoutType.qwerty;
                            _isLayoutMenuOpen = false;
                          });
                        },
                        child: Container(
                          width: double.infinity,
                          padding: const EdgeInsets.symmetric(horizontal: 10.0, vertical: 7.0),
                          decoration: BoxDecoration(
                            color: isSelected
                                ? widget.theme.modifierActiveColor
                                : widget.theme.backgroundColor,
                            borderRadius: BorderRadius.circular(6.0),
                            border: Border.all(
                              color: isSelected
                                  ? widget.theme.accentColor
                                  : widget.theme.modifierInactiveColor,
                              width: isSelected ? 1.2 : 0.6,
                            ),
                          ),
                          child: Text(
                            isSelected ? '✓  ${entry.value}' : '    ${entry.value}',
                            style: TextStyle(
                              color: isSelected ? Colors.white : widget.theme.primaryTextColor,
                              fontSize: 12.5,
                              fontWeight: isSelected ? FontWeight.bold : FontWeight.w500,
                            ),
                          ),
                        ),
                      ),
                    );
                  }),
                  Padding(
                    padding: const EdgeInsets.only(top: 2.0),
                    child: InkWell(
                      onTap: () {
                        widget.imeService.triggerHaptic();
                        setState(() {
                          _researchColorsEnabled = !_researchColorsEnabled;
                        });
                      },
                      child: Container(
                        width: double.infinity,
                        padding: const EdgeInsets.symmetric(horizontal: 10.0, vertical: 7.0),
                        decoration: BoxDecoration(
                          color: _researchColorsEnabled
                              ? widget.theme.modifierActiveColor
                              : widget.theme.backgroundColor,
                          borderRadius: BorderRadius.circular(6.0),
                          border: Border.all(
                            color: _researchColorsEnabled
                                ? widget.theme.accentColor
                                : widget.theme.modifierInactiveColor,
                            width: _researchColorsEnabled ? 1.2 : 0.6,
                          ),
                        ),
                        child: Text(
                          _researchColorsEnabled
                              ? '✓  🎨 Button Color: ON (Tap to OFF)'
                              : '    🎨 Button Color: OFF (Tap to ON)',
                          style: TextStyle(
                            color: _researchColorsEnabled ? Colors.white : widget.theme.primaryTextColor,
                            fontSize: 12.5,
                            fontWeight: _researchColorsEnabled ? FontWeight.bold : FontWeight.w500,
                          ),
                        ),
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final List<List<KeyModel>> rows = (_currentLayout == LayoutType.qwerty)
        ? (_currentLayoutMode == 'qwerty'
            ? KeyboardLayout.version1Layout
            : (_currentLayoutMode == 'right_hand'
                ? KeyboardLayout.rightHandBanglishLayout
                : KeyboardLayout.version2Layout))
        : KeyboardLayout.symbolLayout;
    final currentKeyHeight = _effectiveKeyHeight;
    final effectiveRowHeight = currentKeyHeight * (7.0 / rows.length);

    return Container(
      color: widget.theme.backgroundColor,
      child: SafeArea(
        top: false,
        child: Stack(
          children: [
            Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                if (_isResizeBarOpen) _buildInlineResizeBar(),

                // Top Terminal Modifier Bar (Termux / Hacker Keys + ⊞ Layout Menu + 🌐 Switch Keyboard + ↕ Resize + ⚙️ Settings)
                if (_showTerminalBar)
                  ModifierBar(
                    theme: widget.theme,
                    onKeyPress: _handleKeyPress,
                    isShiftActive: widget.imeService.isShiftActive,
                    isCtrlActive: widget.imeService.isCtrlActive,
                    isAltActive: widget.imeService.isAltActive,
                    isSymbolLayout: _currentLayout == LayoutType.symbols,
                    isLayoutMenuOpen: _isLayoutMenuOpen,
                    keyHeight: currentKeyHeight,
                    isResizeBarOpen: _isResizeBarOpen,
                    onToggleResizeBar: _toggleResizeBar,
                    onSwitchKeyboard: () {
                      widget.imeService.triggerHaptic();
                      widget.imeService.showInputMethodPicker();
                    },
                    onOpenSettings: () {
                      widget.imeService.triggerHaptic();
                      widget.onOpenSettings();
                    },
                    onVerticalResizeDrag: (deltaDp) {
                      _updateKeyHeight(currentKeyHeight + deltaDp);
                    },
                  ),

                // Keyboard Character Rows (Zero gaps between keys — maximum key area, middle-aligned)
                Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: rows.map((row) {
                    return Row(
                      mainAxisAlignment: MainAxisAlignment.center,
                      crossAxisAlignment: CrossAxisAlignment.center,
                      children: row.map((key) {
                        return KeyWidget(
                          keyModel: key,
                          theme: widget.theme,
                          onKeyPress: _handleKeyPress,
                          onCornerSwipeReplace: _handleCornerSwipeReplace,
                          onSpaceLongPress: key.actionType == KeyActionType.space
                              ? () {
                                  widget.imeService.sendBackspace();
                                  widget.imeService.triggerHaptic();
                                  widget.imeService.showInputMethodPicker();
                                }
                              : null,
                          isShiftActive: widget.imeService.isShiftActive,
                          isCtrlActive: widget.imeService.isCtrlActive,
                          isAltActive: widget.imeService.isAltActive,
                          researchColorsEnabled: _researchColorsEnabled,
                          height: effectiveRowHeight,
                          longPressMs: _effectiveLongPressMs,
                        );
                      }).toList(),
                    );
                  }).toList(),
                ),
              ],
            ),
            if (_isLayoutMenuOpen) _buildLayoutPickerPopup(),
          ],
        ),
      ),
    );
  }
}
