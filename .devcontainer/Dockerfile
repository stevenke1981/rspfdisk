FROM mcr.microsoft.com/devcontainers/rust:latest

# System dependencies for rspfdisk development & testing
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive && apt-get install -y --no-install-recommends \
    # QEMU for boot testing
    qemu-system-x86 \
    ovmf \
    # Boot ISO creation
    grub-pc-bin \
    xorriso \
    cpio \
    gzip \
    # Additional utilities
    file \
    && rm -rf /var/lib/apt/lists/*

# Verify Rust toolchain
RUN rustc --version && cargo --version && rustup target list --installed
