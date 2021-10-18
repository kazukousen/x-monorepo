load(":rules.bzl", "go_binary", "go_library")


sh_test(
    name = "hello_test",
    srcs = ["hello_test.sh"],
    args = ["$(location :hello)"],
    data = [":hello"],
)

go_binary(
    name = "hello",
    srcs = [
        "hello.go",
        "message.go",
    ]
)

sh_test(
    name = "bin_with_libs_test",
    srcs = ["bin_with_libs_test.sh"],
    args = ["$(location :bin_with_libs)"],
    data = [":bin_with_libs"],
)

go_binary(
    name = "bin_with_libs",
    srcs = ["bin_with_libs.go"],
    deps = [":foo"],
)

go_library(
    name = "foo",
    srcs = ["foo/foo.go"],
    importpath = "github.com/kazukousen/x-monorepo/tools/toy/rules_go_simple/internal/foo",
    deps = [
        ":bar",
        ":baz",
    ],
)

go_library(
    name = "bar",
    srcs = ["bar/bar.go"],
    importpath = "github.com/kazukousen/x-monorepo/tools/toy/rules_go_simple/internal/bar",
    deps = [":baz"],
)

go_library(
    name = "baz",
    srcs = ["baz/baz.go"],
    importpath = "github.com/kazukousen/x-monorepo/tools/toy/rules_go_simple/internal/baz",
)

sh_test(
    name = "list_data_bin_test",
    srcs = ["list_data_bin_test.sh"],
    args = ["$(location :list_data_bin)"],
    data = [":list_data_bin"],
)

go_binary(
    name = "list_data_bin",
    srcs = ["list_data_bin.go"],
    deps = [":list_data_lib"],
    data = ["foo.txt"],
)

go_library(
    name = "list_data_lib",
    srcs = ["list_data_lib/list_data_lib.go"],
    data = ["bar.txt"],
    importpath = "github.com/kazukousen/x-monorepo/tools/toy/rules_go_simple/internal/list_data_lib",
)
