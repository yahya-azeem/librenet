use std::error::Error;
use tun::{AsyncDevice, Configuration};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};

pub struct LibrenetTun {
    device: AsyncDevice,
}

impl LibrenetTun {
    pub fn new(name: &str) -> Result<Self, Box<dyn Error>> {
        let mut config = Configuration::default();
        config
            .name(name)
            .address((10, 0, 0, 1))
            .netmask((255, 255, 255, 0))
            .up();

        #[cfg(target_os = "linux")]
        config.platform(|config| {
            config.packet_information(true);
        });

        let device = tun::create_as_async(&config)?;
        Ok(Self { device })
    }

    pub fn split(self) -> (ReadHalf<AsyncDevice>, WriteHalf<AsyncDevice>) {
        tokio::io::split(self.device)
    }
}
