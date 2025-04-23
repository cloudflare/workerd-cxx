
load("@@bazel_tools//tools/build_defs/repo:local.bzl", "local_repository")


def _kj_rs(ctx):
    local_repository(
        name = "kj-rs",
        path = "/home/maizatskyi/src/github.com/capnproto/kj-rs",
    )

kj_rs = module_extension(implementation = _kj_rs)
