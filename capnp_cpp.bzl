load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")


URL = "https://github.com/capnproto/capnproto/tarball/7309dd5f2dfdcf23a58207b77b888846ec0e8881"
STRIP_PREFIX = "capnproto-capnproto-7309dd5/c++"
SHA256 = "6d0f393741f26efa18e158c95d9abbeaee466d07090c2b9ecdb6814ba6c39086"
TYPE = "tgz"
COMMIT = "7309dd5f2dfdcf23a58207b77b888846ec0e8881"

def _capnp_cpp(ctx):
    http_archive(
        name = "capnp-cpp",
        url = URL,
        strip_prefix = STRIP_PREFIX,
        type = TYPE,
        sha256 = SHA256,
    )

capnp_cpp = module_extension(implementation = _capnp_cpp)
