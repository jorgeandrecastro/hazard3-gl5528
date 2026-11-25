[![crates.io](https://img.shields.io/crates/v/hazard3-gl5528.svg)](https://crates.io/crates/hazard3-gl5528)
[![docs.rs](https://docs.rs/hazard3-gl5528/badge.svg)](https://docs.rs/hazard3-gl5528)
[![License: GPL v2](https://img.shields.io/badge/License-GPL_v2-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html)

# hazard3-gl5528

An async `no_std` driver for the **GL5528** photoresistor (LDR), part of the
independent [Hazard3 Async](https://github.com/jorgeandrecastro/hazard3-async) ecosystem.
By default, it uses `hazard3-rp` on the RP2350 Hazard3 RISC-V cores.
The upstream Embassy HAL is also available for RP2040 and RP235x.
See the [changelog](CHANGELOG.md) for the project history.

## Description

The **GL5528** is a light dependent resistor (LDR) whose resistance varies with ambient light:

| Condition       | Approx. resistance |
|-----------------|--------------------|
| Bright light    | ~1 kΩ              |
| Darkness        | ~1 MΩ+             |

The driver wraps asynchronous ADC readings from the selected HAL and exposes a simple API.

---

## Wiring

Connect the GL5528 in a voltage divider with a **10 kΩ** pull-down resistor
(GP26 is suitable for RP2040 and RP235xa; use the appropriate pin for RP235xb):

```
3.3V
 │
[GL5528]
 │
 ├──── GP26 (ADC0), for example
 │
[R 10kΩ]
 │
GND
```

> The voltage on GP26 increases with light intensity.

---

## Installation

After version 0.2.2 is published on crates.io:

```toml
[dependencies]
hazard3-gl5528 = "0.2.2"
```

Once the changes are pushed, the Git version can also be used:

```toml
[dependencies]
hazard3-gl5528 = { git = "https://github.com/jorgeandrecastro/hazard3-gl5528" }
```

### Choosing the HAL and chip

By default, `hal-hazard3` and `rp235xa` target the RP2350 Pico 2 with
`riscv32imac-unknown-none-elf`. The application must use the same HAL,
for example:

```toml
[dependencies]
hazard3-gl5528 = "0.2.2"
embassy-rp = { package = "hazard3-rp", version = "0.2", features = ["rp235xa"] }
```

Also configure the runtime, linker, and other compatible Hazard3 crates
according to the [Hazard3 Async guide](https://github.com/jorgeandrecastro/hazard3-async#quick-start).
Do not mix upstream types or runtime drivers with their Hazard3 counterparts.
For RP235xb, disable the default features and select
`features = ["hal-hazard3", "rp235xb"]`.

To use the **upstream Embassy** HAL on RP2040:

```toml
[dependencies]
hazard3-gl5528 = { version = "0.2.2", default-features = false, features = ["hal-upstream", "rp2040"] }
embassy-rp = { version = "0.10", features = ["rp2040"] }
```

For upstream RP235x, replace `rp2040` with `rp235xa` (Pico 2) or `rp235xb`
in both dependencies. `rp235x` remains an alias for `rp235xa` in this driver.
Select exactly one HAL and one chip. The Hazard3 HAL does not support RP2040.

### Migration from `embassy-rp-gl5528`

The former `embassy-rp-gl5528` releases have been yanked, not deleted:
existing lockfiles can continue to use them. Replace the dependency with
`hazard3-gl5528` and `use embassy_rp_gl5528::Gl5528` with
`use hazard3_gl5528::Gl5528`. The default HAL is now Hazard3 rather than
upstream Embassy: explicitly select `hal-upstream` and the chip to retain
the previous environment. The `Gl5528` API and `read_raw` behavior remain unchanged.

## Usage

This example uses GP26 on RP2040 or RP235xa. The application must provide
an executor and linker compatible with the selected HAL.

```rust
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Channel, Config as AdcConfig};
use embassy_rp::bind_interrupts;
use embassy_rp::adc::InterruptHandler;
use hazard3_gl5528::Gl5528;

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
    let channel = Channel::new_pin(p.PIN_26, embassy_rp::gpio::Pull::None);

    let mut sensor = Gl5528::new(adc, channel);

    loop {
        let raw = sensor.read_raw().await;
        // raw: 0 (darkness) → 4095 (bright light)

    }
}
```

### Converting raw readings to voltage

```rust
let voltage = (raw as f32 / 4095.0) * 3.3;

```

### Converting raw readings to LDR resistance

With a pull-down resistor `R_pull = 10 kΩ`:

```rust
// If the LDR is connected to 3.3V and the 10k resistor to GND:
if voltage > 0.0 {
    let r_ldr = 10_000.0 * ((3.3 - voltage) / voltage);
}
```

---

## API

### `Gl5528::new(adc, channel) -> Self`

Creates the driver, taking ownership of the ADC and its channel.

### async fn read_raw(&mut self) -> u16
Reads the sensor's raw ADC value.

- RP2040 : 12 bits (0..=4095).

- RP235x : 12 bits (0..=4095).

- Returns 0 on ADC error.

---

## Compatibility

| Dependency    | Version    |
|---------------|------------|
| `hazard3-rp` (default) | 0.2 |
| `embassy-rp` (upstream alternative) | 0.10 |
| Rust edition  | 2024       |
| `no_std`      |  ✓         |

---


# Pico 2040 example with an OLED display
Historical upstream Embassy example: select `hal-upstream` and `rp2040`,
and adapt the other dependencies and APIs to their respective versions.
It uses [`embassy-ssd1306`](https://crates.io/crates/embassy-ssd1306) and [`sigmoid-q15`](https://crates.io/crates/sigmoid-q15).

````rust
#![no_std]
#![no_main]

use cortex_m_rt as _;
use embassy_executor::Spawner;
use embassy_rp::i2c::{Config as I2cConfig, I2c, Async};
use embassy_time::{Timer, Duration};
use {panic_halt as _, embassy_rp as _};

use embassy_ssd1306::Ssd1306;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;

use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::I2C0;
use rp2040_linker as _;

// GPIO and ADC
use embassy_rp::gpio::{Output, Level, Pull};
use embassy_rp::adc::{Adc, Config as AdcConfig, Channel, InterruptHandler as AdcInterruptHandler};
use embassy_rp::i2c::InterruptHandler as I2cInterruptHandler;

// Light and sigmoid
use sigmoid_q15::sigmoid_q15;
use hazard3_gl5528::Gl5528;

bind_interrupts!(struct Irqs {
    I2C0_IRQ => I2cInterruptHandler<I2C0>;
    ADC_IRQ_FIFO => AdcInterruptHandler;
});

#[embassy_executor::task]
async fn system_task(
    mut oled: Ssd1306<I2cDevice<'static, NoopRawMutex, I2c<'static, I2C0, Async>>>,
    mut light_sensor: Gl5528<'static>,
) {
    if let Ok(_) = oled.init().await {
        oled.clear();
        let _ = oled.flush().await;
    }

    let test_points: [i16; 5] = [i16::MIN, -16384, 0, 16384, i16::MAX];
    let mut idx = 0;

    loop {
        oled.clear();
        oled.draw_rect(0, 0, 127, 63, true);

        let x_input = test_points[idx];
        let y_output = sigmoid_q15(x_input);

        oled.draw_str(10, 1, b"Sigmoid Q15");
        oled.draw_str(10, 3, b"In :");
        oled.draw_i16(50, 3, x_input);
        oled.draw_str(10, 5, b"Out:");
        oled.draw_i16(50, 5, y_output);

        // Read the sensor
        let lux_raw = light_sensor.read_raw().await;

        oled.draw_str(10, 7, b"Light:");
        if lux_raw > 2000 {
            oled.draw_str(75, 7, b"Light");
        } else {
            oled.draw_str(75, 7, b"Dark");
        }

        let _ = oled.flush().await;
        idx = (idx + 1) % test_points.len();

        Timer::after(Duration::from_secs(2)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // I2C & OLED
    let mut i2c_config = I2cConfig::default();
    i2c_config.frequency = 400_000;
    let i2c_bus = I2c::new_async(p.I2C0, p.PIN_5, p.PIN_4, Irqs, i2c_config);

    static I2C_BUS: static_cell::StaticCell<Mutex<NoopRawMutex, I2c<'static, I2C0, Async>>> = static_cell::StaticCell::new();
    let i2c_mutex = I2C_BUS.init(Mutex::new(i2c_bus));

    let i2c_dev_oled = I2cDevice::new(i2c_mutex);
    let oled = Ssd1306::new(i2c_dev_oled, 0x3C);
    let mut led = Output::new(p.PIN_25, Level::Low);

    // ADC and sensor
    let adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
    let p26 = Channel::new_pin(p.PIN_26, Pull::None);

    let light_sensor = Gl5528::new(adc, p26);

    spawner.spawn(system_task(oled, light_sensor)).unwrap();
    // Blink the Pico LED on pin 25
    loop {
        led.toggle();
        Timer::after_millis(200).await;
    };
}

````

## License

This project is distributed under the **GPL-2.0-or-later** license.
See [LICENSE](LICENSE) for details.

---

# 🦅 About
Developed and tested by Jorge Andre Castro.