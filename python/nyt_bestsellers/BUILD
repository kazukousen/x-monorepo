load("@py_deps//:requirements.bzl", "requirement")
load("@rules_python//python:defs.bzl", "py_binary")
load("//tools/python/repl:macro.bzl", "py_repl")

py_binary(
    name = "nyt_bestsellers",
    srcs = glob(["*.py"]),
    main = "main.py",
    deps = [
        requirement("requests"),
        requirement("beautifulsoup4"),
    ],
)

# Try it with:
# $ bazel run //python/nyt_bestsellers:repl
# >>> from python.nyt_bestsellers.main import run
# >>> run()
py_repl(
    name = "repl",
    deps = [":nyt_bestsellers"],
)
