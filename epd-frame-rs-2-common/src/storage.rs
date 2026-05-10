use defmt::Format;
use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes};
use crate::time::NtpConfig;
use crate::types::ByteBool;
use crate::wifi::{NetworkConfig, WifiAccessPointOptions, WifiJoinOptions};

pub const VERSION: u16 = 0001;

#[derive(Debug, Format, IntoBytes, TryFromBytes, Copy, Clone, Immutable, KnownLayout)]
#[repr(align(4))]
pub struct PersistentState {
    pub version: u16,
    pub connect_to_wifi: ByteBool,
    pub wifi_join_options: WifiJoinOptions,
    pub wifi_join_network_config: NetworkConfig,
    pub wifi_access_point_options: WifiAccessPointOptions,
    pub wifi_access_point_network_config: NetworkConfig,
    pub ntp_config: NtpConfig,
}

impl Default for PersistentState {
    fn default() -> Self {
        Self {
            version: VERSION,
            connect_to_wifi: Default::default(),
            wifi_join_options: Default::default(),
            wifi_join_network_config: NetworkConfig::dhcp(),
            wifi_access_point_options: Default::default(),
            wifi_access_point_network_config: NetworkConfig::static_ipv4(),
            ntp_config: Default::default(),
        }
    }
}