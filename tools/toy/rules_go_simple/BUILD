load(":go_binary.bzl", "go_binary")

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
