# Stage 1: Rust build
FROM rust:latest AS rust-build

# Installiere Abhängigkeiten und Protobuf-Compiler
RUN apt-get update && apt-get install -y protobuf-compiler

WORKDIR /usr/src

RUN USER=root cargo new --bin app

# Setze das Arbeitsverzeichnis
WORKDIR /usr/src/app

# Kopiere nur die Cargo.toml und Cargo.lock zum Abhängigkeits-Caching
COPY Cargo.toml Cargo.lock ./

# Caching: Bauen Sie nur die Abhängigkeiten
RUN cargo build --release --bin main
RUN cargo build --bin main
RUN rm -r src

# Kopiere jetzt den Rest des Quellcodes
COPY ./proto ./proto
COPY ./src ./src


# Kompiliere die Protobuf-Dateien für Rust
RUN cargo run --bin compile_protos 

# Baue den Rust gRPC-Server
RUN cargo build --release
# Stage 2: Flutter build using cirrusci/flutter
FROM ghcr.io/cirruslabs/flutter:3.24.3 AS flutter-build

# Setze das Arbeitsverzeichnis für Flutter
WORKDIR /usr/src/app/app

# Installiere protobuf-compiler (protoc) in der Flutter-Build-Umgebung
RUN apt-get update && apt-get install -y protobuf-compiler

# Installiere Dart Protoc-Plugin (protoc-gen-dart)
RUN dart pub global activate protoc_plugin
# Stelle sicher, dass der Dart-Protoc-Plugin im PATH ist
ENV PATH="$PATH:/root/.pub-cache/bin"

# Kopiere nur die pubspec.yaml und installiere Abhängigkeiten
COPY app/pubspec.yaml .
RUN flutter pub get

# Kopiere den Rest der Flutter App
COPY app .
COPY proto ./proto

# Kompiliere die Protobuf-Dateien für Dart
RUN mkdir -p lib/generated
RUN protoc -I=./proto --dart_out=grpc:lib/generated ./proto/*.proto
RUN flutter pub get
# Baue die Flutter-Web-App
RUN flutter build web --release

# Stage 3: Final image
FROM debian:bullseye-slim

# Installiere Nginx
RUN apt-get update && apt-get install -y nginx

# Kopiere den Nginx-Konfigurationsfile
COPY nginx.conf /etc/nginx/conf.d

# Kopiere die Flutter-Web-Build-Dateien in das Nginx-HTML-Verzeichnis
COPY --from=flutter-build /usr/src/app/app/build/web /usr/share/nginx/html

# Kopiere den komprimierten Rust-Server ins finale Image
COPY --from=rust-build /usr/src/app/target/release/main /usr/bin/rust-server

# Erstelle eine Supervisor-Konfiguration
COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf

# Exponiere die Ports für gRPC und Nginx
EXPOSE 50051 80

# Starte Supervisor, um beide Dienste zu starten
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/supervisord.conf"]
