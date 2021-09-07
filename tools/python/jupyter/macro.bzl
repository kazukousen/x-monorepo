def py_jupyter(name, deps, visibility = None):
    native.py_binary(
        name = name,
        srcs = ["//tools/python/jupyter:jupyter.py"],
        main = "//tools/python/jupyter:jupyter.py",
        deps = ["//tools/python/jupyter"] + deps,
    )
