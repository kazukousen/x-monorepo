load("@rules_python//python:defs.bzl", "py_binary")
load("@py_deps//:requirements.bzl", "all_requirements")

py_binary(
    name = "repl",
    srcs = ["repl.py"],
    visibility = ["//visibility:public"],
    deps = all_requirements,
)

exports_files(["repl.py"])
