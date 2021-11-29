package(default_visibility = ["//visibility:public"])

load("@rules_rust//rust:rust.bzl", "rust_library", "rust_test")

rust_library(
    name = "wasm",
    srcs = glob([
        "src/*.rs",
    ]),
    data = [
        "fib.wasm",
    ],
)

rust_test(
    name = "wasm_test",
    srcs = glob([
        "src/*.rs",
    ]),
    deps = [
        ":wasm",
    ],
)
