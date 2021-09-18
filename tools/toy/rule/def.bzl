def _foo_binary_impl(ctx):
    out = ctx.actions.declare_file(ctx.label.name)
    ctx.actions.write(
        output = out,
        content = "Hello {}!\n".format(ctx.attr.username),
    )
    return [DefaultInfo(files = depset([out]))]

foo_binary = rule(
    implementation = _foo_binary_impl,
    attrs = {
        "username": attr.string(),
    },
)

print("bzl file execution")

def _hello_gen_impl(ctx):
    out = ctx.actions.declare_file(ctx.label.name + ".cc")
    ctx.actions.expand_template(
        output = out,
        template = ctx.file.template,
        substitutions = {"{NAME}": ctx.attr.username},
    )
    return [DefaultInfo(files = depset([out]))]

hello_gen = rule(
    implementation = _hello_gen_impl,
    attrs = {
        "username": attr.string(default = "anonymous"),
        "template": attr.label(
            allow_single_file = [".cc.tpl"],
            mandatory = True,
        ),
    },
)

def hello_gen2(**kwargs):
    _hello_gen2(
        source_file = "{name}.cc".format(**kwargs),
        **kwargs
    )

def _hello_gen2_impl(ctx):
    ctx.actions.expand_template(
        output = ctx.outputs.source_file,
        template = ctx.file._template,
        substitutions = {
            "{NAME}": ctx.attr.username,
        },
    )

_hello_gen2 = rule(
    implementation = _hello_gen2_impl,
    attrs = {
        "username": attr.string(default = "anonymous"),
        "_template": attr.label(
            allow_single_file = True,
            default = Label("//tools/toy/rule:hello.cc.tpl"),
        ),
        "source_file": attr.output(mandatory = True),
    }
)
