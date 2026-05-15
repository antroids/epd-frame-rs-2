use core::fmt::Debug;
use core::net::Ipv4Addr;
use core::str::{Utf8Error, from_utf8};
use defmt::{Format, Formatter};
use serde::Serializer;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, NativeEndian, Usize};

pub const DEFAULT_LIMITED_STRING_MAX_LEN: usize = 32;

#[derive(Copy, Clone, FromBytes, IntoBytes, Immutable, KnownLayout, Eq, PartialEq)]
#[repr(C)]
pub struct LimitedString<const MAX_LEN: usize = DEFAULT_LIMITED_STRING_MAX_LEN> {
    str: [u8; MAX_LEN],
    len: Usize<NativeEndian>,
}

impl<const MAX_LEN: usize> Default for LimitedString<MAX_LEN> {
    fn default() -> Self {
        Self {
            len: 0.into(),
            str: [0; MAX_LEN],
        }
    }
}

impl<const MAX_LEN: usize> Format for LimitedString<MAX_LEN> {
    fn format(&self, fmt: Formatter) {
        defmt::write!(
            fmt,
            "{}",
            self.as_utf8_str()
                .unwrap_or_else(|_| "Invalid UTF8 sequence")
        )
    }
}

impl<const MAX_LENGTH: usize> Debug for LimitedString<MAX_LENGTH> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(
            self.as_utf8_str()
                .unwrap_or_else(|_| "Invalid UTF8 sequence"),
        )
    }
}

impl<const MAX_LEN: usize> LimitedString<MAX_LEN> {
    pub fn new(str: [u8; MAX_LEN], len: usize) -> Self {
        Self {
            str,
            len: len.into(),
        }
    }

    pub fn from_bytes(s: &[u8]) -> Self {
        assert!(s.len() <= MAX_LEN, "The string is too long");
        Self::from_bytes_truncate(s)
    }

    pub fn from_str(s: &str) -> Self {
        Self::from_bytes(s.as_bytes())
    }

    pub fn from_str_truncate(s: &str) -> Self {
        Self::from_bytes_truncate(s.as_bytes())
    }

    pub fn from_debug_truncate(d: impl Debug) -> Self {
        Self::from_str_truncate(alloc::format!("{:?}", d).as_str())
    }

    pub fn from_bytes_truncate(s: &[u8]) -> Self {
        let len = s.len().min(MAX_LEN);
        let mut str = [0u8; MAX_LEN];
        str[..len].copy_from_slice(&s[..len]);
        Self::new(str, len.into())
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.str[..self.len.into()]
    }

    pub fn as_utf8_str(&self) -> Result<&str, Utf8Error> {
        from_utf8(self.as_slice())
    }
}

impl<'de, const MAX_LEN: usize> serde::Deserialize<'de> for LimitedString<MAX_LEN> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;
        Ok(Self::from_str_truncate(s))
    }
}

impl<const MAX_LEN: usize> serde::Serialize for LimitedString<MAX_LEN> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_utf8_str().unwrap_or_default())
    }
}

#[derive(
    Default, Copy, Clone, PartialOrd, PartialEq, Eq, FromBytes, IntoBytes, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct ByteBool(u8);

impl ByteBool {
    pub fn as_bool(&self) -> bool {
        self.0 != 0
    }

    pub fn from_bool(v: bool) -> Self {
        Self(if v { 1 } else { 0 })
    }

    pub fn as_inner(&self) -> u8 {
        self.0
    }
}

impl From<bool> for ByteBool {
    fn from(value: bool) -> Self {
        Self::from_bool(value)
    }
}

impl From<ByteBool> for bool {
    fn from(value: ByteBool) -> Self {
        value.as_bool()
    }
}

impl Format for ByteBool {
    fn format(&self, fmt: Formatter) {
        defmt::write!(fmt, "{}", self.as_bool())
    }
}

impl Debug for ByteBool {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_bool())
    }
}

impl<'de> serde::Deserialize<'de> for ByteBool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let b = <bool>::deserialize(deserializer)?;
        Ok(Self::from_bool(b))
    }
}

#[derive(
    Default, Copy, Clone, PartialOrd, PartialEq, Eq, FromBytes, IntoBytes, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct Ipv4Address(u8, u8, u8, u8);

impl Ipv4Address {
    pub fn new(field0: u8, field1: u8, field2: u8, field3: u8) -> Self {
        Self(field0, field1, field2, field3)
    }

    pub fn parse(s: &str) -> Option<Self> {
        let mut octets = s.split('.');
        let a: u8 = octets.next()?.parse().ok()?;
        let b: u8 = octets.next()?.parse().ok()?;
        let c: u8 = octets.next()?.parse().ok()?;
        let d: u8 = octets.next()?.parse().ok()?;
        Some(Self::new(a, b, c, d))
    }
}

impl Format for Ipv4Address {
    fn format(&self, fmt: Formatter) {
        defmt::write!(fmt, "{}.{}.{}.{}", self.0, self.1, self.2, self.3)
    }
}

impl Debug for Ipv4Address {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0, self.1, self.2, self.3)
    }
}

impl From<Ipv4Addr> for Ipv4Address {
    fn from(value: Ipv4Addr) -> Self {
        let octets = value.octets();
        Self(octets[0], octets[1], octets[2], octets[3])
    }
}

impl From<Ipv4Address> for Ipv4Addr {
    fn from(value: Ipv4Address) -> Self {
        Self::new(value.0, value.1, value.2, value.3)
    }
}

impl<'de> serde::Deserialize<'de> for Ipv4Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;
        Self::parse(s).ok_or_else(|| serde::de::Error::custom("invalid IPv4 address"))
    }
}

#[derive(
    Default, Copy, Clone, PartialOrd, PartialEq, Eq, FromBytes, IntoBytes, Immutable, KnownLayout,
)]
#[repr(C)]
pub struct Ipv4CidrAddress(Ipv4Address, u8);

impl Ipv4CidrAddress {
    pub fn new(address: Ipv4Address, len: u8) -> Self {
        Self(address, len)
    }

    pub fn address(&self) -> Ipv4Address {
        self.0
    }

    pub fn prefix(&self) -> u8 {
        self.1
    }

    pub fn parse(s: &str) -> Option<Self> {
        let (addr, prefix) = s.split_once('/')?;
        let prefix: u8 = prefix.parse().ok()?;
        let mut octets = addr.split('.');
        let a: u8 = octets.next()?.parse().ok()?;
        let b: u8 = octets.next()?.parse().ok()?;
        let c: u8 = octets.next()?.parse().ok()?;
        let d: u8 = octets.next()?.parse().ok()?;
        Some(Ipv4CidrAddress::new(Ipv4Address::new(a, b, c, d), prefix))
    }
}

impl Format for Ipv4CidrAddress {
    fn format(&self, fmt: Formatter) {
        self.0.format(fmt);
        defmt::write!(fmt, "/{}", self.1)
    }
}

impl Debug for Ipv4CidrAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)?;
        f.write_str("/")?;
        f.write_fmt(format_args!("{}", self.1))
    }
}

impl From<Ipv4CidrAddress> for embassy_net::Ipv4Cidr {
    fn from(value: Ipv4CidrAddress) -> Self {
        embassy_net::Ipv4Cidr::new(value.0.into(), value.1)
    }
}

impl<'de> serde::Deserialize<'de> for Ipv4CidrAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;
        Self::parse(s).ok_or_else(|| serde::de::Error::custom("invalid IPv4 CIDR address"))
    }
}
