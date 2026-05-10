#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]

extern crate alloc;
pub mod device;

use defmt::{info, Debug2Format};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_alloc::LlffHeap;
use epd_frame_rs_2_common::device::Device;

use defmt_rtt as _;
use panic_probe as _;

#[global_allocator]
static HEAP: LlffHeap = LlffHeap::empty();
const HEAP_SIZE: usize = 1024 * 224;

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    // Initialize the allocator BEFORE you use it
    {
        use core::mem::MaybeUninit;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(&raw mut HEAP_MEM as usize, HEAP_SIZE) }
    }

    let mut device = device::Rp235Device::new(spawner).await.unwrap();

    spawner.spawn(resources_reporter_task().unwrap());
    device.run(spawner.clone()).await;

    info!("Hello world!");
    let delay = Duration::from_millis(2000);
    loop {
        info!("led on!");
        //wifi_stack.wifi_control_mut().gpio_set(0, true).await;
        Timer::after(delay).await;

        info!("led off!");
        //wifi_stack.wifi_control_mut().gpio_set(0, false).await;
        Timer::after(delay).await;
    }
}

// Program metadata for `picotool info`.
// This isn't needed, but it's recommended to have these minimal entries.
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"EPD Frame RS"),
    embassy_rp::binary_info::rp_program_description!(c"EPD Frame RS"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

#[embassy_executor::task]
async fn resources_reporter_task() {
    loop {
        info!(
            "Resources report core {}: memory used {:?} / free {:?} \n{:?} \n{:?} \n{:?} \n{:?} \n{:?} \n{:?}",
            embassy_rp::multicore::current_core(),
            HEAP.used(),
            HEAP.free(),
            Debug2Format(&embassy_rp::pac::POWMAN.state().read()),
            Debug2Format(&embassy_rp::pac::POWMAN.current_pwrup_req().read()),
            Debug2Format(&embassy_rp::pac::POWMAN.last_swcore_pwrup().read()),
            Debug2Format(&embassy_rp::pac::POWMAN.badpasswd().read()),
            Debug2Format(&embassy_rp::pac::POWMAN.timer().read()),
            Debug2Format(&embassy_rp::pac::POWMAN.chip_reset().read()),
        );
        Timer::after_secs(5).await;
    }
}
