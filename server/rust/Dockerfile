# Use the official Rust image as the base image
FROM rust:latest as builder

# Set the working directory
WORKDIR /app

# Copy the entire project
COPY . .

# Build the project in release mode
RUN cargo build --release

# Create a lightweight runtime image
FROM debian:bookworm-slim

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/comic-reader-server /usr/local/bin/app

# Set the entrypoint
ENTRYPOINT ["/usr/local/bin/app"]