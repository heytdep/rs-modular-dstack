# ./examples/ping-host/Dockerfile
FROM rust:latest AS builder

# Set the working directory
WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*
RUN apt-get update && apt-get install -y libssl3 && apt-get clean

# Copy the entire workspace into the container
COPY . . 

# Navigate to the workspace root and build the specific crate
WORKDIR /usr/src/app/examples/ping-host
rustup target add x86_64-unknown-linux-gnu
RUN cargo build --release --target=x86_64-unknown-linux-gnu

# Final runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3 && apt-get clean

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/ping_host /usr/local/bin/ping-host

# Set the entrypoint
CMD ["ping-host"]
