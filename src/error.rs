use core::num;
use std::io;

use thiserror::Error;

pub type PResult<T> = Result<T, PlcError>;

#[derive(Error, Debug)]
pub enum PlcError {
    #[error("Failed connection to the device: {0}")]
    Connect(#[from] serialport::Error),

    #[error("Failed send/receive data, try reconnect device: {0}")]
    IO(#[from] io::Error),

    #[error("Command: Incorrect values")]
    CommandParse(#[from] num::ParseIntError),
}
