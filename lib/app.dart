import 'package:flutter/material.dart';
import 'models/gesture_direction.dart';
import 'models/key_model.dart';
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
                          autofocus: true,
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
