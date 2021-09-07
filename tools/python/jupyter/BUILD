load("@rules_python//python:defs.bzl", "py_binary")
load("@py_deps//:requirements.bzl", "all_requirements")

py_binary(
    name = "jupyter",
    srcs = glob(["*.py"]),
    main = "jupyter.py",
    deps = all_requirements,
    visibility = ["//visibility:public"],
)

exports_files(["jupyter.py"])
