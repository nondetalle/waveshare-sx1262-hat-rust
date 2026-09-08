use std::time::Duration;
use waveshare_sx1262_hat::{rppal_backend::RppalTransport, Driver, Mode};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example BCM pins only. Wire and configure these to match your HAT setup.
    let transport = RppalTransport::new("/dev/serial0", 22, 27, 17)?;
    let radio = Driver::new(Box::new(transport));
    radio.open()?;
    radio.enter_mode(Mode::Configuration)?;
    let pid = radio.read_product_id()?;
    println!("PID: {}", pid.hex());
    let mut cfg = radio.read_configuration()?;
    println!("frequency: {:.3} MHz", cfg.frequency_mhz());
    cfg.network_id = 1;
    radio.apply_configuration(&cfg, false, Mode::Normal)?;
    radio.send_transparent(b"hello")?;
    let frame = radio.receive(256, Duration::from_secs(5), false)?;
    println!("received {} bytes", frame.payload.len());
    radio.close()?;
    Ok(())
}
