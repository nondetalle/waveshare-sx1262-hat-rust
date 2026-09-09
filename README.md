# Waveshare SX1262 868M LoRa HAT Rust driver

A blocking, thread-safe Rust driver for the **UART-controlled modem firmware** on the Waveshare SX1262 868M LoRa HAT. It does not expose the native Semtech SPI API and does not implement LoRaWAN.

## Features

- Raspberry Pi backend with `rppal` GPIO and `serialport`
- Normal, WOR, configuration, and sleep modes
- AUX-ready synchronization plus bounded timeouts
- Strict C0/C1/C2 command framing and echo validation
- Typed register configuration, reserved-bit preservation, temporary and persistent writes
- Product ID, transparent, fixed-address, and broadcast operations
- Conservative RSSI handling and experimental ambient-RSSI query
- Diagnostics, resynchronization, device-wide locking, and mockable transport abstraction

## Raspberry Pi 3B+ setup

1. Put the HAT UART routing jumpers in **B**.
2. Connect/control M0, M1, and AUX using GPIO lines. Remove conflicting M0/M1 jumpers when GPIO drives those lines.
3. Enable the primary UART. On many Raspberry Pi OS installations add `enable_uart=1` to `/boot/firmware/config.txt`, disable the serial console, and reboot.
4. Ensure the user can access `/dev/serial0` and GPIO, commonly through the `dialout` and `gpio` groups.
5. Verify that `/dev/serial0` targets the intended hardware UART. Bluetooth/UART overlays differ by image and configuration.

The example uses BCM 22 for M0, BCM 27 for M1, and BCM 4 for AUX. These are example assignments, not guaranteed Waveshare defaults.

## Build and run

```bash
cargo test
cargo run --example pi_basic
```

## Safe configuration

```rust
radio.enter_mode(Mode::Configuration)?;
let mut cfg = radio.read_configuration()?;
cfg.channel = Configuration::channel_from_frequency_mhz(868.125)?;
cfg.lbt_enabled = true;
radio.apply_configuration(&cfg, false, Mode::Normal)?; // temporary
```

Pass `true` only for an intentional nonvolatile write. Encryption-key registers are write-only and read as zero, so the driver verifies only registers `0x00..=0x06`.

## Receive boundaries and RSSI

UART RX is a byte stream. A host read is not guaranteed to equal one radio packet. `receive(..., known_packet_boundary)` strips an appended RSSI byte only when the caller explicitly says the read has a known packet boundary. Production applications should add framing, length, sequence, and integrity protection.

## Regulatory and security notes

The hardware tuning range is not a list of frequencies legal in every deployment. Validate local band, ERP, duty-cycle/channel-access and antenna requirements. The module's 16-bit proprietary key is not authenticated encryption. Protect sensitive payloads at the application layer.

## Known limitations

- Ambient RSSI query, broadcast interpretation, relay behavior, exact WOR response window, and RSSI conversion are firmware-dependent and require hardware validation.
- No arbitrary SX1262 LoRa modulation, IRQ, packet or register controls are available through the HAT UART firmware.
- Async support is not included in this initial crate; the transport trait makes a later async adapter straightforward.
