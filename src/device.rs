use anyhow::Result;
use serialport::*;

pub struct Device {
    serial: Box<dyn SerialPort>,
}

impl Device {
    pub fn new(port: &str) -> Result<Self> {
        let serial = serialport::new(port, 119_000)
            .data_bits(DataBits::Eight)
            .parity(Parity::Odd)
            .stop_bits(StopBits::One)
            .open()?;
            
            
        Ok(Self {
            serial
        })
    }
}

pub fn get_devices_list() -> Result<Vec<String>> {
    Ok(serialport::available_ports()?
        .into_iter()
        .map(|device| device.port_name)
        .collect())
}
