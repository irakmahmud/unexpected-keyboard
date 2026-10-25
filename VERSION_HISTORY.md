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

## 🧪 Version 2 (Experimental Alphabetical Layout — `a, b, c, d ... y, z`, Release: `v1.2.0`)
Sequential `a` through `z` layout across the 6-key rows, ending with `y, z` on the bottom row:
- **Row 0 (Numbers):** `1  2  3  4  5  6  7  8  9  0`
- **Row 1 (6 keys):** `a  b  c  d  e  f`
- **Row 2 (6 keys):** `g  h  i  j  k  l`
- **Row 3 (6 keys):** `m  n  o  p  q  r`
- **Row 4 (6 keys):** `s  t  u  v  w  x`
- **Row 5 (Bottom):** `,  .  Space  y  z  ⌫  ↵`
