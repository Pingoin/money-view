#!/bin/bash

# Verzeichnis mit .proto Dateien
PROTO_DIR="../proto"
# Zielverzeichnis für die generierten Dart-Dateien
OUT_DIR="lib/generated"

mkdir -p $OUT_DIR

# Kompiliere alle .proto Dateien
for proto_file in $PROTO_DIR/*.proto; do
    echo "Kompiliere $proto_file"

    protoc -I=$PROTO_DIR --dart_out=grpc:$OUT_DIR $proto_file
done

echo "Protobuf-Dateien wurden erfolgreich kompiliert!"
