use crate::device::{DeviceInterface, ERROR_SLEEP_DURATION, IndicatorState, Input};
use crate::display::color::E6Color;
use crate::display::config_mode::draw_configuration_mode;
use crate::display::epd_spectra_6::{FrameBuffer, DisplayDriver};
use crate::display::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::errors::DeviceError;
use crate::http::server::ServerAction;
use crate::providers::open_meteo;
use crate::storage::{LastRunStatistics, PersistentState};
use crate::{display, http};
use core::time::Duration;
use defmt_or_log::{error, info};
use embassy_executor::Spawner;
use embedded_graphics::geometry::Size;
use picoserve::make_static;

#[allow(async_fn_in_trait)]
pub trait Device: DeviceInterface {
    async fn http_client<'a>(&mut self) -> Result<http::client::HttpClient, DeviceError> {
        let seed = self.rand().await;
        let stack = self.network_stack().await?;
        Ok(http::client::HttpClient::new(stack.clone(), seed))
    }

    async fn run(&mut self, spawner: Spawner) {
        let mut indicator = self.indicator_sender();

        let _ = indicator.try_send(IndicatorState::Loading);
        let last_run_statistics = self.read_last_run_statistics().await.unwrap_or_default();
        info!("Last run statistics: {:?}", last_run_statistics);
        if let Err(e) = self.main_loop(spawner, &last_run_statistics).await {
            let _ = indicator.try_send(IndicatorState::Error);
            error!("Main loop finished with error: {:?}", e);
            let run_statistics = LastRunStatistics::from_debug(e);
            if last_run_statistics != run_statistics {
                self.write_last_run_statistics(&run_statistics).await;
            }
            let _ = self.power_off_for(ERROR_SLEEP_DURATION).await;
            error!("Failed to sleep after error");
            let _ = self.reset().await;
        }
    }

    async fn main_loop(
        &mut self,
        spawner: Spawner,
        last_run_statistics: &LastRunStatistics,
    ) -> Result<(), DeviceError> {
        let mut indicator = self.indicator_sender();

        let _ = indicator.try_send(IndicatorState::ReadingConfiguration);
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
        let mut indicator = self.indicator_sender();

        let _ = indicator.try_send(IndicatorState::JoiningWifi);
        self.init_network_stack(&persistent_state.wifi_join_network_config)
            .await?;
        self.join_wifi(&persistent_state.wifi_join_options).await?;

        let _ = indicator.try_send(IndicatorState::HttpRequest);
        let mut http_client = self.http_client().await?;
        let weather = open_meteo::get_weather(
            &mut http_client,
            persistent_state.weather_options.latitude,
            persistent_state.weather_options.longitude,
        )
        .await?;

        self.leave_wifi().await?;
        self.power_off_radio_module().await?;

        let _ = indicator.try_send(IndicatorState::RenderingImage);
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

        let _ = indicator.try_send(IndicatorState::UpdatingScreen);
        display.refresh(frame_buffer.as_bytes()).await?;
        info!("Display refreshed");

        let input = self.input_receiver();
        while let Ok(input_event) = input.try_receive() {
            self.process_input(&input_event, persistent_state).await?;
        }
        let task_scheduler = current_time
            .map(|t| persistent_state.scheduler.task_scheduler(t))
            .unwrap_or_default();

        let _ = indicator.try_send(IndicatorState::WritingConfiguration);
        let run_statistics = LastRunStatistics::successful();
        if run_statistics != *last_run_statistics {
            self.write_last_run_statistics(&run_statistics).await;
        }
        let _ = indicator.try_send(IndicatorState::Off);
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
        let mut indicator = self.indicator_sender();
        {
            let _ = indicator.try_send(IndicatorState::RenderingImage);
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

            let _ = indicator.try_send(IndicatorState::UpdatingScreen);
            display.refresh(frame_buffer.as_bytes()).await?;
        }

        let _ = indicator.try_send(IndicatorState::StartingWifiAccessPoint);
        self.init_network_stack(&persistent_state.wifi_access_point_network_config)
            .await?;
        self.start_wifi_ap(&persistent_state.wifi_access_point_options)
            .await?;

        let _ = indicator.try_send(IndicatorState::ConfigurationMode);
        let action_channel = make_static!(http::server::ActionChannel, crate::device::Channel::new());
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
                            let _ = indicator.try_send(IndicatorState::Off);
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
