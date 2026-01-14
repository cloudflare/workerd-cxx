load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

URL = "https://github.com/capnproto/capnproto/tarball/9ecec9ac6d1780ccb308631b3f15cfbb124795d4"
STRIP_PREFIX = "capnproto-capnproto-9ecec9a/c++"
SHA256 = "0e3a81c31449fd84b69a9cff5979b8fb89288130e2a2fe99f4f387a261657751"
TYPE = "tgz"
COMMIT = "9ecec9ac6d1780ccb308631b3f15cfbb124795d4"

def _capnp_cpp(ctx):
    http_archive(
        name = "capnp-cpp",
        url = URL,
        strip_prefix = STRIP_PREFIX,
        type = TYPE,
        sha256 = SHA256,
    )

capnp_cpp = module_extension(implementation = _capnp_cpp)
