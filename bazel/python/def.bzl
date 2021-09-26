load("@rules_python//python:pip.bzl", "pip_install")

def python_rules_deps():
    pip_install(
        name = "py_deps",
        requirements = "//3rdparty:requirements.txt",
    )

