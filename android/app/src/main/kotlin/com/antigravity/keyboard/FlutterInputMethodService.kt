package com.antigravity.keyboard

import android.inputmethodservice.InputMethodService
import android.os.Build
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

    override fun onCreateInputView(): View {
        initFlutterEngine()
        if (flutterView == null) {
            flutterView = FlutterView(this)
            flutterView?.attachToFlutterEngine(flutterEngine!!)
        } else {
            (flutterView?.parent as? ViewGroup)?.removeView(flutterView)
        }

        val density = resources.displayMetrics.density
        val keyboardHeight = (density * 270).toInt()
        flutterView?.layoutParams = ViewGroup.LayoutParams(
            ViewGroup.LayoutParams.MATCH_PARENT,
            keyboardHeight
        )
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
    }

    override fun onMethodCall(call: MethodCall, result: MethodChannel.Result) {
        val ic: InputConnection? = currentInputConnection
        when (call.method) {
            "commitText" -> {
                val text = call.argument<String>("text") ?: ""
                ic?.commitText(text, 1)
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

    private fun sendDefaultEnter(ic: InputConnection?) {
        sendDownUpKeyEvents(KeyEvent.KEYCODE_ENTER)
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
