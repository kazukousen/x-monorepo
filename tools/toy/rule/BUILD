load(":def.bzl", "foo_binary", "hello_gen", "hello_gen2")
load(":shell.bzl", "convert_to_uppercase", "emit_size")
load(":fortune.bzl", "haiku_fortune")
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
    template = "hello.cc.tpl",
    username = "Carol",
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

haiku_fortune(
    name = "haikus",
    srcs = [
        "correct.txt",
        "extensible.txt",
        "fast.txt",
    ],
)
