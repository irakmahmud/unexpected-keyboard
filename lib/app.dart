import 'package:flutter/material.dart';
import 'services/ime_channel_service.dart';
import 'themes/keyboard_theme.dart';
import 'views/keyboard_view.dart';
import 'views/settings_screen.dart';

class UnexpectedKeyboardApp extends StatefulWidget {
  const UnexpectedKeyboardApp({super.key});

  @override
  State<UnexpectedKeyboardApp> createState() => _UnexpectedKeyboardAppState();
}

class _UnexpectedKeyboardAppState extends State<UnexpectedKeyboardApp> {
  final ImeChannelService _imeService = ImeChannelService();
  final TextEditingController _testTextController = TextEditingController();
  KeyboardThemeMode _themeMode = KeyboardThemeMode.amoled;
  bool _hapticsEnabled = true;
  bool _isSettingsOpen = false;
  double _keyHeight = ImeChannelService.defaultKeyHeight;

  @override
  void initState() {
    super.initState();
    _loadInitialConfig();

    _imeService.onHeightUpdatedFromNative = (newHeight) {
      if (mounted) {
        setState(() {
          _keyHeight = newHeight;
        });
      }
    };

    _imeService.onThemeUpdatedFromNative = (themeName) {
      if (mounted) {
        setState(() {
          _themeMode = KeyboardThemeData.modeFromString(themeName);
        });
      }
    };

    _imeService.onHapticsUpdatedFromNative = (enabled) {
      if (mounted) {
        setState(() {
          _hapticsEnabled = enabled;
        });
      }
    };

    _imeService.onOpenSettingsFromNative = () {
      if (mounted) {
        setState(() {
          _isSettingsOpen = true;
        });
      }
    };

    _imeService.onLocalTextInput = (text) {
      final val = _testTextController.text;
      final sel = _testTextController.selection;
      final start = sel.start < 0 ? val.length : sel.start;
      final end = sel.end < 0 ? val.length : sel.end;
      final newText = val.replaceRange(start, end, text);
      _testTextController.value = TextEditingValue(
        text: newText,
        selection: TextSelection.collapsed(offset: start + text.length),
      );
    };

    _imeService.onLocalBackspace = () {
      final val = _testTextController.text;
      final sel = _testTextController.selection;
      if (val.isEmpty) return;
      if (sel.start != sel.end && sel.start >= 0) {
        final newText = val.replaceRange(sel.start, sel.end, '');
        _testTextController.value = TextEditingValue(
          text: newText,
          selection: TextSelection.collapsed(offset: sel.start),
        );
      } else {
        final start = sel.start <= 0 ? val.length : sel.start;
        if (start > 0) {
          final newText = val.replaceRange(start - 1, start, '');
          _testTextController.value = TextEditingValue(
            text: newText,
            selection: TextSelection.collapsed(offset: start - 1),
          );
        }
      }
    };

    _imeService.onLocalEnter = () {
      final val = _testTextController.text;
      final sel = _testTextController.selection;
      final start = sel.start < 0 ? val.length : sel.start;
      final end = sel.end < 0 ? val.length : sel.end;
      final newText = val.replaceRange(start, end, '\n');
      _testTextController.value = TextEditingValue(
        text: newText,
        selection: TextSelection.collapsed(offset: start + 1),
      );
    };
  }

  Future<void> _loadInitialConfig() async {
    final config = await _imeService.getKeyboardConfig();
    if (mounted) {
      setState(() {
        final rawH = config['keyHeight'];
        if (rawH is num) {
          _keyHeight = ImeChannelService.clampKeyHeight(rawH.toDouble());
        }
        final rawTheme = config['themeMode'];
        if (rawTheme is String) {
          _themeMode = KeyboardThemeData.modeFromString(rawTheme);
        }
        final rawHaptics = config['hapticsEnabled'];
        if (rawHaptics is bool) {
          _hapticsEnabled = rawHaptics;
        }
        if (config['openSettings'] == true) {
          _isSettingsOpen = true;
        }
      });
    }
  }

  void _updateKeyHeight(double newHeight) {
    final clamped = ImeChannelService.clampKeyHeight(newHeight);
    setState(() {
      _keyHeight = clamped;
    });
    _imeService.setKeyboardHeight(clamped);
  }

  void _updateThemeMode(KeyboardThemeMode mode) {
    setState(() {
      _themeMode = mode;
    });
    _imeService.setKeyboardConfig(themeMode: mode.name);
  }

  void _updateHaptics(bool enabled) {
    setState(() {
      _hapticsEnabled = enabled;
    });
    _imeService.setKeyboardConfig(haptics: enabled);
  }

  @override
  void dispose() {
    _testTextController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final themeData = KeyboardThemeData.fromMode(_themeMode);

    return MaterialApp(
      title: 'Unexpected Keyboard',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        brightness: Brightness.dark,
        scaffoldBackgroundColor: themeData.backgroundColor,
        useMaterial3: true,
      ),
      home: _isSettingsOpen
          ? SettingsScreen(
              currentThemeMode: _themeMode,
              onThemeChanged: _updateThemeMode,
              hapticsEnabled: _hapticsEnabled,
              onHapticsChanged: _updateHaptics,
              keyHeight: _keyHeight,
              onKeyHeightChanged: _updateKeyHeight,
              onBackToKeyboard: () async {
                final closedExternal = await _imeService.closeSettingsIfExternal();
                if (!closedExternal && mounted) {
                  setState(() => _isSettingsOpen = false);
                }
              },
            )
          : Scaffold(
              backgroundColor: themeData.backgroundColor,
              appBar: AppBar(
                title: const Text('Unexpected Keyboard'),
                backgroundColor: themeData.keyBackgroundColor,
                actions: [
                  IconButton(
                    icon: const Icon(Icons.height),
                    tooltip: 'Adjust Keyboard Height',
                    onPressed: () {
                      setState(() => _isSettingsOpen = true);
                    },
                  ),
                  IconButton(
                    icon: const Icon(Icons.settings),
                    tooltip: 'Settings',
                    onPressed: () {
                      setState(() => _isSettingsOpen = true);
                    },
                  ),
                ],
              ),
              body: LayoutBuilder(
                builder: (context, constraints) {
                  final totalKbHeight = ImeChannelService.calculateTotalKeyboardHeightDp(_keyHeight).toDouble();
                  final topAreaHeight = (constraints.maxHeight - totalKbHeight - 44.0).clamp(56.0, 400.0);
                  return SingleChildScrollView(
                    child: ConstrainedBox(
                      constraints: BoxConstraints(minHeight: constraints.maxHeight),
                      child: Column(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          // Test Input Field for in-app typing & gesture preview
                          SizedBox(
                            height: topAreaHeight,
                            child: Padding(
                              padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 8.0),
                              child: Container(
                                decoration: BoxDecoration(
                                  color: themeData.keyBackgroundColor.withOpacity(0.5),
                                  borderRadius: BorderRadius.circular(12.0),
                                  border: Border.all(color: themeData.modifierInactiveColor),
                                ),
                                padding: const EdgeInsets.all(12.0),
                                child: TextField(
                                  controller: _testTextController,
                                  maxLines: null,
                                  autofocus: false,
                                  readOnly: false,
                                  showCursor: true,
                                  style: TextStyle(color: themeData.primaryTextColor, fontSize: 16.0, fontFamily: 'monospace'),
                                  decoration: InputDecoration(
                                    border: InputBorder.none,
                                    hintText: 'Tap or swipe diagonally on keys below to test corner symbols...',
                                    hintStyle: TextStyle(color: themeData.cornerTextColor),
                                  ),
                                ),
                              ),
                            ),
                          ),

                          // Quick test buttons row
                          Padding(
                            padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 2.0),
                            child: Row(
                              mainAxisAlignment: MainAxisAlignment.spaceBetween,
                              children: [
                                Flexible(
                                  child: Text(
                                    'Key Height: ${_keyHeight.round()} dp (Tap ↕ on bar to resize)',
                                    style: TextStyle(color: themeData.cornerTextColor, fontSize: 12.0),
                                    overflow: TextOverflow.ellipsis,
                                  ),
                                ),
                                TextButton.icon(
                                  onPressed: () => _testTextController.clear(),
                                  icon: const Icon(Icons.clear_all, size: 16),
                                  label: const Text('Clear'),
                                  style: TextButton.styleFrom(foregroundColor: themeData.accentColor),
                                ),
                              ],
                            ),
                          ),

                          // The Interactive Keyboard View
                          KeyboardView(
                            imeService: _imeService,
                            theme: themeData,
                            keyHeight: _keyHeight,
                            onKeyHeightChanged: _updateKeyHeight,
                            onOpenSettings: () {
                              setState(() => _isSettingsOpen = true);
                            },
                          ),
                        ],
                      ),
                    ),
                  );
                },
              ),
            ),
    );
  }
}
