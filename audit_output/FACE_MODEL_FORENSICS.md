# FACE MODEL FORENSICS — Phase 4 Investigation Report

> **Mission:** Independently determine whether `models/facenet512.onnx` and `models/mtcnn_*.onnx` are authentic pretrained biometric models or generated placeholders, verify their checksums and weights, and evaluate recognition accuracy.

---

## 1. File Size & SHA-256 Checksum Forensics

Read-only inspection of `/home/opc/attendance_system/models/`:

| Model File | Byte Size | SHA-256 Checksum | Authentic Pretrained Size | Verdict |
|---|---:|---|---:|---|
| `models/facenet512.onnx` | `1,049,690` (~1.05 MB) | `a58a4fe8bb65b21c...` | ~92–95 MB (Inception-ResNet-v1, ~23.5M params) | **SYNTHETIC PLACEHOLDER** |
| `models/mtcnn_pnet.onnx` | `10,972` (~11 KB) | `9a0186c51b0869df...` | ~30 KB | **SYNTHETIC RANDOM WEIGHTS** |
| `models/mtcnn_rnet.onnx` | `41,850` (~41 KB) | `b20e6cb7927e557d...` | ~400 KB | **SYNTHETIC RANDOM WEIGHTS** |
| `models/mtcnn_onet.onnx` | `442,831` (~433 KB) | `2049768940b053e7...` | ~1.5 MB | **SYNTHETIC RANDOM WEIGHTS** |

---

## 2. Origin Proof: `generate_onnx_models.py`

Direct source inspection of `/home/opc/attendance_system/generate_onnx_models.py` (lines 1–242) proves beyond doubt how the 4 files in `models/` were created:

1. **Random Seed Initialization (`generate_onnx_models.py:19`):**
   - Uses `rng = np.random.RandomState(42)` to generate Gaussian random floats (`rng.randn(...)`) and orthogonal random projections.
2. **`models/mtcnn_pnet.onnx`, `mtcnn_rnet.onnx`, `mtcnn_onet.onnx` (`generate_onnx_models.py:28–138`):**
   - Constructed via `onnx.helper.make_model` with random convolution kernels (`rng.randn(...) * 0.05`) and negative classification biases so `prob[:, 1]` rarely or arbitrarily fires.
3. **`models/facenet512.onnx` (`generate_onnx_models.py:140–231`):**
   - Instead of the 100+ layer Inception-ResNet-v1 architecture of David Sandberg's / DeepFace's `Facenet512`, `build_facenet512()` creates a **2-node toy graph**:
     - `AveragePool` (`kernel_shape=[10, 10], strides=[10, 10]`) reducing `[1, 160, 160, 3]` (transposed to `[1, 3, 160, 160]`) into $3 	imes 16 	imes 16 = 768$ patch averages.
     - `MatMul` multiplying the $768$-element pooled image vector by a **random orthogonal $768 	imes 512$ matrix (`W_proj`, $768 	imes 512 	imes 4 = 1,572,864$ bytes compressed)**!
   - **Zero trained facial feature weights exist in `models/facenet512.onnx`.** It is a linear random projection of $10 	imes 10$ downsampled pixel averages!

---

## 3. Answers to Phase 4 Forensic Questions

1. **Are these authentic pretrained models or generated placeholders?**
   - They are **generated placeholders** created by `/home/opc/attendance_system/generate_onnx_models.py` using `np.random.RandomState(42)`. (Notably, a legitimate export script exists at `/home/opc/attendance_system/rust_deepFace/export_to_onnx.py` which uses `tf2onnx.convert.from_keras` on DeepFace's real `Facenet512` model, but it was not used to populate `/home/opc/attendance_system/models/`.)
2. **Are the original trained weights actually present?**
   - **No.** Neither `facenet512_weights.h5` (~95 MB) nor a real converted `facenet512.onnx` (~92 MB) is present on disk.
3. **Does inference genuinely depend on image content?**
   - Yes, in the trivial sense that `facenet512.onnx` computes `AveragePool(10x10) -> MatMul(768x512)` on the input pixels. Different pixel patterns yield different 512-D vectors, and identical pixel patterns yield `0.0000` distance — which disguises the model as "working" in naive tests while having **zero facial identity invariance** across lighting, background, or real head pose changes.
4. **Why does MTCNN never fail even with random weights?**
   - Because `crates/attendance-face/src/pipeline.rs:14–31` wraps `mtcnn::detect_face` in `.ok().flatten()` and, whenever MTCNN fails to detect a face (`None`), **unconditionally substitutes a hardcoded center bounding box (`15%..85%` of image width/height)**! Thus, even an image of a blank wall or a synthetic circle "detects a face" and produces a 512-D unit vector.
5. **Does Rust produce comparable embeddings to the original Python implementation?**
   - **No.** Python's `DeepFace.represent(model_name="Facenet512")` runs a deep Inception-ResNet-v1 CNN trained on VGGFace2, whereas Rust runs a 1-layer $10 	imes 10$ average pool + random $768 	imes 512$ matrix multiply. Their embedding spaces are completely unrelated.
6. **Are 20 distinct camera poses genuinely captured and validated?**
   - **No.** `runAuto20PoseCapture()` (`crates/attendance-api/src/ui/scripts.rs:1291–1304`) captures 20 frames in 2.4 seconds (120ms apart) with zero head-pose angle verification, and the backend (`user_routes.rs:76–150`) accepts 20 identical base64 images without checking inter-frame variance.
7. **Can the system incorrectly report a perfect match using the same test image?**
   - **Yes.** In `verify_v8_full_parity.py` (`lines 115–138`), `frames_20[0]` is sent during registration and the **exact same `frames_20[0]` byte buffer** is sent to `/api/v1/face/verify-identity`, guaranteeing `min_distance = 0.0000` (`confidence = 99.9%`) purely by mathematical determinism ($f(x) = f(x)$), proving nothing about face recognition accuracy.
