use crate::display::epd_spectra_6::*;
use crate::errors::DeviceError;
use crate::storage::{LastRunStatistics, PersistentState};
use crate::wifi::{
    NetworkConfig, WifiAccessPointOptions, WifiJoinOptions, WifiNetworkScanRecord,
};
use alloc::vec::Vec;
use core::time::Duration;
use defmt_or_log::derive_format_or_debug;
use embassy_sync::channel::{Channel, Receiver, Sender};

pub use device_impl::Device;

mod device_impl;

#[allow(dead_code)]
const POOL_NTP_ADDR: &str = "pool.ntp.org:123";
const ERROR_SLEEP_DURATION: Duration = Duration::from_mins(15);

pub trait DeviceInterface {
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
    fn leave_wifi(&self) -> impl Future<Output = Result<(), DeviceError>>;
    fn start_wifi_ap(
        &mut self,
        wifi_access_point_options: &WifiAccessPointOptions,
    ) -> impl Future<Output = Result<(), DeviceError>>;
    fn scan(&self) -> impl Future<Output = Result<Vec<WifiNetworkScanRecord>, DeviceError>>;
    fn network_stack(
        &self,
    ) -> impl Future<Output = Result<&embassy_net::Stack<'static>, DeviceError>>;
    fn rand(&mut self) -> impl Future<Output = u64>;
    fn display(&mut self) -> impl Future<Output = Result<&mut impl DisplayDriver, DeviceError>>;
    fn input_receiver(&self) -> DeviceInputReceiver;
    fn indicator_sender(&self) -> DeviceIndicatorSender;
    fn power_off_for(
        &mut self,
        duration: Duration,
    ) -> impl Future<Output = Result<(), DeviceError>>;
    fn power_off_radio_module(&mut self) -> impl Future<Output = Result<(), DeviceError>>;
    fn read_last_run_statistics(&mut self) -> impl Future<Output = Option<LastRunStatistics>>;
    fn write_last_run_statistics(
        &mut self,
        last_run_statistics: &LastRunStatistics,
    ) -> impl Future<Output = ()>;
}

#[derive(Clone, Copy)]
#[derive_format_or_debug]
#[repr(u8)]
pub enum Input {
    Button1Click,
    Button1DoubleClick,
    Button1LongPress,
}
pub const LONG_PRESS_MS: u64 = 1000;
pub const DOUBLE_CLICK_MS: u64 = 400;
const INPUT_CHANNEL_CAPACITY: usize = 16;
pub type DeviceInput = Channel<crate::RawMutex, Input, INPUT_CHANNEL_CAPACITY>;
pub type DeviceInputSender = Sender<'static, crate::RawMutex, Input, INPUT_CHANNEL_CAPACITY>;
pub type DeviceInputReceiver = Receiver<'static, crate::RawMutex, Input, INPUT_CHANNEL_CAPACITY>;


const INDICATOR_STATE_CAPACITY: usize = 16;
pub type DeviceIndicator = Channel<crate::RawMutex, IndicatorState, INDICATOR_STATE_CAPACITY>;
pub type DeviceIndicatorSender = Sender<'static, crate::RawMutex, IndicatorState, INDICATOR_STATE_CAPACITY>;
pub type DeviceIndicatorReceiver = Receiver<'static, crate::RawMutex, IndicatorState, INDICATOR_STATE_CAPACITY>;
#[derive(Default, Copy, Clone)]
#[derive_format_or_debug]
#[repr(u8)]
pub enum IndicatorState {
    #[default]
    Off,
    Loading,
    ReadingConfiguration,
    WritingConfiguration,
    HttpRequest,
    JoiningWifi,
    StartingWifiAccessPoint,
    ConfigurationMode,
    RenderingImage,
    UpdatingScreen,
    Error,
}

