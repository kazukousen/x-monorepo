load("@bazel_skylib//lib:shell.bzl", "shell")

def go_compile(ctx, srcs, out, deps = []):
    """
    Compiles a single Go package from sources.

    Args:
        ctx: analysis context.
        srcs: list of source Files to be compiled.
        out: output .a file. Should have the importpath as a suffix,
            for example, library "example.com/foo" should have the path "somedir/example.com/foo.a"
        deps: list of GoLibraryInfo objects for direct dependencies.
    """

    dep_import_args = []
    dep_archives = []
    for dep in deps:
        suffix_len = len("/" + dep.info.importpath + ".a")
        # trim suffix
        dir = dep.info.archive.path[:-suffix_len]
        dep_import_args.append("-I {}".format(shell.quote(dir)))
        dep_archives.append(dep.info.archive)

    cmd = "go tool compile -o {out} {imports} -- {srcs}".format(
        out = shell.quote(out.path),
        imports = " ".join(dep_import_args),
        srcs = " ".join([shell.quote(src.path) for src in srcs]),
    )

    ctx.actions.run_shell(
        inputs = srcs + dep_archives,
        outputs = [out],
        command = cmd,
        mnemonic = "GoCompile",
        use_default_shell_env = True,
    )

def go_link(ctx, out, main, deps = []):
    """
    Links a Go executable.

    Args:
        ctx: analysis context.
        out: output executable file.
        main: archive file for the main package.
        deps: list of GoLibraryInfo objects for direct dependencies.
    """

    deps_set = depset(
        direct = [d.info for d in deps],
        transitive = [d.deps for d in deps],
    )
    dep_lib_args = []
    dep_archives = []
    for dep in deps_set.to_list():
        suffix_len = len("/" + dep.importpath + ".a")
        # trim suffix
        dir = dep.archive.path[:-suffix_len]
        dep_lib_args.append("-L {}".format(shell.quote(dir)))
        dep_archives.append(dep.archive)

    cmd = "go tool link -o {out} {libs} -- {main}".format(
        out = shell.quote(out.path),
        libs = " ".join(dep_lib_args),
        main = shell.quote(main.path),
    )

    ctx.actions.run_shell(
        inputs = [main] + dep_archives,
        outputs = [out],
        command = cmd,
        mnemonic = "GoLink",
        use_default_shell_env = True,
    )

