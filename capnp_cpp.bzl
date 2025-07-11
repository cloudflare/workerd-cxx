load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")


URL = "https://github.com/capnproto/capnproto/tarball/5b8cae919e37898d47290e5edd777e1aa6be802a"
STRIP_PREFIX = "capnproto-capnproto-5b8cae9/c++"
SHA256 = "bb508a1762ca4af7b01162c38a2153e4b77595f612aeb27d2b560cc16302bf14"
TYPE = "tgz"
COMMIT = "5b8cae919e37898d47290e5edd777e1aa6be802a"

def _capnp_cpp(ctx):
    http_archive(
        name = "capnp-cpp",
        url = URL,
        strip_prefix = STRIP_PREFIX,
        type = TYPE,
        sha256 = SHA256,
    )

capnp_cpp = module_extension(implementation = _capnp_cpp)
