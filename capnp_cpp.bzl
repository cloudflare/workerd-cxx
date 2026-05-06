load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

URL = "https://github.com/capnproto/capnproto/tarball/38322ff31aa90f30dd87906425d18350e6417f88"
STRIP_PREFIX = "capnproto-capnproto-38322ff/c++"
SHA256 = "a3875eb6d311164154567757d032e4fd289d8e66d4acd491b203fcef992785f4"
TYPE = "tgz"
COMMIT = "38322ff31aa90f30dd87906425d18350e6417f88"

def _capnp_cpp(ctx):
    http_archive(
        name = "capnp-cpp",
        url = URL,
        strip_prefix = STRIP_PREFIX,
        type = TYPE,
        sha256 = SHA256,
    )

capnp_cpp = module_extension(implementation = _capnp_cpp)
