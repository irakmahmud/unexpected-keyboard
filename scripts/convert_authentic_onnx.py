import re
import os
import struct
import pickle
import hashlib
import numpy as np
import h5py
import onnx
from onnx import helper, TensorProto, numpy_helper

WEIGHTS_DIR = "/dev/shm/biometric_weights"
OUT_DIR = "/run/user/1000/authentic_models"
os.makedirs(OUT_DIR, exist_ok=True)

def load_legacy_pt(path):
    with open(path, "rb") as f:
        _magic = pickle.load(f)
        _proto = pickle.load(f)
        _sys_info = pickle.load(f)
        class StorageRef:
            def __init__(self, key):
                self.key = key
        class Unpickler(pickle.Unpickler):
            def persistent_load(self, saved_id):
                return StorageRef(saved_id[2])
            def find_class(self, module, name):
                if name == "_rebuild_tensor_v2":
                    def rebuild(storage_ref, offset, size, stride, req_grad, hooks):
                        return (storage_ref.key, offset, size, stride)
                    return rebuild
                if module.startswith("torch"):
                    return type(name, (), {})
                return super().find_class(module, name)
        state_dict_meta = Unpickler(f).load()
        storage_keys = pickle.load(f)
        raw_storages = {}
        for k in storage_keys:
            num_elems = struct.unpack("<q", f.read(8))[0]
            raw_storages[k] = np.frombuffer(f.read(num_elems * 4), dtype=np.float32).copy()
        out = {}
        for k, (skey, offset, size, stride) in state_dict_meta.items():
            arr = np.lib.stride_tricks.as_strided(raw_storages[skey][offset:], shape=size, strides=[st * 4 for st in stride]).copy()
            out[k] = arr
        return out

def build_pnet():
    sd = load_legacy_pt(os.path.join(WEIGHTS_DIR, "pnet.pt"))
    initializers = []
    nodes = []

    def add_init(name, arr):
        initializers.append(numpy_helper.from_array(arr.astype(np.float32), name=name))

    for k, v in sd.items():
        if "prelu" in k:
            add_init(k, v.reshape(-1, 1, 1))
        else:
            add_init(k, v)

    nodes.append(helper.make_node("Conv", ["input", "conv1.weight", "conv1.bias"], ["c1"], kernel_shape=[3, 3], strides=[1, 1], pads=[0, 0, 0, 0]))
    nodes.append(helper.make_node("PRelu", ["c1", "prelu1.weight"], ["p1"]))
    # Note: pads=[0,0,0,0] with kernel 2x2 stride 2
    nodes.append(helper.make_node("MaxPool", ["p1"], ["pool1"], kernel_shape=[2, 2], strides=[2, 2], pads=[0, 0, 0, 0]))
    nodes.append(helper.make_node("Conv", ["pool1", "conv2.weight", "conv2.bias"], ["c2"], kernel_shape=[3, 3], strides=[1, 1], pads=[0, 0, 0, 0]))
    nodes.append(helper.make_node("PRelu", ["c2", "prelu2.weight"], ["p2"]))
    nodes.append(helper.make_node("Conv", ["p2", "conv3.weight", "conv3.bias"], ["c3"], kernel_shape=[3, 3], strides=[1, 1], pads=[0, 0, 0, 0]))
    nodes.append(helper.make_node("PRelu", ["c3", "prelu3.weight"], ["p3"]))
    nodes.append(helper.make_node("Conv", ["p3", "conv4_1.weight", "conv4_1.bias"], ["c4_1"], kernel_shape=[1, 1], strides=[1, 1], pads=[0, 0, 0, 0]))
    nodes.append(helper.make_node("Softmax", ["c4_1"], ["probs"], axis=1))
    nodes.append(helper.make_node("Conv", ["p3", "conv4_2.weight", "conv4_2.bias"], ["reg"], kernel_shape=[1, 1], strides=[1, 1], pads=[0, 0, 0, 0]))

    inp = helper.make_tensor_value_info("input", TensorProto.FLOAT, ["batch", 3, "height", "width"])
    out_reg = helper.make_tensor_value_info("reg", TensorProto.FLOAT, ["batch", 4, "out_h", "out_w"])
    out_probs = helper.make_tensor_value_info("probs", TensorProto.FLOAT, ["batch", 2, "out_h", "out_w"])

    graph = helper.make_graph(nodes, "MTCNN_PNet", [inp], [out_reg, out_probs], initializers)
    model = helper.make_model(graph, producer_name="authentic_facenet_pytorch_exporter", opset_imports=[helper.make_opsetid("", 13)])
    model.ir_version = 7
    onnx.checker.check_model(model)
    onnx.save(model, os.path.join(OUT_DIR, "mtcnn_pnet.onnx"))

def build_rnet():
    sd = load_legacy_pt(os.path.join(WEIGHTS_DIR, "rnet.pt"))
    initializers = []
    nodes = []

    def add_init(name, arr):
        initializers.append(numpy_helper.from_array(arr.astype(np.float32), name=name))

    # Permute dense4.weight from (128, 3, 3, 64) [W, H, C] to (128, 64, 3, 3) [C, H, W]
    w_dense4 = sd["dense4.weight"].reshape(128, 3, 3, 64).transpose(0, 3, 2, 1).reshape(128, 576)

    for k, v in sd.items():
        if k == "dense4.weight":
            add_init(k, w_dense4)
        elif k in ("prelu1.weight", "prelu2.weight", "prelu3.weight"):
            add_init(k, v.reshape(-1, 1, 1))
        elif k == "prelu4.weight":
            add_init(k, v.reshape(1, -1))
        else:
            add_init(k, v)

    nodes.append(helper.make_node("Conv", ["input", "conv1.weight", "conv1.bias"], ["c1"], kernel_shape=[3, 3], strides=[1, 1], pads=[0, 0, 0, 0]))
    nodes.append(helper.make_node("PRelu", ["c1", "prelu1.weight"], ["p1"]))
    # 22x22 with kernel 3x3, stride 2, ceil_mode=True -> pad bottom/right by 1 gives exact 11x11
    nodes.append(helper.make_node("MaxPool", ["p1"], ["pool1"], kernel_shape=[3, 3], strides=[2, 2], pads=[0, 0, 1, 1]))
    nodes.append(helper.make_node("Conv", ["pool1", "conv2.weight", "conv2.bias"], ["c2"], kernel_shape=[3, 3], strides=[1, 1], pads=[0, 0, 0, 0]))
    nodes.append(helper.make_node("PRelu", ["c2", "prelu2.weight"], ["p2"]))
    # 9x9 with kernel 3x3, stride 2 -> 4x4
    nodes.append(helper.make_node("MaxPool", ["p2"], ["pool2"], kernel_shape=[3, 3], strides=[2, 2], pads=[0, 0, 0, 0]))
    nodes.append(helper.make_node("Conv", ["pool2", "conv3.weight", "conv3.bias"], ["c3"], kernel_shape=[2, 2], strides=[1, 1], pads=[0, 0, 0, 0]))
    nodes.append(helper.make_node("PRelu", ["c3", "prelu3.weight"], ["p3"]))
    nodes.append(helper.make_node("Flatten", ["p3"], ["flat"], axis=1))
    nodes.append(helper.make_node("Gemm", ["flat", "dense4.weight", "dense4.bias"], ["d4"], transB=1))
    nodes.append(helper.make_node("PRelu", ["d4", "prelu4.weight"], ["p4"]))
    nodes.append(helper.make_node("Gemm", ["p4", "dense5_1.weight", "dense5_1.bias"], ["d5_1"], transB=1))
    nodes.append(helper.make_node("Softmax", ["d5_1"], ["probs"], axis=1))
    nodes.append(helper.make_node("Gemm", ["p4", "dense5_2.weight", "dense5_2.bias"], ["reg"], transB=1))

    inp = helper.make_tensor_value_info("input", TensorProto.FLOAT, [1, 3, 24, 24])
    out_reg = helper.make_tensor_value_info("reg", TensorProto.FLOAT, [1, 4])
    out_probs = helper.make_tensor_value_info("probs", TensorProto.FLOAT, [1, 2])

    graph = helper.make_graph(nodes, "MTCNN_RNet", [inp], [out_reg, out_probs], initializers)
    model = helper.make_model(graph, producer_name="authentic_facenet_pytorch_exporter", opset_imports=[helper.make_opsetid("", 13)])
    model.ir_version = 7
    onnx.checker.check_model(model)
    onnx.save(model, os.path.join(OUT_DIR, "mtcnn_rnet.onnx"))

def build_onet():
    sd = load_legacy_pt(os.path.join(WEIGHTS_DIR, "onet.pt"))
    initializers = []
    nodes = []

    def add_init(name, arr):
        initializers.append(numpy_helper.from_array(arr.astype(np.float32), name=name))

    # Permute dense5.weight from (256, 3, 3, 128) [W, H, C] to (256, 128, 3, 3) [C, H, W]
    w_dense5 = sd["dense5.weight"].reshape(256, 3, 3, 128).transpose(0, 3, 2, 1).reshape(256, 1152)

    for k, v in sd.items():
        if k == "dense5.weight":
            add_init(k, w_dense5)
        elif k in ("prelu1.weight", "prelu2.weight", "prelu3.weight", "prelu4.weight"):
            add_init(k, v.reshape(-1, 1, 1))
        elif k == "prelu5.weight":
            add_init(k, v.reshape(1, -1))
        else:
            add_init(k, v)

    nodes.append(helper.make_node("Conv", ["input", "conv1.weight", "conv1.bias"], ["c1"], kernel_shape=[3, 3], strides=[1, 1], pads=[0, 0, 0, 0]))
    nodes.append(helper.make_node("PRelu", ["c1", "prelu1.weight"], ["p1"]))
    # 46x46 with kernel 3x3, stride 2, ceil_mode=True -> pad bottom/right by 1 gives 23x23
    nodes.append(helper.make_node("MaxPool", ["p1"], ["pool1"], kernel_shape=[3, 3], strides=[2, 2], pads=[0, 0, 1, 1]))
    nodes.append(helper.make_node("Conv", ["pool1", "conv2.weight", "conv2.bias"], ["c2"], kernel_shape=[3, 3], strides=[1, 1], pads=[0, 0, 0, 0]))
    nodes.append(helper.make_node("PRelu", ["c2", "prelu2.weight"], ["p2"]))
    # 21x21 with kernel 3x3, stride 2 -> 10x10
    nodes.append(helper.make_node("MaxPool", ["p2"], ["pool2"], kernel_shape=[3, 3], strides=[2, 2], pads=[0, 0, 0, 0]))
    nodes.append(helper.make_node("Conv", ["pool2", "conv3.weight", "conv3.bias"], ["c3"], kernel_shape=[3, 3], strides=[1, 1], pads=[0, 0, 0, 0]))
    nodes.append(helper.make_node("PRelu", ["c3", "prelu3.weight"], ["p3"]))
    # 8x8 with kernel 2x2, stride 2 -> 4x4
    nodes.append(helper.make_node("MaxPool", ["p3"], ["pool3"], kernel_shape=[2, 2], strides=[2, 2], pads=[0, 0, 0, 0]))
    nodes.append(helper.make_node("Conv", ["pool3", "conv4.weight", "conv4.bias"], ["c4"], kernel_shape=[2, 2], strides=[1, 1], pads=[0, 0, 0, 0]))
    nodes.append(helper.make_node("PRelu", ["c4", "prelu4.weight"], ["p4"]))
    nodes.append(helper.make_node("Flatten", ["p4"], ["flat"], axis=1))
    nodes.append(helper.make_node("Gemm", ["flat", "dense5.weight", "dense5.bias"], ["d5"], transB=1))
    nodes.append(helper.make_node("PRelu", ["d5", "prelu5.weight"], ["p5"]))
    nodes.append(helper.make_node("Gemm", ["p5", "dense6_1.weight", "dense6_1.bias"], ["d6_1"], transB=1))
    nodes.append(helper.make_node("Softmax", ["d6_1"], ["probs"], axis=1))
    nodes.append(helper.make_node("Gemm", ["p5", "dense6_2.weight", "dense6_2.bias"], ["reg"], transB=1))
    nodes.append(helper.make_node("Gemm", ["p5", "dense6_3.weight", "dense6_3.bias"], ["landmarks"], transB=1))

    inp = helper.make_tensor_value_info("input", TensorProto.FLOAT, [1, 3, 48, 48])
    out_landmarks = helper.make_tensor_value_info("landmarks", TensorProto.FLOAT, [1, 10])
    out_reg = helper.make_tensor_value_info("reg", TensorProto.FLOAT, [1, 4])
    out_probs = helper.make_tensor_value_info("probs", TensorProto.FLOAT, [1, 2])

    # Note: outputs[0] = landmarks [1,10], outputs[1] = reg [1,4], outputs[2] = probs [1,2]
    graph = helper.make_graph(nodes, "MTCNN_ONet", [inp], [out_landmarks, out_reg, out_probs], initializers)
    model = helper.make_model(graph, producer_name="authentic_facenet_pytorch_exporter", opset_imports=[helper.make_opsetid("", 13)])
    model.ir_version = 7
    onnx.checker.check_model(model)
    onnx.save(model, os.path.join(OUT_DIR, "mtcnn_onet.onnx"))

def build_facenet512():
    h5_path = os.path.join(WEIGHTS_DIR, "facenet512_weights.h5")
    h5f = h5py.File(h5_path, "r")

    initializers = []
    nodes = []
    counter = [0]

    def uid(prefix):
        counter[0] += 1
        return f"{prefix}_{counter[0]}"

    def add_init(name, arr):
        initializers.append(numpy_helper.from_array(arr.astype(np.float32), name=name))

    # Input is NHWC [1, 160, 160, 3], transpose to NCHW [1, 3, 160, 160]
    nodes.append(helper.make_node("Transpose", ["input"], ["input_nchw"], perm=[0, 3, 1, 2]))

    def Input(shape):
        return "input_nchw"

    def Conv2D(filters, kernel_size, strides=1, padding="valid", use_bias=False, name=None):
        if isinstance(kernel_size, int):
            kh, kw = kernel_size, kernel_size
        else:
            kh, kw = kernel_size[0], kernel_size[1]
        sh, sw = (strides, strides) if isinstance(strides, int) else (strides[0], strides[1])
        if padding == "same":
            pad_h = (kh - 1) // 2
            pad_w = (kw - 1) // 2
            pads = [pad_h, pad_w, kh - 1 - pad_h, kw - 1 - pad_w]
        else:
            pads = [0, 0, 0, 0]

        h5_name = re.sub(r"Block(17|8)_(\d+)_Branch_\2_", r"Block\1_\2_Branch_1_", name); grp = h5f[h5_name][h5_name]
        # Keras conv kernel is (kH, kW, C_in, C_out) -> ONNX is (C_out, C_in, kH, kW)
        w = np.array(grp["kernel:0"], dtype=np.float32).transpose(3, 2, 0, 1)
        w_name = f"{name}_W"
        add_init(w_name, w)
        inputs = [w_name]
        if use_bias:
            b = np.array(grp["bias:0"], dtype=np.float32)
            b_name = f"{name}_B"
            add_init(b_name, b)
            inputs.append(b_name)

        def call(x):
            out = f"{name}_out"
            nodes.append(helper.make_node(
                "Conv",
                [x] + inputs,
                [out],
                kernel_shape=[kh, kw],
                strides=[sh, sw],
                pads=pads,
                name=name
            ))
            return out
        return call

    def BatchNormalization(axis=3, momentum=0.995, epsilon=0.001, scale=False, name=None):
        h5_name = re.sub(r"Block(17|8)_(\d+)_Branch_\2_", r"Block\1_\2_Branch_1_", name); grp = h5f[h5_name][h5_name]
        beta = np.array(grp["beta:0"], dtype=np.float32)
        mean = np.array(grp["moving_mean:0"], dtype=np.float32)
        var = np.array(grp["moving_variance:0"], dtype=np.float32)
        gamma = np.array(grp["gamma:0"], dtype=np.float32) if "gamma:0" in grp else np.ones_like(beta, dtype=np.float32)

        # Fuse BN into affine scale & bias: y = (x - mean) / sqrt(var + eps) * gamma + beta
        inv_std = gamma / np.sqrt(var + epsilon)
        fused_scale = inv_std
        fused_bias = beta - mean * inv_std

        def call(x):
            # If preceding node is Conv, we can fold directly into Conv weights & bias, OR use BatchNormalization op!
            # Using standard ONNX BatchNormalization op:
            s_name = f"{name}_scale"
            b_name = f"{name}_B"
            m_name = f"{name}_mean"
            v_name = f"{name}_var"
            add_init(s_name, gamma)
            add_init(b_name, beta)
            add_init(m_name, mean)
            add_init(v_name, var)
            out = f"{name}_out"
            nodes.append(helper.make_node(
                "BatchNormalization",
                [x, s_name, b_name, m_name, v_name],
                [out],
                epsilon=float(epsilon),
                momentum=float(momentum),
                name=name
            ))
            return out
        return call

    def Activation(act_type, name=None):
        def call(x):
            out = f"{name}_out" if name else uid("relu")
            nodes.append(helper.make_node("Relu", [x], [out], name=name or out))
            return out
        return call

    def MaxPooling2D(pool_size, strides=2, padding="valid", name=None):
        kh, kw = (pool_size, pool_size) if isinstance(pool_size, int) else (pool_size[0], pool_size[1])
        sh, sw = (strides, strides) if isinstance(strides, int) else (strides[0], strides[1])
        def call(x):
            out = f"{name}_out"
            nodes.append(helper.make_node(
                "MaxPool",
                [x],
                [out],
                kernel_shape=[kh, kw],
                strides=[sh, sw],
                pads=[0, 0, 0, 0],
                name=name
            ))
            return out
        return call

    def Concatenate(axis=3, name=None):
        def call(branches):
            out = f"{name}_out"
            # Keras axis=3 (channels in NHWC) -> ONNX axis=1 (channels in NCHW)
            nodes.append(helper.make_node("Concat", branches, [out], axis=1, name=name))
            return out
        return call

    def Lambda(fn, output_shape=None, arguments=None):
        scale_val = float(arguments["scale"])
        def call(x):
            if scale_val == 1.0:
                return x
            s_name = uid("lambda_scale")
            add_init(s_name, np.array([scale_val], dtype=np.float32))
            out = uid("lambda_out")
            nodes.append(helper.make_node("Mul", [x, s_name], [out]))
            return out
        return call

    def add(tensors):
        out = uid("add_out")
        nodes.append(helper.make_node("Add", tensors, [out]))
        return out

    def GlobalAveragePooling2D(name=None):
        def call(x):
            ga_out = f"{name}_ga"
            out = f"{name}_out"
            nodes.append(helper.make_node("GlobalAveragePool", [x], [ga_out], name=name))
            nodes.append(helper.make_node("Flatten", [ga_out], [out], axis=1))
            return out
        return call

    def Dropout(rate, name=None):
        def call(x):
            return x
        return call

    def Dense(units, use_bias=False, name=None):
        h5_name = re.sub(r"Block(17|8)_(\d+)_Branch_\2_", r"Block\1_\2_Branch_1_", name); grp = h5f[h5_name][h5_name]
        #Wait! Look at GlobalAveragePooling2D in Keras NHWC vs NCHW!
        #GlobalAveragePooling2D averages over H and W, leaving (N, C) in BOTH NHWC and NCHW!
        #Wait: in Keras Dense, kernel:0 has shape (1792, 512) = (in_features, out_features)!
        w = np.array(grp["kernel:0"], dtype=np.float32) # (1792, 512)
        w_name = f"{name}_W"
        add_init(w_name, w)
        def call(x):
            out = f"{name}_out"
            nodes.append(helper.make_node("MatMul", [x, w_name], [out], name=name))
            return out
        return call

    class KStub:
        @staticmethod
        def int_shape(x):
            return (None, None, None, None)

    def Model(inputs, outputs, name=None):
        return (inputs, outputs)

    # Execute actual InceptionResNetV1 body from /dev/shm/facenet_ref.py!
    with open("/dev/shm/facenet_ref.py", "r") as f:
        src = f.read()
    start_idx = src.index("def InceptionResNetV1")
    end_idx = src.index("def load_facenet128d_model")
    fn_src = src[start_idx:end_idx]

    env = {
        "Input": Input,
        "Conv2D": Conv2D,
        "BatchNormalization": BatchNormalization,
        "Activation": Activation,
        "MaxPooling2D": MaxPooling2D,
        "Concatenate": Concatenate,
        "Lambda": Lambda,
        "add": add,
        "GlobalAveragePooling2D": GlobalAveragePooling2D,
        "Dropout": Dropout,
        "Dense": Dense,
        "Model": Model,
        "K": KStub,
        "scaling": lambda x, scale: x,
    }
    exec(fn_src, env)
    _, final_out = env["InceptionResNetV1"](dimension=512)

    # Rename final output to "embedding"
    nodes.append(helper.make_node("Identity", [final_out], ["embedding"]))

    inp = helper.make_tensor_value_info("input", TensorProto.FLOAT, [1, 160, 160, 3])
    out = helper.make_tensor_value_info("embedding", TensorProto.FLOAT, [1, 512])

    graph = helper.make_graph(nodes, "InceptionResNetV1_FaceNet512", [inp], [out], initializers)
    model = helper.make_model(graph, producer_name="authentic_deepface_facenet512_exporter", opset_imports=[helper.make_opsetid("", 13)])
    model.ir_version = 7
    onnx.checker.check_model(model)
    out_path = os.path.join(OUT_DIR, "facenet512.onnx")
    onnx.save(model, out_path)
    h5f.close()

print("Building authentic ONNX models...")
build_pnet()
build_rnet()
build_onet()
build_facenet512()
for fname in ["mtcnn_pnet.onnx", "mtcnn_rnet.onnx", "mtcnn_onet.onnx", "facenet512.onnx"]:
    fpath = os.path.join(OUT_DIR, fname)
    sz = os.path.getsize(fpath)
    sha = hashlib.sha256(open(fpath, "rb").read()).hexdigest()
    print(f"  {fname:20s}: {sz:10,d} bytes | SHA256={sha}")
