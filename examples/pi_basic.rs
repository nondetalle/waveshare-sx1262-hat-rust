use std::time::Duration;
use waveshare_sx1262_hat::{Driver, Error, Mode};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let recv = std::env::args().any(|a| a == "r");

    let radio = Driver::new()?;

    // Configure
    radio.enter_mode(Mode::Configuration)?;
    let mut cfg = radio.read_configuration()?;
    println!("{:#?}", cfg);
    cfg.ambient_rssi_query_enabled = true;
    radio.apply_configuration(&cfg, false, Mode::Normal)?;

    println!("\nquery_ambient_rssi()");
    let rssi = radio.query_ambient_rssi()?;
    println!("RSSI: {} dBm", rssi);

    if !recv {
        println!("\nsend_broadcast()");
        loop {
            radio.send_broadcast(cfg.channel, b"Hello, World!")?;
            std::thread::sleep(Duration::from_secs(2));
        }
    } else {
        println!("\nreceive()");
        loop {
            match radio.receive(240, Duration::from_secs(3), false) {
                Ok(frame) => println!(
                    "received {} bytes: {}",
                    frame.payload.len(),
                    str::from_utf8(&frame.payload[3..]).unwrap_or("not utf-8")
                ),
                Err(Error::UartTimeout { operation: _ }) => {}
                Err(e) => Err(e)?,
            }
        }
    }
}
