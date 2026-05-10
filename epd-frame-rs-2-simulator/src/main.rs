#![feature(type_alias_impl_trait)]

use defmt_or_log::info;
use crate::device::SimulatorDevice;
use embassy_executor::Executor;
use embassy_executor::Spawner;
use epd_frame_rs_2_common::device::Device;
use static_cell::StaticCell;

mod device;

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

fn main() {
    simple_log::console("info").unwrap();

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(main_task(spawner).unwrap());
    });
}

#[embassy_executor::task]
async fn main_task(spawner: Spawner) {
    let mut device = SimulatorDevice::new(spawner);
    loop {
        info!("Starting main loop...");
        device.run(spawner.clone()).await;
    }
}
