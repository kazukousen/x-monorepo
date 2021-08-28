package(default_visibility = ["//visibility:public"])
load("@rules_rust//rust:rust.bzl", "rust_library", "rust_test")

rust_library(
    name = "lox",
    srcs = glob([
        "src/*.rs",
    ]),
)

rust_test(
    name = "lox_test",
    srcs = glob([
        "tests/*.rs",
    ]),
    deps = [
        ":lox",
    ],
)
