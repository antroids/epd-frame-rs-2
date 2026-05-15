#[cfg(feature = "defmt")]
use defmt::Format;
use thiserror::Error;
use crate::{display, http};
use crate::types::LimitedString;

#[derive(Debug, Error)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum DeviceError {
    #[error("Persistent state read error {0:?}")]
    PersistentStateReadError(LimitedString<128>),
    #[error("Persistent state write error {0:?}")]
    PersistentStateWriteError(LimitedString<128>),
    #[error("The persistent state is not valid {0:?}")]
    PersistentStateInvalid(LimitedString<128>),
    #[error("Dhcp configuration error")]
    DhcpConfigurationError,
    #[error("Dhcp server error {0:?}")]
    DhcpServerError(LimitedString<128>),
    #[error("Network stack is not initialized")]
    NetworkStackNotInitialized,
    #[error("Unable to spawn task {0:?}")]
    TaskSpawnError(LimitedString<128>),
    #[error("Unable to join Wifi Network")]
    UnableToJoinWifiNetwork,
    #[error("UTF-8 string error")]
    Utf8Error,
    #[error("Invalid timezone")]
    InvalidTimezone,
    #[error("DNS query error")]
    DnsQueryError,
    #[error("Display initialization error")]
    DisplayInitializationError,
    #[error("Display driver error {0:?}")]
    DisplayDriverError(LimitedString<128>),
    #[error("HTTP Client Error {0:?}")]
    HTTPClientError(#[from] http::HttpError),
    #[error("HTTP Error Code {0:?}")]
    HTTPError(u16),
    #[error("JSON Deserialization error {0:?}")]
    JSONDeserializationError(LimitedString<128>),
    #[error("Timer error {0:?}")]
    TimerError(LimitedString<128>),
}

impl From<core::str::Utf8Error> for DeviceError {
    fn from(_value: core::str::Utf8Error) -> Self {
        Self::Utf8Error
    }
}

#[macro_export]
macro_rules! impl_from_error_with_debug {
    ($error_type:ty, $enum_variant:ident) => {
        impl From<$error_type> for DeviceError {
            fn from(value: $error_type) -> Self {
                let msg = alloc::format!("{:?}", value);
                Self::$enum_variant(LimitedString::from_str_truncate(msg.as_str()))
            }
        }
    };
}

impl_from_error_with_debug!(serde_json::Error, JSONDeserializationError);
impl_from_error_with_debug!(embassy_executor::SpawnError, TaskSpawnError);
impl_from_error_with_debug!(display::epd_spectra_6::Error, DisplayDriverError);