# syntax=docker/dockerfile:1

# Devcontainer for oxivgl — the clco dev environment plus the *exact* toolchain
# CI builds with.
#
# Stage 1 pulls the prebuilt toolchain out of oxivgl's own CI image rather than
# running `espup install` here. Two reasons, in order of importance:
#
#  1. **Devcontainer and CI become the same toolchain.** A local `espup install`
#     resolves whatever is current that day, so a green local build proved
#     nothing about CI (and vice versa). Copying from the image CI runs in
#     removes the drift by construction.
#  2. It turns a 10-30 min `espup install` into a layer copy.
#
# oxivgl-ci is itself `FROM m5stack-core-ci` — the org's reference ESP-BSP image
# — and adds oxivgl's host extras (nightly, SDL2, a native libclang). See
# .forgejo/docker/Dockerfile.
#
# Keep the `:v1` tag in lockstep with .forgejo/workflows/{ci,build-ci-image}.yml.
# The tag is immutable: when the CI image changes, bump it in all three files in
# the same commit and never move an existing tag (the runner fleet shares one
# podman store with forcePull=false, so a moved tag silently goes stale).
FROM registry:3000/emobotics/oxivgl/oxivgl-ci:v1 AS esp-toolchain

# Stage 2: the clco base container.
# Despite the image name this is NOT a Claude Code redistribution — it is the
# clco dev environment (source: emobotics/claude-container): fj/fj-ex, RTK, gh,
# adb and the clco conventions, with Claude Code installed via the official
# installer. Same base every clco repo's devcontainer uses.
FROM registry:3000/emobotics/claude-code:latest

USER root

# ── System packages (only what the base image lacks) ─────────────────
# Base already has: build-essential, pkg-config, libssl-dev, libsdl2-dev,
# python3, python3-pip, curl, ca-certificates, unzip, wget, gh, adb.
#
# libclang-dev/llvm-dev: bindgen for *host* builds (oxivgl-sys). Installed from
#   the distro rather than copied from stage 1 — a shared-library graph does not
#   survive COPY --from (conventions/containers.md §2).
# cmake: LVGL's build scripts.
# libudev-dev: espflash / probe-rs serial + USB enumeration.
# python3-venv: LVGLImage.py tooling.
RUN apt-get update && apt-get install -y --no-install-recommends \
        cmake \
        libclang-dev \
        llvm-dev \
        libudev-dev \
        python3-venv \
    && rm -rf /var/lib/apt/lists/*

# Python deps for LVGL's LVGLImage.py (image asset conversion).
RUN pip3 install --no-cache-dir --break-system-packages pypng lz4

# run_tests.sh / run_host.sh / run_docs.sh default to LIBCLANG_PATH=/usr/lib64
# (a Fedora layout). Symlink so the same path works on this Debian base.
RUN mkdir -p /usr/lib64 \
    && ln -sfn "$(llvm-config --libdir)/libclang.so" /usr/lib64/libclang.so \
    && test -e /usr/lib64/libclang.so

# ── Rust toolchains, copied from the CI image ────────────────────────
# /opt/rustup carries both channels CI uses: `esp` (the Xtensa rustc) and
# `nightly` (host builds/tests, + llvm-tools-preview and rust-src).
# /opt/cargo carries cargo-llvm-cov.
# /opt/esp_elf_bin_path names the dir holding xtensa-esp32-elf-gcc and
# xtensa-esp32s3-elf-gcc, which .cargo/config.toml sets as the linkers.
# `-Zbuild-std` compiles core/alloc from source, so no prebuilt target std is
# needed — the toolchain and linkers are the whole requirement.
COPY --from=esp-toolchain /opt/rustup           /opt/rustup
COPY --from=esp-toolchain /opt/cargo            /opt/cargo
COPY --from=esp-toolchain /opt/esp_elf_bin_path /opt/esp_elf_bin_path

ENV RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo \
    PATH="/opt/cargo/bin:${PATH}"

# Let claude write cargo caches at runtime; keep rustup world-readable.
RUN chown -R claude:claude /opt/cargo \
    && chmod -R a+rX /opt/rustup

# ── Flashing tools ───────────────────────────────────────────────────
# Installed here, not copied: the CI image deliberately carries no flashing
# tools (CI cross-compiles, it never talks to a board), so there is nothing to
# copy. Both boards need one — .cargo/config.toml runs the ESP32 target through
# `espflash` and the ESP32-S3 target through `probe-rs`.
#
# No glibc rebuild dance is needed, unlike m5stack-core's devcontainer: both
# that CI image and this base are Debian 12 bookworm, so the copied /opt/cargo
# binaries (cargo-llvm-cov) run as-is. Re-check that if either base moves.
USER claude
RUN cargo install espflash@4.3.0 --locked \
    && cargo install probe-rs-tools --locked \
    && rm -rf "$CARGO_HOME/registry" "$CARGO_HOME/git" "$CARGO_HOME/.package-cache"

# ── Environment, wired into .clco-env AND .bashrc ────────────────────
# .clco-env is sourced by tooling, .bashrc by interactive shells; both are
# needed or a build works in one context and fails in the other.
#
# BINDGEN_EXTRA_CLANG_ARGS is set **per Xtensa target**, never globally. A
# global sysroot leaks into the host bindgen build, where clang then dies on
# `bits/libc-header-start.h` — the ESP newlib sysroot has no x86_64 headers.
# (That exact failure cost a CI debugging round; see .forgejo/workflows/ci.yml,
# which clears the variable for its host leg.)
#
# LIBCLANG_PATH is likewise pinned back to the system clang: anything that
# points it at the Xtensa-only ESP clang breaks every host build.
# oxivgl-sys/build.rs auto-detects the ESP clang for xtensa targets by itself.
RUN bash -c 'set -eu; \
    ELF_BIN=$(cat /opt/esp_elf_bin_path); \
    SYSROOT=$("$ELF_BIN/xtensa-esp32-elf-gcc" -print-sysroot); \
    { \
      echo "export PATH=\"$ELF_BIN:\$PATH\""; \
      echo "export LIBCLANG_PATH=/usr/lib64"; \
      for t in xtensa_esp32_none_elf xtensa_esp32s2_none_elf xtensa_esp32s3_none_elf; do \
        echo "export BINDGEN_EXTRA_CLANG_ARGS_$t=\"--sysroot=$SYSROOT -I$SYSROOT/include\""; \
      done; \
    } | tee -a /home/claude/.clco-env >> /home/claude/.bashrc'

USER claude
WORKDIR /workspace
