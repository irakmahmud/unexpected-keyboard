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

## 🧪 Version 2 (Default Layout) + Live `V2`/`V1` Layer Button (Release: `v1.2.3`)
- **Default Layout (`V2`):** Sequential `a` through `z` layout with 5 letters per row (`a..e`, `f..j`, `k..o`, `p..t`), a dedicated Action & Symbol Row directly above the Number Row (`⇧  ?123  V2  ?  /  ,  .  Space  ⌫  ↵`), and only letters in the bottom row (`u, v, w, x` + compact `y, z`).
- **Live Layer Switch (`V2` ⇄ `V1` Button):** Placed in both the Top Terminal Bar (right after `ALT`) and in Row 0 (right next to `?123`). Clicking **`V2`** immediately switches the letter alignment to **Version 1 (`V1`)** (`q..y`, `a..h`, `z..n`, `u i j k m o`, `, . Space p l ⌫ ↵`) and highlights the button as **`V1`**. Clicking **`V1`** again switches right back to **Version 2 (`V2` Default)**.
- **Row 0 (Action & Symbol Row above Number Row):** `⇧  ?123  V2/V1  ?  /  ,  .  Space  ⌫  ↵`
- **Row 1 (Numbers):** `1  2  3  4  5  6  7  8  9  0`
- **Row 2 (5 keys in V2 / 6 keys in V1):** `a  b  c  d  e` (V2) / `q  w  e  r  t  y` (V1)
- **Row 3 (5 keys in V2 / 6 keys in V1):** `f  g  h  i  j` (V2) / `a  s  d  f  g  h` (V1)
- **Row 4 (5 keys in V2 / 6 keys in V1):** `k  l  m  n  o` (V2) / `z  x  c  v  b  n` (V1)
- **Row 5 (5 keys in V2 / 6 keys in V1):** `p  q  r  s  t` (V2) / `u  i  j  k  m  o` (V1)
- **Row 6 (Bottom Row):** `u (1.0)  v (1.0)  w (1.0)  x (1.0)  y (0.55)  z (0.55)` (V2) / `,  .  Space  p  l  ⌫  ↵` (V1)
