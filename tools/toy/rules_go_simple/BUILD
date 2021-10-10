load(":rules.bzl", "go_binary", "go_library")

go_binary(
    name = "hello",
    srcs = [
        "hello.go",
        "message.go",
    ]
)

sh_test(
    name = "hello_test",
    srcs = ["hello_test.sh"],
    args = ["$(location :hello)"],
    data = [":hello"],
)

go_binary(
    name = "bin_with_libs",
    srcs = ["bin_with_libs.go"],
    deps = [":foo"],
)

go_library(
    name = "foo",
    srcs = ["foo/foo.go"],
    importpath = "github.com/kazukousen/x-monorepo/tools/toy/rules_go_simple/foo",
    deps = [
        ":bar",
        ":baz",
    ],
)

go_library(
    name = "bar",
    srcs = ["bar/bar.go"],
    importpath = "github.com/kazukousen/x-monorepo/tools/toy/rules_go_simple/bar",
    deps = [":baz"],
)

go_library(
    name = "baz",
    srcs = ["baz/baz.go"],
    importpath = "github.com/kazukousen/x-monorepo/tools/toy/rules_go_simple/baz",
)
