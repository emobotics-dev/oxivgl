# syntax=docker/dockerfile:1

# Devcontainer for oxivgl — Rust (nightly + ESP/Xtensa), libclang,
# SDL2, cmake, and Python image-conversion tools.
FROM ghcr.io/emobotics-dev/claude-code:latest

USER root

# ── System packages (only what base image lacks) ────────────────────
# Base already has: build-essential, pkg-config, libssl-dev, libsdl2-dev,
# python3, python3-pip, curl, ca-certificates, unzip, wget, gh, adb
RUN apt-get update && apt-get install -y --no-install-recommends \
        cmake \
        libclang-dev \
        llvm-dev \
        python3-venv \
        libudev-dev \
    && rm -rf /var/lib/apt/lists/*

# Python deps for LVGLImage.py (image asset conversion)
RUN pip3 install --break-system-packages pypng lz4

# run_tests.sh / run_host.sh hardcode LIBCLANG_PATH=/usr/lib64 (Fedora layout).
# Symlink so the same path works on Debian.
RUN mkdir -p /usr/lib64 \
    && ln -sfn "$(llvm-config --libdir)/libclang.so" /usr/lib64/libclang.so

# ── Rust (nightly, for host builds/tests) ────────────────────────────
USER claude
ENV RUSTUP_HOME=/home/claude/.rustup \
    CARGO_HOME=/home/claude/.cargo \
    PATH="/home/claude/.cargo/bin:${PATH}"

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain nightly --profile minimal \
    && rustup component add llvm-tools-preview \
    && cargo install cargo-llvm-cov

# ── ESP/Xtensa toolchain (via espup) ────────────────────────────────
RUN cargo install espup \
    && cargo install --git https://github.com/esp-rs/espflash espflash \
    && espup install \
    && rm -rf "${CARGO_HOME}/registry" "${CARGO_HOME}/git"

# Wire espup env into .bashrc AND .clco-env so every shell and tool
# call inherits ESP PATH, LIBCLANG_PATH, and BINDGEN sysroot args.
RUN bash -c '\
    EXPORT_SH="$HOME/.espup/espup-export.sh"; \
    [ -f "$EXPORT_SH" ] || EXPORT_SH="$HOME/export-esp.sh"; \
    source "$EXPORT_SH"; \
    cat "$EXPORT_SH" >> "$HOME/.bashrc"; \
    cat "$EXPORT_SH" >> "$HOME/.clco-env"; \
    SYSROOT=$(dirname "$(which xtensa-esp32-elf-gcc)")/../xtensa-esp-elf; \
    echo "export BINDGEN_EXTRA_CLANG_ARGS_xtensa_esp32_none_elf=\"--sysroot=$SYSROOT\"" \
        | tee -a "$HOME/.bashrc" >> "$HOME/.clco-env"; \
    echo "export BINDGEN_EXTRA_CLANG_ARGS_xtensa_esp32s2_none_elf=\"--sysroot=$SYSROOT\"" \
        | tee -a "$HOME/.bashrc" >> "$HOME/.clco-env"; \
    echo "export BINDGEN_EXTRA_CLANG_ARGS_xtensa_esp32s3_none_elf=\"--sysroot=$SYSROOT\"" \
        | tee -a "$HOME/.bashrc" >> "$HOME/.clco-env"'

# Override LIBCLANG_PATH back to the system clang for host builds.
# espup set it to the ESP clang (Xtensa-only), which can't find x86_64
# headers. oxivgl-sys/build.rs auto-detects ESP clang for xtensa targets,
# so host builds just need the system clang.
RUN echo 'export LIBCLANG_PATH=/usr/lib64' \
    | tee -a "$HOME/.bashrc" >> "$HOME/.clco-env"

USER claude
WORKDIR /workspace
