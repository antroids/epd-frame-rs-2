use embassy_executor::Spawner;
use embassy_net::{Config, Ipv4Address, Ipv4Cidr, Stack, StackResources};
use embassy_net_tuntap::TunTapDevice;
use embassy_time::{Duration, Timer};
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::sdl2::MouseButton;
use embedded_graphics_simulator::{OutputSettings, SimulatorDisplay, SimulatorEvent, Window};
use epd_frame_rs_2_common::device::{
    DOUBLE_CLICK_MS, Device, DeviceIndicator, DeviceIndicatorSender, DeviceInput,
    DeviceInputReceiver, DeviceInputSender, Input, LONG_PRESS_MS,
};
use epd_frame_rs_2_common::display::color::E6Color;
use epd_frame_rs_2_common::display::epd_spectra_6::nibbles::Nibbles;
use epd_frame_rs_2_common::display::epd_spectra_6::{DisplayDriver, Error};
use epd_frame_rs_2_common::display::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use epd_frame_rs_2_common::errors::DeviceError;
use epd_frame_rs_2_common::storage::{LastRunStatistics, PersistentState};
use epd_frame_rs_2_common::types::LimitedString;
use epd_frame_rs_2_common::wifi::{
    NetworkConfig, WifiAccessPointOptions, WifiJoinOptions, WifiNetworkScanRecord,
};
use log::info;
use rand::random;
use static_cell::{StaticCell, make_static};
use std::io::{Read, Write};
use std::time::Instant;
use zerocopy::{IntoBytes, TryFromBytes};

const PERSISTENT_STATE_FILE: &str = "state.bin";
const LAST_RUN_STATISTICS_FILE: &str = "last_run.json";

pub struct SimulatorDevice {
    display: SimulatorDeviceDisplay,
    network_stack: Option<Stack<'static>>,
    spawner: Spawner,
    input: &'static DeviceInput,
    indicator: &'static DeviceIndicator,
}

impl SimulatorDevice {
    pub fn new(spawner: Spawner) -> SimulatorDevice {
        let input = make_static!(DeviceInput::new());
        let indicator = make_static!(DeviceIndicator::new());
        let input_sender = input.sender();

        Self {
            display: SimulatorDeviceDisplay::new(input_sender),
            network_stack: None,
            spawner,
            input,
            indicator,
        }
    }
}

impl Device for SimulatorDevice {
    async fn read_persistent_state(&mut self) -> Result<PersistentState, DeviceError> {
        let mut buf = vec![];
        let mut file = std::fs::File::open(PERSISTENT_STATE_FILE).map_err(|e| {
            DeviceError::PersistentStateReadError(LimitedString::from_debug_truncate(e))
        })?;
        file.read_to_end(&mut buf).map_err(|e| {
            DeviceError::PersistentStateReadError(LimitedString::from_debug_truncate(e))
        })?;
        Ok(PersistentState::try_read_from_bytes(&buf).map_err(|e| {
            DeviceError::PersistentStateReadError(LimitedString::from_debug_truncate(e))
        })?)
    }

    async fn write_persistent_state(
        &mut self,
        persistent_state: &PersistentState,
    ) -> Result<(), DeviceError> {
        let mut file = std::fs::File::create(PERSISTENT_STATE_FILE).map_err(|e| {
            DeviceError::PersistentStateReadError(LimitedString::from_debug_truncate(e))
        })?;
        file.write_all(persistent_state.as_bytes()).map_err(|e| {
            DeviceError::PersistentStateWriteError(LimitedString::from_debug_truncate(e))
        })?;
        Ok(())
    }

    async fn reset(&mut self) -> Result<(), DeviceError> {
        info!("resetting simulator device");
        Ok(())
    }

    async fn init_network_stack(
        &mut self,
        _network_config: &NetworkConfig,
    ) -> Result<(), DeviceError> {
        info!("Initializing network stack");
        let device = TunTapDevice::new("tap99").unwrap();
        let dns_servers: heapless::Vec<Ipv4Address, 3> = heapless::Vec::from_slice(&[
            Ipv4Address::new(8, 8, 4, 4).into(),
            Ipv4Address::new(8, 8, 8, 8).into(),
        ])
        .unwrap();
        let config = Config::ipv4_static(embassy_net::StaticConfigV4 {
            address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 69, 1), 24),
            dns_servers,
            gateway: Some(Ipv4Address::new(192, 168, 69, 100)),
        });
        //let config = Config::dhcpv4(Default::default());

        static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
        let (stack, runner) = embassy_net::new(
            device,
            config,
            RESOURCES.init(StackResources::new()),
            self.rand().await,
        );
        self.spawner.spawn(net_task(runner)?);
        self.network_stack = Some(stack);

        Ok(())
    }

    async fn join_wifi(&self, _wifi_join_options: &WifiJoinOptions) -> Result<(), DeviceError> {
        info!("Joining WiFi network");
        Ok(())
    }

    async fn leave_wifi(&self) -> Result<(), DeviceError> {
        info!("Leaving WiFi network");
        Ok(())
    }

    async fn start_wifi_ap(
        &self,
        _wifi_access_point_options: &WifiAccessPointOptions,
    ) -> Result<(), DeviceError> {
        info!("Starting WiFi AP");
        Ok(())
    }

    async fn scan(&self) -> Result<Vec<WifiNetworkScanRecord>, DeviceError> {
        info!("Scanning for WiFi networks");
        Ok(vec![WifiNetworkScanRecord {
            ssid: LimitedString::from_str("TestWifiNetwork"),
        }])
    }

    async fn network_stack(&self) -> Result<&Stack<'static>, DeviceError> {
        self.network_stack
            .as_ref()
            .ok_or_else(|| DeviceError::NetworkStackNotInitialized)
    }

    async fn rand(&mut self) -> u64 {
        random()
    }

    async fn display(&mut self) -> Result<&mut impl DisplayDriver, DeviceError> {
        Ok(&mut self.display)
    }

    fn input_receiver(&self) -> DeviceInputReceiver {
        self.input.receiver()
    }

    fn indicator_sender(&self) -> DeviceIndicatorSender {
        self.indicator.sender()
    }

    async fn power_off_for(&mut self, duration: std::time::Duration) -> Result<(), DeviceError> {
        info!("Powering off for {:?}", duration);
        Ok(())
    }

    async fn power_off_radio_module(&mut self) -> Result<(), DeviceError> {
        info!("Powering off radio module");
        Ok(())
    }

    async fn read_last_run_statistics(&mut self) -> Option<LastRunStatistics> {
        std::fs::File::open(LAST_RUN_STATISTICS_FILE)
            .ok()
            .and_then(|file| serde_json::from_reader(file).ok())
    }

    async fn write_last_run_statistics(&mut self, last_run_statistics: &LastRunStatistics) {
        std::fs::File::open(LAST_RUN_STATISTICS_FILE)
            .unwrap()
            .write_all(serde_json::to_vec(last_run_statistics).unwrap().as_slice())
            .unwrap();
    }
}

pub struct SimulatorDeviceDisplay(SimulatorDisplay<E6Color>, DeviceInputSender);

impl SimulatorDeviceDisplay {
    pub fn new(device_input_sender: DeviceInputSender) -> Self {
        Self(
            SimulatorDisplay::<E6Color>::new(Size::new(
                DISPLAY_WIDTH as u32,
                DISPLAY_HEIGHT as u32,
            )),
            device_input_sender,
        )
    }
}

impl DisplayDriver for SimulatorDeviceDisplay {
    async fn refresh(&mut self, frame_buffer: &[u8]) -> Result<(), Error> {
        let data = frame_buffer.to_vec();
        let nibbles: Nibbles<_, E6Color> =
            Nibbles::new(data, DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize);
        self.0
            .fill_contiguous(&self.0.bounding_box(), nibbles.into_iter())
            .unwrap();

        let output_settings = OutputSettings::default();
        let mut window = Window::new("Hello World", &output_settings);

        window.update(&self.0);

        let mut click_state = ClickState::new();

        'running: loop {
            for event in window.events() {
                match event {
                    SimulatorEvent::Quit => break 'running,
                    SimulatorEvent::MouseButtonDown {
                        mouse_btn: MouseButton::Left,
                        ..
                    } => {
                        click_state.handle_mouse_down();
                    }
                    SimulatorEvent::MouseButtonUp {
                        mouse_btn: MouseButton::Left,
                        ..
                    } => {
                        if let Some(click_type) = click_state.handle_mouse_up() {
                            info!("Sending click event: {:?}", click_type);
                            let _ = self.1.try_send(click_type.into());
                        }
                    }
                    _ => {}
                }
            }

            if let Some(click_type) = click_state.check_timeout() {
                info!("Sending click event: {:?}", click_type);
                let _ = self.1.try_send(click_type.into());
            }

            Timer::after(Duration::from_millis(20)).await;
        }

        Ok(())
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, TunTapDevice>) -> ! {
    runner.run().await
}

struct ClickState {
    last_click: Option<Instant>,
    click_count: u32,
    press_start: Option<Instant>,
}

impl ClickState {
    fn new() -> Self {
        Self {
            last_click: None,
            click_count: 0,
            press_start: None,
        }
    }

    fn handle_mouse_down(&mut self) {
        self.press_start = Some(Instant::now());
    }

    fn handle_mouse_up(&mut self) -> Option<ClickType> {
        let now = Instant::now();

        if let Some(press_start) = self.press_start.take() {
            let press_duration = now.duration_since(press_start);

            if press_duration.as_millis() > LONG_PRESS_MS as u128 {
                self.click_count = 0;
                self.last_click = None;
                return Some(ClickType::Long);
            }

            if let Some(last_click) = self.last_click {
                let time_since_last = now.duration_since(last_click);
                if time_since_last.as_millis() < DOUBLE_CLICK_MS as u128 {
                    self.click_count += 1;
                    if self.click_count >= 2 {
                        self.click_count = 0;
                        self.last_click = None;
                        return Some(ClickType::Double);
                    }
                } else {
                    self.click_count = 1;
                }
            } else {
                self.click_count = 1;
            }
            self.last_click = Some(now);
        }

        None
    }

    fn check_timeout(&mut self) -> Option<ClickType> {
        if self.click_count == 1 {
            if let Some(last_click) = self.last_click {
                let now = Instant::now();
                if now.duration_since(last_click).as_millis() >= 300 {
                    self.click_count = 0;
                    self.last_click = None;
                    return Some(ClickType::Short);
                }
            }
        }
        None
    }
}

#[derive(Copy, Clone, Debug)]
enum ClickType {
    Short,
    Double,
    Long,
}

impl From<ClickType> for Input {
    fn from(value: ClickType) -> Self {
        match value {
            ClickType::Short => Input::Button1Click,
            ClickType::Double => Input::Button1DoubleClick,
            ClickType::Long => Input::Button1LongPress,
        }
    }
}
