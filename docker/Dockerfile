# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.80.1
ARG APP_NAME=rustyheadphones

# Use Debian-based Rust image for the build stage
FROM rust:${RUST_VERSION}-slim AS build

# Define app name as an argument
ARG APP_NAME

# Set the working directory
WORKDIR /app

# Update and install build dependencies using apt-get
# Update and install build dependencies using apt-get
RUN apt-get update && apt-get install -y \
    build-essential \
    clang \
    lld \
    git \
    libasound2-dev \
    pkg-config gcc

# Set environment variables for ALSA and libraries
ENV PKG_CONFIG_PATH=/usr/lib/pkgconfig
ENV LIBRARY_PATH=/usr/lib
ENV LD_LIBRARY_PATH=/usr/lib

# Build the application
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release && \
    cp ./target/release/$APP_NAME /bin/server

# Use the same Rust base image for runtime to avoid glibc version mismatch
FROM rust:${RUST_VERSION}-slim AS final

# Install ALSA runtime dependencies
RUN apt-get update && apt-get install -y \
    libasound2

# Create a non-privileged user
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser

# Copy the built server binary from the build stage
COPY --from=build /bin/server /bin/

# Expose the port your application listens on
EXPOSE 4589

# Run as non-privileged user
USER appuser

# Run the application
CMD ["/bin/server"]