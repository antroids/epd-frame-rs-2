use aligned::{A4, Aligned};
use alloc::string::ToString;
use alloc::vec;
use defmt::{error, info, trace};
use embassy_rp::flash::{Async, ERASE_SIZE};
use embassy_rp::peripherals::FLASH;
use embassy_rp::{Peri, dma, interrupt};
use embedded_storage_async::nor_flash::NorFlash;
use epd_frame_rs_2_common::errors::DeviceError;
use epd_frame_rs_2_common::types::LimitedString;
use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes};

const FLASH_SIZE: usize = 1024 * 1024 * 4;
const BLOCK_SIZE: usize = ERASE_SIZE;
type FlashDriver = embassy_rp::flash::Flash<'static, FLASH, Async, { FLASH_SIZE }>;
pub struct Flash {
    flash_driver: FlashDriver,
    persistent_state_offset: u32,
    last_run_statistics_offset: u32,
}

impl Flash {
    pub fn new<D: dma::ChannelInstance>(
        flash: Peri<'static, FLASH>,
        dma: Peri<'static, D>,
        irq: impl interrupt::typelevel::Binding<D::Interrupt, dma::InterruptHandler<D>> + 'static,
        persistent_state_offset: u32,
        last_run_statistics_offset: u32,
    ) -> Self {
        let flash_driver = FlashDriver::new(flash, dma, irq);

        info!(
            "Flash initialized: persistent state offset: {} and last run statistics offset: {}",
            persistent_state_offset, last_run_statistics_offset
        );

        Self {
            flash_driver,
            persistent_state_offset,
            last_run_statistics_offset,
        }
    }

    pub async fn write<T: IntoBytes + Immutable + KnownLayout>(
        &mut self,
        offset: u32,
        data: &T,
    ) -> Result<(), embassy_rp::flash::Error> {
        let size_of_result = size_of::<T>();
        let buffer_size = (size_of_result + BLOCK_SIZE - 1) / BLOCK_SIZE * BLOCK_SIZE;
        let mut buf: Aligned<A4, _> = Aligned(vec![0u8; buffer_size]);
        let bytes = data.as_bytes();
        trace!(
            "Flash write buffer (hex) with length {} starting at {:08x}: {:02x}",
            bytes.len(),
            offset,
            bytes
        );
        buf[..size_of_result].copy_from_slice(bytes);
        self.flash_driver
            .erase(offset, offset + buffer_size as u32)
            .await?;
        self.flash_driver.write(offset, &buf).await
    }

    pub async fn try_read<T: TryFromBytes + Immutable + KnownLayout>(
        &mut self,
        offset: u32,
    ) -> Result<T, DeviceError> {
        let size_of_result = size_of::<T>();
        let buffer_size = (size_of_result + BLOCK_SIZE - 1) / BLOCK_SIZE * BLOCK_SIZE;
        let mut buf: Aligned<A4, _> = Aligned(vec![0u8; buffer_size]);
        self.flash_driver
            .read(offset, &mut buf)
            .await
            .map_err(|e| {
                DeviceError::PersistentStateReadError(LimitedString::from_debug_truncate(e))
            })?;

        trace!(
            "Flash read buffer (hex) with length {} starting at {:08x}: {:02x}",
            buf.as_slice().len(),
            offset,
            buf.as_slice()[..size_of_result]
        );

        T::try_read_from_bytes(&buf[..size_of_result]).map_err(|e| {
            DeviceError::PersistentStateInvalid(LimitedString::from_str_truncate(
                e.to_string().as_str(),
            ))
        })
    }

    pub async fn write_persistent_state<T: IntoBytes + Immutable + KnownLayout>(
        &mut self,
        data: &T,
    ) -> Result<(), DeviceError> {
        self.write(self.persistent_state_offset, data)
            .await
            .map_err(|e| {
                DeviceError::PersistentStateWriteError(LimitedString::from_debug_truncate(e))
            })
    }

    pub async fn try_read_persistent_state<T: TryFromBytes + Immutable + KnownLayout>(
        &mut self,
    ) -> Result<T, DeviceError> {
        self.try_read(self.persistent_state_offset).await
    }

    pub async fn write_last_run_statistics<T: IntoBytes + Immutable + KnownLayout>(
        &mut self,
        data: &T,
    ) {
        let _ = self
            .write(self.last_run_statistics_offset, data)
            .await
            .inspect_err(|e| {
                error!("Failed to write last run statistics: {}", e);
            });
    }

    pub async fn read_last_run_statistics<T: TryFromBytes + Immutable + KnownLayout>(
        &mut self,
    ) -> Option<T> {
        self.try_read(self.last_run_statistics_offset).await.ok()
    }
}
