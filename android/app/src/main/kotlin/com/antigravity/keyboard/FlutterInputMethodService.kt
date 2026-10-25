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
        const val DEFAULT_KEY_HEIGHT_DP = 46.0f
    }

    override fun onCreateInputView(): View {
        val view = NativeKeyboardView(this, this)
        keyboardView = view
        view.reloadConfigFromPrefs()
        return view
    }

    override fun onStartInputView(info: EditorInfo?, restarting: Boolean) {
        super.onStartInputView(info, restarting)
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
        val resizeH = if (isResizeBarOpen) 40.0f else 0.0f
        val totalDp = (6.0f * clamped + modH + resizeH + 4.0f).toInt().coerceIn(220, 960)
        val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        prefs.edit()
            .putFloat(KEY_ROW_HEIGHT_DP, clamped)
            .putInt(KEY_TOTAL_HEIGHT_DP, totalDp)
            .apply()
    }

    fun isHapticsEnabled(): Boolean {
        val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        return prefs.getBoolean(KEY_HAPTICS_ENABLED, true)
    }

    fun getSavedThemeMode(): String {
        val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        return prefs.getString(KEY_THEME_MODE, "amoled") ?: "amoled"
    }

    fun commitTextImmediate(text: String, metaState: Int, useModifiers: Boolean) {
        val ic: InputConnection? = currentInputConnection ?: return
        if (useModifiers && text.isNotEmpty()) {
            val keyCode = charToKeyCode(text[0])
            if (keyCode != KeyEvent.KEYCODE_UNKNOWN) {
                val now = System.currentTimeMillis()
                ic.sendKeyEvent(KeyEvent(now, now, KeyEvent.ACTION_DOWN, keyCode, 0, metaState))
                ic.sendKeyEvent(KeyEvent(now, now, KeyEvent.ACTION_UP, keyCode, 0, metaState))
                return
            }
        }
        ic.commitText(text, 1)
    }

    fun replaceLastTextImmediate(text: String) {
        val ic: InputConnection? = currentInputConnection ?: return
        ic.beginBatchEdit()
        ic.deleteSurroundingText(1, 0)
        ic.commitText(text, 1)
        ic.endBatchEdit()
    }

    fun sendBackspaceImmediate() {
        val ic: InputConnection? = currentInputConnection
        if (ic != null) {
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
        val action = info?.imeOptions?.and(EditorInfo.IME_MASK_ACTION) ?: EditorInfo.IME_ACTION_UNSPECIFIED
        if (ic != null && action != EditorInfo.IME_ACTION_UNSPECIFIED && action != EditorInfo.IME_ACTION_NONE) {
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
    SWITCH_IME,
    TOGGLE_RESIZE,
    OPEN_SETTINGS,
    RESIZE_MINUS,
    RESIZE_PLUS,
    RESIZE_RESET,
    RESIZE_DONE
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

    private var keyHeightDp: Float = 46.0f
    private var themeMode: String = "amoled"
    private var hapticsEnabled: Boolean = true

    private var isShiftActive: Boolean = false
    private var isCapsLock: Boolean = false
    private var isCtrlActive: Boolean = false
    private var isAltActive: Boolean = false
    private var isSymbolLayout: Boolean = false
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

    private val hitRects = ArrayList<HitRect>(64)

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

    private val topBarSpecs = listOf(
        NativeKeySpec("ESC", action = NativeActionType.ESCAPE, flex = 0.9f, keyCode = 111),
        NativeKeySpec("⇧", action = NativeActionType.SHIFT, flex = 1.0f),
        NativeKeySpec("CTRL", action = NativeActionType.CTRL, flex = 1.0f, keyCode = 113),
        NativeKeySpec("ALT", action = NativeActionType.ALT, flex = 1.0f, keyCode = 57),
        NativeKeySpec("?123", action = NativeActionType.SWITCH_LAYOUT, flex = 1.1f),
        NativeKeySpec("◀", action = NativeActionType.ARROW_LEFT, flex = 0.8f, keyCode = 21),
        NativeKeySpec("▼", action = NativeActionType.ARROW_DOWN, flex = 0.8f, keyCode = 20),
        NativeKeySpec("▲", action = NativeActionType.ARROW_UP, flex = 0.8f, keyCode = 19),
        NativeKeySpec("▶", action = NativeActionType.ARROW_RIGHT, flex = 0.8f, keyCode = 22),
        NativeKeySpec("🌐", action = NativeActionType.SWITCH_IME, flex = 0.85f),
        NativeKeySpec("↕", action = NativeActionType.TOGGLE_RESIZE, flex = 0.85f),
        NativeKeySpec("⚙️", action = NativeActionType.OPEN_SETTINGS, flex = 0.85f)
    )

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

    private val qwertyRows = listOf(
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

    private val symbolRows = listOf(
        numberRow,
        listOf(
            NativeKeySpec("!", "~", "`"),
            NativeKeySpec("@", "#", "$"),
            NativeKeySpec("#", "$", "€"),
            NativeKeySpec("$", "€", "₹"),
            NativeKeySpec("%", "‰", "^"),
            NativeKeySpec("^", "&", "*")
        ),
        listOf(
            NativeKeySpec("+", "±", "×"),
            NativeKeySpec("-", "÷", "—"),
            NativeKeySpec("=", "≠", "≈"),
            NativeKeySpec("/", "\\", "|"),
            NativeKeySpec("<", "≤", "«"),
            NativeKeySpec(">", "≥", "»")
        ),
        listOf(
            NativeKeySpec("|", "¦", "§"),
            NativeKeySpec("_", "—", "-"),
            NativeKeySpec("\\", "/", "|"),
            NativeKeySpec("~", "`", "^"),
            NativeKeySpec("`", "\"", "'"),
            NativeKeySpec(":", ";", ".")
        ),
        listOf(
            NativeKeySpec("&", "*", "("),
            NativeKeySpec("*", "(", ")"),
            NativeKeySpec("{", "[", "("),
            NativeKeySpec("}", "]", ")"),
            NativeKeySpec(";", ":", ","),
            NativeKeySpec("\"", "'", "`")
        ),
        listOf(
            NativeKeySpec("ABC", action = NativeActionType.SWITCH_LAYOUT, flex = 1.0f),
            NativeKeySpec(",", "<"),
            NativeKeySpec(".", ">"),
            NativeKeySpec(" ", topRight = "🌐", action = NativeActionType.SPACE, flex = 2.0f),
            NativeKeySpec("ABC", action = NativeActionType.SWITCH_LAYOUT, flex = 1.0f),
            NativeKeySpec("⌫", action = NativeActionType.BACKSPACE, flex = 1.0f, keyCode = 67),
            NativeKeySpec("↵", action = NativeActionType.ENTER, flex = 1.0f, keyCode = 66)
        )
    )

    fun reloadConfigFromPrefs() {
        keyHeightDp = service.getSavedKeyHeightDp()
        themeMode = service.getSavedThemeMode()
        hapticsEnabled = service.isHapticsEnabled()
        applyThemePalette(themeMode)
        requestLayout()
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
        val resizeH = if (isResizeBarOpen) 40.0f else 0.0f
        val totalDp = 6.0f * keyHeightDp + modH + resizeH + 2.0f
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

        // Optional inline resize control bar
        if (isResizeBarOpen) {
            val resizeH = dp(40.0f)
            val resizeSpecs = listOf(
                NativeKeySpec("− Height", action = NativeActionType.RESIZE_MINUS, flex = 1.0f),
                NativeKeySpec("${keyHeightDp.toInt()}dp (Reset)", action = NativeActionType.RESIZE_RESET, flex = 1.4f),
                NativeKeySpec("+ Height", action = NativeActionType.RESIZE_PLUS, flex = 1.0f),
                NativeKeySpec("Done ✓", action = NativeActionType.RESIZE_DONE, flex = 1.0f)
            )
            val totalFlex = resizeSpecs.fold(0f) { acc, s -> acc + s.flex }
            var left = 0f
            for (spec in resizeSpecs) {
                val kw = viewWidth * (spec.flex / totalFlex)
                hitRects.add(HitRect(spec, RectF(left, currentTop, left + kw, currentTop + resizeH), isTopBar = true))
                left += kw
            }
            currentTop += resizeH
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

        // 6 Main Keyboard Rows
        val rows = if (isSymbolLayout) symbolRows else qwertyRows
        val rowH = dp(keyHeightDp)
        for (row in rows) {
            val totalRowFlex = row.fold(0f) { acc, s -> acc + s.flex }
            var left = 0f
            for (spec in row) {
                val kw = viewWidth * (spec.flex / totalRowFlex)
                hitRects.add(HitRect(spec, RectF(left, currentTop, left + kw, currentTop + rowH), isTopBar = false))
                left += kw
            }
            currentTop += rowH
        }
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

        val strokePx = max(1f, dp(0.5f))
        val activeStrokePx = max(2f, dp(1.2f))
        val cornerPaddingX = dp(3.5f)
        val cornerPaddingY = dp(2.0f)
        cornerTextPaint.textSize = dp(9.5f)

        for (hit in hitRects) {
            val spec = hit.spec
            val r = hit.rect
            val isPressed = (activeHit === hit)

            val isActiveMod = (spec.action == NativeActionType.SHIFT && (isShiftActive || isCapsLock)) ||
                    (spec.action == NativeActionType.CTRL && isCtrlActive) ||
                    (spec.action == NativeActionType.ALT && isAltActive) ||
                    (spec.action == NativeActionType.SWITCH_LAYOUT && spec.primary == "ABC") ||
                    (spec.action == NativeActionType.TOGGLE_RESIZE && isResizeBarOpen) ||
                    (spec.action == NativeActionType.RESIZE_DONE)

            bgPaint.color = when {
                isActiveMod -> modActiveColor
                isPressed -> keyPressedColor
                else -> keyBgColor
            }
            canvas.drawRect(r, bgPaint)

            borderPaint.color = if (isActiveMod) accentColor else modInactiveColor
            borderPaint.strokeWidth = if (isActiveMod) activeStrokePx else strokePx
            canvas.drawRect(r, borderPaint)

            // Primary label
            val displayPrimary = when {
                spec.action == NativeActionType.SPACE -> "␣"
                spec.action == NativeActionType.SHIFT && isCapsLock -> "Caps"
                (isShiftActive || isCapsLock) && spec.primary.length == 1 -> spec.primary.uppercase()
                else -> spec.primary
            }

            val fgColor = when {
                isActiveMod -> 0xFFFFFFFF.toInt()
                spec.action == NativeActionType.SWITCH_IME -> accentColor
                else -> primaryTextColor
            }

            val cellH = r.height()
            val cellW = r.width()
            val isMulti = displayPrimary.length > 1

            val paint = if (hit.isTopBar || isMulti) multiCharTextPaint else primaryTextPaint
            paint.color = fgColor

            var targetTextSize = when {
                hit.isTopBar -> if (isMulti) cellH * 0.34f else cellH * 0.46f
                isMulti -> cellH * 0.34f
                spec.action == NativeActionType.SPACE -> cellH * 0.65f
                else -> cellH * 0.72f
            }
            val maxAllowedW = cellW * 0.86f
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

            // 4 Corner Symbols
            if (!hit.isTopBar) {
                val cFm = cornerTextPaint.fontMetrics
                if (spec.topLeft != null) {
                    cornerTextPaint.color = if (isPressed && activeCornerDir == 1) accentColor else cornerTextColor
                    cornerTextPaint.textAlign = Paint.Align.LEFT
                    canvas.drawText(spec.topLeft, r.left + cornerPaddingX, r.top + cornerPaddingY - cFm.ascent, cornerTextPaint)
                }
                if (spec.topRight != null) {
                    cornerTextPaint.color = if (isPressed && activeCornerDir == 2) accentColor else cornerTextColor
                    cornerTextPaint.textAlign = Paint.Align.RIGHT
                    canvas.drawText(spec.topRight, r.right - cornerPaddingX, r.top + cornerPaddingY - cFm.ascent, cornerTextPaint)
                }
                if (spec.bottomLeft != null) {
                    cornerTextPaint.color = if (isPressed && activeCornerDir == 3) accentColor else cornerTextColor
                    cornerTextPaint.textAlign = Paint.Align.LEFT
                    canvas.drawText(spec.bottomLeft, r.left + cornerPaddingX, r.bottom - cornerPaddingY - cFm.descent, cornerTextPaint)
                }
                if (spec.bottomRight != null) {
                    cornerTextPaint.color = if (isPressed && activeCornerDir == 4) accentColor else cornerTextColor
                    cornerTextPaint.textAlign = Paint.Align.RIGHT
                    canvas.drawText(spec.bottomRight, r.right - cornerPaddingX, r.bottom - cornerPaddingY - cFm.descent, cornerTextPaint)
                }
            }
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
    }

    private fun updateKeyHeightLive(newHeightDp: Float) {
        keyHeightDp = newHeightDp.coerceIn(34.0f, 136.0f)
        service.saveKeyHeightDp(keyHeightDp, isResizeBarOpen)
        rebuildHitRects(width.toFloat())
        requestLayout()
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
            }
            NativeActionType.SPACE -> {
                if (isShiftActive && !isCapsLock) isShiftActive = false
                isCtrlActive = false
                isAltActive = false
                service.commitTextImmediate(" ", 0, false)
                triggerHapticNow()
                invalidate()

                // Hold Spacebar 450ms to switch keyboard (Ridmik / Gboard style)
                val runnable = Runnable {
                    if (activeHit === hit) {
                        service.sendBackspaceImmediate()
                        triggerHapticNow()
                        service.showSystemKeyboardPicker()
                    }
                }
                spaceLongPressRunnable = runnable
                handler.postDelayed(runnable, 450L)
            }
            NativeActionType.BACKSPACE -> {
                service.sendBackspaceImmediate()
                triggerHapticNow()
                invalidate()
                val runnable = object : Runnable {
                    override fun run() {
                        if (activeHit === hit) {
                            service.sendBackspaceImmediate()
                            handler.postDelayed(this, 45L)
                        }
                    }
                }
                repeatRunnable = runnable
                handler.postDelayed(runnable, 350L)
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
                val runnable = object : Runnable {
                    override fun run() {
                        if (activeHit === hit) {
                            service.sendKeyCodeImmediate(spec.keyCode, meta)
                            handler.postDelayed(this, 45L)
                        }
                    }
                }
                repeatRunnable = runnable
                handler.postDelayed(runnable, 350L)
            }
            NativeActionType.SWITCH_LAYOUT -> {
                isSymbolLayout = !isSymbolLayout
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
                val hit = findHit(x, y) ?: return true

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
