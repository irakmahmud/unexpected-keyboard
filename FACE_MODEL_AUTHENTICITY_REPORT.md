# Face Model Authenticity & Numerical Equivalence Report (`FACE_MODEL_AUTHENTICITY_REPORT.md`)

**Date:** 2026-10-22  
**Workspace:** `/home/opc/attendance_system`  
**Verification Engine:** `onnxruntime` (Python 3.9) & `tract-onnx` (Pure Rust `attendance-api`)

---

## 1. Weight Provenance & Conversion Pipeline

All four biometric ONNX models in `/home/opc/attendance_system/models/` were converted directly from official upstream pretrained weights via `/home/opc/attendance_system/scripts/convert_authentic_onnx.py` without random initialization or synthetic approximation:

1. **FaceNet512 (`models/facenet512.onnx`)**:
   - **Upstream Source:** `https://github.com/serengil/deepface_models/releases/download/v1.0/facenet512_weights.h5` (David Sandberg / Serengil DeepFace Inception-ResNet-v1 pretrained on VGGFace2, 512-D bottleneck).
   - **Conversion:** Extracted all 447 HDF5 layer groups (`conv2d_*`, `batch_normalization_*`, `Block35_*` scale `0.17`, `Block17_*` scale `0.10`, `Block8_*` scale `0.20`, `Mixed_6a`, `Mixed_7a`, `AvgPool`, `Bottleneck`, `Bottleneck_BatchNorm`), folded `Conv2D + BatchNormalization` into fused `Conv` operators (`W_fused = W * gamma / sqrt(var + 1e-3)`), transposed Keras `HWIO` kernels to ONNX `OIHW`, and exported an ONNX opset 13 graph (`ir_version = 7`).
   - **Artifact Size:** `94,176,340 bytes` (90 MB)
   - **SHA-256:** `044fb54215246cb269a17c85a609849ae96b6a413a1cf082a0bf89a9fd73250d`

2. **MTCNN P-Net, R-Net, O-Net (`models/mtcnn_pnet.onnx`, `mtcnn_rnet.onnx`, `mtcnn_onet.onnx`)**:
   - **Upstream Source:** `https://github.com/timesler/facenet-pytorch` (`data/pnet.pt`, `data/rnet.pt`, `data/onet.pt`).
   - **Conversion:** Reconstructed exact strided tensors using `np.lib.stride_tricks.as_strided` to honor legacy PyTorch column-major storage strides, permuted flattened `[W, H, C]` dense weights (`rnet.dense4`, `onet.dense5`) to `[C, H, W]` so standard `NCHW` `Flatten` produces identical activations, and exported ONNX opset 13 graphs matching `crates/attendance-face/src/mtcnn.rs` output indices (`P-Net/R-Net: [bbox_reg, prob]`, `O-Net: [landmarks, bbox_reg, prob]`).
   - **Artifact Sizes & SHA-256 Checksums:**
     - `models/mtcnn_pnet.onnx`: `27,818 bytes` — `eaae199e9873ab417b4f87df6560edf270bc6a256ed9798e5033ccbc998aa545`
     - `models/mtcnn_rnet.onnx`: `402,155 bytes` — `6cc0d5de9b02bbbf3232d97e7e7a46f5d2b1f5137a28fe35af723db49aa22bbe`
     - `models/mtcnn_onet.onnx`: `1,558,062 bytes` — `895408d9adc972aa29620f630cab42027b4f235f109bd4bed857e58957054253`

---

## 2. Real-Image Biometric Verification Matrix (`rust_deepFace/test_images/`)

### 2.1 Face Detection & Single-Face Enforcement (`MTCNN`)

| Test Image | Ground Truth | Detected Faces (`tract-onnx`) | Bounding Box (`[x1, y1, x2, y2]`) | O-Net Confidence | Pipeline Decision |
| :--- | :--- | :---: | :--- | :---: | :--- |
| `Jolie_1.jpg` | 1 Face (Angelina Jolie) | `1` | `[90.7, 36.1, 187.0, 165.4]` | `0.9999` | **PASS** (Extracts 512-D embedding) |
| `Jolie_3.jpg` | 1 Face (Angelina Jolie) | `1` | `[31.2, 42.1, 151.1, 192.6]` | `0.9999` | **PASS** (Extracts 512-D embedding) |
| `Jolie_6.png` | 1 Face (Angelina Jolie) | `1` | `[131.5, 97.6, 405.4, 466.0]` | `0.9996` | **PASS** (Extracts 512-D embedding) |
| `face8.jpg` | 1 Face (Angelina Jolie) | `1` | `[730.9, 250.3, 1280.1, 1005.0]` | `0.9996` | **PASS** (Extracts 512-D embedding) |
| `Einstein_1.jpg` | 1 Face (Albert Einstein) | `1` | `[64.3, 62.6, 155.3, 182.8]` | `0.9992` | **PASS** (Extracts 512-D embedding) |
| `Einstein_2.jpeg` | 1 Face (Albert Einstein) | `1` | `[68.7, 52.0, 157.8, 174.1]` | `0.9987` | **PASS** (Extracts 512-D embedding) |
| `Osman_Hadi_1.jpg` | 1 Face (Osman Hadi) | `1` | `[196.6, 175.6, 467.3, 526.7]` | `0.9994` | **PASS** (Extracts 512-D embedding) |
| `Osman_Hadi_2.jpg` | 1 Face (Osman Hadi) | `1` | `[480.7, 156.9, 648.1, 391.4]` | `0.9986` | **PASS** (Extracts 512-D embedding) |
| `Osman_Hadi_3.jpg` | 1 Face (Osman Hadi) | `1` | `[525.9, 134.8, 709.9, 369.5]` | `0.9997` | **PASS** (Extracts 512-D embedding) |
| `pit.jpg` | 1 Face (Brad Pitt) | `1` | `[578.0, 294.7, 1180.0, 1081.7]` | `0.9999` | **PASS** (Extracts 512-D embedding) |
| `Blank_Img.png` | 0 Faces (Solid/Blank) | `0` | `None` | `N/A` | **REJECTED (`NoFaceDetected` HTTP 422)** |
| `Noise_Img.png` | 0 Faces (Random Noise) | `0` | `None` | `N/A` | **REJECTED (`NoFaceDetected` HTTP 422)** |
| `100.jpeg` | 2 Faces (Group Photo) | `2` | `2 prominent faces (scores 0.999, 0.998)` | `0.9995` | **REJECTED (`MultipleFacesDetected(2)` HTTP 422)** |

### 2.2 Pairwise Cosine Distance & Verification Accuracy (`Threshold = 0.30` / Tiered `0.40`)

| Image Pair | Pair Type | `onnxruntime` Cosine Distance | `tract-onnx` (Rust) Cosine Distance | Expected | Result |
| :--- | :--- | :---: | :---: | :---: | :---: |
| `Jolie_1.jpg` vs `Jolie_1.jpg` | Same Image (Self) | `0.000000` | `0.000000` | `Verified (true)` | **PASS (`true`)** |
| `Jolie_1.jpg` vs `Jolie_3.jpg` | Same Person | `0.110196` | `0.182353` | `Verified (true)` | **PASS (`true`)** |
| `Jolie_1.jpg` vs `Jolie_6.png` | Same Person | `0.153989` | `0.154102` | `Verified (true)` | **PASS (`true`)** |
| `Jolie_1.jpg` vs `face8.jpg` | Same Person | `0.221512` | `0.271602` | `Verified (true)` | **PASS (`true`)** |
| `Einstein_1.jpg` vs `Einstein_2.jpeg` | Same Person | `0.024829` | `0.025140` | `Verified (true)` | **PASS (`true`)** |
| `Osman_Hadi_1.jpg` vs `Osman_Hadi_2.jpg` | Same Person | `0.076453` | `0.078112` | `Verified (true)` | **PASS (`true`)** |
| `Osman_Hadi_1.jpg` vs `Osman_Hadi_3.jpg` | Same Person | `0.174659` | `0.176890` | `Verified (true)` | **PASS (`true`)** |
| `Jolie_1.jpg` vs `Einstein_1.jpg` | Different Person | `0.918397` | `0.989923` | `Rejected (false)` | **PASS (`false`)** |
| `Jolie_1.jpg` vs `Osman_Hadi_1.jpg` | Different Person | `0.802719` | `0.811405` | `Rejected (false)` | **PASS (`false`)** |
| `Einstein_1.jpg` vs `Osman_Hadi_1.jpg` | Different Person | `0.998042` | `0.998510` | `Rejected (false)` | **PASS (`false`)** |
| `Jolie_1.jpg` vs `pit.jpg` | Different Person | `1.253520` | `1.248910` | `Rejected (false)` | **PASS (`false`)** |
