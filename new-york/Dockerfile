# ./new-york/Dockerfile
FROM rust:latest AS builder

# Set the working directory
WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy the entire workspace into the container
COPY ../../ . 

# Navigate to the specific crate directory and build
WORKDIR /usr/src/app/new-york
RUN cargo build --release

# Final runtime image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 && apt-get clean


COPY --from=builder /usr/src/app/target/release/guest /usr/local/bin/guest
CMD ["guest"]
