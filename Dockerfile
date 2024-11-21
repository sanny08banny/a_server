# Use the Rust official image
FROM rust:latest as builder

# Set the working directory
WORKDIR /usr/src/app

# Copy the application code
COPY . .

# Build the application in release mode
RUN cargo build --release

# Use a minimal base image for the final build
FROM debian:bullseye-slim

# Install OpenSSL runtime (required for PostgreSQL)
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the compiled binary from the builder
COPY --from=builder /usr/target/release/house_server .

# Expose the port the application listens on
EXPOSE 4000

# Command to run the application
CMD ["./house_server"]
