use embassy_rp::gpio::Input;
use embassy_time::{Duration, Instant, Timer};
use epd_frame_rs_2_common::device::{DeviceInputSender, Input as DeviceInputEvent, DOUBLE_CLICK_MS, LONG_PRESS_MS};

const DEBOUNCE_MS: u64 = 20;

struct Debouncer<'d> {
    pin: Input<'d>,
}

impl<'d> Debouncer<'d> {
    fn new(pin: Input<'d>) -> Self {
        Self { pin }
    }

    async fn debounce(&mut self) -> bool {
        loop {
            self.pin.wait_for_any_edge().await;
            Timer::after(Duration::from_millis(DEBOUNCE_MS)).await;
            return self.pin.is_high();
        }
    }
}

pub fn spawn_button_task(
    pin: Input<'static>,
    sender: DeviceInputSender,
    spawner: &embassy_executor::Spawner,
) -> Result<(), embassy_executor::SpawnError> {
    Ok(spawner.spawn(button_task(pin, sender)?))
}

/// Pin convention: button connects the pin to GND; the pin must be pulled up
/// externally or via `Input::new(pin, Pull::Up)`.
#[embassy_executor::task]
async fn button_task(pin: Input<'static>, channel: DeviceInputSender) {
    let mut btn = Debouncer::new(pin);

    loop {
        while btn.debounce().await {}
        let press_start = Instant::now();
        while !btn.debounce().await {}
        let press_duration = Instant::now() - press_start;

        if press_duration >= Duration::from_millis(LONG_PRESS_MS) {
            let _ = channel.try_send(DeviceInputEvent::Button1LongPress);
            continue;
        }

        let double_click_deadline = Instant::now() + Duration::from_millis(DOUBLE_CLICK_MS);

        let second_click = loop {
            let now = Instant::now();
            if now >= double_click_deadline {
                break false;
            }
            let remaining = double_click_deadline - now;

            match embassy_futures::select::select(btn.debounce(), Timer::after(remaining)).await {
                embassy_futures::select::Either::First(false) => {
                    while !btn.debounce().await {}
                    break true;
                }
                embassy_futures::select::Either::Second(_) => {
                    break false;
                }
                _ => {}
            }
        };

        let event = if second_click {
            DeviceInputEvent::Button1DoubleClick
        } else {
            DeviceInputEvent::Button1Click
        };

        let _ = channel.try_send(event);
    }
}
