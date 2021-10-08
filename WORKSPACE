workspace(name = "x-monorepo")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

skylib_version = "1.0.3"

http_archive(
    name = "bazel_skylib",
    sha256 = "1c531376ac7e5a180e0237938a2536de0c54d93f5c278634818e0efc952dd56c",
    type = "tar.gz",
    url = "https://mirror.bazel.build/github.com/bazelbuild/bazel-skylib/releases/download/{version}/bazel-skylib-{version}.tar.gz".format(version = skylib_version),
)

local_repository(
    name = "bazel_rules_python",
    path = "bazel/python/",
)

load("@bazel_rules_python//:repos.bzl", "python_repos")

python_repos()

load("@bazel_rules_python//:def.bzl", "python_rules_deps")

python_rules_deps()

load("//bazel/go:repos.bzl", "go_repos")

go_repos()

load("//bazel/go:def.bzl", "go_rules_deps")

go_rules_deps()

# rules_rust
http_archive(
    name = "rules_rust",
    sha256 = "531bdd470728b61ce41cf7604dc4f9a115983e455d46ac1d0c1632f613ab9fc3",
    strip_prefix = "rules_rust-d8238877c0e552639d3e057aadd6bfcf37592408",
    urls = [
        # `main` branch as of 2021-08-23
        "https://github.com/bazelbuild/rules_rust/archive/d8238877c0e552639d3e057aadd6bfcf37592408.tar.gz",
    ],
)

load("@rules_rust//rust:repositories.bzl", "rust_repositories")

rust_repositories(
    edition = "2018",
    version = "1.53.0",
)

# rules_scala

rules_scala_version = "e7a948ad1948058a7a5ddfbd9d1629d6db839933"

http_archive(
    name = "io_bazel_rules_scala",
    sha256 = "76e1abb8a54f61ada974e6e9af689c59fd9f0518b49be6be7a631ce9fa45f236",
    strip_prefix = "rules_scala-%s" % rules_scala_version,
    type = "zip",
    url = "https://github.com/bazelbuild/rules_scala/archive/{version}.zip".format(version = rules_scala_version),
)

load("@io_bazel_rules_scala//:scala_config.bzl", "scala_config")

scala_config()

load("@io_bazel_rules_scala//scala:scala.bzl", "scala_repositories")

scala_repositories()

load("@io_bazel_rules_scala//scala:toolchains.bzl", "scala_register_toolchains")

scala_register_toolchains()

load("@io_bazel_rules_scala//testing:scalatest.bzl", "scalatest_repositories", "scalatest_toolchain")

scalatest_repositories()

scalatest_toolchain()

# maven
rules_jvm_external_version = "4.1"

http_archive(
    name = "rules_jvm_external",
    sha256 = "f36441aa876c4f6427bfb2d1f2d723b48e9d930b62662bf723ddfb8fc80f0140",
    strip_prefix = "rules_jvm_external-{version}".format(version = rules_jvm_external_version),
    url = "https://github.com/bazelbuild/rules_jvm_external/archive/{version}.zip".format(version = rules_jvm_external_version),
)

load("@rules_jvm_external//:defs.bzl", "maven_install")

maven_install(
    name = "maven",
    artifacts = [
        "org.apache.lucene:lucene-core:8.9.0",
    ],
    fetch_sources = True,
    repositories = [
        "https://repo1.maven.org/maven2",
        "https://maven.google.com",
    ],
)

# rules_docker

http_archive(
    name = "io_bazel_rules_docker",
    sha256 = "1f4e59843b61981a96835dc4ac377ad4da9f8c334ebe5e0bb3f58f80c09735f4",
    strip_prefix = "rules_docker-0.19.0",
    urls = ["https://github.com/bazelbuild/rules_docker/releases/download/v0.19.0/rules_docker-v0.19.0.tar.gz"],
)

load("@io_bazel_rules_docker//repositories:repositories.bzl", container_repositories = "repositories")

container_repositories()

load("@io_bazel_rules_docker//repositories:deps.bzl", container_deps = "deps")

container_deps()

load("@io_bazel_rules_docker//container:pull.bzl", "container_pull")

container_pull(
    name = "alpine_linux_amd64",
    registry = "index.docker.io",
    repository = "library/alpine",
    tag = "3.13",
)

load("//3rdparty:go_repositories.bzl", "go_repositories")
# gazelle:repository_macro 3rdparty/go_repositories.bzl%go_repositories
go_repositories()

