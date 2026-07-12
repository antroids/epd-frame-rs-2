use crate::{types, Validate};
use crate::types::{ByteBool, LimitedString};
use defmt::Format;
use serde::Deserialize;
use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes};
use crate::errors::DeviceError;

const MAX_SSID_LEN: usize = 32;
const MAX_PASSPHRASE_LEN: usize = 64;

#[derive(
    Default,
    Copy,
    Clone,
    IntoBytes,
    TryFromBytes,
    Immutable,
    KnownLayout,
    Debug,
    Format,
    PartialEq,
    Eq,
    Deserialize,
)]
#[repr(u8)]
pub enum Auth {
    #[default]
    Open,
    Wpa,
    Wpa2,
    Wpa3,
    Wpa2Wpa3,
}

#[derive(
    Default,
    Copy,
    Clone,
    IntoBytes,
    TryFromBytes,
    Debug,
    Format,
    Eq,
    PartialEq,
    Immutable,
    KnownLayout,
    Deserialize,
)]
#[repr(C)]
pub struct WifiJoinOptions {
    pub ssid: LimitedString<MAX_SSID_LEN>,
    pub auth: Auth,
    pub cipher_tkip: ByteBool,
    pub cipher_aes: ByteBool,
    pub passphrase: LimitedString<MAX_PASSPHRASE_LEN>,
    pub passphrase_is_prehashed: ByteBool,
}

impl Validate for WifiJoinOptions {
    fn validate(&self) -> Result<(), DeviceError> {
        let ssid = self.ssid.as_slice();
        if ssid.is_empty() {
            Err(DeviceError::NetworkConfigurationError(LimitedString::from_str("Access Point SSID cannot be empty")))
        } else {
            Ok(())
        }
    }
}

#[derive(
    Copy,
    Clone,
    IntoBytes,
    TryFromBytes,
    Immutable,
    KnownLayout,
    Debug,
    Format,
    Eq,
    PartialEq,
    Deserialize,
)]
#[repr(C)]
pub struct WifiAccessPointOptions {
    pub ssid: LimitedString<MAX_SSID_LEN>,
    pub channel: u8,
}

impl Validate for WifiAccessPointOptions {
    fn validate(&self) -> Result<(), DeviceError> {
        let ssid = self.ssid.as_slice();
        if ssid.is_empty() {
            Err(DeviceError::NetworkConfigurationError(LimitedString::from_str("Access Point SSID cannot be empty")))
        } else if self.channel < 1 || self.channel > 14 {
            Err(DeviceError::NetworkConfigurationError(LimitedString::from_str("Access Point channel invalid")))
        } else {
            Ok(())
        }
    }
}

impl Default for WifiAccessPointOptions {
    fn default() -> Self {
        Self {
            ssid: LimitedString::from_str("EPD_Frame_AP"),
            channel: 5,
        }
    }
}

#[derive(
    Copy,
    Clone,
    IntoBytes,
    TryFromBytes,
    Immutable,
    KnownLayout,
    Debug,
    Format,
    Eq,
    PartialEq,
    Deserialize,
)]
#[repr(C)]
pub struct NetworkConfig {
    pub ipv4_address: types::Ipv4CidrAddress,
    pub dhcp: ByteBool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self::static_ipv4()
    }
}

impl NetworkConfig {
    pub fn static_ipv4() -> Self {
        Self {
            ipv4_address: types::Ipv4CidrAddress::new(types::Ipv4Address::new(192, 168, 1, 1), 16),
            dhcp: false.into(),
        }
    }

    pub fn dhcp() -> Self {
        Self {
            ipv4_address: types::Ipv4CidrAddress::default(),
            dhcp: true.into(),
        }
    }
}

#[derive(
    Copy, Clone, IntoBytes, TryFromBytes, Debug, Format, Eq, PartialEq, Immutable, KnownLayout, Deserialize,
)]
#[repr(C)]
pub struct WifiNetworkScanRecord {
    pub ssid: LimitedString<MAX_SSID_LEN>,
}
