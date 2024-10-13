# Use a minimal base image for running the server
FROM debian:bullseye-slim

# Install dependencies
RUN apt-get update && apt-get install -y nginx && rm -rf /var/lib/apt/lists/*

# Copy Rust server binary
COPY ./target/aarch64-unknown-linux-gnu/release/my_rust_server /usr/local/bin/my_rust_server

# Copy Flutter web build
COPY ./app/build/web /var/www/html

# Expose ports
EXPOSE 50051 80

# Start NGINX and the Rust server
CMD ["sh", "-c", "nginx && my_rust_server"]
