use crate::display::color::{BinaryColorAdapter, E6Color};
use crate::display::config_mode::draw_configuration_mode;
use crate::display::epd_spectra_6::nibbles::Nibbles;
use crate::display::epd_spectra_6::*;
use crate::display::image::{E6Image, E6ImageSource};
use crate::display::weather::Weather;
use crate::display::{CroppedDrawTarget, DISPLAY_HEIGHT, DISPLAY_WIDTH, weather};
use crate::errors::DeviceError;
use crate::http::server::ServerAction;
use crate::providers::open_meteo;
use crate::scheduler::WeeklyScheduler;
use crate::storage::{LastRunStatistics, LastRunStatus, PersistentState};
use crate::time::NtpConfig;
use crate::types::{Ipv4CidrAddress, LimitedString};
use crate::wifi::{
    Auth, NetworkConfig, WifiAccessPointOptions, WifiJoinOptions, WifiNetworkScanRecord,
};
use crate::{display, http, time};
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::{format, vec};
use chrono::{FixedOffset, Timelike};
use core::net::{IpAddr, SocketAddr};
use core::time::Duration;
use defmt_or_log::{derive_format_or_debug, error, info};
use embassy_executor::Spawner;
use embassy_net::dns::DnsQueryType;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_sync::channel::{Channel, Receiver, Sender};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::pixelcolor::{BinaryColor, Rgb888};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{
    CornerRadii, PrimitiveStyle, Rectangle, RoundedRectangle, StyledDrawable,
};
use embedded_graphics::text::Text;
use mplusfonts::BitmapFont;
use mplusfonts::style::{BitmapFontStyle, BitmapFontStyleBuilder};
use mplusfonts_macros::mplus;
use picoserve::make_static;
use sntpc::{NtpContext, NtpTimestampGenerator, get_time};
use sntpc_net_embassy::UdpSocketWrapper;
use zerocopy::FromBytes;

#[allow(dead_code)]
const POOL_NTP_ADDR: &str = "pool.ntp.org:123";
const BUFFER_SIZE: usize = 1024 * 8;
const REQUEST_SIZE: usize = 1024 * 2;

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
    fn leave_wifi(&self) -> impl Future<Output = Result<(), DeviceError>>;
    fn start_wifi_ap(
        &self,
        wifi_access_point_options: &WifiAccessPointOptions,
    ) -> impl Future<Output = Result<(), DeviceError>>;
    fn scan(&self) -> impl Future<Output = Result<Vec<WifiNetworkScanRecord>, DeviceError>>;
    fn network_stack(
        &self,
    ) -> impl Future<Output = Result<&embassy_net::Stack<'static>, DeviceError>>;
    fn rand(&mut self) -> impl Future<Output = u64>;
    fn display(&mut self) -> impl Future<Output = Result<&mut impl DisplayDriver, DeviceError>>;
    fn input_receiver(&self) -> DeviceInputReceiver;
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

    async fn http_client<'a>(&mut self) -> Result<http::client::HttpClient, DeviceError> {
        let seed = self.rand().await;
        let stack = self.network_stack().await?;
        Ok(http::client::HttpClient::new(stack.clone(), seed))
    }

    async fn run(&mut self, spawner: Spawner) {
        let last_run_statistics = self.read_last_run_statistics().await.unwrap_or_default();
        info!("Last run statistics: {:?}", last_run_statistics);
        if let Err(e) = self.main_loop(spawner, &last_run_statistics).await {
            error!("Main loop finished with error: {:?}", e);
            let run_statistics = LastRunStatistics::from_debug(e);
            if last_run_statistics != run_statistics {
                self.write_last_run_statistics(&run_statistics).await;
            }
            // TODO: use timer
            let _ = self.reset().await;
        }
    }

    async fn main_loop(
        &mut self,
        spawner: Spawner,
        last_run_statistics: &LastRunStatistics,
    ) -> Result<(), DeviceError> {
        let mut persistent_state = self.read_persistent_state().await.unwrap_or_else(|e| {
            error!(
                "Persistent state read error: {:?}, falling back to default",
                e
            );
            PersistentState::default()
        });

        if persistent_state.connect_to_wifi.as_bool() {
            self.online_mode_loop(persistent_state, last_run_statistics)
                .await?;
        } else {
            self.config_mode_loop(spawner, persistent_state, last_run_statistics)
                .await?;
        }

        Ok(())
    }

    async fn online_mode_loop(
        &mut self,
        mut persistent_state: PersistentState,
        last_run_statistics: &LastRunStatistics,
    ) -> Result<(), DeviceError> {
        self.init_network_stack(&persistent_state.wifi_join_network_config)
            .await?;
        self.join_wifi(&persistent_state.wifi_join_options).await?;

        let mut http_client = self.http_client().await?;
        let weather = open_meteo::get_weather(&mut http_client, 51.1, 17.039999).await?;

        self.leave_wifi().await?;
        self.power_off_radio_module().await?;

        let seed = self.rand().await;
        let mut rand = fastrand::Rng::with_seed(seed);
        let mut display = self.display().await?;
        let mut frame_buffer = FrameBuffer::new(
            Size::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32),
            E6Color::White,
        );
        let current_time = Some(weather.current.time);

        display::draw_weather(&mut frame_buffer, &weather, &mut rand).await?;
        display::draw_last_run_statistics(&mut frame_buffer, last_run_statistics).await?;

        display.refresh(frame_buffer.as_bytes()).await?;
        info!("Display refreshed");

        let input = self.input_receiver();
        while let Ok(input_event) = input.try_receive() {
            self.process_input(&input_event, persistent_state).await?;
        }
        let task_scheduler = current_time
            .map(|t| persistent_state.scheduler.task_scheduler(t))
            .unwrap_or_default();
        let run_statistics = LastRunStatistics::successful();
        if run_statistics != *last_run_statistics {
            self.write_last_run_statistics(&run_statistics).await;
        }
        info!("Powering off for {} minutes", task_scheduler.minutes_delay);
        self.power_off_for(Duration::from_mins(task_scheduler.minutes_delay as u64))
            .await?;

        Ok(())
    }

    async fn config_mode_loop(
        &mut self,
        spawner: Spawner,
        persistent_state: PersistentState,
        _last_run_statistics: &LastRunStatistics,
    ) -> Result<(), DeviceError> {
        {
            let mut display = self.display().await?;

            let mut frame_buffer = FrameBuffer::new(
                Size::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32),
                E6Color::White,
            );

            draw_configuration_mode(
                &persistent_state.wifi_access_point_options,
                &mut frame_buffer,
            )
            .await?;

            display.refresh(frame_buffer.as_bytes()).await?;
        }

        self.init_network_stack(&persistent_state.wifi_access_point_network_config)
            .await?;
        self.start_wifi_ap(&persistent_state.wifi_access_point_options)
            .await?;

        let action_channel = make_static!(http::server::ActionChannel, Channel::new());
        let seed = self.rand().await;
        let network_stack = self.network_stack().await?.clone();
        info!("Starting HTTP server");
        http::server::start_http_server(
            &spawner,
            network_stack,
            seed,
            persistent_state,
            action_channel,
        )?;
        info!("HTTP server started");

        let input_receiver = self.input_receiver();
        loop {
            use embassy_futures::select::{Either, select};

            match select(action_channel.receive(), input_receiver.receive()).await {
                Either::First(action) => {
                    info!("Received action: {:?}", action);
                    match action {
                        ServerAction::WriteState(state) => {
                            self.write_persistent_state(&state).await?
                        }
                        ServerAction::Restart => {
                            self.reset().await?;
                            break;
                        }
                    }
                }
                Either::Second(input) => {
                    info!("Received input: {:?}", input);
                    self.process_input(&input, persistent_state).await?;
                }
            }
        }

        Ok(())
    }

    async fn process_input(
        &mut self,
        input: &Input,
        mut persistent_state: PersistentState,
    ) -> Result<(), DeviceError> {
        match input {
            Input::Button1Click => self.reset().await,
            Input::Button1DoubleClick => {
                persistent_state.connect_to_wifi =
                    (!persistent_state.connect_to_wifi.as_bool()).into();
                self.write_persistent_state(&persistent_state).await?;
                self.reset().await
            }
            Input::Button1LongPress => {
                info!("Resetting the device configuration");
                self.write_persistent_state(&PersistentState::default())
                    .await?;
                self.reset().await
            }
        }
    }
}

#[derive(Clone, Copy)]
#[derive_format_or_debug]
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
