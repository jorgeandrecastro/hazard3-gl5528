// Copyright (C) 2026 Jorge Andre Castro
// GPL-2.0-or-later
//! # hazard3-gl5528
//!
//! Minimal async `no_std` driver for the **GL5528** photoresistor (LDR),
//! part of the independent [Hazard3 Async](https://github.com/jorgeandrecastro/hazard3-async) ecosystem.
//! The upstream Embassy HAL is available as an alternative for RP2040 and RP235x.
//! The `hal-hazard3` and `hal-upstream` features are mutually exclusive; select
//! exactly one chip from `rp2040`, `rp235xa`, and `rp235xb`.
//! By default, `hal-hazard3` and `rp235xa` are enabled. The application must
//! use the same HAL, for example `embassy-rp = { package = "hazard3-rp",
//! version = "0.2", features = ["rp235xa"] }` for Hazard3.
//!
//! ## Component description
//!
//! The GL5528 is a light dependent resistor (LDR) whose resistance varies
//! between ~1 kΩ (bright light) and ~10 MΩ (total darkness). It is typically
//! used in a voltage divider with a fixed resistor (10 kΩ recommended) to
//! convert the resistance variation into a voltage readable by an ADC.
//!
//! ## Wiring diagram
//!
//! ```text
//! 3.3V
//!  │
//! [GL5528]
//!  │
//!  ├──── GP26 (ADC0)
//!  │
//! [R 10kΩ]
//!  │
//! GND
//! ```
//!
//! The ADC pin voltage increases with light intensity (the GL5528 becomes
//! less resistive and the midpoint voltage rises toward 3.3 V).
//!
//! ## Usage example
//!
//! The application must also provide a suitable executor and linker.
//!
//! ```rust,ignore
//! #![no_std]
//! #![no_main]
//!
//! use embassy_executor::Spawner;
//! use embassy_rp::adc::{Adc, Channel, Config as AdcConfig};
//! use embassy_rp::bind_interrupts;
//! use embassy_rp::adc::InterruptHandler;
//! use hazard3_gl5528::Gl5528;
//!
//! bind_interrupts!(struct Irqs {
//!     ADC_IRQ_FIFO => InterruptHandler;
//! });
//!
//! #[embassy_executor::main]
//! async fn main(_spawner: Spawner) {
//!     let p = embassy_rp::init(Default::default());
//!
//!     let adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
//!     let channel = Channel::new_pin(p.PIN_26, embassy_rp::gpio::Pull::None);
//!
//!     let mut sensor = Gl5528::new(adc, channel);
//!
//!     loop {
//!         let raw = sensor.read_raw().await;
//!         // Value between 0 (darkness) and 4095 (bright light, 12 bits)
//!         let _ = raw;
//!     }
//! }
//! ```
//!
//! ## Calculating light intensity
//!
//! The raw ADC value can be converted to voltage (12 bits):
//!
//! ```text
//! V = raw × 3.3 / 4095
//! ```
//!
//! The LDR resistance follows from the voltage divider (R_pull = 10 kΩ):
//!
//! ```text
//! R_ldr = R_pull × (3.3 - V) / V, for V > 0
//! ```
//!
//! ## Specifications
//!
//! | Parameter              | Value                                 |
//! |------------------------|---------------------------------------|
//! | Supply voltage         | 3.3 V (RP2040 / RP235x)               |
//! | ADC resolution         | 12 bits (0–4095)                      |
//! | Light resistance       | ~1 kΩ @ 10 lux                        |
//! | Dark resistance        | ~1 MΩ minimum                         |
//! | Pull-down resistance   | 10 kΩ recommended                     |
//!
//! ## `no_std`
//!
//! This crate does not depend on the standard library and is designed to run
//! on bare-metal microcontrollers with a compatible runtime.

#![no_std]
#![forbid(unsafe_code)]

#[cfg(all(feature = "hal-hazard3", feature = "hal-upstream"))]
compile_error!("Select exactly one HAL: hal-hazard3 or hal-upstream.");
#[cfg(not(any(feature = "hal-hazard3", feature = "hal-upstream")))]
compile_error!("Select a HAL: hal-hazard3 or hal-upstream.");
#[cfg(all(feature = "hal-hazard3", feature = "rp2040"))]
compile_error!("The Hazard3 HAL does not support RP2040.");
#[cfg(not(any(feature = "rp2040", feature = "rp235xa", feature = "rp235xb")))]
compile_error!("Select a chip: rp2040, rp235xa or rp235xb.");
#[cfg(any(
    all(feature = "rp2040", feature = "rp235xa"),
    all(feature = "rp2040", feature = "rp235xb"),
    all(feature = "rp235xa", feature = "rp235xb")
))]
compile_error!("Select exactly one chip: rp2040, rp235xa or rp235xb.");

#[cfg(all(feature = "hal-upstream", not(feature = "hal-hazard3")))]
use embassy_rp as selected_hal;
#[cfg(feature = "hal-hazard3")]
use hazard3_rp_backend as selected_hal;
#[cfg(all(feature = "hal-upstream", not(feature = "hal-hazard3")))]
use selected_hal::adc::Async;
use selected_hal::adc::{Adc, Channel};
#[cfg(feature = "hal-hazard3")]
use selected_hal::mode::Async;

/// GL5528 photoresistor driver using the RP2040 or RP235x ADC.
///
/// This driver wraps an ADC channel and provides asynchronous readings
/// of the raw sensor value.
///
/// # Example
///
/// ```rust,no_run
/// # use hazard3_gl5528::Gl5528;
/// // Create the driver with the ADC and channel obtained from the HAL.
/// // let mut sensor = Gl5528::new(adc, channel);
/// // let raw: u16 = sensor.read_raw().await;
/// ```
pub struct Gl5528<'d> {
    adc: Adc<'d, Async>,
    channel: Channel<'d>,
}

impl<'d> Gl5528<'d> {
    /// Creates a new GL5528 driver instance.
    ///
    /// # Arguments
    ///
    /// * `adc` — ADC peripheral in asynchronous mode.
    /// * `channel` — ADC channel connected to the GL5528 reading pin.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use hazard3_gl5528::Gl5528;
    /// // let adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
    /// // let channel = Channel::new_pin(p.PIN_26, embassy_rp::gpio::Pull::None);
    /// // let sensor = Gl5528::new(adc, channel);
    /// ```
    #[inline]
    pub fn new(adc: Adc<'d, Async>, channel: Channel<'d>) -> Self {
        Self { adc, channel }
    }

    /// Reads the raw ADC value.
    ///
    /// Returns the raw reading (`0..=4095` on RP2040 and RP235x).
    /// A higher value corresponds to brighter light.
    ///
    /// Returns `0` if an ADC error occurs.
    ///
    /// # Returns
    ///
    /// * u16 — Raw ADC value.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use hazard3_gl5528::Gl5528;
    /// # async fn example(mut sensor: Gl5528<'_>) {
    /// let raw: u16 = sensor.read_raw().await;
    ///
    /// // Convert to voltage (V)
    /// let voltage = raw as f32 * 3.3 / 4095.0;
    ///
    /// // Convert to LDR resistance (kΩ), with R_pull = 10 kΩ
    /// if voltage > 0.0 {
    ///     // If the LDR is connected to 3.3V and R_pull (10k) to GND:
    ///     let r_ldr = 10.0 * (3.3 - voltage) / voltage;
    ///     let _ = r_ldr;
    /// }
    /// # }
    /// ```

    #[inline]
    pub async fn read_raw(&mut self) -> u16 {
        self.adc.read(&mut self.channel).await.unwrap_or(0)
    }
}
