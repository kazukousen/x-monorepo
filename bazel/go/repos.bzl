load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

def go_repos():
    rules_go_version = "v0.28.0"

    http_archive(
        name = "io_bazel_rules_go",
        sha256 = "8e968b5fcea1d2d64071872b12737bbb5514524ee5f0a4f54f5920266c261acb",
        urls = [
            "https://mirror.bazel.build/github.com/bazelbuild/rules_go/releases/download/{version}/rules_go-{version}.zip".format(version = rules_go_version),
            "https://github.com/bazelbuild/rules_go/releases/download/{version}/rules_go-{version}.zip".format(version = rules_go_version),
        ],
    )

    # gazelle
    rules_gazelle_version = "v0.23.0"

    http_archive(
        name = "bazel_gazelle",
        sha256 = "62ca106be173579c0a167deb23358fdfe71ffa1e4cfdddf5582af26520f1c66f",
        urls = [
            "https://mirror.bazel.build/github.com/bazelbuild/bazel-gazelle/releases/download/{version}/bazel-gazelle-{version}.tar.gz".format(version = rules_gazelle_version),
            "https://github.com/bazelbuild/bazel-gazelle/releases/download/{version}/bazel-gazelle-{version}.tar.gz".format(version = rules_gazelle_version),
        ],
    )
