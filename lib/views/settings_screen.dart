import 'package:flutter/material.dart';
import '../themes/keyboard_theme.dart';

class SettingsScreen extends StatelessWidget {
  final KeyboardThemeMode currentThemeMode;
  final ValueChanged<KeyboardThemeMode> onThemeChanged;
  final bool hapticsEnabled;
  final ValueChanged<bool> onHapticsChanged;
  final VoidCallback onBackToKeyboard;

  const SettingsScreen({
    super.key,
    required this.currentThemeMode,
    required this.onThemeChanged,
    required this.hapticsEnabled,
    required this.onHapticsChanged,
    required this.onBackToKeyboard,
  });

  @override
  Widget build(BuildContext context) {
    final themeData = KeyboardThemeData.fromMode(currentThemeMode);

    return Scaffold(
      backgroundColor: themeData.backgroundColor,
      appBar: AppBar(
        title: const Text('Unexpected Keyboard Settings'),
        backgroundColor: themeData.keyBackgroundColor,
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: onBackToKeyboard,
        ),
      ),
      body: ListView(
        padding: const EdgeInsets.all(16.0),
        children: [
          _buildSectionHeader('Appearance & Theme', themeData),
          _buildThemeTile(
            title: 'AMOLED Pure Dark',
            mode: KeyboardThemeMode.amoled,
            themeData: themeData,
          ),
          _buildThemeTile(
            title: 'Terminal Hacker Green',
            mode: KeyboardThemeMode.terminalGreen,
            themeData: themeData,
          ),
          _buildThemeTile(
            title: 'Cyberpunk Neon',
            mode: KeyboardThemeMode.cyberpunk,
            themeData: themeData,
          ),
          _buildThemeTile(
            title: 'Material Dark',
            mode: KeyboardThemeMode.materialDark,
            themeData: themeData,
          ),
          const Divider(height: 32),
          _buildSectionHeader('Feedback & Behavior', themeData),
          SwitchListTile(
            title: Text('Haptic Vibration Feedback', style: TextStyle(color: themeData.primaryTextColor)),
            subtitle: Text('Vibrate on key tap and corner swipes', style: TextStyle(color: themeData.cornerTextColor)),
            value: hapticsEnabled,
            activeColor: themeData.accentColor,
            onChanged: onHapticsChanged,
          ),
          const Divider(height: 32),
          _buildSectionHeader('Privacy & Security', themeData),
          Card(
            color: themeData.keyBackgroundColor,
            child: Padding(
              padding: const EdgeInsets.all(16.0),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Icon(Icons.security, color: themeData.accentColor),
                      const SizedBox(width: 8),
                      Text(
                        '100% Offline & Private',
                        style: TextStyle(
                          color: themeData.primaryTextColor,
                          fontWeight: FontWeight.bold,
                          fontSize: 16,
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'This keyboard requires ZERO internet permissions (android.permission.INTERNET is completely absent from the Android Manifest). Everything you type never leaves your device.',
                    style: TextStyle(color: themeData.cornerTextColor, fontSize: 13.5),
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 16),
          Card(
            color: themeData.keyBackgroundColor,
            child: Padding(
              padding: const EdgeInsets.all(16.0),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Icon(Icons.keyboard_alt_outlined, color: themeData.accentColor),
                      const SizedBox(width: 8),
                      Text(
                        'Inspired by Unexpected Keyboard',
                        style: TextStyle(
                          color: themeData.primaryTextColor,
                          fontWeight: FontWeight.bold,
                          fontSize: 16,
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'Built with Flutter & Android IME Platform Channels. Swipe diagonally towards the 4 corners of any key to access numbers, brackets, and special characters effortlessly.',
                    style: TextStyle(color: themeData.cornerTextColor, fontSize: 13.5),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSectionHeader(String title, KeyboardThemeData themeData) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8.0),
      child: Text(
        title.toUpperCase(),
        style: TextStyle(
          color: themeData.accentColor,
          fontWeight: FontWeight.bold,
          fontSize: 12.5,
          letterSpacing: 1.2,
        ),
      ),
    );
  }

  Widget _buildThemeTile({
    required String title,
    required KeyboardThemeMode mode,
    required KeyboardThemeData themeData,
  }) {
    return RadioListTile<KeyboardThemeMode>(
      title: Text(title, style: TextStyle(color: themeData.primaryTextColor)),
      value: mode,
      groupValue: currentThemeMode,
      activeColor: themeData.accentColor,
      onChanged: (newMode) {
        if (newMode != null) onThemeChanged(newMode);
      },
    );
  }
}
