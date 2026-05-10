use alloc::vec::Vec;
use core::time::Duration;
use defmt::warn;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, DMA_CH2, DMA_CH3, PIO0, SPI1, TRNG};
use embassy_rp::pio::InterruptHandler;
use embassy_rp::spi::Async;
use embassy_rp::trng::Trng;
use embassy_rp::watchdog::Watchdog;
use embassy_rp::{bind_interrupts, dma, gpio, pac};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Delay, Timer};
use epd_frame_rs_2_common::device::{Device, DeviceInput, DeviceInputReceiver};
use epd_frame_rs_2_common::errors::DeviceError;
use epd_frame_rs_2_common::storage::{LastRunStatistics, PersistentState};
use epd_frame_rs_2_common::types::LimitedString;
use epd_frame_rs_2_common::wifi::{
    NetworkConfig, WifiAccessPointOptions, WifiJoinOptions, WifiNetworkScanRecord,
};
use log::info;
use static_cell::{StaticCell, make_static};

const LAST_RUN_STATISTICS_OFFSET: u32 = 1024 * 4;
const POWMAN_PASSWORD: u32 = 0x5AFE << 16;

pub mod button;
pub mod flash;
pub mod wifi;

type DisplayDriver<'d> = epd_frame_rs_2_common::display::epd_spectra_6::AsyncE6Display<
    gpio::Output<'d>,
    gpio::Output<'d>,
    gpio::Input<'d>,
    SpiDevice<'d, CriticalSectionRawMutex, embassy_rp::spi::Spi<'d, SPI1, Async>, gpio::Output<'d>>,
    Delay,
>;

pub struct Rp235Device {
    flash: flash::Flash,
    network_stack: wifi::WifiStack,
    trng: Trng<'static, TRNG>,
    watchdog: Watchdog,
    display: DisplayDriver<'static>,
    device_input: &'static DeviceInput,
    spawner: Spawner,
    aon_timer: embassy_rp::aon_timer::AonTimer<'static>,
}

impl Device for Rp235Device {
    async fn read_persistent_state(&mut self) -> Result<PersistentState, DeviceError> {
        self.flash.try_read_persistent_state().await
    }

    async fn write_persistent_state(
        &mut self,
        persistent_state: &PersistentState,
    ) -> Result<(), DeviceError> {
        self.flash.write_persistent_state(persistent_state).await
    }

    async fn reset(&mut self) -> Result<(), DeviceError> {
        Ok(self.watchdog.trigger_reset())
    }

    async fn init_network_stack(
        &mut self,
        network_config: &NetworkConfig,
    ) -> Result<(), DeviceError> {
        let seed = self.rand().await;
        self.network_stack
            .init_network_stack(seed, network_config)
            .await
    }

    async fn join_wifi(&self, wifi_join_options: &WifiJoinOptions) -> Result<(), DeviceError> {
        self.network_stack.join(wifi_join_options).await
    }

    async fn leave_wifi(&self) -> Result<(), DeviceError> {
        self.network_stack.leave().await
    }

    async fn start_wifi_ap(
        &self,
        wifi_access_point_options: &WifiAccessPointOptions,
    ) -> Result<(), DeviceError> {
        self.network_stack
            .start_ap(&self.spawner, wifi_access_point_options)
            .await
    }

    async fn scan(&self) -> Result<Vec<WifiNetworkScanRecord>, DeviceError> {
        self.network_stack.scan().await
    }

    async fn network_stack(&self) -> Result<&embassy_net::Stack<'static>, DeviceError> {
        self.network_stack.stack()
    }

    async fn rand(&mut self) -> u64 {
        self.trng.blocking_next_u64()
    }

    async fn display(
        &mut self,
    ) -> Result<&mut impl epd_frame_rs_2_common::display::epd_spectra_6::DisplayDriver, DeviceError>
    {
        Ok(&mut self.display)
    }

    fn input_receiver(&self) -> DeviceInputReceiver {
        self.device_input.receiver()
    }

    async fn power_off_for(&mut self, duration: Duration) -> Result<(), DeviceError> {
        info!("Powering off the board for {:?}", duration);

        if !duration.is_zero() {
            info!("Powering off the board for {} seconds", duration.as_secs());
            if self.aon_timer.is_running() {
                self.aon_timer.stop();
            }
            self.aon_timer.set_counter(0);
            self.aon_timer.start();
            self.aon_timer
                .set_alarm_after(embassy_time::Duration::from_secs(duration.as_secs()))
                .map_err(|e| DeviceError::TimerError(LimitedString::from_debug_truncate(e)))?;
        } else {
            warn!("Power off duration is zero, sleeping forever");
        }
        defmt::flush();

        low_power_mode();
        //embassy_rp::clocks::dormant_sleep();
        self.aon_timer.wait_for_alarm().await;
        //self.reset().await?;

        Ok(())
    }

    async fn power_off_radio_module(&mut self) -> Result<(), DeviceError> {
        self.network_stack.deinitialize_network_stack();
        unsafe {
            let pwr = embassy_rp::peripherals::PIN_23::steal();
            gpio::Output::new(pwr, gpio::Level::Low);
        }
        Ok(())
    }

    async fn read_last_run_statistics(&mut self) -> Option<LastRunStatistics> {
        self.flash.read_last_run_statistics().await
    }

    async fn write_last_run_statistics(&mut self, last_run_statistics: &LastRunStatistics) {
        self.flash
            .write_last_run_statistics(last_run_statistics)
            .await;
    }
}

impl Rp235Device {
    pub async fn new(spawner: Spawner) -> Result<Self, DeviceError> {
        let peripherals = embassy_rp::init(Default::default());
        let watchdog = Watchdog::new(peripherals.WATCHDOG);

        info!(
            "Config start: {}, length: {}",
            unsafe { &__config_start as *const u32 as u32 },
            unsafe { &__config_length as *const u32 as u32 }
        );

        let trng = Trng::new(peripherals.TRNG, Irqs, embassy_rp::trng::Config::default());
        let config_offset =
            unsafe { &__config_start as *const u32 as u32 - embassy_rp::flash::FLASH_BASE as u32 };
        let flash = flash::Flash::new(
            peripherals.FLASH,
            peripherals.DMA_CH1,
            Irqs,
            config_offset,
            config_offset + LAST_RUN_STATISTICS_OFFSET,
        );
        let network_stack = wifi::WifiStack::new(
            Irqs,
            peripherals.PIN_25,
            peripherals.PIO0,
            peripherals.PIN_24,
            peripherals.PIN_29,
            peripherals.DMA_CH0,
            peripherals.PIN_23,
            spawner,
        )
        .await?;

        let display_spi = embassy_rp::spi::Spi::new(
            peripherals.SPI1,
            peripherals.PIN_10,
            peripherals.PIN_11,
            peripherals.PIN_12,
            peripherals.DMA_CH2,
            peripherals.DMA_CH3,
            Irqs,
            embassy_rp::spi::Config::default(),
        );
        static DISPLAY_SPI_MUTEX: StaticCell<
            Mutex<CriticalSectionRawMutex, embassy_rp::spi::Spi<SPI1, Async>>,
        > = StaticCell::new();
        let display_spi = DISPLAY_SPI_MUTEX.init(Mutex::new(display_spi));
        let display_cs = gpio::Output::new(peripherals.PIN_13, gpio::Level::High);
        let display_dc = gpio::Output::new(peripherals.PIN_6, gpio::Level::High);
        let display_rst = gpio::Output::new(peripherals.PIN_7, gpio::Level::High);
        let display_busy = gpio::Input::new(peripherals.PIN_8, gpio::Pull::Up);
        let spi_device = SpiDevice::new(display_spi, display_cs);
        let display = DisplayDriver::new(spi_device, display_dc, display_rst, display_busy, Delay);
        let button = gpio::Input::new(peripherals.PIN_18, gpio::Pull::Up);
        let led = gpio::Output::new(peripherals.PIN_9, gpio::Level::High);
        let device_input = make_static!(DeviceInput::new());
        let aon_timer = embassy_rp::aon_timer::AonTimer::new(
            peripherals.POWMAN,
            Irqs,
            embassy_rp::aon_timer::Config {
                clock_source: embassy_rp::aon_timer::ClockSource::Lposc,
                clock_freq_khz: 32,
                alarm_wake_mode: embassy_rp::aon_timer::AlarmWakeMode::DormantOnly,
            },
        );

        button::spawn_button_task(button, device_input.sender(), &spawner)?;
        spawner.spawn(blink_led(led)?);

        Ok(Self {
            flash,
            network_stack,
            trng,
            watchdog,
            display,
            device_input,
            spawner,
            aon_timer,
        })
    }
}

fn low_power_mode() {
    for boot in 0..4 {
        pac::POWMAN.boot(boot).write_value(POWMAN_PASSWORD);
    }

    pac::POWMAN.dbg_pwrcfg().modify(|r| {
        r.set_ignore(true);
        r.0 |= POWMAN_PASSWORD;
    });

    pac::POWMAN.seq_cfg().modify(|r| {
        r.set_use_fast_powck(true);
        r.set_run_lposc_in_lp(true);
        r.set_hw_pwrup_sram0(false);
        r.set_hw_pwrup_sram1(false);
        r.0 |= POWMAN_PASSWORD;
    });

    pac::CLOCKS.clk_adc_ctrl().modify(|r| r.set_enabled(false));
    pac::CLOCKS.clk_usb_ctrl().modify(|r| r.set_enabled(false));
    pac::CLOCKS.clk_peri_ctrl().modify(|r| r.set_enabled(false));

    pac::POWMAN.state().modify(|r| {
        r.set_req(0b00001111);
        r.0 = POWMAN_PASSWORD | (r.0 & 0xFFFF);
    });
}

unsafe extern "C" {
    // Flash storage used for configuration
    static __config_start: u32;
    static __config_length: u32;
}

bind_interrupts!(pub struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    TRNG_IRQ => embassy_rp::trng::InterruptHandler<TRNG>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, dma::InterruptHandler<DMA_CH1>, dma::InterruptHandler<DMA_CH2>, dma::InterruptHandler<DMA_CH3>;
    POWMAN_IRQ_TIMER => embassy_rp::aon_timer::InterruptHandler;
});

#[embassy_executor::task]
async fn blink_led(mut led: gpio::Output<'static>) {
    loop {
        led.set_low();
        Timer::after_millis(500).await;
        led.set_high();
        Timer::after_millis(500).await;
    }
}
