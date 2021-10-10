load(":go_actions.bzl", "go_compile", "go_link")

def _go_binary_impl(ctx):
    # Our output files will start with a prefix to avoid conflicting with other rules.
    prefix = ctx.label.name + "%/"
    main_archive = ctx.actions.declare_file(prefix + "main.a")
    # go_compile
    go_compile(
        ctx,
        srcs = ctx.files.srcs,
        out = main_archive,
    )

    # Note that output files may not have the same name as the rule, so we still need to use the prefix here.
    executable = ctx.actions.declare_file(prefix + ctx.label.name)

    go_link(
        ctx,
        main = main_archive,
        out = executable,
    )

    return [DefaultInfo(
        files = depset([executable]),
        executable = executable,
    )]

go_binary = rule(
    _go_binary_impl,
    attrs = {
        "srcs": attr.label_list(
            allow_files = [".go"],
            doc = "Source files to compile for the main package of this binary",
        ),
    },
    doc = "Builds an executable program from Go source code",
    executable = True,
)

