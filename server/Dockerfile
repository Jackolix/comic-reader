# Use the official Rust image as the base image
FROM rust:latest as builder

# Set the working directory
WORKDIR /app

# Copy the entire project
COPY . .

# Build the project in release mode
RUN cargo build --release
RUN ldd /app/target/release/comic-reader-server
RUN readelf -V /app/target/release/comic-reader-server

FROM ubuntu:oracular

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/comic-reader-server /usr/local/bin/app

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000

# Set the entrypoint
ENTRYPOINT ["/usr/local/bin/app"]