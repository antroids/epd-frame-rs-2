use core::fmt::Debug;
use crate::scheduler::WeeklyScheduler;
use crate::time::NtpConfig;
use crate::types::{ByteBool, LimitedString};
use crate::wifi::{NetworkConfig, WifiAccessPointOptions, WifiJoinOptions};
use defmt::Format;
use serde::{Deserialize, Serialize};
use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes};

pub const VERSION: u16 = 0001;

#[derive(
    Debug, Format, IntoBytes, TryFromBytes, Copy, Clone, Immutable, KnownLayout, Deserialize,
)]
#[repr(C, align(4))]
pub struct PersistentState {
    pub version: u16,
    pub connect_to_wifi: ByteBool,
    pub wifi_join_options: WifiJoinOptions,
    pub wifi_join_network_config: NetworkConfig,
    pub wifi_access_point_options: WifiAccessPointOptions,
    pub wifi_access_point_network_config: NetworkConfig,
    pub ntp_config: NtpConfig,
    pub scheduler: WeeklyScheduler,
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
            scheduler: Default::default(),
        }
    }
}

#[derive(
    Debug,
    Format,
    IntoBytes,
    TryFromBytes,
    Copy,
    Clone,
    Immutable,
    KnownLayout,
    Deserialize,
    Serialize,
    Eq,
    PartialEq,
    Default
)]
#[repr(C, align(4))]
pub struct LastRunStatistics {
    pub status: LastRunStatus,
    pub failed_cause: LimitedString<256>,
}

impl LastRunStatistics {
    pub fn successful() -> Self {
        Self {
            status: LastRunStatus::Successful,
            failed_cause: Default::default(),
        }
    }
    pub fn from_debug(debug: impl Debug) -> Self {
        Self {
            status: LastRunStatus::Failed,
            failed_cause: LimitedString::from_debug_truncate(debug),
        }
    }
}

#[derive(
    Debug,
    Format,
    IntoBytes,
    TryFromBytes,
    Copy,
    Clone,
    Immutable,
    KnownLayout,
    Deserialize,
    Serialize,
    Eq,
    PartialEq,
    Default
)]
#[repr(u32)]
pub enum LastRunStatus {
    #[default]
    None = 0,
    Successful = 0b01001001_01001001_01001001_01001001,
    Failed = 0b11001100_11001100_11001100_11001100,
}
