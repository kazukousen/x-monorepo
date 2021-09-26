load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

def python_repos():
    rules_python_version = "0.3.0"

    http_archive(
        name = "rules_python",
        sha256 = "934c9ceb552e84577b0faf1e5a2f0450314985b4d8712b2b70717dc679fdc01b",
        url = "https://github.com/bazelbuild/rules_python/releases/download/{version}/rules_python-{version}.tar.gz".format(version = rules_python_version),
    )
