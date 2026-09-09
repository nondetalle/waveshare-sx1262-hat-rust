use std::time::Duration;
use waveshare_sx1262_hat::{Driver, Mode};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example BCM pins only. Wire and configure these to match your HAT setup.
    let radio = Driver::new("/dev/serial0", 22, 27, 4)?;
    radio.enter_mode(Mode::Configuration)?;
    let pid = radio.read_product_id()?;
    println!("PID: {}", pid.hex());
    let mut cfg = radio.read_configuration()?;
    println!("frequency: {:.3} MHz", cfg.frequency_mhz());
    cfg.network_id = 1;
    radio.apply_configuration(&cfg, false, Mode::Normal)?;
    radio.send_transparent(b"hello")?;
    let frame = radio.receive(256, Duration::ZERO, false)?;
    println!("received {} bytes", frame.payload.len());
    Ok(())
}
