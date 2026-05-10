use crate::display::color::E6Color;
use crate::wifi::WifiAccessPointOptions;
use alloc::format;
use embedded_graphics::geometry::Size;
use embedded_graphics::prelude::{DrawTarget, Point};
use embedded_graphics::primitives::Rectangle;
use qrcodegen_no_heap::QrCodeEcc;

const QR_CODE_VERSION: qrcodegen_no_heap::Version = qrcodegen_no_heap::Version::new(10);
const QR_CODE_MODULE_SIZE: Size = Size::new(6, 6);

pub async fn draw_configuration_mode<D>(
    wifi_config: &WifiAccessPointOptions,
    draw_target: &mut D,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = E6Color>,
{
    let connection_string = format!("WIFI:T:nopass;S:{:?};;", wifi_config.ssid);
    let mut qr_code_temp_buf = [0u8; QR_CODE_VERSION.buffer_len()];
    let mut qr_code_out_buf = [0u8; QR_CODE_VERSION.buffer_len()];
    let qr_code = qrcodegen_no_heap::QrCode::encode_text(
        &connection_string,
        &mut qr_code_temp_buf,
        &mut qr_code_out_buf,
        QrCodeEcc::High,
        QR_CODE_VERSION,
        QR_CODE_VERSION,
        None,
        false,
    )
    .unwrap();
    let qr_code_width = qr_code.size() as u32 * QR_CODE_MODULE_SIZE.width;
    let qr_code_height = qr_code.size() as u32 * QR_CODE_MODULE_SIZE.height;
    let qr_code_left_padding = (draw_target.bounding_box().size.width - qr_code_width) / 2;
    let qr_code_top_padding = (draw_target.bounding_box().size.height - qr_code_height) / 2;

    for x in 0..qr_code.size() {
        for y in 0..qr_code.size() {
            let color = if qr_code.get_module(x, y) {
                E6Color::Black
            } else {
                E6Color::White
            };
            let area = Rectangle::new(
                Point::new(
                    (qr_code_left_padding + x as u32 * QR_CODE_MODULE_SIZE.width) as i32,
                    (qr_code_top_padding + y as u32 * QR_CODE_MODULE_SIZE.height) as i32,
                ),
                QR_CODE_MODULE_SIZE,
            );
            draw_target.fill_solid(&area, color)?;
        }
    }

    Ok(())
}
