package com.antigravity.keyboard

import android.content.Context
import android.content.Intent
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.RectF
import android.graphics.Typeface
import android.inputmethodservice.InputMethodService
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.provider.Settings
import android.view.HapticFeedbackConstants
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.View
import android.view.ViewGroup
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputConnection
import android.view.inputmethod.InputMethodManager
import kotlin.math.abs
import kotlin.math.hypot
import kotlin.math.max
import kotlin.math.min

class FlutterInputMethodService : InputMethodService() {

    private var keyboardView: NativeKeyboardView? = null

    companion object {
        const val PREFS_NAME = "unexpected_keyboard_prefs"
        const val KEY_ROW_HEIGHT_DP = "key_height_dp"
        const val KEY_TOTAL_HEIGHT_DP = "total_height_dp"
        const val KEY_THEME_MODE = "theme_mode"
        const val KEY_HAPTICS_ENABLED = "haptics_enabled"
        const val KEY_LONG_PRESS_MS = "long_press_ms"
        const val KEY_LAYOUT_MODE = "layout_mode"
        const val KEY_RESEARCH_COLORS = "research_colors_enabled"
        const val DEFAULT_KEY_HEIGHT_DP = 42.0f
        const val DEFAULT_LONG_PRESS_MS = 400
        const val DEFAULT_LAYOUT_MODE = "abcd"
        const val DEFAULT_RESEARCH_COLORS = true
    }

    override fun onEvaluateFullscreenMode(): Boolean = false

    override fun onComputeInsets(outInsets: Insets) {
        super.onComputeInsets(outInsets)
        if (!isFullscreenMode) {
            outInsets.contentTopInsets = outInsets.visibleTopInsets
            outInsets.touchableInsets = Insets.TOUCHABLE_INSETS_CONTENT
        }
    }

    override fun onCreateInputView(): View {
        val view = NativeKeyboardView(this, this)
        keyboardView = view
        view.reloadConfigFromPrefs()
        return view
    }

    override fun onStartInput(attribute: EditorInfo?, restarting: Boolean) {
        super.onStartInput(attribute, restarting)
        try {
            currentInputConnection?.finishComposingText()
        } catch (_: Exception) {}
    }

    override fun onStartInputView(info: EditorInfo?, restarting: Boolean) {
        super.onStartInputView(info, restarting)
        try {
            currentInputConnection?.finishComposingText()
        } catch (_: Exception) {}
        keyboardView?.reloadConfigFromPrefs()
    }

    override fun onWindowShown() {
        super.onWindowShown()
        keyboardView?.reloadConfigFromPrefs()
    }

    fun getSavedKeyHeightDp(): Float {
        val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        return prefs.getFloat(KEY_ROW_HEIGHT_DP, DEFAULT_KEY_HEIGHT_DP).coerceIn(34.0f, 136.0f)
    }

    fun saveKeyHeightDp(keyHeightDp: Float, isResizeBarOpen: Boolean) {
        val clamped = keyHeightDp.coerceIn(34.0f, 136.0f)
        val modH = (clamped * 0.85f).coerceIn(36.0f, 68.0f)
        val resizeH = if (isResizeBarOpen) 72.0f else 0.0f
        val totalDp = (7.0f * clamped + modH + resizeH + 4.0f).toInt().coerceIn(220, 1080)
        val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        prefs.edit()
            .putFloat(KEY_ROW_HEIGHT_DP, clamped)
            .putInt(KEY_TOTAL_HEIGHT_DP, totalDp)
            .apply()
    }

    fun getSavedLongPressMs(): Int {
        val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        return prefs.getInt(KEY_LONG_PRESS_MS, DEFAULT_LONG_PRESS_MS).coerceIn(0, 3000)
    }

    fun saveLongPressMs(ms: Int) {
        val clamped = ms.coerceIn(0, 3000)
        val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        prefs.edit().putInt(KEY_LONG_PRESS_MS, clamped).apply()
    }

    fun isHapticsEnabled(): Boolean {
        val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        return prefs.getBoolean(KEY_HAPTICS_ENABLED, true)
    }

    fun getSavedThemeMode(): String {
        val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        return prefs.getString(KEY_THEME_MODE, "amoled") ?: "amoled"
    }

    fun getSavedLayoutMode(): String {
        val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        val raw = prefs.getString(KEY_LAYOUT_MODE, DEFAULT_LAYOUT_MODE) ?: DEFAULT_LAYOUT_MODE
        return when (raw) {
            "qwerty", "abcd", "left_hand", "right_hand", "both_hand" -> raw
            else -> DEFAULT_LAYOUT_MODE
        }
    }

    fun saveLayoutMode(mode: String) {
        val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        prefs.edit().putString(KEY_LAYOUT_MODE, mode).apply()
    }

    fun isResearchColorsEnabled(): Boolean {
        val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        return prefs.getBoolean(KEY_RESEARCH_COLORS, DEFAULT_RESEARCH_COLORS)
    }

    fun saveResearchColorsEnabled(enabled: Boolean) {
        val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        prefs.edit().putBoolean(KEY_RESEARCH_COLORS, enabled).apply()
    }

    fun commitTextImmediate(text: String, metaState: Int, useModifiers: Boolean) {
        val ic: InputConnection = currentInputConnection ?: return
        if (useModifiers && text.isNotEmpty()) {
            val keyCode = charToKeyCode(text[0])
            if (keyCode != KeyEvent.KEYCODE_UNKNOWN) {
                val now = System.currentTimeMillis()
                ic.sendKeyEvent(KeyEvent(now, now, KeyEvent.ACTION_DOWN, keyCode, 0, metaState))
                ic.sendKeyEvent(KeyEvent(now, now, KeyEvent.ACTION_UP, keyCode, 0, metaState))
                return
            }
        }
        try {
            ic.finishComposingText()
        } catch (_: Exception) {}
        ic.commitText(text, 1)
    }

    fun replaceLastTextImmediate(text: String) {
        val ic: InputConnection = currentInputConnection ?: return
        ic.beginBatchEdit()
        try {
            ic.finishComposingText()
        } catch (_: Exception) {}
        ic.deleteSurroundingText(1, 0)
        ic.commitText(text, 1)
        ic.endBatchEdit()
    }

    fun sendBackspaceImmediate() {
        val ic: InputConnection? = currentInputConnection
        if (ic != null) {
            try {
                ic.finishComposingText()
            } catch (_: Exception) {}
            val selected = ic.getSelectedText(0)
            if (!selected.isNullOrEmpty()) {
                ic.commitText("", 1)
            } else if (!ic.deleteSurroundingText(1, 0)) {
                sendDownUpKeyEvents(KeyEvent.KEYCODE_DEL)
            }
        } else {
            sendDownUpKeyEvents(KeyEvent.KEYCODE_DEL)
        }
    }

    fun sendEnterImmediate() {
        val ic: InputConnection? = currentInputConnection
        val info = currentInputEditorInfo
        val inputType = info?.inputType ?: EditorInfo.TYPE_NULL
        if (ic != null && inputType != EditorInfo.TYPE_NULL) {
            try {
                ic.finishComposingText()
            } catch (_: Exception) {}
            // Always insert a literal newline "\n" so the keyboard NEVER hides
            // (calling performEditorAction(IME_ACTION_DONE) closes the keyboard, and
            // sending raw KEYCODE_ENTER in WebViews/chat boxes clears/submits the message!)
            ic.commitText("\n", 1)
        } else {
            sendDownUpKeyEvents(KeyEvent.KEYCODE_ENTER)
        }
    }

    fun sendEditorActionImmediate() {
        val ic: InputConnection? = currentInputConnection
        val info = currentInputEditorInfo
        val action = info?.imeOptions?.and(EditorInfo.IME_MASK_ACTION) ?: EditorInfo.IME_ACTION_UNSPECIFIED
        if (ic != null && action != EditorInfo.IME_ACTION_UNSPECIFIED && action != EditorInfo.IME_ACTION_NONE && action != EditorInfo.IME_ACTION_DONE) {
            ic.performEditorAction(action)
        } else {
            sendDownUpKeyEvents(KeyEvent.KEYCODE_ENTER)
        }
    }

    fun sendKeyCodeImmediate(keyCode: Int, metaState: Int) {
        val ic: InputConnection? = currentInputConnection
        if (ic != null) {
            val now = System.currentTimeMillis()
            ic.sendKeyEvent(KeyEvent(now, now, KeyEvent.ACTION_DOWN, keyCode, 0, metaState))
            ic.sendKeyEvent(KeyEvent(now, now, KeyEvent.ACTION_UP, keyCode, 0, metaState))
        } else {
            sendDownUpKeyEvents(keyCode)
        }
    }

    fun showSystemKeyboardPicker() {
        try {
            val imm = getSystemService(Context.INPUT_METHOD_SERVICE) as? InputMethodManager
            imm?.showInputMethodPicker()
        } catch (_: Exception) {}
    }

    fun openKeyboardSettings() {
        try {
            val intent = Intent(this, MainActivity::class.java).apply {
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP)
                putExtra("open_settings", true)
            }
            startActivity(intent)
            requestHideSelf(0)
        } catch (_: Exception) {}
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
}

enum class NativeActionType {
    INPUT_CHAR,
    SPACE,
    BACKSPACE,
    ENTER,
    SHIFT,
    CTRL,
    ALT,
    ESCAPE,
    ARROW_LEFT,
    ARROW_DOWN,
    ARROW_UP,
    ARROW_RIGHT,
    SWITCH_LAYOUT,
    OPEN_LAYOUT_MENU,
    CLOSE_LAYOUT_MENU,
    SELECT_LAYOUT_QWERTY,
    SELECT_LAYOUT_ABCD,
    SELECT_LAYOUT_LEFT,
    SELECT_LAYOUT_RIGHT,
    SELECT_LAYOUT_BOTH,
    TOGGLE_RESEARCH_COLORS,
    SWITCH_IME,
    TOGGLE_RESIZE,
    OPEN_SETTINGS,
    RESIZE_MINUS,
    RESIZE_PLUS,
    RESIZE_RESET,
    RESIZE_DONE,
    LONG_PRESS_MINUS,
    LONG_PRESS_PLUS,
    LONG_PRESS_RESET,
    LONG_PRESS_ZERO
}

data class NativeKeySpec(
    val primary: String,
    val topLeft: String? = null,
    val topRight: String? = null,
    val bottomLeft: String? = null,
    val bottomRight: String? = null,
    val action: NativeActionType = NativeActionType.INPUT_CHAR,
    val flex: Float = 1.0f,
    val keyCode: Int = 0
)

private class HitRect(
    val spec: NativeKeySpec,
    val rect: RectF,
    val isTopBar: Boolean = false
)

class NativeKeyboardView(
    context: Context,
    private val service: FlutterInputMethodService
) : View(context) {

    private var keyHeightDp: Float = 42.0f
    private var longPressMs: Int = 400
    private var themeMode: String = "amoled"
    private var hapticsEnabled: Boolean = true

    private var isShiftActive: Boolean = false
    private var isCapsLock: Boolean = false
    private var isCtrlActive: Boolean = false
    private var isAltActive: Boolean = false
    private var isSymbolLayout: Boolean = false
    private var layoutMode: String = "abcd" // Default "abcd" => 2. ABCD Serially
    private var researchColorsEnabled: Boolean = true
    private var isLayoutMenuOpen: Boolean = false
    private var isResizeBarOpen: Boolean = false

    // Theme colors
    private var bgColor: Int = 0xFF000000.toInt()
    private var keyBgColor: Int = 0xFF141414.toInt()
    private var keyPressedColor: Int = 0xFF2C2C2C.toInt()
    private var modActiveColor: Int = 0xFF00E676.toInt()
    private var modInactiveColor: Int = 0xFF262626.toInt()
    private var primaryTextColor: Int = 0xFFFFFFFF.toInt()
    private var cornerTextColor: Int = 0xFF9E9E9E.toInt()
    private var accentColor: Int = 0xFF00E676.toInt()

    private val bgPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply { style = Paint.Style.FILL }
    private val borderPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply { style = Paint.Style.STROKE }
    private val primaryTextPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        textAlign = Paint.Align.CENTER
        typeface = Typeface.create("sans-serif-thin", Typeface.NORMAL)
    }
    private val multiCharTextPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        textAlign = Paint.Align.CENTER
        typeface = Typeface.create("sans-serif", Typeface.BOLD)
    }
    private val cornerTextPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        typeface = Typeface.create("sans-serif-thin", Typeface.NORMAL)
    }
    private val popupBgPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply { style = Paint.Style.FILL }
    private val popupBorderPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        style = Paint.Style.STROKE
        color = 0xFFFFFFFF.toInt()
    }
    private val popupTextPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        textAlign = Paint.Align.CENTER
        typeface = Typeface.create("sans-serif", Typeface.BOLD)
    }
    private val menuItemTextPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        textAlign = Paint.Align.LEFT
        typeface = Typeface.create("sans-serif-medium", Typeface.NORMAL)
    }

    private val hitRects = ArrayList<HitRect>(80)
    private val menuHitRects = ArrayList<HitRect>(8)
    private var menuCardRect = RectF()

    // Active pointer tracking
    private var activePointerId: Int = MotionEvent.INVALID_POINTER_ID
    private var activeHit: HitRect? = null
    private var startX: Float = 0f
    private var startY: Float = 0f
    private var activeCornerDir: Int = 0 // 0=center, 1=TL, 2=TR, 3=BL, 4=BR
    private var wasShiftedOnDown: Boolean = false
    private var wasModifierComboOnDown: Boolean = false
    private var currentCommittedOutput: String = ""
    private var popupLabel: String? = null
    private var popupAnchorRect: RectF? = null
    private var isDraggingResizeButton: Boolean = false
    private var resizeDragLastY: Float = 0f

    private val handler = Handler(Looper.getMainLooper())
    private var repeatRunnable: Runnable? = null
    private var spaceLongPressRunnable: Runnable? = null
    private var charLongPressRunnable: Runnable? = null

    private val topBarSpecs = listOf(
        NativeKeySpec("ESC", action = NativeActionType.ESCAPE, flex = 0.95f, keyCode = 111),
        NativeKeySpec("CTRL", action = NativeActionType.CTRL, flex = 1.05f, keyCode = 113),
        NativeKeySpec("ALT", action = NativeActionType.ALT, flex = 0.95f, keyCode = 57),
        NativeKeySpec("◀", action = NativeActionType.ARROW_LEFT, flex = 0.8f, keyCode = 21),
        NativeKeySpec("▼", action = NativeActionType.ARROW_DOWN, flex = 0.8f, keyCode = 20),
        NativeKeySpec("▲", action = NativeActionType.ARROW_UP, flex = 0.8f, keyCode = 19),
        NativeKeySpec("▶", action = NativeActionType.ARROW_RIGHT, flex = 0.8f, keyCode = 22),
        NativeKeySpec("⊞", action = NativeActionType.OPEN_LAYOUT_MENU, flex = 0.9f),
        NativeKeySpec("🌐", action = NativeActionType.SWITCH_IME, flex = 0.85f),
        NativeKeySpec("↕", action = NativeActionType.TOGGLE_RESIZE, flex = 0.85f),
        NativeKeySpec("⚙️", action = NativeActionType.OPEN_SETTINGS, flex = 0.85f)
    )

    // Row 0 (Directly above Number Row): Shift, ?123, ?, /, ,, ., Space, Backspace (Cross), Enter
    private val actionSymbolRow = listOf(
        NativeKeySpec("⇧", action = NativeActionType.SHIFT, flex = 1.0f),
        NativeKeySpec("?123", action = NativeActionType.SWITCH_LAYOUT, flex = 1.0f),
        NativeKeySpec("?", "!", "¿", "¡", "@", flex = 0.85f),
        NativeKeySpec("/", "\\", "|", "-", "_", flex = 0.85f),
        NativeKeySpec(",", "<", ";", "'", "\"", flex = 0.85f),
        NativeKeySpec(".", ">", ":", "(", ")", flex = 0.85f),
        NativeKeySpec(" ", topRight = "🌐", action = NativeActionType.SPACE, flex = 1.5f),
        NativeKeySpec("⌫", action = NativeActionType.BACKSPACE, flex = 1.0f, keyCode = 67),
        NativeKeySpec("↵", action = NativeActionType.ENTER, flex = 1.0f, keyCode = 66)
    )

    // Row 1 (Number Row): 1 2 3 4 5 6 7 8 9 0
    private val numberRow = listOf(
        NativeKeySpec("1", "!", "~", "`", "¡"),
        NativeKeySpec("2", "@", "#", "<", ">"),
        NativeKeySpec("3", "#", "$", "{", "}"),
        NativeKeySpec("4", "$", "€", "[", "]"),
        NativeKeySpec("5", "%", "‰", "(", ")"),
        NativeKeySpec("6", "^", "&", "=", "+"),
        NativeKeySpec("7", "&", "*", "-", "_"),
        NativeKeySpec("8", "*", "+", "/", "\\"),
        NativeKeySpec("9", "(", "[", "{", "«"),
        NativeKeySpec("0", ")", "]", "}", "»")
    )

    // VERSION 1 LAYER ALIGNMENT (Activated by clicking the V2/V1 Layer button):
    // Row 0: Action & Symbol Row above Number Row (⇧, ?123, ?, /, ,, ., Space, ⌫, ↵)
    // Row 1: Number Row (1..0)
    // Rows 2..6: Version 1 Letter Alignment (q..y, a..h, z..n, u i j k m o, and , . Space p l ⌫ ↵)
    private val version1Rows = listOf(
        actionSymbolRow,
        numberRow,
        listOf(
            NativeKeySpec("q", "1", "`", "~", "\\"),
            NativeKeySpec("w", "2", "|", "<", ">"),
            NativeKeySpec("e", "3", "€", "{", "}"),
            NativeKeySpec("r", "4", "₹", "[", "]"),
            NativeKeySpec("t", "5", "%", "(", ")"),
            NativeKeySpec("y", "6", "^", "=", "+")
        ),
        listOf(
            NativeKeySpec("a", "@", "!", "#", "$"),
            NativeKeySpec("s", "#", "$", "%", "^"),
            NativeKeySpec("d", "$", "%", "&", "*"),
            NativeKeySpec("f", "%", "^", "_", "+"),
            NativeKeySpec("g", "&", "*", "-", "="),
            NativeKeySpec("h", "-", "_", ":", ";")
        ),
        listOf(
            NativeKeySpec("z", "*", "/", "\\", "|"),
            NativeKeySpec("x", "\"", "'", "`", "~"),
            NativeKeySpec("c", "'", "`", "©", "¢"),
            NativeKeySpec("v", ":", ";", "•", "°"),
            NativeKeySpec("b", ";", ":", "§", "¶"),
            NativeKeySpec("n", "!", "?", "¿", "¡")
        ),
        listOf(
            NativeKeySpec("u", "7", "&", "-", "_"),
            NativeKeySpec("i", "8", "*", "/", "*"),
            NativeKeySpec("j", "+", "=", "\"", "'"),
            NativeKeySpec("k", "(", ")", "<", ">"),
            NativeKeySpec("m", "?", "!", "«", "»"),
            NativeKeySpec("o", "9", "(", "[", "]")
        ),
        listOf(
            NativeKeySpec(",", "<", ";", "[", "{"),
            NativeKeySpec(".", ":", ">", "}", "]"),
            NativeKeySpec(" ", topRight = "🌐", action = NativeActionType.SPACE, flex = 2.0f),
            NativeKeySpec("p", "0", ")", "{", "}"),
            NativeKeySpec("l", ")", ":", "[", "]"),
            NativeKeySpec("⌫", action = NativeActionType.BACKSPACE, flex = 1.0f, keyCode = 67),
            NativeKeySpec("↵", action = NativeActionType.ENTER, flex = 1.0f, keyCode = 66)
        )
    )

    // VERSION 2 DEFAULT ALIGNMENT (Default on startup):
    // Row 0: Action & Symbol Row above Number Row (⇧, ?123, ?, /, ,, ., Space, ⌫, ↵)
    // Row 1: Number Row (1..0)
    // Rows 2..5: 5-letter sequential rows (a..e, f..j, k..o, p..t)
    // Row 6 (Bottom Row): Only letters — u, v, w, x (flex 1.0) + compact y, z (flex 0.55)
    private val version2Rows = listOf(
        actionSymbolRow,
        numberRow,
        listOf(
            NativeKeySpec("a", "1", "`", "~", "\\", flex = 1.0f),
            NativeKeySpec("b", "2", "|", "<", ">", flex = 1.0f),
            NativeKeySpec("c", "3", "€", "{", "}", flex = 1.0f),
            NativeKeySpec("d", "4", "₹", "[", "]", flex = 1.0f),
            NativeKeySpec("e", "5", "%", "(", ")", flex = 1.0f)
        ),
        listOf(
            NativeKeySpec("f", "6", "^", "=", "+", flex = 1.0f),
            NativeKeySpec("g", "@", "!", "#", "$", flex = 1.0f),
            NativeKeySpec("h", "#", "$", "%", "^", flex = 1.0f),
            NativeKeySpec("i", "$", "%", "&", "*", flex = 1.0f),
            NativeKeySpec("j", "%", "^", "_", "+", flex = 1.0f)
        ),
        listOf(
            NativeKeySpec("k", "&", "*", "-", "=", flex = 1.0f),
            NativeKeySpec("l", "-", "_", ":", ";", flex = 1.0f),
            NativeKeySpec("m", "*", "/", "\\", "|", flex = 1.0f),
            NativeKeySpec("n", "\"", "'", "`", "~", flex = 1.0f),
            NativeKeySpec("o", "'", "`", "©", "¢", flex = 1.0f)
        ),
        listOf(
            NativeKeySpec("p", ":", ";", "•", "°", flex = 1.0f),
            NativeKeySpec("q", ";", ":", "§", "¶", flex = 1.0f),
            NativeKeySpec("r", "!", "?", "¿", "¡", flex = 1.0f),
            NativeKeySpec("s", "7", "&", "-", "_", flex = 1.0f),
            NativeKeySpec("t", "8", "*", "/", "*", flex = 1.0f)
        ),
        listOf(
            NativeKeySpec("u", "+", "=", "\"", "'", flex = 1.0f),
            NativeKeySpec("v", "(", ")", "<", ">", flex = 1.0f),
            NativeKeySpec("w", "?", "!", "«", "»", flex = 1.0f),
            NativeKeySpec("x", "9", "(", "[", "]", flex = 1.0f),
            NativeKeySpec("y", "0", ")", "{", "}", flex = 0.55f),
            NativeKeySpec("z", ")", ":", "[", "]", flex = 0.55f)
        )
    )

    // RIGHT HAND BANGLISH LAYOUT (9x5 Ergonomic Grid based on 55 Banglish Mobile Keys Rank #1..#55):
    // Row 1: [ #55 (%) | #54 (&) | #53 (@) | #52 (;) | #17 (u) ]
    // Row 2: [ #50/#51 (8/9) | #49 (/) | #31/#32 (!/w) | #14 (m) | #10 (n) ]
    // Row 3: [ #47/#48 (6/7) | #28 (?) | #13 (b) | #9 (k) | #5 (r — moved up!) ]
    // Row 4: [ #41/#42 (4/5) | #24 (y) | #11 (t) | #2 (a) | #7 (i — moved up!) ]
    // Row 5: [ #34/#40 (2/3) | #21 (↵) | #19 (⇧ — in o's place!) | #1 (Space — wider!) | #3 (⌫ — right of Space!) ]
    // Row 6: [ #36/#33 (0/1) | #18 (c) | #12 (s) | #4 (e) | #8 (h) ]
    // Row 7: [ #37/#38 (-/x) | #25 (.) | #16 (d) | #15 (l) | #6 (o — below h!) ]
    // Row 8: [ #43/#44 (q/)) | #30 (') | #22 (g) | #20 (p) | #23 (j) ]
    // Row 9: [ #45/#46 ((/:) | #35/#39 (z/") | #29 (f) | #27 (v) | #26 (,) ]
    private val rightHandBanglishRows = listOf(
        // Row 1: [ 55(%) | 54(&) | 53(@) | 52(;) | 17(u) ]
        listOf(
            NativeKeySpec("%", flex = 1.0f),
            NativeKeySpec("&", flex = 1.0f),
            NativeKeySpec("@", flex = 1.0f),
            NativeKeySpec(";", flex = 1.0f),
            NativeKeySpec("u", "7", "&", "-", "_", flex = 1.0f)
        ),
        // Row 2: [ 50/51 (8/9) | 49 (/) | 31/32 (!/w) | 14 (m) | 10 (n) ]
        listOf(
            NativeKeySpec("8", "9", flex = 0.5f),
            NativeKeySpec("9", flex = 0.5f),
            NativeKeySpec("/", "?", "\\", "|", flex = 1.0f),
            NativeKeySpec("!", "w", flex = 0.5f),
            NativeKeySpec("w", "!", flex = 0.5f),
            NativeKeySpec("m", "?", "!", "«", "»", flex = 1.0f),
            NativeKeySpec("n", "!", "?", "¿", "¡", flex = 1.0f)
        ),
        // Row 3: [ 47/48 (6/7) | 28 (?) | 13 (b) | 9 (k) | 5 (r moved up!) ]
        listOf(
            NativeKeySpec("6", "7", flex = 0.5f),
            NativeKeySpec("7", flex = 0.5f),
            NativeKeySpec("?", "!", "¿", "¡", "@", flex = 1.0f),
            NativeKeySpec("b", ";", ":", "§", "¶", flex = 1.0f),
            NativeKeySpec("k", "(", ")", "<", ">", flex = 1.0f),
            NativeKeySpec("r", "4", "₹", "[", "]", flex = 1.0f)
        ),
        // Row 4: [ 41/42 (4/5) | 24 (y) | 11 (t) | 2 (a) | 7 (i moved up!) ]
        listOf(
            NativeKeySpec("4", "5", flex = 0.5f),
            NativeKeySpec("5", flex = 0.5f),
            NativeKeySpec("y", "6", "^", "=", "+", flex = 1.0f),
            NativeKeySpec("t", "5", "%", "(", ")", flex = 1.0f),
            NativeKeySpec("a", "@", "!", "#", "$", flex = 1.0f),
            NativeKeySpec("i", "8", "*", "/", "*", flex = 1.0f)
        ),
        // Row 5: [ 34/40 (2/3) | 21 (↵) | 19 (⇧ in o's place!) | 1 (wider Space!) | 3 (⌫ Cross right of Space!) ]
        listOf(
            NativeKeySpec("2", "3", flex = 0.44f),
            NativeKeySpec("3", flex = 0.44f),
            NativeKeySpec("↵", action = NativeActionType.ENTER, flex = 0.88f, keyCode = 66),
            NativeKeySpec("⇧", action = NativeActionType.SHIFT, flex = 0.88f),
            NativeKeySpec(" ", topRight = "🌐", action = NativeActionType.SPACE, flex = 1.48f),
            NativeKeySpec("⌫", action = NativeActionType.BACKSPACE, flex = 0.88f, keyCode = 67)
        ),
        // Row 6: [ 36/33 (0/1) | 18 (c) | 12 (s) | 4 (e) | 8 (h) ]
        listOf(
            NativeKeySpec("0", "1", flex = 0.5f),
            NativeKeySpec("1", flex = 0.5f),
            NativeKeySpec("c", "'", "`", "©", "¢", flex = 1.0f),
            NativeKeySpec("s", "#", "$", "%", "^", flex = 1.0f),
            NativeKeySpec("e", "3", "€", "{", "}", flex = 1.0f),
            NativeKeySpec("h", "-", "_", ":", ";", flex = 1.0f)
        ),
        // Row 7: [ 37/38 (-/x) | 25 (.) | 16 (d) | 15 (l) | 6 (o below h!) ]
        listOf(
            NativeKeySpec("-", "x", flex = 0.5f),
            NativeKeySpec("x", "-", flex = 0.5f),
            NativeKeySpec(".", ">", ":", "(", ")", flex = 1.0f),
            NativeKeySpec("d", "$", "%", "&", "*", flex = 1.0f),
            NativeKeySpec("l", ")", ":", "[", "]", flex = 1.0f),
            NativeKeySpec("o", "9", "(", "[", "]", flex = 1.0f)
        ),
        // Row 8: [ 43/44 (q/)) | 30 (') | 22 (g) | 20 (p) | 23 (j) ]
        listOf(
            NativeKeySpec("q", ")", flex = 0.5f),
            NativeKeySpec(")", flex = 0.5f),
            NativeKeySpec("'", "\"", "`", "~", flex = 1.0f),
            NativeKeySpec("g", "&", "*", "-", "=", flex = 1.0f),
            NativeKeySpec("p", "0", ")", "{", "}", flex = 1.0f),
            NativeKeySpec("j", "+", "=", "\"", "'", flex = 1.0f)
        ),
        // Row 9: [ 45/46 ((/:) | 35/39 (z/") | 29 (f) | 27 (v) | 26 (,) ]
        listOf(
            NativeKeySpec("(", ":", flex = 0.5f),
            NativeKeySpec(":", flex = 0.5f),
            NativeKeySpec("z", "\"", flex = 0.5f),
            NativeKeySpec("\"", flex = 0.5f),
            NativeKeySpec("f", "%", "^", "_", "+", flex = 1.0f),
            NativeKeySpec("v", ":", ";", "•", "°", flex = 1.0f),
            NativeKeySpec(",", "<", ";", "'", "\"", flex = 1.0f)
        )
    )

    private val symbolRows = listOf(
        actionSymbolRow,
        numberRow,
        listOf(
            NativeKeySpec("!", "~", "`"),
            NativeKeySpec("@", "#", "$"),
            NativeKeySpec("#", "$", "€"),
            NativeKeySpec("$", "€", "₹"),
            NativeKeySpec("%", "‰", "^")
        ),
        listOf(
            NativeKeySpec("^", "&", "*"),
            NativeKeySpec("&", "*", "("),
            NativeKeySpec("*", "(", ")"),
            NativeKeySpec("(", "[", "{"),
            NativeKeySpec(")", "]", "}")
        ),
        listOf(
            NativeKeySpec("+", "±", "×"),
            NativeKeySpec("-", "÷", "—"),
            NativeKeySpec("=", "≠", "≈"),
            NativeKeySpec("_", "—", "-"),
            NativeKeySpec("|", "¦", "§")
        ),
        listOf(
            NativeKeySpec("\\", "/", "|"),
            NativeKeySpec("~", "`", "^"),
            NativeKeySpec("`", "\"", "'"),
            NativeKeySpec("{", "[", "("),
            NativeKeySpec("}", "]", ")")
        ),
        listOf(
            NativeKeySpec("[", "{", "("),
            NativeKeySpec("]", "}", ")"),
            NativeKeySpec("<", "≤", "«"),
            NativeKeySpec(">", "≥", "»"),
            NativeKeySpec(";", ":", ",", flex = 0.55f),
            NativeKeySpec(":", ";", ".", flex = 0.55f)
        )
    )

    fun reloadConfigFromPrefs() {
        val prevHeightPx = calculateTotalHeightPx()
        keyHeightDp = service.getSavedKeyHeightDp()
        longPressMs = service.getSavedLongPressMs()
        themeMode = service.getSavedThemeMode()
        hapticsEnabled = service.isHapticsEnabled()
        layoutMode = service.getSavedLayoutMode()
        researchColorsEnabled = service.isResearchColorsEnabled()
        applyThemePalette(themeMode)
        if (width > 0) {
            rebuildHitRects(width.toFloat())
        }
        if (calculateTotalHeightPx() != prevHeightPx || height == 0) {
            requestLayout()
        }
        invalidate()
    }

    private fun applyThemePalette(mode: String) {
        when (mode) {
            "terminalGreen" -> {
                bgColor = 0xFF050B05.toInt()
                keyBgColor = 0xFF0D190D.toInt()
                keyPressedColor = 0xFF1B331B.toInt()
                modActiveColor = 0xFF00FF41.toInt()
                modInactiveColor = 0xFF142814.toInt()
                primaryTextColor = 0xFF00FF41.toInt()
                cornerTextColor = 0xFF4CAF50.toInt()
                accentColor = 0xFF00FF41.toInt()
            }
            "cyberpunk" -> {
                bgColor = 0xFF0B001A.toInt()
                keyBgColor = 0xFF1A0933.toInt()
                keyPressedColor = 0xFF321261.toInt()
                modActiveColor = 0xFFFF007F.toInt()
                modInactiveColor = 0xFF290F4D.toInt()
                primaryTextColor = 0xFF00F0FF.toInt()
                cornerTextColor = 0xFFFF77FF.toInt()
                accentColor = 0xFFFF007F.toInt()
            }
            "materialDark" -> {
                bgColor = 0xFF121212.toInt()
                keyBgColor = 0xFF1E1E1E.toInt()
                keyPressedColor = 0xFF383838.toInt()
                modActiveColor = 0xFF64B5F6.toInt()
                modInactiveColor = 0xFF2C2C2C.toInt()
                primaryTextColor = 0xFFE0E0E0.toInt()
                cornerTextColor = 0xFF9E9E9E.toInt()
                accentColor = 0xFF64B5F6.toInt()
            }
            else -> {
                bgColor = 0xFF000000.toInt()
                keyBgColor = 0xFF141414.toInt()
                keyPressedColor = 0xFF2C2C2C.toInt()
                modActiveColor = 0xFF00E676.toInt()
                modInactiveColor = 0xFF262626.toInt()
                primaryTextColor = 0xFFFFFFFF.toInt()
                cornerTextColor = 0xFF9E9E9E.toInt()
                accentColor = 0xFF00E676.toInt()
            }
        }
    }

    private fun dp(value: Float): Float = value * resources.displayMetrics.density

    private fun calculateTotalHeightPx(): Int {
        val modH = (keyHeightDp * 0.85f).coerceIn(36.0f, 68.0f)
        val resizeH = if (isResizeBarOpen) 72.0f else 0.0f
        val totalDp = 7.0f * keyHeightDp + modH + resizeH + 8.0f
        return dp(totalDp).toInt()
    }

    override fun onMeasure(widthMeasureSpec: Int, heightMeasureSpec: Int) {
        val w = MeasureSpec.getSize(widthMeasureSpec)
        val h = calculateTotalHeightPx()
        setMeasuredDimension(w, h)
    }

    private fun rebuildHitRects(viewWidth: Float) {
        hitRects.clear()
        var currentTop = 0f

        // Optional inline resize & long-press (0ms..3000ms) control bar
        if (isResizeBarOpen) {
            val rowH = dp(36.0f)
            val resizeSpecs = listOf(
                NativeKeySpec("− Height", action = NativeActionType.RESIZE_MINUS, flex = 0.9f),
                NativeKeySpec("${keyHeightDp.toInt()}dp", action = NativeActionType.RESIZE_RESET, flex = 1.0f),
                NativeKeySpec("+ Height", action = NativeActionType.RESIZE_PLUS, flex = 0.9f),
                NativeKeySpec(
                    if (researchColorsEnabled) "🎨 Color: ON" else "🎨 Color: OFF",
                    action = NativeActionType.TOGGLE_RESEARCH_COLORS,
                    flex = 1.25f
                ),
                NativeKeySpec("Done ✓", action = NativeActionType.RESIZE_DONE, flex = 0.85f)
            )
            val totalFlex = resizeSpecs.fold(0f) { acc, s -> acc + s.flex }
            var left = 0f
            for (spec in resizeSpecs) {
                val kw = viewWidth * (spec.flex / totalFlex)
                hitRects.add(HitRect(spec, RectF(left, currentTop, left + kw, currentTop + rowH), isTopBar = true))
                left += kw
            }
            currentTop += rowH

            val longPressSpecs = listOf(
                NativeKeySpec("− LongPress", action = NativeActionType.LONG_PRESS_MINUS, flex = 1.0f),
                NativeKeySpec("Hold: ${longPressMs}ms", action = NativeActionType.LONG_PRESS_RESET, flex = 1.4f),
                NativeKeySpec("+ LongPress", action = NativeActionType.LONG_PRESS_PLUS, flex = 1.0f),
                NativeKeySpec(if (longPressMs == 0) "400ms" else "0ms", action = NativeActionType.LONG_PRESS_ZERO, flex = 1.0f)
            )
            val totalLpFlex = longPressSpecs.fold(0f) { acc, s -> acc + s.flex }
            var lpLeft = 0f
            for (spec in longPressSpecs) {
                val kw = viewWidth * (spec.flex / totalLpFlex)
                hitRects.add(HitRect(spec, RectF(lpLeft, currentTop, lpLeft + kw, currentTop + rowH), isTopBar = true))
                lpLeft += kw
            }
            currentTop += rowH
        }

        // Top Terminal Modifier Bar
        val modH = dp((keyHeightDp * 0.85f).coerceIn(36.0f, 68.0f))
        val totalModFlex = topBarSpecs.fold(0f) { acc, s -> acc + s.flex }
        var modLeft = 0f
        for (rawSpec in topBarSpecs) {
            val spec = if (rawSpec.action == NativeActionType.SWITCH_LAYOUT && isSymbolLayout) {
                rawSpec.copy(primary = "ABC")
            } else {
                rawSpec
            }
            val kw = viewWidth * (spec.flex / totalModFlex)
            hitRects.add(HitRect(spec, RectF(modLeft, currentTop, modLeft + kw, currentTop + modH), isTopBar = true))
            modLeft += kw
        }
        currentTop += modH
        val bodyTop = currentTop

        // Main Keyboard Rows (7 rows for QWERTY/ABCD/Symbols, or 9 rows for Right Hand Banglish)
        val rows = when {
            isSymbolLayout -> symbolRows
            layoutMode == "qwerty" -> version1Rows
            layoutMode == "right_hand" -> rightHandBanglishRows
            else -> version2Rows
        }
        val rowH = dp(keyHeightDp * 7.0f / rows.size.toFloat())
        for (row in rows) {
            val totalRowFlex = row.fold(0f) { acc, s -> acc + s.flex }
            var left = 0f
            for (rawSpec in row) {
                val spec = if (rawSpec.action == NativeActionType.SWITCH_LAYOUT && isSymbolLayout) {
                    rawSpec.copy(primary = "ABC")
                } else {
                    rawSpec
                }
                val kw = viewWidth * (spec.flex / totalRowFlex)
                hitRects.add(HitRect(spec, RectF(left, currentTop, left + kw, currentTop + rowH), isTopBar = false))
                left += kw
            }
            currentTop += rowH
        }

        // Build Layout Selection Popup Menu hit rects
        menuHitRects.clear()
        val bodyH = currentTop - bodyTop
        val cardW = min(viewWidth * 0.88f, dp(320f))
        val cardH = min(bodyH - dp(8f), dp(276f))
        val cardLeft = (viewWidth - cardW) / 2f
        val cardTop = bodyTop + max(dp(4f), (bodyH - cardH) / 2f)
        menuCardRect = RectF(cardLeft, cardTop, cardLeft + cardW, cardTop + cardH)

        val headerH = dp(32f)
        val closeSpec = NativeKeySpec("✕", action = NativeActionType.CLOSE_LAYOUT_MENU)
        menuHitRects.add(
            HitRect(
                closeSpec,
                RectF(cardLeft + cardW - dp(40f), cardTop, cardLeft + cardW, cardTop + headerH),
                isTopBar = true
            )
        )

        val menuItems = listOf(
            NativeKeySpec("1. QWERTY Type", action = NativeActionType.SELECT_LAYOUT_QWERTY),
            NativeKeySpec("2. ABCD Serially", action = NativeActionType.SELECT_LAYOUT_ABCD),
            NativeKeySpec("3. Left Hand", action = NativeActionType.SELECT_LAYOUT_LEFT),
            NativeKeySpec("4. Right Hand Banglish", action = NativeActionType.SELECT_LAYOUT_RIGHT),
            NativeKeySpec("5. Both Hand", action = NativeActionType.SELECT_LAYOUT_BOTH),
            NativeKeySpec(
                if (researchColorsEnabled) "🎨 Button Color: ON (Tap to OFF)" else "🎨 Button Color: OFF (Tap to ON)",
                action = NativeActionType.TOGGLE_RESEARCH_COLORS
            )
        )
        val availItemsH = cardH - headerH - dp(8f)
        val itemH = availItemsH / menuItems.size
        var itemTop = cardTop + headerH + dp(2f)
        val itemLeft = cardLeft + dp(10f)
        val itemRight = cardLeft + cardW - dp(10f)
        for (itemSpec in menuItems) {
            menuHitRects.add(
                HitRect(
                    itemSpec,
                    RectF(itemLeft, itemTop + dp(1.5f), itemRight, itemTop + itemH - dp(1.5f)),
                    isTopBar = true
                )
            )
            itemTop += itemH
        }
    }

    // Scientifically Tested 12-Color Palette from User's Research:
    // #1 Spacebar: Blue (নীল বোতাম — Journal of Marketing #1, 12.7% CTR)
    // #2 'a': Grey (ধূসর বোতাম — Journal of Marketing #2, 9.4% CTR)
    // #3..#12: 2022 Button-Press Preference Study Top 10:
    //   1. উজ্জ্বল সবুজ (Bright Green)
    //   2. গাঢ় উজ্জ্বল হলুদ (Deep Bright Yellow)
    //   3. উজ্জ্বল সবুজাভ নীল (Bright Teal-Blue)
    //   4. কালো (Pure Black)
    //   5. সাদা (Crisp White)
    //   6. হালকা নীল (Light Blue)
    //   7. তীব্র নীল (Intense Royal Blue)
    //   8. গাঢ় লাল (Dark Red)
    //   9. হালকা ধূসর (Light Grey)
    //  10. তীব্র হলুদাভ কমলা (Intense Yellowish Orange)
    private val researchColorPalette = intArrayOf(
        0xFF1565C0.toInt(), // #1: নীল বোতাম (Blue — Journal of Marketing #1)
        0xFF607D8B.toInt(), // #2: ধূসর বোতাম (Grey — Journal of Marketing #2)
        0xFF00C853.toInt(), // #3 (Study #1): উজ্জ্বল সবুজ (Bright Green)
        0xFFFBC02D.toInt(), // #4 (Study #2): গাঢ় উজ্জ্বল হলুদ (Deep Bright Yellow)
        0xFF00ACC1.toInt(), // #5 (Study #3): উজ্জ্বল সবুজাভ নীল (Bright Teal-Blue)
        0xFF141414.toInt(), // #6 (Study #4): কালো (Black)
        0xFFF5F5F5.toInt(), // #7 (Study #5): সাদা (White)
        0xFF29B6F6.toInt(), // #8 (Study #6): হালকা নীল (Light Blue)
        0xFF1A237E.toInt(), // #9 (Study #7): তীব্র নীল (Intense Blue)
        0xFFC62828.toInt(), // #10 (Study #8): গাঢ় লাল (Dark Red)
        0xFF90A4AE.toInt(), // #11 (Study #9): হালকা ধূসর (Light Grey)
        0xFFFF6D00.toInt()  // #12 (Study #10): তীব্র হলুদাভ কমলা (Intense Yellowish Orange)
    )

    private fun getBanglishRank(spec: NativeKeySpec): Int {
        return when (spec.action) {
            NativeActionType.SPACE -> 1
            NativeActionType.BACKSPACE -> 3
            NativeActionType.SHIFT -> 19
            NativeActionType.ENTER -> 21
            NativeActionType.INPUT_CHAR -> when (spec.primary.lowercase()) {
                "a" -> 2
                "e" -> 4
                "r" -> 5
                "o" -> 6
                "i" -> 7
                "h" -> 8
                "k" -> 9
                "n" -> 10
                "t" -> 11
                "s" -> 12
                "b" -> 13
                "m" -> 14
                "l" -> 15
                "d" -> 16
                "u" -> 17
                "c" -> 18
                "p" -> 20
                "g" -> 22
                "j" -> 23
                "y" -> 24
                "." -> 25
                "," -> 26
                "v" -> 27
                "?" -> 28
                "f" -> 29
                "'" -> 30
                "!" -> 31
                "w" -> 32
                "1" -> 33
                "2" -> 34
                "z" -> 35
                "0" -> 36
                "-" -> 37
                "x" -> 38
                "\"" -> 39
                "3" -> 40
                "4" -> 41
                "5" -> 42
                "q" -> 43
                ")" -> 44
                "(" -> 45
                ":" -> 46
                "6" -> 47
                "7" -> 48
                "/" -> 49
                "8" -> 50
                "9" -> 51
                ";" -> 52
                "@" -> 53
                "&" -> 54
                "%" -> 55
                else -> 0
            }
            else -> 0
        }
    }

    private fun getResearchColorForSpec(spec: NativeKeySpec): Int? {
        if (!researchColorsEnabled || layoutMode != "right_hand") return null
        val rank = getBanglishRank(spec)
        if (rank <= 0) return null
        return researchColorPalette[(rank - 1) % researchColorPalette.size]
    }

    private fun isBrightColor(color: Int): Boolean {
        val r = (color shr 16) and 0xFF
        val g = (color shr 8) and 0xFF
        val b = color and 0xFF
        val lum = (0.299 * r + 0.587 * g + 0.114 * b)
        return lum > 150.0
    }

    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) {
        super.onSizeChanged(w, h, oldw, oldh)
        if (w > 0) {
            rebuildHitRects(w.toFloat())
        }
    }

    override fun onDraw(canvas: Canvas) {
        super.onDraw(canvas)
        if (hitRects.isEmpty() && width > 0) {
            rebuildHitRects(width.toFloat())
        }

        canvas.drawColor(bgColor)

        val isBanglish = (layoutMode == "right_hand")
        val strokePx = max(1f, dp(0.5f))
        val activeStrokePx = max(2f, dp(1.2f))
        val cornerPaddingX = dp(3.5f)
        val cornerPaddingY = dp(2.0f)
        val baseCornerSize = dp(9.5f)
        cornerTextPaint.textSize = baseCornerSize

        for (hit in hitRects) {
            val spec = hit.spec
            val r = hit.rect
            val isPressed = (activeHit === hit)

            val isActiveMod = (spec.action == NativeActionType.SHIFT && (isShiftActive || isCapsLock)) ||
                    (spec.action == NativeActionType.CTRL && isCtrlActive) ||
                    (spec.action == NativeActionType.ALT && isAltActive) ||
                    (spec.action == NativeActionType.SWITCH_LAYOUT && spec.primary == "ABC") ||
                    (spec.action == NativeActionType.OPEN_LAYOUT_MENU && isLayoutMenuOpen) ||
                    (spec.action == NativeActionType.TOGGLE_RESIZE && isResizeBarOpen) ||
                    (spec.action == NativeActionType.TOGGLE_RESEARCH_COLORS && researchColorsEnabled) ||
                    (spec.action == NativeActionType.RESIZE_DONE)

            val researchColor = if (!hit.isTopBar) getResearchColorForSpec(spec) else null
            val effectiveBgColor = when {
                isActiveMod -> modActiveColor
                isPressed -> keyPressedColor
                researchColor != null -> researchColor
                else -> keyBgColor
            }
            bgPaint.color = effectiveBgColor
            canvas.drawRect(r, bgPaint)

            borderPaint.color = if (isActiveMod) accentColor else modInactiveColor
            borderPaint.strokeWidth = if (isActiveMod) activeStrokePx else strokePx
            canvas.drawRect(r, borderPaint)

            if (spec.action == NativeActionType.OPEN_LAYOUT_MENU) {
                // Draw crisp 2x2 Layout Grid Icon (50% larger in Banglish layout)
                val iconColor = if (isActiveMod) 0xFFFFFFFF.toInt() else accentColor
                bgPaint.color = iconColor
                val iconScale = if (isBanglish) 1.5f else 1.0f
                val iconSize = min(r.width(), r.height()) * (0.44f * iconScale).coerceAtMost(0.82f)
                val gap = max(dp(1.8f), iconSize * 0.18f)
                val sq = (iconSize - gap) / 2f
                val startXIcon = r.centerX() - iconSize / 2f
                val startYIcon = r.centerY() - iconSize / 2f
                val rad = dp(1.5f)
                canvas.drawRoundRect(RectF(startXIcon, startYIcon, startXIcon + sq, startYIcon + sq), rad, rad, bgPaint)
                canvas.drawRoundRect(RectF(startXIcon + sq + gap, startYIcon, startXIcon + iconSize, startYIcon + sq), rad, rad, bgPaint)
                canvas.drawRoundRect(RectF(startXIcon, startYIcon + sq + gap, startXIcon + sq, startYIcon + iconSize), rad, rad, bgPaint)
                canvas.drawRoundRect(RectF(startXIcon + sq + gap, startYIcon + sq + gap, startXIcon + iconSize, startYIcon + iconSize), rad, rad, bgPaint)
                continue
            }

            // Primary label
            val displayPrimary = when {
                spec.action == NativeActionType.SPACE -> "␣"
                spec.action == NativeActionType.SHIFT && isCapsLock -> "Caps"
                (isShiftActive || isCapsLock) && spec.primary.length == 1 -> spec.primary.uppercase()
                else -> spec.primary
            }

            val isBrightKeyBg = (!isActiveMod && !isPressed && researchColor != null && isBrightColor(researchColor))
            val fgColor = when {
                isActiveMod -> 0xFFFFFFFF.toInt()
                spec.action == NativeActionType.SWITCH_IME -> accentColor
                isBrightKeyBg -> 0xFF0A0A0A.toInt()
                researchColor != null -> 0xFFFFFFFF.toInt()
                else -> primaryTextColor
            }

            val cellH = r.height()
            val cellW = r.width()
            val isMulti = displayPrimary.length > 1
            val isIconKey = spec.action == NativeActionType.SPACE ||
                    spec.action == NativeActionType.BACKSPACE ||
                    spec.action == NativeActionType.SHIFT ||
                    spec.action == NativeActionType.ENTER ||
                    spec.action == NativeActionType.SWITCH_IME ||
                    spec.action == NativeActionType.TOGGLE_RESIZE ||
                    spec.action == NativeActionType.OPEN_SETTINGS ||
                    spec.action == NativeActionType.ARROW_LEFT ||
                    spec.action == NativeActionType.ARROW_DOWN ||
                    spec.action == NativeActionType.ARROW_UP ||
                    spec.action == NativeActionType.ARROW_RIGHT

            val paint = if (hit.isTopBar || isMulti || (isBanglish && isIconKey)) multiCharTextPaint else primaryTextPaint
            paint.color = fgColor

            var targetTextSize = when {
                hit.isTopBar -> if (isMulti) cellH * 0.34f else cellH * 0.46f
                isMulti -> cellH * 0.34f
                spec.action == NativeActionType.SPACE -> cellH * 0.65f
                else -> cellH * 0.72f
            }
            if (isBanglish && isIconKey) {
                // Increase all icons by 50% in Banglish layout
                targetTextSize *= 1.5f
            }
            val maxAllowedW = if (isBanglish && isIconKey) cellW * 0.96f else cellW * 0.86f
            paint.textSize = targetTextSize
            val measuredW = paint.measureText(displayPrimary)
            if (measuredW > maxAllowedW && measuredW > 0f) {
                targetTextSize *= (maxAllowedW / measuredW)
                paint.textSize = targetTextSize
            }

            val fm = paint.fontMetrics
            var centerY = r.centerY() - (fm.ascent + fm.descent) / 2f
            if (displayPrimary == "," || displayPrimary == ".") {
                centerY -= cellH * 0.16f
            }
            canvas.drawText(displayPrimary, r.centerX(), centerY, paint)

            // 4 Corner Symbols (with 50% larger 🌐 icon on Spacebar in Banglish layout)
            if (!hit.isTopBar) {
                val defaultCornerCol = when {
                    isBrightKeyBg -> 0xFF263238.toInt()
                    researchColor != null -> 0xFFE0E0E0.toInt()
                    else -> cornerTextColor
                }
                if (spec.topLeft != null) {
                    cornerTextPaint.textSize = baseCornerSize
                    val cFm = cornerTextPaint.fontMetrics
                    cornerTextPaint.color = if (isPressed && activeCornerDir == 1) accentColor else defaultCornerCol
                    cornerTextPaint.textAlign = Paint.Align.LEFT
                    canvas.drawText(spec.topLeft, r.left + cornerPaddingX, r.top + cornerPaddingY - cFm.ascent, cornerTextPaint)
                }
                if (spec.topRight != null) {
                    cornerTextPaint.textSize = if (isBanglish && spec.action == NativeActionType.SPACE) baseCornerSize * 1.5f else baseCornerSize
                    val cFm = cornerTextPaint.fontMetrics
                    cornerTextPaint.color = if (isPressed && activeCornerDir == 2) accentColor else defaultCornerCol
                    cornerTextPaint.textAlign = Paint.Align.RIGHT
                    canvas.drawText(spec.topRight, r.right - cornerPaddingX, r.top + cornerPaddingY - cFm.ascent, cornerTextPaint)
                }
                if (spec.bottomLeft != null) {
                    cornerTextPaint.textSize = baseCornerSize
                    val cFm = cornerTextPaint.fontMetrics
                    cornerTextPaint.color = if (isPressed && activeCornerDir == 3) accentColor else defaultCornerCol
                    cornerTextPaint.textAlign = Paint.Align.LEFT
                    canvas.drawText(spec.bottomLeft, r.left + cornerPaddingX, r.bottom - cornerPaddingY - cFm.descent, cornerTextPaint)
                }
                if (spec.bottomRight != null) {
                    cornerTextPaint.textSize = baseCornerSize
                    val cFm = cornerTextPaint.fontMetrics
                    cornerTextPaint.color = if (isPressed && activeCornerDir == 4) accentColor else defaultCornerCol
                    cornerTextPaint.textAlign = Paint.Align.RIGHT
                    canvas.drawText(spec.bottomRight, r.right - cornerPaddingX, r.bottom - cornerPaddingY - cFm.descent, cornerTextPaint)
                }
            }
        }

        // Draw Layout Picker Popup Modal when open
        if (isLayoutMenuOpen) {
            bgPaint.color = 0xCC000000.toInt()
            canvas.drawRect(0f, 0f, width.toFloat(), height.toFloat(), bgPaint)

            val cardRad = dp(10f)
            bgPaint.color = keyBgColor
            canvas.drawRoundRect(menuCardRect, cardRad, cardRad, bgPaint)
            borderPaint.color = accentColor
            borderPaint.strokeWidth = dp(1.5f)
            canvas.drawRoundRect(menuCardRect, cardRad, cardRad, borderPaint)

            // Title text
            multiCharTextPaint.color = accentColor
            multiCharTextPaint.textSize = dp(13.5f)
            val titleFm = multiCharTextPaint.fontMetrics
            val titleY = menuCardRect.top + dp(17f) - (titleFm.ascent + titleFm.descent) / 2f
            canvas.drawText("Choose Keyboard Layout", menuCardRect.centerX() - dp(10f), titleY, multiCharTextPaint)

            for (mHit in menuHitRects) {
                val mSpec = mHit.spec
                val mr = mHit.rect
                if (mSpec.action == NativeActionType.CLOSE_LAYOUT_MENU) {
                    multiCharTextPaint.color = cornerTextColor
                    multiCharTextPaint.textSize = dp(14f)
                    val cFm = multiCharTextPaint.fontMetrics
                    canvas.drawText("✕", mr.centerX(), mr.centerY() - (cFm.ascent + cFm.descent) / 2f, multiCharTextPaint)
                    continue
                }

                val isSelected = (mSpec.action == NativeActionType.SELECT_LAYOUT_QWERTY && layoutMode == "qwerty") ||
                        (mSpec.action == NativeActionType.SELECT_LAYOUT_ABCD && layoutMode == "abcd") ||
                        (mSpec.action == NativeActionType.SELECT_LAYOUT_LEFT && layoutMode == "left_hand") ||
                        (mSpec.action == NativeActionType.SELECT_LAYOUT_RIGHT && layoutMode == "right_hand") ||
                        (mSpec.action == NativeActionType.SELECT_LAYOUT_BOTH && layoutMode == "both_hand") ||
                        (mSpec.action == NativeActionType.TOGGLE_RESEARCH_COLORS && researchColorsEnabled)

                val itemRad = dp(6f)
                bgPaint.color = if (isSelected) modActiveColor else bgColor
                canvas.drawRoundRect(mr, itemRad, itemRad, bgPaint)
                borderPaint.color = if (isSelected) accentColor else modInactiveColor
                borderPaint.strokeWidth = if (isSelected) dp(1.2f) else dp(0.6f)
                canvas.drawRoundRect(mr, itemRad, itemRad, borderPaint)

                menuItemTextPaint.color = if (isSelected) 0xFFFFFFFF.toInt() else primaryTextColor
                menuItemTextPaint.textSize = min(dp(13.5f), mr.height() * 0.44f)
                val iFm = menuItemTextPaint.fontMetrics
                val textY = mr.centerY() - (iFm.ascent + iFm.descent) / 2f
                val prefix = if (isSelected) "✓  " else "    "
                canvas.drawText(prefix + mSpec.primary, mr.left + dp(12f), textY, menuItemTextPaint)
            }
            return
        }

        // Draw live key press popup preview
        val pLabel = popupLabel
        val pRect = popupAnchorRect
        if (pLabel != null && pRect != null) {
            val popW = dp(56.0f)
            val popH = dp(50.0f)
            val popLeft = (pRect.centerX() - popW / 2f).coerceIn(dp(4f), max(dp(4f), width - popW - dp(4f)))
            val popTop = max(dp(2f), pRect.top - popH - dp(4f))
            val popBox = RectF(popLeft, popTop, popLeft + popW, popTop + popH)

            popupBgPaint.color = accentColor
            popupBorderPaint.strokeWidth = dp(1.5f)
            val radius = dp(8.0f)
            canvas.drawRoundRect(popBox, radius, radius, popupBgPaint)
            canvas.drawRoundRect(popBox, radius, radius, popupBorderPaint)

            popupTextPaint.color = bgColor
            popupTextPaint.textSize = if (pLabel.length > 2) dp(16.0f) else dp(28.0f)
            val pFm = popupTextPaint.fontMetrics
            val pCenterY = popBox.centerY() - (pFm.ascent + pFm.descent) / 2f
            canvas.drawText(pLabel, popBox.centerX(), pCenterY, popupTextPaint)
        }
    }

    private fun findHit(x: Float, y: Float): HitRect? {
        if (isLayoutMenuOpen) {
            for (mHit in menuHitRects) {
                if (mHit.rect.contains(x, y)) {
                    return mHit
                }
            }
            for (hit in hitRects) {
                if (hit.spec.action == NativeActionType.OPEN_LAYOUT_MENU && hit.rect.contains(x, y)) {
                    return hit
                }
            }
            return null
        }
        for (hit in hitRects) {
            if (hit.rect.contains(x, y)) {
                return hit
            }
        }
        return null
    }

    private fun triggerHapticNow() {
        if (hapticsEnabled) {
            performHapticFeedback(
                HapticFeedbackConstants.KEYBOARD_TAP,
                HapticFeedbackConstants.FLAG_IGNORE_GLOBAL_SETTING
            )
        }
    }

    private fun calculateMetaState(shifted: Boolean): Int {
        var state = 0
        if (shifted) state = state or KeyEvent.META_SHIFT_ON
        if (isAltActive) state = state or KeyEvent.META_ALT_ON
        if (isCtrlActive) state = state or KeyEvent.META_CTRL_ON
        return state
    }

    private fun resolveCharOutput(spec: NativeKeySpec, dir: Int, shifted: Boolean): String {
        val raw = when (dir) {
            1 -> spec.topLeft ?: spec.primary
            2 -> spec.topRight ?: spec.primary
            3 -> spec.bottomLeft ?: spec.primary
            4 -> spec.bottomRight ?: spec.primary
            else -> spec.primary
        }
        return if (shifted && dir == 0 && raw.length == 1) raw.uppercase() else raw
    }

    private fun cancelTimers() {
        repeatRunnable?.let { handler.removeCallbacks(it) }
        repeatRunnable = null
        spaceLongPressRunnable?.let { handler.removeCallbacks(it) }
        spaceLongPressRunnable = null
        charLongPressRunnable?.let { handler.removeCallbacks(it) }
        charLongPressRunnable = null
    }

    private fun updateKeyHeightLive(newHeightDp: Float) {
        keyHeightDp = newHeightDp.coerceIn(34.0f, 136.0f)
        service.saveKeyHeightDp(keyHeightDp, isResizeBarOpen)
        rebuildHitRects(width.toFloat())
        requestLayout()
        invalidate()
    }

    private fun updateLongPressMsLive(newMs: Int) {
        longPressMs = newMs.coerceIn(0, 3000)
        service.saveLongPressMs(longPressMs)
        rebuildHitRects(width.toFloat())
        invalidate()
    }

    private fun selectLayoutMode(mode: String) {
        layoutMode = mode
        isSymbolLayout = false
        isLayoutMenuOpen = false
        service.saveLayoutMode(mode)
        rebuildHitRects(width.toFloat())
        triggerHapticNow()
        invalidate()
    }

    private fun executeKeyActionOnDown(hit: HitRect) {
        val spec = hit.spec
        when (spec.action) {
            NativeActionType.INPUT_CHAR -> {
                wasShiftedOnDown = isShiftActive || isCapsLock
                val useMods = isCtrlActive || isAltActive
                wasModifierComboOnDown = useMods
                val meta = calculateMetaState(wasShiftedOnDown)
                val out = resolveCharOutput(spec, 0, wasShiftedOnDown)
                currentCommittedOutput = out

                // Reset single-use modifiers immediately
                if (isShiftActive && !isCapsLock) isShiftActive = false
                isCtrlActive = false
                isAltActive = false

                // 0.0ms direct commit to Android InputConnection
                service.commitTextImmediate(out, meta, useMods)
                popupLabel = out
                popupAnchorRect = hit.rect
                triggerHapticNow()
                invalidate()

                // Customizable Long Press (0ms..3000ms): Hold key to replace with topLeft symbol
                if (!useMods && longPressMs > 0 && spec.topLeft != null) {
                    val lpRunnable = Runnable {
                        if (activeHit === hit && activeCornerDir == 0) {
                            activeCornerDir = 1
                            val lpOut = spec.topLeft
                            if (lpOut != currentCommittedOutput) {
                                currentCommittedOutput = lpOut
                                service.replaceLastTextImmediate(lpOut)
                                popupLabel = lpOut
                                triggerHapticNow()
                                invalidate()
                            }
                        }
                    }
                    charLongPressRunnable = lpRunnable
                    handler.postDelayed(lpRunnable, longPressMs.toLong())
                }
            }
            NativeActionType.SPACE -> {
                if (isShiftActive && !isCapsLock) isShiftActive = false
                isCtrlActive = false
                isAltActive = false
                service.commitTextImmediate(" ", 0, false)
                triggerHapticNow()
                invalidate()

                // Hold Spacebar (customizable 0ms..3000ms) to switch keyboard
                val spaceDelay = if (longPressMs <= 0) 80L else longPressMs.toLong()
                val runnable = Runnable {
                    if (activeHit === hit) {
                        service.sendBackspaceImmediate()
                        triggerHapticNow()
                        service.showSystemKeyboardPicker()
                    }
                }
                spaceLongPressRunnable = runnable
                handler.postDelayed(runnable, spaceDelay)
            }
            NativeActionType.BACKSPACE -> {
                service.sendBackspaceImmediate()
                triggerHapticNow()
                invalidate()
                val initialDelay = if (longPressMs <= 0) 35L else longPressMs.toLong()
                val runnable = object : Runnable {
                    override fun run() {
                        if (activeHit === hit) {
                            service.sendBackspaceImmediate()
                            handler.postDelayed(this, 45L)
                        }
                    }
                }
                repeatRunnable = runnable
                handler.postDelayed(runnable, initialDelay)
            }
            NativeActionType.ENTER -> {
                service.sendEnterImmediate()
                triggerHapticNow()
                invalidate()
            }
            NativeActionType.SHIFT -> {
                if (!isShiftActive && !isCapsLock) {
                    isShiftActive = true
                    isCapsLock = false
                } else if (isShiftActive && !isCapsLock) {
                    isShiftActive = true
                    isCapsLock = true
                } else {
                    isShiftActive = false
                    isCapsLock = false
                }
                triggerHapticNow()
                invalidate()
            }
            NativeActionType.CTRL -> {
                isCtrlActive = !isCtrlActive
                triggerHapticNow()
                invalidate()
            }
            NativeActionType.ALT -> {
                isAltActive = !isAltActive
                triggerHapticNow()
                invalidate()
            }
            NativeActionType.ESCAPE -> {
                val meta = calculateMetaState(isShiftActive || isCapsLock)
                if (isShiftActive && !isCapsLock) isShiftActive = false
                isCtrlActive = false
                isAltActive = false
                service.sendKeyCodeImmediate(KeyEvent.KEYCODE_ESCAPE, meta)
                triggerHapticNow()
                invalidate()
            }
            NativeActionType.ARROW_LEFT,
            NativeActionType.ARROW_DOWN,
            NativeActionType.ARROW_UP,
            NativeActionType.ARROW_RIGHT -> {
                val meta = calculateMetaState(isShiftActive || isCapsLock)
                service.sendKeyCodeImmediate(spec.keyCode, meta)
                triggerHapticNow()
                invalidate()
                val initialDelay = if (longPressMs <= 0) 35L else longPressMs.toLong()
                val runnable = object : Runnable {
                    override fun run() {
                        if (activeHit === hit) {
                            service.sendKeyCodeImmediate(spec.keyCode, meta)
                            handler.postDelayed(this, 45L)
                        }
                    }
                }
                repeatRunnable = runnable
                handler.postDelayed(runnable, initialDelay)
            }
            NativeActionType.SWITCH_LAYOUT -> {
                isSymbolLayout = !isSymbolLayout
                rebuildHitRects(width.toFloat())
                triggerHapticNow()
                invalidate()
            }
            NativeActionType.OPEN_LAYOUT_MENU -> {
                isLayoutMenuOpen = !isLayoutMenuOpen
                rebuildHitRects(width.toFloat())
                triggerHapticNow()
                invalidate()
            }
            NativeActionType.CLOSE_LAYOUT_MENU -> {
                isLayoutMenuOpen = false
                rebuildHitRects(width.toFloat())
                triggerHapticNow()
                invalidate()
            }
            NativeActionType.SELECT_LAYOUT_QWERTY -> {
                selectLayoutMode("qwerty")
            }
            NativeActionType.SELECT_LAYOUT_ABCD -> {
                selectLayoutMode("abcd")
            }
            NativeActionType.SELECT_LAYOUT_LEFT -> {
                selectLayoutMode("left_hand")
            }
            NativeActionType.SELECT_LAYOUT_RIGHT -> {
                selectLayoutMode("right_hand")
            }
            NativeActionType.SELECT_LAYOUT_BOTH -> {
                selectLayoutMode("both_hand")
            }
            NativeActionType.TOGGLE_RESEARCH_COLORS -> {
                researchColorsEnabled = !researchColorsEnabled
                service.saveResearchColorsEnabled(researchColorsEnabled)
                rebuildHitRects(width.toFloat())
                triggerHapticNow()
                invalidate()
            }
            NativeActionType.SWITCH_IME -> {
                triggerHapticNow()
                service.showSystemKeyboardPicker()
                invalidate()
            }
            NativeActionType.TOGGLE_RESIZE -> {
                isResizeBarOpen = !isResizeBarOpen
                service.saveKeyHeightDp(keyHeightDp, isResizeBarOpen)
                rebuildHitRects(width.toFloat())
                requestLayout()
                triggerHapticNow()
                invalidate()
            }
            NativeActionType.OPEN_SETTINGS -> {
                triggerHapticNow()
                service.openKeyboardSettings()
                invalidate()
            }
            NativeActionType.RESIZE_MINUS -> {
                triggerHapticNow()
                updateKeyHeightLive(keyHeightDp - 4.0f)
            }
            NativeActionType.RESIZE_PLUS -> {
                triggerHapticNow()
                updateKeyHeightLive(keyHeightDp + 4.0f)
            }
            NativeActionType.RESIZE_RESET -> {
                triggerHapticNow()
                updateKeyHeightLive(FlutterInputMethodService.DEFAULT_KEY_HEIGHT_DP)
            }
            NativeActionType.RESIZE_DONE -> {
                isResizeBarOpen = false
                service.saveKeyHeightDp(keyHeightDp, false)
                rebuildHitRects(width.toFloat())
                requestLayout()
                triggerHapticNow()
                invalidate()
            }
            NativeActionType.LONG_PRESS_MINUS -> {
                triggerHapticNow()
                val step = if (longPressMs <= 300) 50 else 100
                updateLongPressMsLive(longPressMs - step)
            }
            NativeActionType.LONG_PRESS_PLUS -> {
                triggerHapticNow()
                val step = if (longPressMs < 300) 50 else 100
                updateLongPressMsLive(longPressMs + step)
            }
            NativeActionType.LONG_PRESS_RESET -> {
                triggerHapticNow()
                updateLongPressMsLive(FlutterInputMethodService.DEFAULT_LONG_PRESS_MS)
            }
            NativeActionType.LONG_PRESS_ZERO -> {
                triggerHapticNow()
                updateLongPressMsLive(if (longPressMs == 0) FlutterInputMethodService.DEFAULT_LONG_PRESS_MS else 0)
            }
        }
    }

    override fun onTouchEvent(event: MotionEvent): Boolean {
        val action = event.actionMasked
        val index = event.actionIndex

        when (action) {
            MotionEvent.ACTION_DOWN, MotionEvent.ACTION_POINTER_DOWN -> {
                cancelTimers()
                val ptrId = event.getPointerId(index)
                val x = event.getX(index)
                val y = event.getY(index)
                val hit = findHit(x, y)
                if (hit == null) {
                    if (isLayoutMenuOpen) {
                        isLayoutMenuOpen = false
                        rebuildHitRects(width.toFloat())
                        triggerHapticNow()
                        invalidate()
                    }
                    return true
                }

                activePointerId = ptrId
                activeHit = hit
                startX = x
                startY = y
                activeCornerDir = 0
                popupLabel = null
                popupAnchorRect = null
                isDraggingResizeButton = (hit.spec.action == NativeActionType.TOGGLE_RESIZE)
                resizeDragLastY = y

                executeKeyActionOnDown(hit)
                return true
            }

            MotionEvent.ACTION_MOVE -> {
                val ptrIdx = event.findPointerIndex(activePointerId)
                if (ptrIdx < 0) return true
                val x = event.getX(ptrIdx)
                val y = event.getY(ptrIdx)
                val hit = activeHit ?: return true

                if (isDraggingResizeButton) {
                    val dyPx = y - resizeDragLastY
                    if (abs(dyPx) > dp(4.0f)) {
                        val deltaDp = -(dyPx / resources.displayMetrics.density) * 0.35f
                        resizeDragLastY = y
                        updateKeyHeightLive(keyHeightDp + deltaDp)
                    }
                    return true
                }

                if (hit.spec.action == NativeActionType.INPUT_CHAR && !wasModifierComboOnDown) {
                    val dx = x - startX
                    val dy = y - startY
                    val dist = hypot(dx.toDouble(), dy.toDouble()).toFloat()
                    val newDir = if (dist < dp(13.0f)) {
                        0
                    } else if (dx < 0 && dy < 0) {
                        1 // Top-Left
                    } else if (dx >= 0 && dy < 0) {
                        2 // Top-Right
                    } else if (dx < 0 && dy >= 0) {
                        3 // Bottom-Left
                    } else {
                        4 // Bottom-Right
                    }

                    if (newDir != activeCornerDir) {
                        if (newDir != 0) {
                            charLongPressRunnable?.let { handler.removeCallbacks(it) }
                            charLongPressRunnable = null
                        }
                        activeCornerDir = newDir
                        val newOut = resolveCharOutput(hit.spec, newDir, wasShiftedOnDown)
                        if (newOut != currentCommittedOutput) {
                            currentCommittedOutput = newOut
                            service.replaceLastTextImmediate(newOut)
                            popupLabel = newOut
                            triggerHapticNow()
                        }
                        invalidate()
                    }
                }
                return true
            }

            MotionEvent.ACTION_UP, MotionEvent.ACTION_POINTER_UP, MotionEvent.ACTION_CANCEL -> {
                val ptrId = event.getPointerId(index)
                if (ptrId == activePointerId || action == MotionEvent.ACTION_CANCEL) {
                    cancelTimers()
                    activePointerId = MotionEvent.INVALID_POINTER_ID
                    activeHit = null
                    activeCornerDir = 0
                    popupLabel = null
                    popupAnchorRect = null
                    isDraggingResizeButton = false
                    invalidate()
                }
                return true
            }
        }
        return true
    }
}
