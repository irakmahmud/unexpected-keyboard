package com.antigravity.keyboard

import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.provider.Settings
import android.view.inputmethod.InputMethodManager
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

class MainActivity: FlutterActivity() {
    private val CHANNEL_NAME = "com.antigravity.keyboard/ime"
    private val PREFS_NAME = "unexpected_keyboard_prefs"
    private val KEY_ROW_HEIGHT_DP = "key_height_dp"
    private val KEY_TOTAL_HEIGHT_DP = "total_height_dp"
    private val KEY_THEME_MODE = "theme_mode"
    private val KEY_HAPTICS_ENABLED = "haptics_enabled"

    private var methodChannel: MethodChannel? = null
    private var openedFromKeyboardSettings: Boolean = false

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        checkOpenSettingsIntent(intent)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        if (checkOpenSettingsIntent(intent)) {
            methodChannel?.invokeMethod("onOpenSettings", true)
        }
    }

    private fun checkOpenSettingsIntent(intent: Intent?): Boolean {
        if (intent == null) return false
        val fromExtra = intent.getBooleanExtra("open_settings", false)
        val isLauncher = intent.categories?.contains(Intent.CATEGORY_LAUNCHER) == true
        if (fromExtra || (!isLauncher && intent.action == Intent.ACTION_MAIN)) {
            openedFromKeyboardSettings = true
            return true
        }
        return false
    }

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        val channel = MethodChannel(flutterEngine.dartExecutor.binaryMessenger, CHANNEL_NAME)
        methodChannel = channel
        channel.setMethodCallHandler { call, result ->
            val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            when (call.method) {
                "getKeyboardHeight" -> {
                    val saved = prefs.getFloat(KEY_ROW_HEIGHT_DP, 46.0f).coerceIn(34.0f, 136.0f)
                    result.success(saved.toDouble())
                }
                "setKeyboardHeight" -> {
                    val keyHeight = call.argument<Double>("keyHeight")?.toFloat() ?: 46.0f
                    val totalHeightDp = call.argument<Int>("totalHeightDp") ?: 265
                    prefs.edit()
                        .putFloat(KEY_ROW_HEIGHT_DP, keyHeight.coerceIn(34.0f, 136.0f))
                        .putInt(KEY_TOTAL_HEIGHT_DP, totalHeightDp.coerceIn(190, 900))
                        .apply()
                    result.success(true)
                }
                "getKeyboardConfig" -> {
                    val shouldOpenSettings = openedFromKeyboardSettings
                    result.success(
                        mapOf(
                            "keyHeight" to prefs.getFloat(KEY_ROW_HEIGHT_DP, 46.0f).coerceIn(34.0f, 136.0f).toDouble(),
                            "themeMode" to (prefs.getString(KEY_THEME_MODE, "amoled") ?: "amoled"),
                            "hapticsEnabled" to prefs.getBoolean(KEY_HAPTICS_ENABLED, true),
                            "openSettings" to shouldOpenSettings
                        )
                    )
                }
                "setKeyboardConfig" -> {
                    val editor = prefs.edit()
                    call.argument<String>("themeMode")?.let { editor.putString(KEY_THEME_MODE, it) }
                    call.argument<Boolean>("hapticsEnabled")?.let { editor.putBoolean(KEY_HAPTICS_ENABLED, it) }
                    editor.apply()
                    result.success(true)
                }
                "openSettings" -> {
                    openedFromKeyboardSettings = true
                    methodChannel?.invokeMethod("onOpenSettings", true)
                    result.success(true)
                }
                "closeSettingsIfExternal" -> {
                    if (openedFromKeyboardSettings) {
                        openedFromKeyboardSettings = false
                        finish()
                        result.success(true)
                    } else {
                        result.success(false)
                    }
                }
                "enableKeyboardInSystemSettings" -> {
                    try {
                        val intent = Intent(Settings.ACTION_INPUT_METHOD_SETTINGS)
                        intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                        startActivity(intent)
                        result.success(true)
                    } catch (e: Exception) {
                        result.error("SETTINGS_ERR", e.message, null)
                    }
                }
                "showInputMethodPicker" -> {
                    try {
                        val imm = getSystemService(Context.INPUT_METHOD_SERVICE) as? InputMethodManager
                        imm?.showInputMethodPicker()
                        result.success(true)
                    } catch (e: Exception) {
                        result.error("IME_PICKER_ERR", e.message, null)
                    }
                }
                "commitText", "replaceLastText", "sendKeyEvent", "deleteSurroundingText", "sendEnterAction", "sendTextWithModifiers", "hideKeyboard" -> {
                    result.success(true)
                }
                else -> result.notImplemented()
            }
        }
    }
}
