load("@io_bazel_rules_go//go:deps.bzl", "go_register_toolchains", "go_rules_dependencies")
load("@bazel_gazelle//:deps.bzl", "gazelle_dependencies")

def go_rules_deps():
    go_rules_dependencies()
    go_register_toolchains(version = "1.17")
    gazelle_dependencies()

