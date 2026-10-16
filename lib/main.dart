import 'package:flutter/material.dart';
import 'app.dart';
import 'services/ime_channel_service.dart';
import 'themes/keyboard_theme.dart';
import 'views/keyboard_view.dart';

/// Standalone launcher application for preview, typing testing, and settings
void main() {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const UnexpectedKeyboardApp());
}

/// Dedicated System IME entrypoint: zero TextField, zero autofocus, zero focus stealing
@pragma('vm:entry-point')
void keyboardMain() {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const StandaloneImeView());
}

class StandaloneImeView extends StatefulWidget {
  const StandaloneImeView({super.key});

  @override
  State<StandaloneImeView> createState() => _StandaloneImeViewState();
}

class _StandaloneImeViewState extends State<StandaloneImeView> {
  final ImeChannelService _imeService = ImeChannelService();
  double _keyHeight = ImeChannelService.defaultKeyHeight;

  @override
  void initState() {
    super.initState();
    _loadHeight();
    _imeService.onHeightUpdatedFromNative = (newHeight) {
      if (mounted) {
        setState(() {
          _keyHeight = newHeight;
        });
      }
    };
  }

  Future<void> _loadHeight() async {
    final saved = await _imeService.getKeyboardHeight();
    if (mounted) {
      setState(() {
        _keyHeight = saved;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final themeData = KeyboardThemeData.fromMode(KeyboardThemeMode.amoled);
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        brightness: Brightness.dark,
        scaffoldBackgroundColor: themeData.backgroundColor,
      ),
      home: Scaffold(
        backgroundColor: themeData.backgroundColor,
        body: Align(
          alignment: Alignment.bottomCenter,
          child: KeyboardView(
            imeService: _imeService,
            theme: themeData,
            keyHeight: _keyHeight,
            onKeyHeightChanged: (val) {
              setState(() {
                _keyHeight = val;
              });
            },
            onOpenSettings: () {},
          ),
        ),
      ),
    );
  }
}
