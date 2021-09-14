def _foo_binary_impl():
    pass

foo_binary = rule(
    implementation = _foo_binary_impl,
)

print("bzl file execution")
