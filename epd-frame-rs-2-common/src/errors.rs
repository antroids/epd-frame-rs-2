use defmt::Format;
use thiserror::Error;

#[derive(Debug, Format, Copy, Clone, Eq, PartialEq, Error)]
pub enum DeviceError {
    #[error("Unable to read persistent state")]
    PersistentStateReadError,
    #[error("Unable to write persistent state")]
    PersistentStateWriteError,
    #[error("The persistent state is not valid")]
    PersistentStateInvalid,
    #[error("Dhcp configuration error")]
    DhcpConfigurationError,
    #[error("Network stack is not initialized")]
    NetworkStackNotInitialized,
    #[error("Unable to spawn task")]
    TaskSpawnError,
    #[error("Unable to join Wifi Network")]
    UnableToJoinWifiNetwork,
    #[error("UTF-8 string error")]
    Utf8Error,
    #[error("Invalid timezone")]
    InvalidTimezone,
    #[error("DNS query error")]
    DnsQueryError,
}

impl From<core::str::Utf8Error> for DeviceError {
    fn from(_value: core::str::Utf8Error) -> Self {
        Self::Utf8Error
    }
}