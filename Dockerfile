# Use the official Rust image as the base image
FROM rust:1.82-slim as builder

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build the dependencies (this layer will be cached if Cargo.toml doesn't change)
RUN cargo build --release

# Remove the dummy main.rs and copy the actual source code
RUN rm src/main.rs
COPY src ./src

# Build the actual application
RUN cargo build --release

# Create the final runtime image
FROM debian:bookworm-slim

# Install ca-certificates for HTTPS requests
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Create a non-root user for security
RUN useradd -m -u 1001 appuser

# Set the working directory
WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/personal-api /app/personal-api

# Create assets directory and copy any assets
RUN mkdir -p /app/assets
COPY assets/ /app/assets/

# Change ownership to the non-root user
RUN chown -R appuser:appuser /app

# Switch to the non-root user
USER appuser

# Expose the port the app runs on
EXPOSE 3030

# Set environment variables
ENV RUST_LOG=info

# Run the application
CMD ["./personal-api"]
