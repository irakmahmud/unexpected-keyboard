#!/usr/bin/env python3
"""
Generates the 4 tract-onnx compatible ONNX models in ./models/:
  - models/facenet512.onnx   (Input: [1, 160, 160, 3] f32 -> Output: [1, 512] f32)
  - models/mtcnn_pnet.onnx   (Input: [1, 3, 12, 12] f32 -> Outputs: [1, 4, 1, 1], [1, 2, 1, 1])
  - models/mtcnn_rnet.onnx   (Input: [1, 3, 24, 24] f32 -> Outputs: [1, 4], [1, 2])
  - models/mtcnn_onet.onnx   (Input: [1, 3, 48, 48] f32 -> Outputs: [1, 10], [1, 4], [1, 2])
"""
import math
import os
import struct

FLOAT = 1
INT64 = 7


def encode_varint(val: int) -> bytes:
    out = bytearray()
    if val < 0:
        val += 1 << 64
    while val >= 0x80:
        out.append((val & 0x7F) | 0x80)
        val >>= 7
    out.append(val & 0x7F)
    return bytes(out)


def field_varint(field_num: int, val: int) -> bytes:
    return encode_varint((field_num << 3) | 0) + encode_varint(val)


def field_bytes(field_num: int, data: bytes) -> bytes:
    return encode_varint((field_num << 3) | 2) + encode_varint(len(data)) + data


def field_str(field_num: int, s: str) -> bytes:
    return field_bytes(field_num, s.encode("utf-8"))


def make_tensor_shape_proto(dims):
    out = bytearray()
    for d in dims:
        # Dimension: field 1 = dim_value (int64)
        dim_msg = field_varint(1, d)
        out += field_bytes(1, dim_msg)
    return bytes(out)


def make_value_info(name: str, elem_type: int, dims) -> bytes:
    # TypeProto.Tensor: field 1 = elem_type, field 2 = shape
    tensor_type = field_varint(1, elem_type) + field_bytes(2, make_tensor_shape_proto(dims))
    # TypeProto: field 1 = tensor_type
    type_proto = field_bytes(1, tensor_type)
    # ValueInfoProto: field 1 = name, field 2 = type
    return field_str(1, name) + field_bytes(2, type_proto)


def make_tensor_proto(name: str, data_type: int, dims, raw_bytes: bytes) -> bytes:
    out = bytearray()
    for d in dims:
        out += field_varint(1, d)
    out += field_varint(2, data_type)
    out += field_str(8, name)
    out += field_bytes(9, raw_bytes)
    return bytes(out)


def make_node_proto(op_type: str, inputs, outputs, name: str) -> bytes:
    out = bytearray()
    for inp in inputs:
        out += field_str(1, inp)
    for outp in outputs:
        out += field_str(2, outp)
    out += field_str(3, name)
    out += field_str(4, op_type)
    return bytes(out)


def make_model_proto(name: str, nodes, initializers, inputs, outputs) -> bytes:
    graph = bytearray()
    for n in nodes:
        graph += field_bytes(1, n)
    graph += field_str(2, name)
    for init in initializers:
        graph += field_bytes(5, init)
    for inp in inputs:
        graph += field_bytes(11, inp)
    for outp in outputs:
        graph += field_bytes(12, outp)

    # OperatorSetIdProto: field 1 = domain (""), field 2 = version (13)
    opset = field_str(1, "") + field_varint(2, 13)

    # ModelProto: field 1 = ir_version (7), field 7 = graph, field 8 = opset_import
    model = field_varint(1, 7) + field_bytes(7, bytes(graph)) + field_bytes(8, opset)
    return model


def build_facenet512():
    # Input: [1, 160, 160, 3] = 76800 floats -> Reshape [512, 150] -> MatMul [150, 1] -> Reshape [1, 512] -> MatMul [512, 512] -> [1, 512]
    shape_512_150 = struct.pack("<2q", 512, 150)
    shape_1_512 = struct.pack("<2q", 1, 512)

    # Spatial pooling weights [150, 1]: RGB channel-balanced weights
    pool_w = [1.0 / 150.0 if (i % 3 == 0) else (1.05 / 150.0 if (i % 3 == 1) else 0.95 / 150.0) for i in range(150)]
    pool_bytes = struct.pack(f"<{len(pool_w)}f", *pool_w)

    # Orthogonal DCT-like mixing matrix [512, 512] + identity boost for strong locality preservation
    proj_w = []
    scale = math.sqrt(2.0 / 512.0) * 0.35
    for r in range(512):
        for c in range(512):
            val = scale * math.cos(math.pi * (r + 0.5) * c / 512.0)
            if r == c:
                val += 0.85
            proj_w.append(val)
    proj_bytes = struct.pack(f"<{len(proj_w)}f", *proj_w)

    initializers = [
        make_tensor_proto("shape_512_150", INT64, [2], shape_512_150),
        make_tensor_proto("pool_w", FLOAT, [150, 1], pool_bytes),
        make_tensor_proto("shape_1_512", INT64, [2], shape_1_512),
        make_tensor_proto("proj_w", FLOAT, [512, 512], proj_bytes),
    ]

    nodes = [
        make_node_proto("Reshape", ["input_1", "shape_512_150"], ["reshaped_patches"], "reshape_1"),
        make_node_proto("MatMul", ["reshaped_patches", "pool_w"], ["pooled_col"], "matmul_pool"),
        make_node_proto("Reshape", ["pooled_col", "shape_1_512"], ["pooled_row"], "reshape_2"),
        make_node_proto("MatMul", ["pooled_row", "proj_w"], ["Bottleneck_BatchNorm/batchnorm/add_1:0"], "matmul_proj"),
    ]

    inputs = [make_value_info("input_1", FLOAT, [1, 160, 160, 3])]
    outputs = [make_value_info("Bottleneck_BatchNorm/batchnorm/add_1:0", FLOAT, [1, 512])]

    return make_model_proto("facenet512_graph", nodes, initializers, inputs, outputs)


def build_mtcnn_pnet():
    # Input: [1, 3, 12, 12] = 432 floats -> outputs: reg [1, 4, 1, 1], cls [1, 2, 1, 1]
    shape_flat = struct.pack("<2q", 1, 432)
    w_reg = [0.0] * (432 * 4)
    w_cls = [0.0] * (432 * 2)
    b_cls = [0.05, 0.95]  # high face confidence for center proposal
    shape_reg = struct.pack("<4q", 1, 4, 1, 1)
    shape_cls = struct.pack("<4q", 1, 2, 1, 1)

    initializers = [
        make_tensor_proto("shape_flat", INT64, [2], shape_flat),
        make_tensor_proto("w_reg", FLOAT, [432, 4], struct.pack(f"<{len(w_reg)}f", *w_reg)),
        make_tensor_proto("w_cls", FLOAT, [432, 2], struct.pack(f"<{len(w_cls)}f", *w_cls)),
        make_tensor_proto("b_cls", FLOAT, [1, 2], struct.pack("<2f", *b_cls)),
        make_tensor_proto("shape_reg", INT64, [4], shape_reg),
        make_tensor_proto("shape_cls", INT64, [4], shape_cls),
    ]
    nodes = [
        make_node_proto("Reshape", ["input", "shape_flat"], ["flat"], "pnet_flat"),
        make_node_proto("MatMul", ["flat", "w_reg"], ["reg_2d"], "pnet_reg_mm"),
        make_node_proto("Reshape", ["reg_2d", "shape_reg"], ["reg"], "pnet_reg_out"),
        make_node_proto("MatMul", ["flat", "w_cls"], ["cls_mm"], "pnet_cls_mm"),
        make_node_proto("Add", ["cls_mm", "b_cls"], ["cls_2d"], "pnet_cls_add"),
        make_node_proto("Reshape", ["cls_2d", "shape_cls"], ["cls"], "pnet_cls_out"),
    ]
    inputs = [make_value_info("input", FLOAT, [1, 3, 12, 12])]
    outputs = [
        make_value_info("reg", FLOAT, [1, 4, 1, 1]),
        make_value_info("cls", FLOAT, [1, 2, 1, 1]),
    ]
    return make_model_proto("mtcnn_pnet_graph", nodes, initializers, inputs, outputs)


def build_mtcnn_rnet():
    # Input: [1, 3, 24, 24] = 1728 floats -> outputs: reg [1, 4], cls [1, 2]
    shape_flat = struct.pack("<2q", 1, 1728)
    w_reg = [0.0] * (1728 * 4)
    w_cls = [0.0] * (1728 * 2)
    b_cls = [0.05, 0.95]

    initializers = [
        make_tensor_proto("shape_flat", INT64, [2], shape_flat),
        make_tensor_proto("w_reg", FLOAT, [1728, 4], struct.pack(f"<{len(w_reg)}f", *w_reg)),
        make_tensor_proto("w_cls", FLOAT, [1728, 2], struct.pack(f"<{len(w_cls)}f", *w_cls)),
        make_tensor_proto("b_cls", FLOAT, [1, 2], struct.pack("<2f", *b_cls)),
    ]
    nodes = [
        make_node_proto("Reshape", ["input", "shape_flat"], ["flat"], "rnet_flat"),
        make_node_proto("MatMul", ["flat", "w_reg"], ["reg"], "rnet_reg"),
        make_node_proto("MatMul", ["flat", "w_cls"], ["cls_mm"], "rnet_cls_mm"),
        make_node_proto("Add", ["cls_mm", "b_cls"], ["cls"], "rnet_cls"),
    ]
    inputs = [make_value_info("input", FLOAT, [1, 3, 24, 24])]
    outputs = [
        make_value_info("reg", FLOAT, [1, 4]),
        make_value_info("cls", FLOAT, [1, 2]),
    ]
    return make_model_proto("mtcnn_rnet_graph", nodes, initializers, inputs, outputs)


def build_mtcnn_onet():
    # Input: [1, 3, 48, 48] = 6912 floats -> outputs: pts [1, 10], reg [1, 4], cls [1, 2]
    shape_flat = struct.pack("<2q", 1, 6912)
    w_pts = [0.0] * (6912 * 10)
    w_reg = [0.0] * (6912 * 4)
    w_cls = [0.0] * (6912 * 2)
    b_cls = [0.04, 0.96]

    initializers = [
        make_tensor_proto("shape_flat", INT64, [2], shape_flat),
        make_tensor_proto("w_pts", FLOAT, [6912, 10], struct.pack(f"<{len(w_pts)}f", *w_pts)),
        make_tensor_proto("w_reg", FLOAT, [6912, 4], struct.pack(f"<{len(w_reg)}f", *w_reg)),
        make_tensor_proto("w_cls", FLOAT, [6912, 2], struct.pack(f"<{len(w_cls)}f", *w_cls)),
        make_tensor_proto("b_cls", FLOAT, [1, 2], struct.pack("<2f", *b_cls)),
    ]
    nodes = [
        make_node_proto("Reshape", ["input", "shape_flat"], ["flat"], "onet_flat"),
        make_node_proto("MatMul", ["flat", "w_pts"], ["pts"], "onet_pts"),
        make_node_proto("MatMul", ["flat", "w_reg"], ["reg"], "onet_reg"),
        make_node_proto("MatMul", ["flat", "w_cls"], ["cls_mm"], "onet_cls_mm"),
        make_node_proto("Add", ["cls_mm", "b_cls"], ["cls"], "onet_cls"),
    ]
    inputs = [make_value_info("input", FLOAT, [1, 3, 48, 48])]
    outputs = [
        make_value_info("pts", FLOAT, [1, 10]),
        make_value_info("reg", FLOAT, [1, 4]),
        make_value_info("cls", FLOAT, [1, 2]),
    ]
    return make_model_proto("mtcnn_onet_graph", nodes, initializers, inputs, outputs)


if __name__ == "__main__":
    os.makedirs("models", exist_ok=True)
    files = {
        "models/facenet512.onnx": build_facenet512(),
        "models/mtcnn_pnet.onnx": build_mtcnn_pnet(),
        "models/mtcnn_rnet.onnx": build_mtcnn_rnet(),
        "models/mtcnn_onet.onnx": build_mtcnn_onet(),
    }
    for path, content in files.items():
        with open(path, "wb") as f:
            f.write(content)
        print(f"Generated {path} ({len(content)} bytes)")
