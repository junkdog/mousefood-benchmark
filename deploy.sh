#!/bin/bash

# ESP32-S3 deployment script
# Usage: ./deploy.sh [release|debug] [port]

PROFILE=${1:-release}
PORT=${2:-/dev/ttyACM0}

if [[ "$PROFILE" != "release" && "$PROFILE" != "debug" ]]; then
    echo "Error: Profile must be 'release' or 'debug'"
    echo "Usage: $0 [release|debug] [port]"
    exit 1
fi

echo "Deploying to ESP32-S3 with profile: $PROFILE, port: $PORT"

if [[ "$PROFILE" == "release" ]]; then
    MCU=esp32s3 cargo build --target xtensa-esp32s3-espidf --release
    espflash flash --port "$PORT" --chip esp32s3 target/xtensa-esp32s3-espidf/release/mousefood-benchmark
else
    MCU=esp32s3 cargo build --target xtensa-esp32s3-espidf
    espflash flash --port "$PORT" --chip esp32s3 target/xtensa-esp32s3-espidf/debug/mousefood-benchmark
fi
