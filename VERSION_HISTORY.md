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

## 🧪 Version 2 (Experimental Alphabetical 5-Key Layout + Top Action/Symbol Row — Release: `v1.2.2`)
Sequential `a` through `z` layout with 5 letters per row, a dedicated Action & Symbol Row directly above the Number Row, and only letters in the bottom row (`u, v, w, x` + compact `y, z`):
- **Row 0 (Action & Symbol Row above Number Row):** `⇧  ?123  ?  /  ,  .  Space  ⌫  ↵`
- **Row 1 (Numbers):** `1  2  3  4  5  6  7  8  9  0`
- **Row 2 (5 keys):** `a  b  c  d  e`
- **Row 3 (5 keys):** `f  g  h  i  j`
- **Row 4 (5 keys):** `k  l  m  n  o`
- **Row 5 (5 keys):** `p  q  r  s  t`
- **Row 6 (Bottom Row — ONLY letters):** `u (1.0)  v (1.0)  w (1.0)  x (1.0)  y (0.55)  z (0.55)`
