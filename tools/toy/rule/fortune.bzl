SCRIPT_TEMPLATE="""\
#!/bin/bash
R=$(( $RANDOM % {num_fortunes} * 3 + 1 ))
cat {fortunes_files} | sed -n "$R,$(( $R + 2 ))p"
"""

def _haiku_fortune_impl(ctx):
    # Generate a datafile of concatenated fortunes.
    datafile = ctx.actions.declare_file(ctx.label.name + ".fortunes")
    ctx.actions.run_shell(
        inputs = ctx.files.srcs,
        outputs = [datafile],
        command = "cat {srcs} > {output}".format(
            srcs = " ".join([f.path for f in ctx.files.srcs]),
            output = datafile.path,
        )
    )

    # Emit the executable shell script.
    script = ctx.actions.declare_file(ctx.label.name + "-fortunes")
    script_content = SCRIPT_TEMPLATE.format(
        num_fortunes = len(ctx.attr.srcs),
        fortunes_files = datafile.short_path,
    )
    ctx.actions.write(output = script, content = script_content, is_executable = True)

    # The datafile must be in the runfiles for the executable to see it.
    runfiles = ctx.runfiles([datafile])
    return [DefaultInfo(executable = script, runfiles = runfiles)]

haiku_fortune = rule(
    implementation = _haiku_fortune_impl,
    attrs = {
        "srcs": attr.label_list(
            allow_files = True,
            doc = "The haiku files. Each file must have exactly three lines." +
            "The last line must be terminated by a newline character.",
        ),
    },
    executable = True,
)