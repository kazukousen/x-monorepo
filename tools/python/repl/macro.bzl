def py_repl(name, deps, visibility = None):
    native.py_binary(
        name = name,
        srcs = ["//tools/python/repl:repl.py"],
        main = "//tools/python/repl:repl.py",
        deps = ["//tools/python/repl"] + deps,
    )
