use waveshare_sx1262_hat::*;
#[test] fn default_encoding(){let c=Configuration::default();let r=c.encode().unwrap();assert_eq!(r[3],0x62);assert_eq!(r[5],0x12)}
#[test] fn frequency_mapping(){assert_eq!(Configuration::channel_from_frequency_mhz(868.125).unwrap(),0x12);assert!(Configuration::channel_from_frequency_mhz(868.5).is_err())}
#[test] fn roundtrip_readable_fields(){let c=Configuration::default();let d=Configuration::decode(c.encode().unwrap());assert_eq!(d.address,c.address);assert_eq!(d.air_rate,c.air_rate);assert_eq!(d.channel,c.channel)}
