use crate::types;
use crate::types::{ByteBool, LimitedString};
use defmt::Format;
use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes};

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

#[derive(
    Copy, Clone, IntoBytes, TryFromBytes, Immutable, KnownLayout, Debug, Format, Eq, PartialEq,
)]
#[repr(C)]
pub struct WifiAccessPointOptions {
    pub ssid: LimitedString<MAX_SSID_LEN>,
    pub channel: u8,
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
    Copy, Clone, IntoBytes, TryFromBytes, Immutable, KnownLayout, Debug, Format, Eq, PartialEq,
)]
#[repr(C)]
pub struct NetworkConfig {
    pub ipv4_address: types::Ipv4CidrAddress,
    pub dhcp: ByteBool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            ipv4_address: types::Ipv4CidrAddress::new(types::Ipv4Address::new(192, 168, 1, 1), 32),
            dhcp: ByteBool::default(),
        }
    }
}

impl NetworkConfig {
    pub fn static_ipv4() -> Self {
        Self {
            ipv4_address: types::Ipv4CidrAddress::new(types::Ipv4Address::new(192, 168, 1, 1), 32),
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
    Copy, Clone, IntoBytes, TryFromBytes, Debug, Format, Eq, PartialEq, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct WifiNetworkScanRecord {
    pub ssid: LimitedString<MAX_SSID_LEN>,
}
