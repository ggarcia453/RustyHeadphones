# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.80.1
ARG APP_NAME=rustyheadphones

FROM ubuntu:20.04 AS build

# Install dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libasound2-dev \
    alsa-utils \
    pulseaudio

# Install Rust using rustup
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain ${RUST_VERSION}

# Set environment variables for Rust
ENV PATH=/root/.cargo/bin:$PATH

# Create application directory
WORKDIR /app

# Copy the source code into the container
COPY . .

# Build the Rust application using cargo
RUN cargo build --locked --release && \
    cp ./target/release/$APP_NAME /bin/server

FROM debian:bullseye-slim AS final

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libasound2 \
    pulseaudio \
    && rm -rf /var/lib/apt/lists/*

# Add ALSA null output configuration
RUN echo "pcm.!default { type null }" > /etc/asound.conf

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

# Expose the necessary ports
EXPOSE 4589

# Run as the non-privileged user
USER appuser

CMD ["/bin/server"]