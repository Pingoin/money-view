default: serve-web

# Compiles the protobuf files for the Rust subproject
compile-protos-rust:
    cargo run --bin compile_protos

# Starts the server and compiles the protobuf files beforehand
serve-server: compile-protos-rust
    cargo watch -x "run --bin main"

# Formats the Rust code
format-rust:
    cargo fmt

# Formats the Flutter code
format-flutter:
    cd app && dart format .

# Runs formatting for both Rust and Flutter
format: format-rust format-flutter

compile-protos-dart:
    #!/bin/bash
    PROTO_DIR="proto"
    # Zielverzeichnis für die generierten Dart-Dateien
    OUT_DIR="app/lib/generated"

    mkdir -p $OUT_DIR

    # Kompiliere alle .proto Dateien
    for proto_file in $PROTO_DIR/*.proto; do
        echo "Kompiliere $proto_file"

        protoc -I=$PROTO_DIR --dart_out=grpc:$OUT_DIR $proto_file
    done

    echo "Protobuf-Dateien wurden erfolgreich kompiliert!"

# starts flutter dev in web mode
serve-flutter-web: compile-protos-dart
    cd app && flutter run -d chrome

# Run server and client in a split terminal using tmux
serve-web:
    tmux new-session -d -s mysession 'just serve-server' \; split-window -h 'just serve-flutter-web' \; attach

compile-protos: compile-protos-dart compile-protos-rust