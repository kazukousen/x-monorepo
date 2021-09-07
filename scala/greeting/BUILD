load("@io_bazel_rules_scala//scala:scala.bzl", "scala_binary", "scala_test")

scala_binary(
    name = "greeter",
    srcs = glob(["src/main/scala/app/nitta/x/greeting/*.scala"]),
    main_class = "app.nitta.x.greeting.Greeter",
)

scala_test(
    name = "test",
    timeout = "short",
    srcs = glob(["src/test/scala/app/nitta/x/greeting/*.scala"]),
)
