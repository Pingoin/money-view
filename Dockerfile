# Use a minimal base image for running the server
FROM debian:bookworm-slim

ARG TARGETARCH

# Copy Rust server binary
COPY ./bin/$TARGETARCH/money-view /opt/money-view/money-view
RUN chmod +x /opt/money-view/money-view

# Copy Flutter web build
COPY ./web /opt/money-view/web

# Exponiere die Ports für gRPC und Nginx
EXPOSE 50051 8080

# Starte Supervisor, um beide Dienste zu starten
CMD ["/opt/money-view/money-view"]
