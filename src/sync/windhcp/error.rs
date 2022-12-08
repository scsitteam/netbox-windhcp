use std::{error::Error, fmt::Display};

use windows::Win32::Foundation::WIN32_ERROR;

pub type WinDhcpResult<'a, T> = Result<T, Box<WinDhcpError>>;

#[derive(Debug)]
pub struct WinDhcpError {
    message: &'static str,
    error: WIN32_ERROR,
}

impl WinDhcpError {
    pub fn new(message: &'static str, error: u32) -> Box<Self> {
        let message = message;
        let error = WIN32_ERROR(error);
        Box::new(Self { message, error })
    }
}

impl Error for WinDhcpError {}

impl Display for WinDhcpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error {}: {}", self.message, ::windows::core::Error::from(self.error).message())
    }
}