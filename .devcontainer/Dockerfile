# Devcontainer for oxivgl — extends clco base with Rust, ESP/Xtensa, SDL2,
# libclang, cmake, and Python image-conversion tools.
FROM ghcr.io/emobotics-dev/claude-code:latest

USER root

# ── System build dependencies ────────────────────────────────────────────────
RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential \
        cmake \
        pkg-config \
        libclang-dev \
        llvm-dev \
        libsdl2-dev \
        python3 \
        python3-pip \
        python3-venv \
        curl \
        ca-certificates \
        unzip \
    && rm -rf /var/lib/apt/lists/*

# Python deps for LVGLImage.py (image asset conversion)
RUN pip3 install --break-system-packages pypng lz4

# Project scripts (run_tests.sh, run_host.sh) hardcode LIBCLANG_PATH=/usr/lib64
# which is the Fedora layout. Symlink so the same path works on Debian.
RUN ln -sfn "$(llvm-config --libdir)/libclang.so" /usr/lib64/libclang.so

# ── Rust (nightly, for host builds/tests) ────────────────────────────────────
USER claude
ENV RUSTUP_HOME=/home/claude/.rustup \
    CARGO_HOME=/home/claude/.cargo \
    PATH="/home/claude/.cargo/bin:${PATH}"

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain nightly \
    && rustup component add llvm-tools-preview \
    && cargo install cargo-llvm-cov

# ── ESP/Xtensa toolchain (via espup) ────────────────────────────────────────
# espflash v4.3.0 has a nix-crate build bug — pin to 3.3.0 for now.
RUN cargo install espup espflash@3.3.0 \
    && espup install \
    && rm -rf "${CARGO_HOME}/registry" "${CARGO_HOME}/git"

# Bake espup env into the image. The export script sets PATH (xtensa-elf gcc),
# LIBCLANG_PATH (xtensa clang), etc. We source it and persist the paths into
# .bashrc so every shell inherits them, plus write individual path files
# (mirrors CI's /opt/esp_*_path convention).
RUN bash -c '\
    EXPORT_SH="$HOME/.espup/espup-export.sh"; \
    [ -f "$EXPORT_SH" ] || EXPORT_SH="$HOME/export-esp.sh"; \
    source "$EXPORT_SH"; \
    echo "$LIBCLANG_PATH"          > "$HOME/.espup/esp_libclang_path"; \
    SYSROOT=$(dirname "$(which xtensa-esp32-elf-gcc)")/../xtensa-esp-elf; \
    echo "$SYSROOT"                > "$HOME/.espup/esp_sysroot_path"; \
    ELF_BIN=$(dirname "$(which xtensa-esp32-elf-gcc)"); \
    echo "$ELF_BIN"               > "$HOME/.espup/esp_elf_bin_path"; \
    cat "$EXPORT_SH" >> "$HOME/.bashrc"; \
    cat "$EXPORT_SH" >> "$HOME/.clco-env"'

# ── Host LIBCLANG_PATH (default) ────────────────────────────────────────────
# For host builds, override the ESP libclang with the system one.
# run_tests.sh and run_host.sh already do this; set the default here too
# so `cargo +nightly check` works without sourcing a script.
RUN echo "export LIBCLANG_PATH=$(llvm-config --libdir)" >> "$HOME/.bashrc"

WORKDIR /workspace
