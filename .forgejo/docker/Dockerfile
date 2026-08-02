# SPDX-License-Identifier: MIT OR Apache-2.0
# The oxivgl CI image (registry:3000/emobotics/oxivgl/oxivgl-ci) — the container
# both CI legs run in. Everything is baked at BUILD time; no job ever installs a
# toolchain or package at runtime (conventions/containers.md §1).
#
# Built ON TOP of m5stack-core-ci, the org's reference ESP-BSP CI image, reusing
# it rather than re-deriving the Xtensa toolchain: it already carries the
# dual-board `esp` toolchain, the LVGL/bindgen build deps (cmake, libclang,
# pypng/lz4 — it notes these are partly for oxivgl-sys), Node 22 for
# actions/checkout, and llvm-cov/cargo-llvm-cov, and it exports the full ESP
# cross-build env so the firmware legs need no plumbing. This image adds only the
# three things that base lacks for oxivgl's host suites.
#
# SDL2 is added here for now; it has been proposed for the shared base in
# m5stack-core#67. If that lands, drop the libsdl2-dev layer and bump the base
# tag instead.
#
# Tagged by IMAGE_VERSION in build-ci-image.yml, never `:latest` — the runner
# fleet shares one podman store with forcePull=false, so a mutable tag goes stale
# the moment any runner has pulled it once. Bump the tag in build-ci-image.yml +
# ci.yml together, in the same commit, whenever this Dockerfile changes; never
# move an existing tag (emobotics/forgejo-instance docs/runners.md).
FROM registry:3000/emobotics/m5stack-core/m5stack-core-ci:v6

ENV DEBIAN_FRONTEND=noninteractive

# libsdl2-dev: oxivgl-sys builds the LVGL SDL2 backend for the host target, and
# the integration / leak / screenshot suites link and run against it (headless
# via SDL_VIDEODRIVER=dummy, so no display server is needed). Installed from the
# distro archive, not COPY --from: SDL2 has a shared-library graph, which the
# multi-stage-copy trick does not carry (conventions/containers.md §2).
RUN apt-get update -q && \
    apt-get install -y --no-install-recommends libsdl2-dev && \
    rm -rf /var/lib/apt/lists/*

# Host `nightly` toolchain: rust-toolchain.toml pins nightly for host builds and
# the suites run `cargo +nightly …`. Installed into the base image's shared
# /opt/rustup so every job user sees it. llvm-tools-preview + the base image's
# cargo-llvm-cov give host coverage; the firmware legs keep using `cargo +esp`.
RUN rustup toolchain install nightly --profile minimal && \
    rustup component add --toolchain nightly llvm-tools-preview rust-src

# Native libclang at the Fedora-style path run_docs.sh / run_host.sh default to
# (LIBCLANG_PATH=${LIBCLANG_PATH:-/usr/lib64}). The base image's exported
# LIBCLANG_PATH is the Xtensa-only ESP clang (no x86_64 headers); the host job
# points LIBCLANG_PATH back here (see ci.yml). `find` fails the build loudly if
# the base ever drops libclang.
RUN mkdir -p /usr/lib64 && \
    ln -sfn "$(find /usr/lib -name 'libclang.so*' | sort | tail -1)" /usr/lib64/libclang.so && \
    test -e /usr/lib64/libclang.so

WORKDIR /workspace
