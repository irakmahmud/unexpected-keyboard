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

  @override
  void initState() {
    super.initState();
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
              onThemeChanged: (mode) {
                setState(() => _themeMode = mode);
              },
              hapticsEnabled: _hapticsEnabled,
              onHapticsChanged: (val) {
                setState(() => _hapticsEnabled = val);
              },
              onBackToKeyboard: () {
                setState(() => _isSettingsOpen = false);
              },
            )
          : Scaffold(
              backgroundColor: themeData.backgroundColor,
              appBar: AppBar(
                title: const Text('Unexpected Keyboard'),
                backgroundColor: themeData.keyBackgroundColor,
                actions: [
                  IconButton(
                    icon: const Icon(Icons.settings),
                    tooltip: 'Settings',
                    onPressed: () {
                      setState(() => _isSettingsOpen = true);
                    },
                  ),
                ],
              ),
              body: Column(
                children: [
                  // Test Input Field for in-app typing & gesture preview
                  Expanded(
                    child: Padding(
                      padding: const EdgeInsets.all(16.0),
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
                    padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 4.0),
                    child: Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Text(
                          'Corner Gestures Enabled',
                          style: TextStyle(color: themeData.cornerTextColor, fontSize: 12.0),
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
                    onOpenSettings: () {
                      setState(() => _isSettingsOpen = true);
                    },
                  ),
                ],
              ),
            ),
    );
  }
}
