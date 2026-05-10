use crate::display::driver::nibbles::{Nibble, Nibbles};
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::time::Duration;
use defmt::{info, Format};
use embedded_graphics::Pixel;
use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::{BinaryColor, PixelColor, Rgb888};
use embedded_graphics::prelude::{Dimensions, DrawTarget, OriginDimensions, RgbColor};
use embedded_graphics::primitives::Rectangle;
use embedded_hal::digital::{OutputPin, PinState};
use embedded_hal::spi::Operation;
use embedded_hal::{digital, spi};
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::SpiDevice;

pub mod nibbles;

const RESET_DELAY_MS: u32 = 30;
const BUSY_WAIT_DELAY_MS: u32 = 100;
const BUSY_WAIT_TIMEOUT_MS: Duration = Duration::from_millis(20_000);
const INIT_SEQUENCE: &[(CommandCode, &[u8])] = &[
    (CommandCode::INIT, &[0x49, 0x55, 0x20, 0x08, 0x09, 0x18]),
    (CommandCode::PWR, &[0x3F]),
    (CommandCode::PSR, &[0x5F, 0x69]),
    (CommandCode::BTST1, &[0x40, 0x1F, 0x1F, 0x2C]),
    (CommandCode::BTST3, &[0x6F, 0x1F, 0x1F, 0x22]),
    (CommandCode::BTST2, &[0x6F, 0x1F, 0x17, 0x17]),
    (CommandCode::POFS, &[0x00, 0x54, 0x00, 0x44]),
    (CommandCode::TCON, &[0x02, 0x00]),
    (CommandCode::PLL, &[0x08]),
    (CommandCode::CDI, &[0x3F]),
    (CommandCode::TRES, &[0x03, 0x20, 0x01, 0xE0]),
    (CommandCode::PWS, &[0x2F]),
    (CommandCode::VDCS, &[0x01]),
];

pub trait DisplayDriver {
    fn refresh(&mut self, frame_buffer: &[u8]) -> impl Future<Output = Result<(), Error>>;
}

#[derive(Debug)]
pub enum Error {
    SpiError(spi::ErrorKind),
    DigitalPinError(digital::ErrorKind),
}

impl Error {
    pub fn from_spi_error<ERR: spi::Error>(err: ERR) -> Self {
        Self::SpiError(err.kind())
    }

    pub fn from_digital_pin_error<ERR: digital::Error>(err: ERR) -> Self {
        Self::DigitalPinError(err.kind())
    }
}

pub type DisplayRgbColor = (u8, u8, u8);

const E6_PALETTE: [DisplayRgbColor; 6] = {
    [
        (0, 0, 0),
        (255, 255, 255),
        (255, 255, 0),
        (255, 0, 0),
        (0, 0, 255),
        (0, 255, 0),
    ]
};

#[derive(Format, Copy, Clone, PartialOrd, PartialEq, Debug)]
#[repr(u8)]
pub enum E6Color {
    Black = 0,
    White = 1,
    Yellow = 2,
    Red = 3,
    Blue = 5,
    Green = 6,
}

// Embedded Graphics Impl
impl PixelColor for E6Color {
    type Raw = ();
}

impl From<u8> for E6Color {
    fn from(value: u8) -> Self {
        match value {
            0 => E6Color::Black,
            1 => E6Color::White,
            2 => E6Color::Yellow,
            3 => E6Color::Red,
            5 => E6Color::Blue,
            6 => E6Color::Green,
            _ => panic!("Unknown E6 color index {}", value),
        }
    }
}

impl From<E6Color> for u8 {
    fn from(value: E6Color) -> Self {
        value as u8
    }
}

impl From<E6Color> for Rgb888 {
    fn from(value: E6Color) -> Self {
        let triplet = E6_PALETTE[value as usize];
        Self::new(triplet.0, triplet.1, triplet.2)
    }
}

impl From<Rgb888> for E6Color {
    fn from(value: Rgb888) -> Self {
        let color: DisplayRgbColor = (value.r(), value.g(), value.b()).into();
        for (index, c) in E6_PALETTE.iter().enumerate() {
            if color == *c {
                return E6Color::from(index as u8);
            }
        }
        panic!("Invalid E6Color: {:?}", color);
    }
}

impl From<BinaryColor> for E6Color {
    fn from(value: BinaryColor) -> Self {
        match value {
            BinaryColor::Off => E6Color::White,
            BinaryColor::On => E6Color::Black,
        }
    }
}

pub struct AsyncE6Display<DC: OutputPin, RST: OutputPin, BUSY: Wait, SPI: SpiDevice, DELAY: DelayNs>
{
    spi: SPI,
    dc_pin: DC,
    rst_pin: RST,
    busy_pin: BUSY,
    delay_source: DELAY,
}

#[allow(dead_code)]
impl<DC: OutputPin, RST: OutputPin, BUSY: Wait, SPI: SpiDevice, DELAY: DelayNs>
    AsyncE6Display<DC, RST, BUSY, SPI, DELAY>
{
    pub fn new(spi: SPI, dc_pin: DC, rst_pin: RST, busy_pin: BUSY, delay_source: DELAY) -> Self {
        Self {
            spi,
            dc_pin,
            rst_pin,
            busy_pin,
            delay_source,
        }
    }

    async fn power_off(&mut self) -> Result<(), Error> {
        self.spi_write_command(CommandCode::POF).await
    }

    async fn power_on(&mut self) -> Result<(), Error> {
        self.spi_write_command(CommandCode::PON).await
    }

    async fn display_refresh(&mut self) -> Result<(), Error> {
        self.spi_write_command_and_data(CommandCode::DRF, &[0x00])
            .await
    }

    fn set_data_command(&mut self, data_command: DataCommand) -> Result<(), Error> {
        set_data_command(&mut self.dc_pin, data_command)
    }

    async fn spi_write_command(&mut self, command: CommandCode) -> Result<(), Error> {
        self.set_data_command(DataCommand::Command)?;
        self.spi
            .write(&[command as u8])
            .await
            .map_err(Error::from_spi_error)
    }

    async fn spi_write_data(&mut self, data: &[u8]) -> Result<(), Error> {
        info!("Sending data chunk: {}", data.len());
        self.set_data_command(DataCommand::Data)?;
        self.spi.write(&data).await.map_err(Error::from_spi_error)?;
        Ok(())
    }

    async fn spi_write_frame_buffer(&mut self, frame_buffer: &[u8]) -> Result<(), Error> {
        self.set_data_command(DataCommand::Data)?;
        info!("Sending data chunk: {}", frame_buffer.len());
        self.spi
            .write(frame_buffer)
            .await
            .map_err(Error::from_spi_error)?;

        Ok(())
    }

    async fn spi_write_command_and_read<const R: usize>(
        &mut self,
        command: CommandCode,
    ) -> Result<[u8; R], Error> {
        let mut result = [0u8; R];
        self.set_data_command(DataCommand::Command)?;
        self.spi
            .transfer(&mut result, &[command as u8])
            .await
            .map_err(Error::from_spi_error)?;
        Ok(result)
    }

    async fn spi_write_command_and_data(
        &mut self,
        command: CommandCode,
        data: &[u8],
    ) -> Result<(), Error> {
        self.spi_write_command(command).await?;
        self.spi_write_data(data).await?;
        Ok(())
    }

    async fn reset(&mut self) -> Result<(), Error> {
        self.rst_pin
            .set_low()
            .map_err(Error::from_digital_pin_error)?;
        self.delay_source.delay_ms(RESET_DELAY_MS).await;
        self.rst_pin
            .set_high()
            .map_err(Error::from_digital_pin_error)?;
        self.delay_source.delay_ms(RESET_DELAY_MS).await;
        self.busy_wait().await?;
        Ok(())
    }

    async fn busy_wait(&mut self) -> Result<(), Error> {
        info!("The display could be busy, waiting...");
        self.busy_pin
            .wait_for_high()
            .await
            .map_err(Error::from_digital_pin_error)?;
        info!("The display is free, continue...");
        Ok(())
    }

    async fn refresh_display(&mut self) -> Result<(), Error> {
        self.power_on().await?;
        self.busy_wait().await?;
        //self.spi_write_command_and_data(CommandCode::BTST2, &[0x6F, 0x1F, 0x17, 0x49])?;
        self.display_refresh().await?;
        self.busy_wait().await?;
        self.power_off().await?;
        self.busy_wait().await?;
        Ok(())
    }

    async fn send_frame_buffer(&mut self, frame_buffer: &[u8]) -> Result<(), Error> {
        self.spi_write_command(CommandCode::DTM1).await?;
        self.spi_write_frame_buffer(frame_buffer).await?;
        let result: [u8; 1] = self.spi_write_command_and_read(CommandCode::DSP).await?;
        info!("Frame buffer sent, result: {}", result);
        self.busy_wait().await?;
        Ok(())
    }

    async fn initialize(&mut self) -> Result<(), Error> {
        info!("Initialize display");
        self.reset().await?;
        for (command_code, data) in INIT_SEQUENCE {
            self.spi_write_command_and_data(*command_code, data).await?;
        }
        Ok(())
    }
}

impl<DC: OutputPin, RST: OutputPin, BUSY: Wait, SPI: SpiDevice, DELAY: DelayNs> DisplayDriver
    for AsyncE6Display<DC, RST, BUSY, SPI, DELAY>
{
    async fn refresh(&mut self, frame_buffer: &[u8]) -> Result<(), Error> {
        self.initialize().await?;
        self.send_frame_buffer(frame_buffer).await?;
        self.refresh_display().await
    }
}
fn set_data_command(dc_pin: &mut impl OutputPin, data_command: DataCommand) -> Result<(), Error> {
    dc_pin
        .set_state(if let DataCommand::Data = data_command {
            PinState::High
        } else {
            PinState::Low
        })
        .map_err(Error::from_digital_pin_error)
}

#[repr(u8)]
#[derive(Format, Copy, Clone)]
#[allow(dead_code)]
pub(crate) enum CommandCode {
    PSR = 0x00,
    PWR = 0x01,
    POF = 0x02,
    POFS = 0x03,
    PON = 0x04,
    BTST1 = 0x05,
    BTST2 = 0x06,
    DSLP = 0x07,
    BTST3 = 0x08,
    DTM1 = 0x10,
    DSP = 0x11,
    DRF = 0x12,
    PLL = 0x30,
    CDI = 0x50,
    TCON = 0x60,
    TRES = 0x61,
    REV = 0x70,
    VDCS = 0x82,
    PTL = 0x83,
    PWS = 0xE3,
    INIT = 0xAA,
}

pub(crate) enum DataCommand {
    Data,
    Command,
}

pub struct FrameBuffer<S: AsMut<[u8]> + AsRef<[u8]>>(Size, Nibbles<S, E6Color>);

impl FrameBuffer<Vec<u8>> {
    pub fn new(size: Size, color: E6Color) -> Self {
        Self(
            size,
            Nibbles::allocate(color.into(), size.width as usize * size.height as usize),
        )
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.1.as_underlying_data().as_slice()
    }
}

impl<S: AsMut<[u8]> + AsRef<[u8]>> OriginDimensions for FrameBuffer<S> {
    fn size(&self) -> Size {
        self.0
    }
}

impl<S: AsMut<[u8]> + AsRef<[u8]>> DrawTarget for FrameBuffer<S> {
    type Color = E6Color;
    type Error = Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let width = self.0.width as usize;
        for Pixel(p, c) in pixels.into_iter().take(self.1.len()) {
            let pixel_index = p.y as usize * width + p.x as usize;
            self.1.set(pixel_index, c);
        }
        Ok(())
    }
}
