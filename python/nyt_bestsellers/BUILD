load("@rules_python//python:defs.bzl", "py_binary")
load("@py_deps//:requirements.bzl", "requirement")
load("//tools/python/repl:macro.bzl", "py_repl")
load("//tools/python/jupyter:macro.bzl", "py_jupyter")

py_binary(
    name = "nyt_bestsellers",
    srcs = glob(["*.py"]),
    main = "main.py",
    deps = [
        requirement("requests"),
        requirement("beautifulsoup4"),
    ],
)

# $ bazel build //...
# $ bazel-bin/python/nyt_bestsellers/jupyter
py_jupyter(
    name = "jupyter",
    deps = [":nyt_bestsellers"],
)

# Try it with:
# $ bazel run //python/nyt_bestsellers:repl
# >>> from python.nyt_bestsellers.main import run
# >>> run()
py_repl(
    name = "repl",
    deps = [":nyt_bestsellers"],
)
