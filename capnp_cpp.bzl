load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

URL = "https://github.com/capnproto/capnproto/tarball/9f9f87feafa03de954a1c446e836cc51b35696cc"
STRIP_PREFIX = "capnproto-capnproto-9f9f87f/c++"
SHA256 = "289617a365f8d68e75a055374584a72e598bdb191eb847800f9b7fe4338db36a"
TYPE = "tgz"
COMMIT = "9f9f87feafa03de954a1c446e836cc51b35696cc"

def _capnp_cpp(_ctx):
    http_archive(
        name = "capnp-cpp",
        url = URL,
        strip_prefix = STRIP_PREFIX,
        type = TYPE,
        sha256 = SHA256,
    )

capnp_cpp = module_extension(implementation = _capnp_cpp)
