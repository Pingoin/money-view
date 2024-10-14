# Use a minimal base image for running the server
FROM debian:bullseye-slim

ARG TARGETARCH

# Install dependencies
RUN apt-get update && apt-get install -y nginx supervisor && rm -rf /var/lib/apt/lists/*

# Copy Rust server binary
COPY ./bin/$TARGETARCH/money-view /usr/local/bin/money-view
RUN chmod +x /usr/local/bin/money-view

# Copy Flutter web build
COPY ./web /usr/share/nginx/html

# Kopiere den Nginx-Konfigurationsfile
COPY nginx.conf /etc/nginx/conf.d

# Erstelle eine Supervisor-Konfiguration
COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf

# Exponiere die Ports für gRPC und Nginx
EXPOSE 50051 80

# Starte Supervisor, um beide Dienste zu starten
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/supervisord.conf"]
