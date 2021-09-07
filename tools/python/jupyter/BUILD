load("@rules_python//python:defs.bzl", "py_binary")
load("@py_deps//:requirements.bzl", "requirement")

py_binary(
    name = "jupyter",
    srcs = glob(["*.py"]),
    main = "jupyter.py",
    deps = [
        requirement("jupyterlab"),
    ],
)
