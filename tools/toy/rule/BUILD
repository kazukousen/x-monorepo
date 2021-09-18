load(":def.bzl", "foo_binary", "hello_gen", "hello_gen2")
load(":shell.bzl", "emit_size", "convert_to_uppercase")
load("@rules_cc//cc:defs.bzl", "cc_binary")

print("BUILD file")

foo_binary(
    name = "bin1",
    username = "Alice",
)

foo_binary(
    name = "bin2",
    username = "Bob",
)

hello_gen(
    name = "Carol",
    username = "Carol",
    template = "hello.cc.tpl",
)

hello_gen2(
    name = "david_src",
    username = "David",
)

cc_binary(
    name = "david",
    srcs = [":david_src"],
)

emit_size(
    name = "foo",
    file = "foo.txt",
)

convert_to_uppercase(
    name = "foo_upper",
    input = "foo.txt",
    output = "foo_upper.txt",
)
