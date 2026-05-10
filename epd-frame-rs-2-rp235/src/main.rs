#![no_std]
#![no_main]
extern crate alloc;

use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::{PIO0, TRNG};
use embassy_rp::pio::{InterruptHandler};
use embassy_time::{Duration, Timer};
use defmt::{error, info, unwrap, warn};
use {defmt_rtt as _, panic_probe as _};
use embedded_alloc::LlffHeap;

bind_interrupts!(pub struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    TRNG_IRQ => embassy_rp::trng::InterruptHandler<TRNG>;
});

unsafe extern "C" {
    // Flash storage used for configuration
    static __config_start: u32;
    static __config_length: u32;
}

#[global_allocator]
static HEAP: LlffHeap = LlffHeap::empty();
const HEAP_SIZE: usize = 1024 * 32;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    // Initialize the allocator BEFORE you use it
    {
        use core::mem::MaybeUninit;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(&raw mut HEAP_MEM as usize, HEAP_SIZE) }
    }


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