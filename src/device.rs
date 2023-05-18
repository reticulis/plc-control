use serialport::*;

use crate::{app::DataMode, error::PResult};

pub struct Device {
    pub serial: Box<dyn SerialPort>,
}

impl Device {
    pub fn new(port: &str) -> PResult<Self> {
        let serial = serialport::new(port, 115_200)
            .data_bits(DataBits::Eight)
            .parity(Parity::Odd)
            .stop_bits(StopBits::One)
            .open()?;

        Ok(Self { serial })
    }

    pub fn send(&mut self, data: &str, data_type: DataMode) -> PResult<()> {
        match data_type {
            DataMode::Hex => self.serial.write_all(&str_to_hex(data)?)?,
            _ => self.serial.write_all(data.as_bytes())?,
        }

        Ok(())
    }

    pub fn read(&mut self) -> PResult<Vec<u8>> {
        let mut buf = Vec::new();
        match self.serial.read_to_end(&mut buf) {
            Ok(_) => Ok(buf),
            Err(err) => {
                if buf.is_empty() {
                    Err(err)?
                } else {
                    Ok(buf)
                }
            }
        }
    }

    pub fn get_devices_list() -> Result<Vec<String>> {
        Ok(serialport::available_ports()?
            .into_iter()
            .map(|device| device.port_name)
            .collect())
    }
}

pub fn str_to_hex(data: &str) -> PResult<Vec<u8>> {
    data.split_whitespace()
        .map(|n| Ok(u8::from_str_radix(n, 16)?))
        .collect()
}
