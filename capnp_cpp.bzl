load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

URL = "https://github.com/capnproto/capnproto/tarball/a7654b2041731d0ee323aa2bdb7975c7ab903f5e"
STRIP_PREFIX = "capnproto-capnproto-a7654b2/c++"
SHA256 = "f502ac3ada732800929706f3d6dacb9bcf3639d3bffde6387f4dd5e0e11984f5"
TYPE = "tgz"
COMMIT = "a7654b2041731d0ee323aa2bdb7975c7ab903f5e"

def _capnp_cpp(ctx):
    http_archive(
        name = "capnp-cpp",
        url = URL,
        strip_prefix = STRIP_PREFIX,
        type = TYPE,
        sha256 = SHA256,
    )

capnp_cpp = module_extension(implementation = _capnp_cpp)
