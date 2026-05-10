#![no_std]
#![feature(impl_trait_in_assoc_type)]

pub mod storage;
pub mod types;
pub mod errors;
pub mod wifi;
pub mod device;
pub mod time;
pub mod display;
pub mod http;
pub mod providers;
pub mod scheduler;

extern crate alloc;

#[cfg(not(feature = "std"))]
pub type RawMutex = embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
#[cfg(feature = "std")]
pub type RawMutex = embassy_sync::blocking_mutex::raw::NoopRawMutex;