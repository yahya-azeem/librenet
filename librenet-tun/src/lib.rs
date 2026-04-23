use std::error::Error;
use tun::AsyncDevice;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct LibrenetTun {
    device: AsyncDevice,
}

impl LibrenetTun {
    pub fn new(name: &str) -> Result<Self, Box<dyn Error>> {
        let mut config = tun::Configuration::default();
        config
            .name(name)
            .address((10, 0, 0, 1)) // Default internal IP for the bridge
            .netmask((255, 255, 255, 0))
            .up();

        #[cfg(target_os = "linux")]
        config.platform(|config| {
            config.packet_information(true);
        });

        let device = tun::create_as_async(&config)?;
        Ok(Self { device })
    }

    pub async fn read_packet(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        self.device.read(buf).await
    }

    pub async fn write_packet(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        self.device.write(buf).await
    }
}
