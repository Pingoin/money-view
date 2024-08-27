#!/bin/bash
cd app

# Führe das Protobuf-Skript aus
./compile-proto.sh

# Führe Flutter aus
flutter run -d chrome
