package com.antigravity.keyboard

import android.content.Context
import android.content.Intent
import android.inputmethodservice.InputMethodService
import android.view.KeyEvent
import android.view.View
import android.view.ViewGroup
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputConnection
import io.flutter.FlutterInjector
import io.flutter.embedding.android.FlutterView
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.embedding.engine.dart.DartExecutor
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel

class FlutterInputMethodService : InputMethodService(), MethodChannel.MethodCallHandler {

    private var flutterEngine: FlutterEngine? = null
    private var flutterView: FlutterView? = null
    private var methodChannel: MethodChannel? = null

    companion object {
        private const val CHANNEL_NAME = "com.antigravity.keyboard/ime"
        private const val PREFS_NAME = "unexpected_keyboard_prefs"
        private const val KEY_ROW_HEIGHT_DP = "key_height_dp"
        private const val KEY_TOTAL_HEIGHT_DP = "total_height_dp"
        private const val KEY_THEME_MODE = "theme_mode"
        private const val KEY_HAPTICS_ENABLED = "haptics_enabled"
        private const val DEFAULT_KEY_HEIGHT_DP = 46.0f
        private const val DEFAULT_TOTAL_HEIGHT_DP = 265
    }

    override fun onCreate() {
        super.onCreate()
        initFlutterEngine()
    }

    private fun initFlutterEngine() {
        if (flutterEngine == null) {
            val loader = FlutterInjector.instance().flutterLoader()
            if (!loader.initialized()) {
                loader.startInitialization(applicationContext)
                loader.ensureInitializationComplete(applicationContext, null)
            }
            flutterEngine = FlutterEngine(this)
            val bundlePath = loader.findAppBundlePath()
            val entrypoint = DartExecutor.DartEntrypoint(bundlePath, "keyboardMain")
            flutterEngine?.dartExecutor?.executeDartEntrypoint(entrypoint)

            methodChannel = MethodChannel(flutterEngine!!.dartExecutor.binaryMessenger, CHANNEL_NAME)
            methodChannel?.setMethodCallHandler(this)
        }
    }

    private fun getSavedKeyHeightDp(): Float {
        val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        return prefs.getFloat(KEY_ROW_HEIGHT_DP, DEFAULT_KEY_HEIGHT_DP).coerceIn(34.0f, 136.0f)
    }

    private fun getSavedTotalHeightDp(): Int {
        val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        return prefs.getInt(KEY_TOTAL_HEIGHT_DP, DEFAULT_TOTAL_HEIGHT_DP).coerceIn(190, 900)
    }

    private fun saveKeyboardHeight(keyHeightDp: Float, totalHeightDp: Int) {
        val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        prefs.edit()
            .putFloat(KEY_ROW_HEIGHT_DP, keyHeightDp.coerceIn(34.0f, 136.0f))
            .putInt(KEY_TOTAL_HEIGHT_DP, totalHeightDp.coerceIn(190, 900))
            .apply()
    }

    private fun applyKeyboardHeightDp(totalHeightDp: Int) {
        val view = flutterView ?: return
        val density = resources.displayMetrics.density
        val heightPx = (density * totalHeightDp.coerceIn(190, 900)).toInt()
        val params = view.layoutParams
        if (params == null) {
            view.layoutParams = ViewGroup.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                heightPx
            )
        } else if (params.height != heightPx) {
            params.height = heightPx
            view.layoutParams = params
            view.requestLayout()
        }
    }

    override fun onCreateInputView(): View {
        initFlutterEngine()
        if (flutterView == null) {
            flutterView = FlutterView(this)
            flutterView?.attachToFlutterEngine(flutterEngine!!)
        } else {
            (flutterView?.parent as? ViewGroup)?.removeView(flutterView)
        }

        applyKeyboardHeightDp(getSavedTotalHeightDp())
        return flutterView!!
    }

    override fun onComputeInsets(outInsets: Insets) {
        super.onComputeInsets(outInsets)
        val view = flutterView ?: return
        val loc = IntArray(2)
        view.getLocationInWindow(loc)
        outInsets.contentTopInsets = loc[1]
        outInsets.visibleTopInsets = loc[1]
        outInsets.touchableInsets = Insets.TOUCHABLE_INSETS_CONTENT
    }

    override fun onStartInputView(info: EditorInfo?, restarting: Boolean) {
        super.onStartInputView(info, restarting)
        val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        val savedKeyHeight = getSavedKeyHeightDp()
        val savedTotalDp = getSavedTotalHeightDp()
        val savedTheme = prefs.getString(KEY_THEME_MODE, "amoled") ?: "amoled"
        val savedHaptics = prefs.getBoolean(KEY_HAPTICS_ENABLED, true)
        applyKeyboardHeightDp(savedTotalDp)
        methodChannel?.invokeMethod("onHeightChanged", savedKeyHeight.toDouble())
        methodChannel?.invokeMethod(
            "onConfigChanged",
            mapOf(
                "keyHeight" to savedKeyHeight.toDouble(),
                "themeMode" to savedTheme,
                "hapticsEnabled" to savedHaptics
            )
        )
    }

    override fun onMethodCall(call: MethodCall, result: MethodChannel.Result) {
        val ic: InputConnection? = currentInputConnection
        val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        when (call.method) {
            "getKeyboardHeight" -> {
                result.success(getSavedKeyHeightDp().toDouble())
            }
            "setKeyboardHeight" -> {
                val keyHeight = call.argument<Double>("keyHeight")?.toFloat() ?: DEFAULT_KEY_HEIGHT_DP
                val totalHeightDp = call.argument<Int>("totalHeightDp") ?: DEFAULT_TOTAL_HEIGHT_DP
                saveKeyboardHeight(keyHeight, totalHeightDp)
                applyKeyboardHeightDp(totalHeightDp)
                result.success(true)
            }
            "getKeyboardConfig" -> {
                result.success(
                    mapOf(
                        "keyHeight" to getSavedKeyHeightDp().toDouble(),
                        "themeMode" to (prefs.getString(KEY_THEME_MODE, "amoled") ?: "amoled"),
                        "hapticsEnabled" to prefs.getBoolean(KEY_HAPTICS_ENABLED, true),
                        "openSettings" to false
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
                try {
                    val intent = Intent(this, MainActivity::class.java).apply {
                        addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP)
                        putExtra("open_settings", true)
                    }
                    startActivity(intent)
                    requestHideSelf(0)
                    result.success(true)
                } catch (e: Exception) {
                    result.error("OPEN_SETTINGS_ERR", e.message, null)
                }
            }
            "commitText" -> {
                val text = call.argument<String>("text") ?: ""
                ic?.commitText(text, 1)
                result.success(true)
            }
            "replaceLastText" -> {
                val text = call.argument<String>("text") ?: ""
                if (ic != null) {
                    ic.beginBatchEdit()
                    ic.deleteSurroundingText(1, 0)
                    ic.commitText(text, 1)
                    ic.endBatchEdit()
                }
                result.success(true)
            }
            "sendKeyEvent" -> {
                val keyCode = call.argument<Int>("keyCode") ?: KeyEvent.KEYCODE_UNKNOWN
                val metaState = call.argument<Int>("metaState") ?: 0
                val eventTime = System.currentTimeMillis()
                val downEvent = KeyEvent(eventTime, eventTime, KeyEvent.ACTION_DOWN, keyCode, 0, metaState)
                val upEvent = KeyEvent(eventTime, eventTime, KeyEvent.ACTION_UP, keyCode, 0, metaState)
                ic?.sendKeyEvent(downEvent)
                ic?.sendKeyEvent(upEvent)
                result.success(true)
            }
            "deleteSurroundingText" -> {
                val before = call.argument<Int>("beforeLength") ?: 1
                val after = call.argument<Int>("afterLength") ?: 0
                if (ic != null) {
                    val selected = ic.getSelectedText(0)
                    if (!selected.isNullOrEmpty()) {
                        ic.commitText("", 1)
                    } else if (!ic.deleteSurroundingText(before, after)) {
                        sendDownUpKeyEvents(KeyEvent.KEYCODE_DEL)
                    }
                } else {
                    sendDownUpKeyEvents(KeyEvent.KEYCODE_DEL)
                }
                result.success(true)
            }
            "sendEnterAction" -> {
                val info = currentInputEditorInfo
                val action = info?.imeOptions?.and(EditorInfo.IME_MASK_ACTION) ?: EditorInfo.IME_ACTION_UNSPECIFIED
                if (ic != null && action != EditorInfo.IME_ACTION_UNSPECIFIED && action != EditorInfo.IME_ACTION_NONE) {
                    ic.performEditorAction(action)
                } else {
                    sendDownUpKeyEvents(KeyEvent.KEYCODE_ENTER)
                }
                result.success(true)
            }
            "sendTextWithModifiers" -> {
                val text = call.argument<String>("text") ?: ""
                val metaState = call.argument<Int>("metaState") ?: 0
                if (text.isNotEmpty()) {
                    val char = text[0]
                    val keyCode = charToKeyCode(char)
                    if (keyCode != KeyEvent.KEYCODE_UNKNOWN) {
                        val eventTime = System.currentTimeMillis()
                        val downEvent = KeyEvent(eventTime, eventTime, KeyEvent.ACTION_DOWN, keyCode, 0, metaState)
                        val upEvent = KeyEvent(eventTime, eventTime, KeyEvent.ACTION_UP, keyCode, 0, metaState)
                        ic?.sendKeyEvent(downEvent)
                        ic?.sendKeyEvent(upEvent)
                    } else {
                        ic?.commitText(text, 1)
                    }
                }
                result.success(true)
            }
            "hideKeyboard" -> {
                requestHideSelf(0)
                result.success(true)
            }
            else -> result.notImplemented()
        }
    }

    private fun charToKeyCode(c: Char): Int {
        return when (c.lowercaseChar()) {
            'a' -> KeyEvent.KEYCODE_A
            'b' -> KeyEvent.KEYCODE_B
            'c' -> KeyEvent.KEYCODE_C
            'd' -> KeyEvent.KEYCODE_D
            'e' -> KeyEvent.KEYCODE_E
            'f' -> KeyEvent.KEYCODE_F
            'g' -> KeyEvent.KEYCODE_G
            'h' -> KeyEvent.KEYCODE_H
            'i' -> KeyEvent.KEYCODE_I
            'j' -> KeyEvent.KEYCODE_J
            'k' -> KeyEvent.KEYCODE_K
            'l' -> KeyEvent.KEYCODE_L
            'm' -> KeyEvent.KEYCODE_M
            'n' -> KeyEvent.KEYCODE_N
            'o' -> KeyEvent.KEYCODE_O
            'p' -> KeyEvent.KEYCODE_P
            'q' -> KeyEvent.KEYCODE_Q
            'r' -> KeyEvent.KEYCODE_R
            's' -> KeyEvent.KEYCODE_S
            't' -> KeyEvent.KEYCODE_T
            'u' -> KeyEvent.KEYCODE_U
            'v' -> KeyEvent.KEYCODE_V
            'w' -> KeyEvent.KEYCODE_W
            'x' -> KeyEvent.KEYCODE_X
            'y' -> KeyEvent.KEYCODE_Y
            'z' -> KeyEvent.KEYCODE_Z
            else -> KeyEvent.KEYCODE_UNKNOWN
        }
    }

    override fun onDestroy() {
        flutterView?.detachFromFlutterEngine()
        flutterView = null
        flutterEngine?.destroy()
        flutterEngine = null
        super.onDestroy()
    }
}
