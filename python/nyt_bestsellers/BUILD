load("@py_deps//:requirements.bzl", "requirement")
load("@rules_python//python:defs.bzl", "py_binary")

py_binary(
    name = "nyt_bestsellers",
    srcs = glob(["*.py"]),
    main = "main.py",
    deps = [
        requirement("requests"),
        requirement("beautifulsoup4"),
    ],
)
