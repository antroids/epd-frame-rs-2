use crate::errors::DeviceError;
use crate::storage::PersistentState;
use crate::time;
use crate::time::NtpConfig;
use crate::types::{Ipv4CidrAddress, LimitedString};
use crate::wifi::{
    Auth, NetworkConfig, WifiAccessPointOptions, WifiJoinOptions, WifiNetworkScanRecord,
};
use alloc::vec::Vec;
use core::net::{IpAddr, SocketAddr};
use core::time::Duration;
use defmt::{error, info};
use embassy_net::dns::DnsQueryType;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use sntpc::{NtpContext, NtpTimestampGenerator, get_time};
use sntpc_net_embassy::UdpSocketWrapper;

#[allow(dead_code)]
const POOL_NTP_ADDR: &str = "pool.ntp.org:123";

pub type HttpClient<'a> = nanofish::client::DefaultHttpClient<'a>;

pub trait Device {
    fn read_persistent_state(
        &mut self,
    ) -> impl Future<Output = Result<PersistentState, DeviceError>>;
    fn write_persistent_state(
        &mut self,
        persistent_state: &PersistentState,
    ) -> impl Future<Output = Result<(), DeviceError>>;
    fn reset(&mut self) -> impl Future<Output = Result<(), DeviceError>>;
    fn init_network_stack(
        &mut self,
        network_config: &NetworkConfig,
    ) -> impl Future<Output = Result<(), DeviceError>>;
    fn join_wifi(
        &self,
        wifi_join_options: &WifiJoinOptions,
    ) -> impl Future<Output = Result<(), DeviceError>>;
    fn start_wifi_ap(
        &self,
        wifi_access_point_options: &WifiAccessPointOptions,
    ) -> impl Future<Output = Result<(), DeviceError>>;
    fn scan(&self) -> impl Future<Output = Result<Vec<WifiNetworkScanRecord>, DeviceError>>;
    fn network_stack(&self) -> impl Future<Output = Result<&embassy_net::Stack, DeviceError>>;
    fn rand(&mut self) -> impl Future<Output = u64>;
    async fn http_client<'a>(&'a self) -> Result<HttpClient<'a>, DeviceError> {
        let stack = self.network_stack().await?;
        Ok(HttpClient::new(stack))
    }

    async fn main_loop(&mut self) -> Result<(), DeviceError> {
        let mut config = self.read_persistent_state().await.unwrap_or_else(|e| {
            error!(
                "Persistent state read error: {:?}, falling back to default",
                e
            );
            PersistentState::default()
        });

        config.connect_to_wifi = true.into();
        config.wifi_join_options = WifiJoinOptions {
            ssid: LimitedString::from_str("Orange_2g"),
            auth: Auth::Wpa2,
            cipher_tkip: Default::default(),
            cipher_aes: Default::default(),
            passphrase: LimitedString::from_str("RYMXUA7HP99A"),
            passphrase_is_prehashed: Default::default(),
        };
        config.ntp_config = Default::default();

        if config.connect_to_wifi.as_bool() {
            self.init_network_stack(&config.wifi_join_network_config)
                .await?;
            self.join_wifi(&config.wifi_join_options).await?;
            self.online_mode_loop(&config.ntp_config).await?;
        } else {
            self.init_network_stack(&config.wifi_access_point_network_config)
                .await?;
            self.start_wifi_ap(&config.wifi_access_point_options)
                .await?;
        }

        let scan_result = self.scan().await?;

        for n in scan_result {
            info!("Network SSID: '{}'", n.ssid)
        }

        self.write_persistent_state(&config).await?;

        Ok(())
    }

    async fn online_mode_loop(&mut self, ntp_config: &NtpConfig) -> Result<(), DeviceError> {
        let utc_time = time::ntp_get_time_utc(
            self.network_stack().await?.clone(),
            ntp_config.ntp_server.as_utf8_str()?,
        ).await?;

        info!("Time received: {:?}", utc_time);
        Ok(())
    }
}
