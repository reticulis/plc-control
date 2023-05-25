use std::time::Duration;

use serialport::*;

use crate::{
    app::{DataMode, Preferences},
    error::PResult,
    utils::PushCrc,
};

pub struct Device {
    pub serial: Box<dyn SerialPort>,
}

impl Device {
    pub fn new(port: &str, preferences: &Preferences) -> PResult<Self> {
        let serial = serialport::new(port, preferences.baud_rate)
            .data_bits(preferences.data_bits)
            .parity(preferences.parity)
            .stop_bits(preferences.stop_bits)
            .timeout(Duration::from_micros((preferences.timeout * 1000.) as u64))
            .open()?;

        Ok(Self { serial })
    }

    pub fn send(&mut self, data: &str, data_type: DataMode, crc: bool) -> PResult<()> {
        match data_type {
            DataMode::Hex => self.serial.write_all(&str_to_hex(data)?.push_crc(crc))?,
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
