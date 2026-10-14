import 'package:flutter/material.dart';

enum KeyboardThemeMode {
  amoled,
  terminalGreen,
  cyberpunk,
  materialDark,
}

class KeyboardThemeData {
  final Color backgroundColor;
  final Color keyBackgroundColor;
  final Color keyPressedColor;
  final Color primaryTextColor;
  final Color cornerTextColor;
  final Color modifierActiveColor;
  final Color modifierInactiveColor;
  final Color accentColor;
  final double borderRadius;

  const KeyboardThemeData({
    required this.backgroundColor,
    required this.keyBackgroundColor,
    required this.keyPressedColor,
    required this.primaryTextColor,
    required this.cornerTextColor,
    required this.modifierActiveColor,
    required this.modifierInactiveColor,
    required this.accentColor,
    this.borderRadius = 6.0,
  });

  static KeyboardThemeData fromMode(KeyboardThemeMode mode) {
    switch (mode) {
      case KeyboardThemeMode.amoled:
        return const KeyboardThemeData(
          backgroundColor: Color(0xFF000000),
          keyBackgroundColor: Color(0xFF161618),
          keyPressedColor: Color(0xFF333338),
          primaryTextColor: Color(0xFFFFFFFF),
          cornerTextColor: Color(0xFF8E8E93),
          modifierActiveColor: Color(0xFF0A84FF),
          modifierInactiveColor: Color(0xFF2C2C2E),
          accentColor: Color(0xFF0A84FF),
          borderRadius: 5.0,
        );
      case KeyboardThemeMode.terminalGreen:
        return const KeyboardThemeData(
          backgroundColor: Color(0xFF0D1117),
          keyBackgroundColor: Color(0xFF161B22),
          keyPressedColor: Color(0xFF21262D),
          primaryTextColor: Color(0xFF39D353),
          cornerTextColor: Color(0xFF238636),
          modifierActiveColor: Color(0xFF39D353),
          modifierInactiveColor: Color(0xFF21262D),
          accentColor: Color(0xFF39D353),
          borderRadius: 4.0,
        );
      case KeyboardThemeMode.cyberpunk:
        return const KeyboardThemeData(
          backgroundColor: Color(0xFF0F051D),
          keyBackgroundColor: Color(0xFF1F1138),
          keyPressedColor: Color(0xFF3D1F6D),
          primaryTextColor: Color(0xFF00FFF5),
          cornerTextColor: Color(0xFFFF007F),
          modifierActiveColor: Color(0xFFFF007F),
          modifierInactiveColor: Color(0xFF29174B),
          accentColor: Color(0xFF00FFF5),
          borderRadius: 8.0,
        );
      case KeyboardThemeMode.materialDark:
        return const KeyboardThemeData(
          backgroundColor: Color(0xFF1E1E24),
          keyBackgroundColor: Color(0xFF2B2B36),
          keyPressedColor: Color(0xFF404050),
          primaryTextColor: Color(0xFFEDEDED),
          cornerTextColor: Color(0xFF9E9EA7),
          modifierActiveColor: Color(0xFF6C63FF),
          modifierInactiveColor: Color(0xFF353542),
          accentColor: Color(0xFF6C63FF),
          borderRadius: 6.0,
        );
    }
  }
}
