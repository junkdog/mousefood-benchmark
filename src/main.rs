
mod gauge;
mod lorem;
mod fps;
mod catpuccin;
mod stats;
mod benchmark;
mod nonsense;
mod compute;
mod glyph_mapping;
mod string_ops;
mod embedded_str;
mod header;
mod worm_buffer;

use crate::gauge::GaugeApp;
use crate::benchmark::Benchmark;
use crate::nonsense::Nonsense;
use crate::compute::ComputeApp;
use crate::glyph_mapping::GlyphMappingApp;
use crate::string_ops::StringOpsApp;
use esp_idf_svc::hal::adc::Resolution;
use esp_idf_svc::hal::adc::attenuation::DB_11;
use esp_idf_svc::hal::adc::oneshot::config::{AdcChannelConfig, Calibration};
use esp_idf_svc::hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_svc::hal::delay::Ets;
use esp_idf_svc::hal::gpio::{AnyIOPin, InterruptType, PinDriver};
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::hal::spi::config::MODE_3;
use esp_idf_svc::hal::spi::{SpiConfig, SpiDeviceDriver, SpiDriverConfig};
use esp_idf_svc::hal::task::notification::Notification;
use mipidsi::Builder;
use mipidsi::interface::SpiInterface;
use mipidsi::models::ST7789;
use mipidsi::options::{ColorInversion, Orientation, Rotation};
use mousefood::prelude::*;
use std::num::{NonZeroU32, NonZeroUsize};
use std::thread;
use std::time::Duration;
use embedded_graphics::prelude::{DrawTarget, Point, Size};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics_unicodefonts::{mono_6x10_atlas, mono_6x10_optimized_atlas, mono_6x13_bold_atlas};
use ratatui::Terminal;
use ratatui::layout::Layout;
use crate::stats::Stats;

const DISPLAY_OFFSET: (u16, u16) = (0, 0);
const DISPLAY_SIZE: (u16, u16) = (
    240 - DISPLAY_OFFSET.0 * 2,
    320 - DISPLAY_OFFSET.1 * 0,
);

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    // Turn on display backlight
    let mut backlight = PinDriver::output(peripherals.pins.gpio15).unwrap();
    backlight.set_high().unwrap();

    // Configure SPI - Optimized for display refresh performance
    let config = SpiConfig::new()
        .write_only(true)
        .baudrate(80u32.MHz().into())  // Increased from 40MHz to maximum 80MHz
        .data_mode(MODE_3);
    let spi_device = SpiDeviceDriver::new_single(
        peripherals.spi2,
        peripherals.pins.gpio6,
        peripherals.pins.gpio7,
        Option::<AnyIOPin>::None,
        Some(peripherals.pins.gpio5),
        &SpiDriverConfig::new(),
        &config,
    ).unwrap();

    let buffer = Box::leak(Box::new([0_u8; 8192]));  // Increased buffer size for better throughput
    let spi_interface = SpiInterface::new(
        spi_device,
        PinDriver::output(peripherals.pins.gpio4).unwrap(),
        buffer,
    );

    // Configure display
    let mut delay = Ets;
    let mut display = Builder::new(ST7789, spi_interface)
        .invert_colors(ColorInversion::Inverted)
        .reset_pin(PinDriver::output(peripherals.pins.gpio8).unwrap())
        .display_offset(DISPLAY_OFFSET.0, DISPLAY_OFFSET.1)
        .display_size(DISPLAY_SIZE.0, DISPLAY_SIZE.1)
        .orientation(Orientation::new().rotate(Rotation::Deg90))
        .init(&mut delay)
        .expect("Failed to init display");

    // Setup button interrupt
    let mut button = PinDriver::input(peripherals.pins.gpio0).unwrap();
    button.set_interrupt_type(InterruptType::NegEdge).unwrap();
    let mut notification = Notification::new();
    let notifier = notification.notifier();
    unsafe {
        button
            .subscribe(move || {
                notifier.notify_and_yield(NonZeroU32::new(1).unwrap());
            })
            .unwrap();
    }

    // Setup battery voltage reader
    let adc_driver = AdcDriver::new(peripherals.adc1).unwrap();
    let mut battery_adc_channel = AdcChannelDriver::new(
        &adc_driver,
        peripherals.pins.gpio1,
        &AdcChannelConfig {
            attenuation: DB_11,
            calibration: Calibration::Curve,
            resolution: Resolution::Resolution12Bit,
        },
    )
    .unwrap();

    // Setup Mousefood and Ratatui
    let mut config = EmbeddedBackendConfig::default();
    config.font_regular = mono_6x10_optimized_atlas();
    config.font_bold = Some(mono_6x13_bold_atlas());

    display.fill_solid(
        &Rectangle::new(
            Point::new(0, 0),
            Size::new(DISPLAY_SIZE.1 as _, DISPLAY_SIZE.0 as _)
        ),
        Rgb565::new(0, 0, 0)
    ).unwrap();

    let backend = EmbeddedBackend::new(&mut display, config);
    let mut terminal = Terminal::new(backend).unwrap();
    Layout::init_cache(NonZeroUsize::new(20).unwrap()); // default is 500

    loop {
        Stats::new()
            .run(&mut terminal, &mut notification, &mut button)
            .unwrap();
        thread::sleep(Duration::from_millis(200));

        Nonsense::new()
            .run(&mut terminal, &mut notification, &mut button)
            .unwrap();

        thread::sleep(Duration::from_millis(200));

        ComputeApp::new()
            .run(&mut terminal, &mut notification, &mut button)
            .unwrap();

        thread::sleep(Duration::from_millis(200));

        GlyphMappingApp::new(&mono_6x10_optimized_atlas())
            .run(&mut terminal, &mut notification, &mut button)
            .unwrap();

        thread::sleep(Duration::from_millis(200));

        StringOpsApp::new(&mono_6x10_atlas())
            .run(&mut terminal, &mut notification, &mut button)
            .unwrap();

        thread::sleep(Duration::from_millis(200));

        Benchmark::new()
            .run(&mut terminal, &mut notification, &mut button)
            .unwrap();

        thread::sleep(Duration::from_millis(200));

        GaugeApp::new()
            .run(&mut terminal, &mut notification, &mut button)
            .unwrap();

        thread::sleep(Duration::from_millis(200));
    }
}
