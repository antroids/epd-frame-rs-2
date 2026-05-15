use crate::errors::DeviceError;
use crate::types;
use chrono::{DateTime, TimeZone, Utc};
use core::net::{IpAddr, SocketAddr};
use defmt::{Format, error};
use embassy_net::dns::DnsQueryType;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use serde::Deserialize;
use sntpc::{NtpContext, NtpTimestampGenerator, get_time};
use sntpc_net_embassy::UdpSocketWrapper;
use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes};

const NTP_LOCAL_SOCKET: u16 = 123;

#[derive(
    Copy,
    Clone,
    IntoBytes,
    TryFromBytes,
    Immutable,
    KnownLayout,
    Debug,
    Format,
    Eq,
    PartialEq,
    Deserialize,
)]
#[repr(C)]
pub struct NtpConfig {
    pub ntp_server: types::LimitedString,
}

impl Default for NtpConfig {
    fn default() -> Self {
        Self {
            ntp_server: types::LimitedString::from_str("pool.ntp.org"),
        }
    }
}

#[derive(Copy, Clone, Default)]
struct TimestampGen {
    duration: u64,
}

impl NtpTimestampGenerator for TimestampGen {
    fn init(&mut self) {
        self.duration = 0u64;
    }

    fn timestamp_sec(&self) -> u64 {
        self.duration >> 32
    }

    fn timestamp_subsec_micros(&self) -> u32 {
        (self.duration & 0xff_ff_ff_ffu64) as u32
    }
}

pub async fn ntp_get_time_utc(
    stack: embassy_net::Stack<'_>,
    ntp_server: &str,
) -> Result<DateTime<Utc>, DeviceError> {
    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buffer = [0; 4096];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_buffer = [0; 4096];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );
    socket.bind(NTP_LOCAL_SOCKET).unwrap();
    let socket = UdpSocketWrapper::new(socket);

    let context = NtpContext::new(TimestampGen::default());

    let ntp_addrs = stack
        .dns_query(ntp_server, DnsQueryType::A)
        .await
        .map_err(|e| {
            error!("DNS query error: {:?}", e);
            DeviceError::DnsQueryError
        })?;
    if ntp_addrs.is_empty() {
        error!("Failed to resolve DNS");
        return Err(DeviceError::DnsQueryError);
    }

    let addr: IpAddr = ntp_addrs[0].into();
    let result = get_time(SocketAddr::from((addr, NTP_LOCAL_SOCKET)), &socket, context).await;

    result
        .map(|time| Utc.timestamp_opt(time.seconds as i64, 0).unwrap())
        .map_err(|e| {
            error!("Error getting time: {:?}", e);
            DeviceError::InvalidTimezone
        })
}
