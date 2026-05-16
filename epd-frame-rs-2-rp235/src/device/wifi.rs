use aligned::{A4, Aligned};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::mem;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use cyw43_pio::{DEFAULT_CLOCK_DIVIDER, PioSpi};
use defmt::{Format, error};
use edge_nal::UdpBind;
use embassy_executor::Spawner;
use embassy_net::StaticConfigV4;
use embassy_rp::dma::ChannelInstance;
use embassy_rp::gpio::{Level, Output, Pin};
use embassy_rp::interrupt::typelevel::Binding;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{Instance, InterruptHandler, Pio, PioPin};
use embassy_rp::{Peri, dma};
use epd_frame_rs_2_common::errors::DeviceError;
use epd_frame_rs_2_common::types::{Ipv4Address, LimitedString};
use epd_frame_rs_2_common::wifi::{
    Auth, NetworkConfig, WifiAccessPointOptions, WifiJoinOptions, WifiNetworkScanRecord,
};
use static_cell::StaticCell;

const NET_STACK_RESOURCES_SIZE: usize = 8;
const DHCP_UDP_STACK_RESOURCES_SIZE: usize = 1024;
const CAPTIVE_PORTAL_UDP_STACK_RESOURCES_SIZE: usize = 1024;

type DhcpUdpBuffers = edge_nal_embassy::UdpBuffers<
    2,
    DHCP_UDP_STACK_RESOURCES_SIZE,
    DHCP_UDP_STACK_RESOURCES_SIZE,
    2,
>;

type CaptivePortalUdpBuffers = edge_nal_embassy::UdpBuffers<
    2,
    CAPTIVE_PORTAL_UDP_STACK_RESOURCES_SIZE,
    CAPTIVE_PORTAL_UDP_STACK_RESOURCES_SIZE,
    2,
>;

unsafe extern "C" {
    static __cyw43_firmware_start: u32;
    static __cyw43_firmware_length: u32;
    static __cyw43_clm_start: u32;
    static __cyw43_clm_length: u32;
}

pub struct WifiStack {
    state: State,
    spawner: Spawner,
}

#[derive(Debug, Format, Clone, Copy, Eq, PartialEq)]
pub enum WifiMode {
    Join(WifiJoinOptions),
    AccessPoint(WifiAccessPointOptions),
}

impl WifiStack {
    pub async fn new<'d, DMA, IRQS>(
        irqs: IRQS,
        cs: Peri<'d, impl Pin>,
        pio: Peri<'d, PIO0>,
        dio: Peri<'d, impl PioPin>,
        clk: Peri<'d, impl PioPin>,
        dma: Peri<'d, DMA>,
        pwr: Peri<'d, impl Pin>,
        spawner: Spawner,
    ) -> Result<Self, DeviceError>
    where
        DMA: ChannelInstance,
        IRQS: Binding<<PIO0 as Instance>::Interrupt, InterruptHandler<PIO0>>
            + Binding<<DMA as ChannelInstance>::Interrupt, dma::InterruptHandler<DMA>>
            + 'd,
        'd: 'static,
    {
        let cyw43_spi = init_cyw_43_pio_spi(irqs, cs, pio, dio, clk, dma);
        let pwr_output = Output::new(pwr, Level::High);
        let (wifi_device, wifi_control) = init_cyw43(pwr_output, cyw43_spi, &spawner).await?;
        let state = State::DeviceInitialized {
            wifi_device,
            wifi_control,
        };

        Ok(Self { state, spawner })
    }

    pub async fn init_network_stack(
        &mut self,
        seed: u64,
        network_config: &NetworkConfig,
    ) -> Result<(), DeviceError> {
        let state = mem::replace(&mut self.state, State::NotInitialized);

        match state {
            State::DeviceInitialized {
                wifi_device,
                wifi_control,
            } => {
                let wifi_control = Arc::new(RefCell::new(wifi_control));
                static RESOURCES: StaticCell<
                    embassy_net::StackResources<NET_STACK_RESOURCES_SIZE>,
                > = StaticCell::new();
                let net_config = network_configuration(network_config);
                let (wifi_stack, runner) = embassy_net::new(
                    wifi_device,
                    net_config,
                    RESOURCES.init(embassy_net::StackResources::new()),
                    seed,
                );
                self.spawner.spawn(net_stack_task(runner)?);

                self.state = State::NetworkStackInitialized {
                    wifi_control,
                    wifi_stack,
                    network_config: network_config.clone(),
                };
                Ok(())
            }

            _ => Err(DeviceError::NetworkStackNotInitialized),
        }
    }

    pub fn deinitialize_network_stack(&mut self) {
        self.state = State::NotInitialized;
    }

    pub async fn scan(&self) -> Result<Vec<WifiNetworkScanRecord>, DeviceError> {
        let control = self.control()?;
        let mut control = control.borrow_mut();
        let mut scanner = control.scan(cyw43::ScanOptions::default()).await;
        let mut result = Vec::new();

        while let Some(info) = scanner.next().await {
            let info = WifiNetworkScanRecord {
                ssid: LimitedString::from_bytes_truncate(&info.ssid[0..(info.ssid_len as usize)]),
            };
            result.push(info.into());
        }

        Ok(result)
    }

    pub async fn join(&self, wifi_join_options: &WifiJoinOptions) -> Result<(), DeviceError> {
        if let State::NetworkStackInitialized { wifi_control, .. } = &self.state {
            join(wifi_control.clone(), wifi_join_options).await?;
            Ok(self.stack()?.wait_config_up().await)
        } else {
            Err(DeviceError::NetworkStackNotInitialized)
        }
    }

    pub async fn leave(&self) -> Result<(), DeviceError> {
        if let State::NetworkStackInitialized { wifi_control, .. } = &self.state {
            wifi_control.borrow_mut().leave().await;
            Ok(())
        } else {
            Err(DeviceError::NetworkStackNotInitialized)
        }
    }

    pub async fn start_ap(
        &self,
        spawner: &Spawner,
        wifi_access_point_options: &WifiAccessPointOptions,
    ) -> Result<(), DeviceError> {
        if let State::NetworkStackInitialized {
            wifi_control,
            wifi_stack,
            network_config,
        } = &self.state
        {
            start_open_ap(wifi_control.clone(), wifi_access_point_options).await?;
            start_dhcp_server(
                wifi_stack.clone(),
                network_config.ipv4_address.address(),
                spawner,
            )
            .await
        } else {
            Err(DeviceError::NetworkStackNotInitialized)
        }
    }

    pub fn stack(&self) -> Result<&embassy_net::Stack<'static>, DeviceError> {
        match &self.state {
            State::NetworkStackInitialized { wifi_stack, .. } => Ok(wifi_stack),
            _ => Err(DeviceError::NetworkStackNotInitialized),
        }
    }

    pub fn control(&self) -> Result<Arc<RefCell<cyw43::Control<'static>>>, DeviceError> {
        match &self.state {
            State::NetworkStackInitialized { wifi_control, .. } => Ok(wifi_control.clone()),
            _ => Err(DeviceError::NetworkStackNotInitialized),
        }
    }
}

enum State {
    NotInitialized,
    DeviceInitialized {
        wifi_device: cyw43::NetDriver<'static>,
        wifi_control: cyw43::Control<'static>,
    },
    NetworkStackInitialized {
        wifi_stack: embassy_net::Stack<'static>,
        wifi_control: Arc<RefCell<cyw43::Control<'static>>>,
        network_config: NetworkConfig,
    },
}

async fn start_open_ap(
    wifi_control: Arc<RefCell<cyw43::Control<'static>>>,
    wifi_access_point_options: &WifiAccessPointOptions,
) -> Result<(), DeviceError> {
    let ssid = wifi_access_point_options.ssid.as_utf8_str()?;
    Ok(wifi_control
        .borrow_mut()
        .start_ap_open(ssid, wifi_access_point_options.channel)
        .await)
}

async fn join(
    wifi_control: Arc<RefCell<cyw43::Control<'static>>>,
    wifi_join_options: &WifiJoinOptions,
) -> Result<(), DeviceError> {
    let ssid = wifi_join_options.ssid.as_utf8_str()?;
    let mut cyw43_join_options = cyw43::JoinOptions::default();
    let auth = match wifi_join_options.auth {
        Auth::Open => cyw43::JoinAuth::Open,
        Auth::Wpa => cyw43::JoinAuth::Wpa,
        Auth::Wpa2 => cyw43::JoinAuth::Wpa2,
        Auth::Wpa3 => cyw43::JoinAuth::Wpa3,
        Auth::Wpa2Wpa3 => cyw43::JoinAuth::Wpa2Wpa3,
    };
    let passphrase = wifi_join_options.passphrase.as_slice();

    cyw43_join_options.auth = auth;
    cyw43_join_options.passphrase = passphrase;

    wifi_control
        .borrow_mut()
        .join(ssid, cyw43_join_options)
        .await
        .map_err(|_| DeviceError::UnableToJoinWifiNetwork)
}

fn network_configuration(network_config: &NetworkConfig) -> embassy_net::Config {
    if network_config.dhcp.as_bool() {
        embassy_net::Config::dhcpv4(Default::default())
    } else {
        let ipv4_address = network_config.ipv4_address;
        embassy_net::Config::ipv4_static(StaticConfigV4 {
            address: ipv4_address.into(),
            gateway: Some(ipv4_address.address().into()),
            dns_servers: Default::default(),
        })
    }
}

fn init_cyw_43_pio_spi<'d, PIO, DMA, IRQS>(
    irqs: IRQS,
    cs: Peri<'d, impl Pin>,
    pio: Peri<'d, PIO>,
    dio: Peri<'d, impl PioPin>,
    clk: Peri<'d, impl PioPin>,
    dma: Peri<'d, DMA>,
) -> PioSpi<'d, PIO, 0>
where
    PIO: Instance,
    DMA: ChannelInstance,
    IRQS: Binding<PIO::Interrupt, InterruptHandler<PIO>>
        + Binding<<DMA as ChannelInstance>::Interrupt, dma::InterruptHandler<DMA>>
        + 'd,
{
    let mut pio_instance = Pio::new(pio, irqs);
    PioSpi::new(
        &mut pio_instance.common,
        pio_instance.sm0,
        DEFAULT_CLOCK_DIVIDER,
        pio_instance.irq0,
        Output::new(cs, Level::High),
        dio,
        clk,
        dma::Channel::new(dma, irqs),
    )
}

async fn init_cyw43<'a>(
    pwr: Output<'static>,
    spi: PioSpi<'static, PIO0, 0>,
    spawner: &Spawner,
) -> Result<(cyw43::NetDriver<'a>, cyw43::Control<'a>), DeviceError> {
    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (wifi_device, mut wifi_control, wifi_runner) = cyw43::new(
        state,
        pwr,
        spi,
        cyw43_firmware_content(),
        cyw43_nvram_content(),
    )
    .await;
    spawner.spawn(cyw43_task(wifi_runner)?);
    wifi_control.init(cyw43_clm_content()).await;
    wifi_control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;
    Ok((wifi_device, wifi_control))
}

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, cyw43::SpiBus<Output<'static>, PioSpi<'static, PIO0, 0>>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_stack_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}

fn bytes_aligned_to_4(start: usize, length: usize) -> &'static Aligned<A4, [u8]> {
    unsafe {
        let slice = core::slice::from_raw_parts(start as *const u8, length);
        mem::transmute(slice)
    }
}

fn cyw43_firmware_content() -> &'static Aligned<A4, [u8]> {
    let cyw43_firmware_start = unsafe { &__cyw43_firmware_start as *const u32 as usize };
    let cyw43_firmware_length = unsafe { &__cyw43_firmware_length as *const u32 as usize };
    bytes_aligned_to_4(cyw43_firmware_start, cyw43_firmware_length)
}

fn cyw43_clm_content() -> &'static Aligned<A4, [u8]> {
    let cyw43_clm_start = unsafe { &__cyw43_clm_start as *const u32 as usize };
    let cyw43_clm_length = unsafe { &__cyw43_clm_length as *const u32 as usize };
    bytes_aligned_to_4(cyw43_clm_start, cyw43_clm_length)
}

fn cyw43_nvram_content() -> &'static Aligned<A4, [u8]> {
    cyw43::aligned_bytes!("../../cyw43-firmware/cyw43439-firmware/nvram_rp2040.bin")
}

async fn start_dhcp_server(
    stack: embassy_net::Stack<'static>,
    address: Ipv4Address,
    spawner: &Spawner,
) -> Result<(), DeviceError> {
    spawner.spawn(dhcp_server_task(stack, address.into())?);
    spawner.spawn(captive_portal_task(stack, address.into())?);
    Ok(())
}

#[embassy_executor::task]
async fn dhcp_server_task(stack: embassy_net::Stack<'static>, address: Ipv4Addr) {
    static UDP_BUFFERS: StaticCell<DhcpUdpBuffers> = StaticCell::new();
    let udp = edge_nal_embassy::Udp::new(stack, UDP_BUFFERS.init(DhcpUdpBuffers::new()));
    let mut server: edge_dhcp::server::Server<_, 8> =
        edge_dhcp::server::Server::new_with_et(address);
    let mut gateway_buf = [address];
    let options = edge_dhcp::server::ServerOptions::new(address, Some(&mut gateway_buf));
    let mut buf = [0; DHCP_UDP_STACK_RESOURCES_SIZE];

    let mut socket = match udp
        .bind(SocketAddr::new(
            address.into(),
            edge_dhcp::io::DEFAULT_SERVER_PORT,
        ))
        .await
    {
        Ok(socket) => socket,
        Err(e) => {
            error!("DHCP server bind error: {:?}", e);
            return;
        }
    };

    if let Err(e) = edge_dhcp::io::server::run(&mut server, &options, &mut socket, &mut buf).await {
        error!("DHCP server run error: {:?}", e);
    }
}

#[embassy_executor::task]
pub async fn captive_portal_task(wifi_stack: embassy_net::Stack<'static>, address: Ipv4Addr) {
    static UDP_BUFFERS: StaticCell<CaptivePortalUdpBuffers> = StaticCell::new();
    let udp_stack = edge_nal_embassy::Udp::new(
        wifi_stack,
        UDP_BUFFERS.init(edge_nal_embassy::UdpBuffers::new()),
    );
    let mut tx_buf = vec![0; CAPTIVE_PORTAL_UDP_STACK_RESOURCES_SIZE];
    let mut rx_buf = vec![0; CAPTIVE_PORTAL_UDP_STACK_RESOURCES_SIZE];

    let result = edge_captive::io::run(
        &udp_stack,
        SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 80),
        &mut tx_buf,
        &mut rx_buf,
        address,
        core::time::Duration::from_secs(60),
    )
    .await;

    error!("Captive portal task error: {:?}", result);
}
