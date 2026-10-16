package com.antigravity.keyboard

import android.content.Context
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

class MainActivity: FlutterActivity() {
    private val CHANNEL_NAME = "com.antigravity.keyboard/ime"
    private val PREFS_NAME = "unexpected_keyboard_prefs"
    private val KEY_ROW_HEIGHT_DP = "key_height_dp"
    private val KEY_TOTAL_HEIGHT_DP = "total_height_dp"

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, CHANNEL_NAME).setMethodCallHandler { call, result ->
            val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            when (call.method) {
                "getKeyboardHeight" -> {
                    val saved = prefs.getFloat(KEY_ROW_HEIGHT_DP, 46.0f).coerceIn(34.0f, 68.0f)
                    result.success(saved.toDouble())
                }
                "setKeyboardHeight" -> {
                    val keyHeight = call.argument<Double>("keyHeight")?.toFloat() ?: 46.0f
                    val totalHeightDp = call.argument<Int>("totalHeightDp") ?: 265
                    prefs.edit()
                        .putFloat(KEY_ROW_HEIGHT_DP, keyHeight.coerceIn(34.0f, 68.0f))
                        .putInt(KEY_TOTAL_HEIGHT_DP, totalHeightDp.coerceIn(190, 420))
                        .apply()
                    result.success(true)
                }
                "commitText", "sendKeyEvent", "deleteSurroundingText", "sendEnterAction", "sendTextWithModifiers", "hideKeyboard" -> {
                    result.success(true)
                }
                else -> result.notImplemented()
            }
        }
    }
}
