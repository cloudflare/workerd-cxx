load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("@bazel_tools//tools/build_defs/repo:local.bzl", "local_repository")


URL = "https://github.com/capnproto/capnproto/tarball/c525eb1680bdd94f3d814f81f148643ee5086525"
STRIP_PREFIX = "capnproto-capnproto-c525eb1/c++"
SHA256 = "7b60e840dbc62bb3140a9dff9ae38253688fc63722dcea6769108e9356e6362b"
TYPE = "tgz"
COMMIT = "c525eb1680bdd94f3d814f81f148643ee5086525"

def _capnp_cpp(ctx):
    http_archive(
        name = "capnp-cpp",
        url = URL,
        strip_prefix = STRIP_PREFIX,
        type = TYPE,
        sha256 = SHA256,
    )

    # For local development
    # local_repository(
    #     name = "capnp-cpp",
    #     path = "/full-path-to-capnproto-repo/c++",
    # )

capnp_cpp = module_extension(implementation = _capnp_cpp)
