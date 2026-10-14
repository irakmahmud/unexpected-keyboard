package com.antigravity.keyboard

import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

class MainActivity: FlutterActivity() {
    private val CHANNEL_NAME = "com.antigravity.keyboard/ime"

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, CHANNEL_NAME).setMethodCallHandler { call, result ->
            // In standalone preview app mode, typing is handled in-app via Dart local callbacks.
            // Acknowledge platform channel calls to prevent MissingPluginException.
            when (call.method) {
                "commitText", "sendKeyEvent", "deleteSurroundingText", "sendEnterAction", "sendTextWithModifiers", "hideKeyboard" -> {
                    result.success(true)
                }
                else -> result.notImplemented()
            }
        }
    }
}
