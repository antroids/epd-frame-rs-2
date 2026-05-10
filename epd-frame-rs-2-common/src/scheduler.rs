use chrono::{DateTime, Datelike, FixedOffset, Timelike};
use defmt::Format;
use embassy_time::Duration;
use serde::Deserialize;
use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes};

#[derive(
    Debug,
    Format,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    TryFromBytes,
    IntoBytes,
    KnownLayout,
    Immutable,
    Deserialize,
)]
pub struct TaskFlags(pub u8);

impl TaskFlags {
    pub const NONE: Self = Self(0);
    pub const UPDATE_WEATHER: Self = Self(0b00000001);

    pub const fn contains(self, flag: TaskFlags) -> bool {
        self.0 & flag.0 == flag.0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn union(self, other: TaskFlags) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn intersect(self, other: TaskFlags) -> Self {
        Self(self.0 & other.0)
    }

    pub const fn remove(self, other: TaskFlags) -> Self {
        Self(self.0 & !other.0)
    }
}

impl core::ops::BitOr for TaskFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitAnd for TaskFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl core::ops::Not for TaskFlags {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}

#[derive(
    Debug,
    Format,
    Clone,
    Copy,
    PartialEq,
    Eq,
    TryFromBytes,
    IntoBytes,
    KnownLayout,
    Immutable,
    Deserialize,
)]
pub struct HourlyScheduler {
    pub tasks: TaskFlags,
    pub minutes_delay: u8,
}

impl Default for HourlyScheduler {
    fn default() -> Self {
        Self {
            tasks: TaskFlags::UPDATE_WEATHER,
            minutes_delay: 15,
        }
    }
}

impl HourlyScheduler {
    pub const fn new(tasks: TaskFlags, minutes_delay: u8) -> Self {
        Self {
            tasks,
            minutes_delay,
        }
    }

    pub fn delay(&self) -> Duration {
        Duration::from_secs(self.minutes_delay as u64 * 60)
    }
}

#[derive(
    Debug,
    Format,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    TryFromBytes,
    IntoBytes,
    KnownLayout,
    Immutable,
    Deserialize,
)]
pub struct DailyScheduler {
    pub hourly: [HourlyScheduler; 24],
}

impl DailyScheduler {
    pub const fn new(hourly: [HourlyScheduler; 24]) -> Self {
        Self { hourly }
    }

    pub fn scheduler_for_hour(&self, hour: u8) -> &HourlyScheduler {
        &self.hourly[hour as usize % 24]
    }
}

#[derive(
    Debug,
    Format,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    TryFromBytes,
    IntoBytes,
    KnownLayout,
    Immutable,
    Deserialize,
)]
pub struct WeeklyScheduler {
    pub daily: [DailyScheduler; 7],
}

impl WeeklyScheduler {
    pub const fn new(daily: [DailyScheduler; 7]) -> Self {
        Self { daily }
    }

    pub fn scheduler_for_weekday(&self, weekday_iso: u32) -> &DailyScheduler {
        &self.daily[(weekday_iso.saturating_sub(1) % 7) as usize]
    }

    pub fn task_scheduler(&self, now: DateTime<FixedOffset>) -> HourlyScheduler {
        let weekday_iso = now.weekday().number_from_monday(); // 1–7
        let hour = now.hour() as u8;

        self.scheduler_for_weekday(weekday_iso)
            .scheduler_for_hour(hour)
            .clone()
    }
}
