# Use a minimal base image for running the server
FROM debian:bookworm-slim

ARG TARGETARCH
WORKDIR /opt/money-view
# Copy Rust server binary
COPY ./bin/$TARGETARCH/money-view .
RUN chmod +x ./money-view

# Copy Flutter web build
COPY ./web .

# Exponiere die Ports für gRPC und Nginx
EXPOSE 50051 8080

# Starte Supervisor, um beide Dienste zu starten
CMD ["./money-view"]
