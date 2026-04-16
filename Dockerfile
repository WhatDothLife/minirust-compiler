# Wir starten direkt mit einem offiziellen Rust-Image auf Debian-Basis
FROM rust:1.85-slim-bookworm

# Update und Installation der RISC-V Tools + QEMU
# Wir kombinieren die Befehle, um die Image-Größe klein zu halten
RUN apt-get update && apt-get install -y \
    gcc-riscv64-linux-gnu \
    qemu-user-static \
    vim \
    && rm -rf /var/lib/apt/lists/*

RUN rustup default nightly

# Arbeitsverzeichnis im Container setzen
WORKDIR /cc
