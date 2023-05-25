use std::ops::{Deref, DerefMut};
use crc::{Crc, CRC_16_MODBUS};

// Check if the value has been changed since the last use
#[derive(Default)]
pub struct CValue<T> {
    value: T,
    changed: bool,
}

impl<T> CValue<T> {
    pub fn is_changed(&mut self) -> bool {
        let b = self.changed;
        self.changed = false;
        b
    }
}

impl<T> Deref for CValue<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for CValue<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.changed = true;
        &mut self.value
    }
}

pub const MODBUS: Crc<u16> = Crc::<u16>::new(&CRC_16_MODBUS);

pub trait PushCrc {
    fn push_crc(&mut self, crc: bool) -> Self;
}

impl PushCrc for Vec<u8> {
    fn push_crc(&mut self, crc: bool) -> Self {
        if crc {
            let checksum = MODBUS.checksum(self);
        
            self.push(((checksum << 8) >> 8) as u8);
            self.push((checksum >> 8) as u8);
        
            return self.to_vec()
        }
        
        
        self.to_vec()
    }
}