# Mousefood Benchmark

An ESP32-S3 embedded Rust benchmarking suite for [`mousefood`](https://github.com/j-g00da/mousefood), a no-std embedded-graphics backend for Ratatui, and [`tachyonfx`](https://github.com/junkdog/tachyonfx) effects and animation library for ratatui applications, targeting the Waveshare ESP32-S3-LCD-1.69 development board.

Originally forked from [mousefood-esp32-demo](https://github.com/j-g00da/mousefood-esp32-demo).

## Features

- Text rendering performance tests with different styling modes
- Gauge widget benchmarks  
- System statistics display
- Real-time frame rate calculations
- Tachyonfx effects

## Quick Start

```bash
# Build and deploy (release)
./deploy.sh

# Build only
MCU=esp32s3 cargo build --target xtensa-esp32s3-espidf --release
```
