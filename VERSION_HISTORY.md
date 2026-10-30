# Unexpected Keyboard — Version 1 vs Version 2 Alignment Reference

## 📌 Version 1 (Saved Baseline — Tag: `version-1`, Commit: `53cf92dea28d946573434b5430ab4c0a0040551b`, Release: `v1.1.5`)
When the user asks to revert to **Version 1** or uncomment Version 1, restore this exact layout:
- **Row 0 (Numbers):** `1  2  3  4  5  6  7  8  9  0`
- **Row 1 (6 keys):** `q  w  e  r  t  y`
- **Row 2 (6 keys):** `a  s  d  f  g  h`
- **Row 3 (6 keys):** `z  x  c  v  b  n`
- **Row 4 (6 keys):** `u  i  j  k  m  o`
- **Row 5 (Bottom):** `,  .  Space  p  l  ⌫  ↵`
- **Features included in Version 1:**
  - 100% Native Android Canvas IME (`NativeKeyboardView` in `FlutterInputMethodService.kt`) for 0.0ms external app typing.
  - Ultra-thin (`sans-serif-thin`) font with 5x large letter height and middle-aligned bottom row.
  - Customizable Long Press Delay (`0 ms` to `3000 ms`) in Settings and inline `↕` quick-config bar.
  - External keyboard switcher (`🌐` button + Spacebar long-press).

---

## 🧪 Version 2 (`2. ABCD Serially` Default) + `4. Right Hand Banglish` Official 150-Key Final Rank Layout + Top Number Row (Release: `v1.3.2`)
- **Single Layout Icon Button (`⊞`):** Placed in the Top Terminal Bar (`ESC  CTRL  ALT  ◀  ▼  ▲  ▶  ⊞  🌐  ↕  ⚙️`). Clicking the **`⊞` Layout Icon Button** opens a floating **Popup Menu** over the keyboard with 5 selectable layout options:
  1. **`1. QWERTY Type`** (`"qwerty"` — Version 1 alignment: `q..y`, `a..h`, `z..n`, `u i j k m o`, `, . Space p l ⌫ ↵`)
  2. **`2. ABCD Serially`** (`"abcd"` — **Default** Version 2 alignment: 5 letters per row `a..e`, `f..j`, `k..o`, `p..t`, and bottom row `u, v, w, x` + compact `y, z`)
  3. **`3. Left Hand`** (`"left_hand"` — ready for custom left-handed design)
  4. **`4. Right Hand Banglish`** (`"right_hand"` — Top Number Row `1..9, 0` with uniform color directly below `ESC CTRL` + 9×5 Ergonomic Grid from Official 150-Key Final Rank Table with number gaps `0, 1, 2` filled by `J` #56, `F` #59, `L` #60)
  5. **`5. Both Hand`** (`"both_hand"` — ready for custom two-handed design)
- **Row 0 (Action & Symbol Row above Number Row):** `⇧  ?123  ?  /  ,  .  Space  ⌫  ↵`
- **Row 1 (Numbers):** `1  2  3  4  5  6  7  8  9  0`
- **Row 2 (`2. ABCD Serially` / `1. QWERTY Type`):** `a  b  c  d  e` / `q  w  e  r  t  y`
- **Row 3 (`2. ABCD Serially` / `1. QWERTY Type`):** `f  g  h  i  j` / `a  s  d  f  g  h`
- **Row 4 (`2. ABCD Serially` / `1. QWERTY Type`):** `k  l  m  n  o` / `z  x  c  v  b  n`
- **Row 5 (`2. ABCD Serially` / `1. QWERTY Type`):** `p  q  r  s  t` / `u  i  j  k  m  o`
- **Row 6 (Bottom Row):** `u (1.0)  v (1.0)  w (1.0)  x (1.0)  y (0.55)  z (0.55)` (`2. ABCD Serially`) / `,  .  Space  p  l  ⌫  ↵` (`1. QWERTY Type`)
