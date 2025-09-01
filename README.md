# Mousefood Benchmark

An ESP32-S3 embedded Rust benchmarking suite for [`mousefood`](https://github.com/j-g00da/mousefood), a no-std embedded-graphics backend for Ratatui, and [`tachyonfx`](https://github.com/junkdog/tachyonfx) effects and animation library for ratatui applications, targeting the Waveshare ESP32-S3-LCD-1.69 development board.

Originally forked from [mousefood-esp32-demo](https://github.com/j-g00da/mousefood-esp32-demo).

## Features

- Text rendering performance tests with different styling modes
- Gauge widget benchmarks  
- System statistics display
- Real-time frame rate calculations
- Hardware integration (SPI display, button controls, battery monitoring)

## Hardware

- **Board**: Waveshare ESP32-S3-LCD-1.69
- **Display**: ST7789 (240x280, 90° rotation)
- **Controls**: GPIO0 button for scene transitions
- **Monitoring**: ADC battery voltage on GPIO1

## Quick Start

```bash
# Build and deploy (release)
./deploy.sh

# Build only
MCU=esp32s3 cargo build --target xtensa-esp32s3-espidf --release
```

See `CLAUDE.md` for detailed build and deployment instructions.