FROM ubuntu:jammy
# as rust-base

# NB: This can be "stable" or another version.
ARG RUST_TOOLCHAIN="1.75.0"

WORKDIR /tmp

# NB: cmake is required for freetype-sys-0.13.1, which in turn has only been added for egui.
# NB: fontconfig is required by servo-fontconfig-sys, which is in the dependency chain for egui.
# NB: libfontconfig-dev is required by servo-fontconfig-sys, which is in the dependency chain for egui.
# NB: pkg-config and libssl are for container TLS; we may switch to rustls in the future.
# BEVY SPECIFIC -
# NB: libasound2-dev for alsa-sys
# NB: libudev-dev for libudev-sys
RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive TZ=Etc/UTC apt-get install -y \
        build-essential \
        cmake \
        libasound2-dev \
        libudev-dev \
        curl \
        fontconfig \
        libfontconfig1-dev \
        libssl-dev \
        librust-wayland-commons-dev \
        librust-wayland-sys-dev \
        libwayland-client++0 \
        libwayland-client0 \
        pkg-config

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh  -s -- --default-toolchain $RUST_TOOLCHAIN -y

# Report Rust version for build debugging
RUN $HOME/.cargo/bin/rustup show
RUN $HOME/.cargo/bin/rustc --version
RUN $HOME/.cargo/bin/cargo --version

# Install NPM
# https://nodejs.org/en
# https://github.com/nodesource/distributions
#RUN curl -fsSL https://deb.nodesource.com/setup_21.x | bash - && \
#  apt-get install -y nodejs

# NB: Now we build and test our code.
COPY Cargo.lock .
COPY Cargo.toml .
COPY ./ .


RUN LD_LIBRARY_PATH=/usr/lib:${LD_LIBRARY_PATH} \
  $HOME/.cargo/bin/cargo build \
  --release \
  --bin studio-headless \
  --features headless
