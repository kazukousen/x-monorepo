def _emit_size_impl(ctx):
    # The input file is given to us from the BUILD file via an attribute.
    in_file = ctx.file.file
    # The output file is declared with a name based on the target's name.
    out_file = ctx.actions.declare_file("{name}.size".format(name = ctx.label.name))

    ctx.actions.run_shell(
        inputs = [in_file],
        outputs = [out_file],
        progress_message = "Getting size of {}".format(in_file.short_path),
        command = "wc -c {in_file} | awk '{{print $1}}' > {out_file}".format(
            in_file = in_file.path,
            out_file = out_file.path,
        )
    )
    return [DefaultInfo(files = depset([out_file]))]

emit_size = rule(
    implementation = _emit_size_impl,
    attrs = {
        "file": attr.label(
            mandatory = True,
            allow_single_file = True,
            doc = "The file whose size is computed",
        ),
    },
    doc = """
Given an input file, creates an output file with the extenstion `.size`
containing the file's size in bytes.
"""
)

def _convert_to_uppercase_impl(ctx):
    in_file = ctx.file.input
    out_file = ctx.outputs.output
    ctx.actions.run_shell(
        inputs = [in_file],
        outputs = [out_file],
        arguments = [in_file.path, out_file.path],
        command = "tr '[:lower:]' '[:upper:]' < $1 > $2",
    )

convert_to_uppercase = rule(
    implementation = _convert_to_uppercase_impl,
    attrs = {
        "input": attr.label(
            mandatory = True,
            allow_single_file = True,
            doc = "The file to transform",
        ),
        "output": attr.output(doc = "The generated file"),
    },
    doc = """
Transform a text file by changing its characters to uppercase.
"""
)
