load(":actions.bzl", "go_compile", "go_link")
load(":providers.bzl", "GoLibraryInfo")

def _go_binary_impl(ctx):
    # Our output files will start with a prefix to avoid conflicting with other rules.
    main_archive = ctx.actions.declare_file("{name}%/main.a".format(name=ctx.label.name))
    go_compile(
        ctx,
        srcs = ctx.files.srcs,
        deps = [dep[GoLibraryInfo] for dep in ctx.attr.deps],
        out = main_archive,
    )

    # Note that output files may not have the same name as the rule, so we still need to use the prefix here.
    executable = ctx.actions.declare_file("{name}%/{name}".format(name=ctx.label.name))

    go_link(
        ctx,
        main = main_archive,
        deps = [dep[GoLibraryInfo] for dep in ctx.attr.deps],
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
        "deps": attr.label_list(
            providers = [GoLibraryInfo],
            doc = "Direct dependencies of the library",
        ),
    },
    doc = "Builds an executable program from Go source code",
    executable = True,
)

def _go_library_impl(ctx):
    """
    Declare an output file for the library package and compile it from srcs.
    """
    archive = ctx.actions.declare_file("{name}%/{importpath}.a".format(
        name = ctx.label.name,
        importpath = ctx.attr.importpath,
    ))

    go_compile(
        ctx,
        srcs = ctx.files.srcs,
        deps = [dep[GoLibraryInfo] for dep in ctx.attr.deps],
        out = archive,
    )

    return [
        DefaultInfo(files = depset([archive])),
        GoLibraryInfo(
            info = struct(
                importpath = ctx.attr.importpath,
                archive = archive,
            ),
            deps = depset(
                direct = [dep[GoLibraryInfo].info for dep in ctx.attr.deps],
                transitive = [dep[GoLibraryInfo].deps for dep in ctx.attr.deps],
            ),
        ),
    ]

go_library = rule(
    _go_library_impl,
    attrs = {
        "srcs": attr.label_list(
            allow_files = [".go"],
            doc = "Source files to compile",
        ),
        "deps": attr.label_list(
            providers = [GoLibraryInfo],
            doc = "Direct dependencies of the library",
        ),
        "importpath": attr.string(
            mandatory = True,
            doc = "Name by which the libary may be imported",
        ),
    },
    doc = "Compiles a Go archive from Go sources and dependencies",
)
