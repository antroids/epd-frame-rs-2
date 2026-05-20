use crate::display::color::E6Color;
use crate::display::weather::frog::Frog130x180;
use crate::display::weather::{CurrentWeather, DailyWeather, HourlyWeather, Icon64};
use crate::display::widgets::weather::styles::{
    TEMPERATURE_FONT_12, TEMPERATURE_FONT_20, TEMPERATURE_FONT_50,
};
use crate::display::widgets::{Icon, RoundWidgetBorder, Text, Widget};
use crate::display::{BinaryFontStyleType, DEFAULT_FONT_10_STYLE, Icon16};
use alloc::format;
use alloc::string::ToString;
use core::f32::consts::PI;
use embedded_graphics::Drawable;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Primitive;
use embedded_graphics::primitives::{Line, PrimitiveStyle, Rectangle, StyledDrawable};
use embedded_layout::align::horizontal;
use embedded_layout::prelude::{Align, vertical};
use embedded_layout::{View, ViewGroup};
#[cfg(not(feature = "std"))]
use micromath::F32Ext;
use mplusfonts::style::BitmapFontStyle;

pub(super) mod styles {
    use crate::display;
    use crate::display::BinaryFontStyleType;
    use crate::display::color::E6Color;
    use crate::display::weather::TemperatureCelsium;
    use embedded_graphics::pixelcolor::BinaryColor;
    use mplusfonts::BitmapFont;
    use mplusfonts::style::BitmapFontStyle;
    use mplusfonts_macros::mplus;

    pub const TEMPERATURE_FONT_50: BitmapFont<BinaryColor, 2> =
        mplus!(1, MEDIUM, cap_height(50), true, 2, 1, '0'..='9', ["+-C°"]);
    pub const TEMPERATURE_FONT_20: BitmapFont<BinaryColor, 2> =
        mplus!(1, BOLD, cap_height(20), true, 2, 1, '0'..='9', ["+-C°"]);
    pub const TEMPERATURE_FONT_12: BitmapFont<BinaryColor, 2> =
        mplus!(1, BOLD, cap_height(12), true, 2, 1, '0'..='9', ["+-C°"]);

    pub fn date_time_style() -> BinaryFontStyleType {
        BitmapFontStyle::new(&display::DEFAULT_FONT_12, BinaryColor::On)
    }

    pub fn current_weather_details_style() -> BinaryFontStyleType {
        date_time_style()
    }

    pub fn temperature_color(temperature: TemperatureCelsium) -> E6Color {
        match temperature as i32 {
            ..1 => E6Color::Blue,
            18..26 => E6Color::Green,
            26.. => E6Color::Red,
            _ => E6Color::Black,
        }
    }
}

#[derive(Clone)]
pub struct WindArrow {
    position: Point,
    radius: f32,
    head_len: f32,
    head_angle: f32,
    direction: f32,
    stroke_width: u32,
}

impl WindArrow {
    pub fn new(direction: f32) -> Self {
        Self {
            position: Default::default(),
            radius: 7.0,
            head_len: 6.0,
            head_angle: 145.0,
            direction,
            stroke_width: 3,
        }
    }
}

impl View for WindArrow {
    fn translate_impl(&mut self, by: Point) {
        self.position += by;
    }

    fn bounds(&self) -> Rectangle {
        Rectangle::new(self.position, (16, 16).into())
    }
}

impl Drawable for WindArrow {
    type Color = E6Color;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let rad = self.direction * PI / 180.0;
        let sin_a = rad.sin();
        let cos_a = rad.cos();
        let radius = self.radius;
        let head_len = self.head_len;
        let head_angle = self.head_angle * PI / 180.0;
        let tip_x = self.position.x as f32 + radius * sin_a;
        let tip_y = self.position.y as f32 - radius * cos_a;
        let base_x = self.position.x as f32 - radius * sin_a;
        let base_y = self.position.y as f32 + radius * cos_a;
        let w1_rad = rad + head_angle;
        let w2_rad = rad - head_angle;
        let w1_x = tip_x + head_len * w1_rad.sin();
        let w1_y = tip_y - head_len * w1_rad.cos();
        let w2_x = tip_x + head_len * w2_rad.sin();
        let w2_y = tip_y - head_len * w2_rad.cos();

        let p_base = Point::new(base_x.round() as i32, base_y.round() as i32);
        let p_tip = Point::new(tip_x.round() as i32, tip_y.round() as i32);
        let p_w1 = Point::new(w1_x.round() as i32, w1_y.round() as i32);
        let p_w2 = Point::new(w2_x.round() as i32, w2_y.round() as i32);

        let style = PrimitiveStyle::with_stroke(E6Color::Black, self.stroke_width);

        Line::new(p_base, p_tip).into_styled(style).draw(target)?;
        Line::new(p_tip, p_w1).into_styled(style).draw(target)?;
        Line::new(p_tip, p_w2).into_styled(style).draw(target)?;

        Ok(())
    }
}

impl Widget for WindArrow {}

#[derive(Clone)]
pub struct WeatherIconBackground {
    position: Point,
    size: Size,
    rand: fastrand::Rng,
}

impl WeatherIconBackground {
    pub fn new(size: Size, rand: fastrand::Rng) -> Self {
        Self {
            position: Default::default(),
            size,
            rand,
        }
    }
}

impl View for WeatherIconBackground {
    fn translate_impl(&mut self, by: Point) {
        self.position += by;
    }

    fn bounds(&self) -> Rectangle {
        Rectangle::new(self.position, self.size)
    }
}

impl Drawable for WeatherIconBackground {
    type Color = E6Color;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let mut rand = self.rand.clone().fork();
        let height = self.size.height as i32;
        let shape_part = self.size.width as f32 / 6f32;
        let random_part = (self.size.width / 8) as u8;
        let start_x = self.position.x;
        let end_x = self.position.x + self.size.width as i32;
        let style = PrimitiveStyle::with_stroke(E6Color::Blue, 1);
        for y in (0..height).step_by(5) {
            let shape_diff = y.abs_diff(height / 2) as f32 / (height / 2) as f32;
            let y = self.position.y - y;
            let length = (shape_diff * shape_part) as i32 + rand.u8(0..random_part) as i32;
            Line::new((start_x, y).into(), (start_x + length, y).into())
                .draw_styled(&style, target)?;

            let length = (shape_diff * shape_part) as i32 + rand.u8(0..random_part) as i32;
            Line::new((end_x - length, y).into(), (end_x, y).into()).draw_styled(&style, target)?;
        }
        Ok(())
    }
}

impl Widget for WeatherIconBackground {}

#[derive(ViewGroup, Clone)]
pub struct IconValue16<'a> {
    icon: Icon<'a, Icon16>,
    text: Text<'a, BinaryFontStyleType>,
}

impl<'a> IconValue16<'a> {
    pub fn new(icon: &'a Icon16, text: &'a str, color: E6Color) -> Self {
        let icon = Icon::new(icon);
        let text = Text::new(text, DEFAULT_FONT_10_STYLE, color).translate((18, 14).into());
        Self { icon, text }
    }
}

impl<'a> Drawable for IconValue16<'a> {
    type Color = E6Color;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.icon.draw(target)?;
        self.text.draw(target)?;
        Ok(())
    }
}

impl<'a> Widget for IconValue16<'a> {}

#[derive(ViewGroup, Clone)]
pub struct HourlyWeatherWidget<'a> {
    frame: RoundWidgetBorder,
    date_time: Text<'a, BinaryFontStyleType>,
    weather_icon_background: WeatherIconBackground,
    weather_icon: Icon<'a, Icon64>,
    wind_arrow: WindArrow,
    temperature_small: Text<'a, BinaryFontStyleType>,
    apparent_temperature: IconValue16<'a>,
    wind_speed: IconValue16<'a>,
    precipitation_probability: IconValue16<'a>,
    uv_index: IconValue16<'a>,
    humidity: IconValue16<'a>,
}

impl<'a> HourlyWeatherWidget<'a> {
    pub fn new(hourly_weather: &'a HourlyWeather, rand: &'a mut fastrand::Rng) -> Self {
        let time = hourly_weather.time.format("%_H:%M").to_string();

        let frame = RoundWidgetBorder::new(Rectangle::new((0, 0).into(), (110, 219).into()));
        let mut date_time = Text::new(time.leak(), styles::date_time_style(), E6Color::Black);
        let mut weather_icon_background = WeatherIconBackground::new((102, 54).into(), rand.fork());
        let mut weather_icon = Icon::new(&hourly_weather.weather_icon);
        let mut wind_arrow = WindArrow::new(hourly_weather.wind_direction as f32);
        let mut temperature_small = Text::new(
            format!("{:+}C°", hourly_weather.temperature.round() as u32).leak(),
            BitmapFontStyle::new(&TEMPERATURE_FONT_20, BinaryColor::On),
            styles::temperature_color(hourly_weather.temperature),
        );
        let mut apparent_temperature = IconValue16::new(
            &Icon16::Temperature,
            format!(
                "{:+}/{:+}C°",
                hourly_weather.apparent_temperature.round() as u32,
                hourly_weather.apparent_temperature.round() as u32
            )
            .leak(),
            E6Color::Black,
        );
        let mut wind_speed = IconValue16::new(
            &Icon16::Wind,
            format!("{:}km/h", hourly_weather.wind_speed).leak(),
            E6Color::Black,
        );
        let mut precipitation_probability = IconValue16::new(
            &Icon16::Water,
            format!(
                "{:.1}mm {:}%",
                hourly_weather.precipitation, hourly_weather.precipitation_probability
            )
            .leak(),
            E6Color::Black,
        );
        let mut humidity = IconValue16::new(
            &Icon16::Humidity,
            format!("{}%", hourly_weather.humidity).leak(),
            E6Color::Black,
        );
        let mut uv_index = IconValue16::new(
            &Icon16::Uv,
            format!("{}", hourly_weather.uv_index).leak(),
            E6Color::Black,
        );

        date_time.translate_mut((0, 18).into()).align_to_mut(
            &frame,
            horizontal::Center,
            vertical::NoAlignment,
        );
        weather_icon_background.translate_mut((4, 80).into());
        weather_icon.translate_mut((23, 20).into());
        wind_arrow.translate_mut((90, 34).into());
        temperature_small
            .translate_mut((15, 114).into())
            .align_to_mut(&frame, horizontal::Center, vertical::NoAlignment);
        let details_offset = 127;
        apparent_temperature.translate_mut((4, details_offset).into());
        wind_speed.translate_mut((4, details_offset + 18).into());
        precipitation_probability.translate_mut((4, details_offset + 18 * 2).into());
        humidity.translate_mut((4, details_offset + 18 * 3).into());
        uv_index.translate_mut((4, details_offset + 18 * 4).into());

        Self {
            frame,
            date_time,
            weather_icon_background,
            weather_icon,
            wind_arrow,
            temperature_small,
            apparent_temperature,
            wind_speed,
            precipitation_probability,
            humidity,
            uv_index,
        }
    }
}

impl Drawable for HourlyWeatherWidget<'_> {
    type Color = E6Color;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.frame.draw(target)?;
        self.date_time.draw(target)?;
        self.weather_icon_background.draw(target)?;
        self.weather_icon.draw(target)?;
        self.wind_arrow.draw(target)?;
        self.temperature_small.draw(target)?;
        self.apparent_temperature.draw(target)?;
        self.wind_speed.draw(target)?;
        self.precipitation_probability.draw(target)?;
        self.humidity.draw(target)?;
        self.uv_index.draw(target)?;
        Ok(())
    }
}

#[derive(ViewGroup, Clone)]
pub struct DailyWeatherWidget<'a> {
    frame: RoundWidgetBorder,
    date: Text<'a, BinaryFontStyleType>,
    weather_icon_background: WeatherIconBackground,
    weather_icon: Icon<'a, Icon64>,
    wind_arrow: WindArrow,
    temperature_small: Text<'a, BinaryFontStyleType>,
    temperature_small_secondary: Text<'a, BinaryFontStyleType>,
    apparent_temperature: IconValue16<'a>,
    wind_speed: IconValue16<'a>,
    precipitation_probability: IconValue16<'a>,
    humidity: IconValue16<'a>,
    uv_index_max: IconValue16<'a>,
}

impl<'a> DailyWeatherWidget<'a> {
    pub fn new(daily_weather: &'a DailyWeather, rand: &'a mut fastrand::Rng) -> Self {
        let date = daily_weather.time.format("%A").to_string();

        let frame = RoundWidgetBorder::new(Rectangle::new((0, 0).into(), (110, 239).into()));
        let mut date = Text::new(date.leak(), styles::date_time_style(), E6Color::Black);
        let mut weather_icon_background = WeatherIconBackground::new((102, 54).into(), rand.fork());
        let mut weather_icon = Icon::new(&daily_weather.weather_icon);
        let mut wind_arrow = WindArrow::new(daily_weather.wind_direction as f32);
        let mut temperature_small = Text::new(
            format!("{:+}C°", daily_weather.temperature_max.round() as u32).leak(),
            BitmapFontStyle::new(&TEMPERATURE_FONT_20, BinaryColor::On),
            styles::temperature_color(daily_weather.temperature_max),
        );
        let mut temperature_small_secondary = Text::new(
            format!("{:+}C°", daily_weather.temperature_min.round() as u32).leak(),
            BitmapFontStyle::new(&TEMPERATURE_FONT_12, BinaryColor::On),
            styles::temperature_color(daily_weather.temperature_min),
        );
        let mut apparent_temperature = IconValue16::new(
            &Icon16::Temperature,
            format!(
                "{:+}/{:+}C°",
                daily_weather.apparent_temperature_max.round() as u32,
                daily_weather.apparent_temperature_min.round() as u32
            )
            .leak(),
            E6Color::Black,
        );
        let mut wind_speed = IconValue16::new(
            &Icon16::Wind,
            format!("{:}km/h", daily_weather.wind_speed).leak(),
            E6Color::Black,
        );
        let mut precipitation_probability = IconValue16::new(
            &Icon16::Water,
            format!(
                "{:.1}mm {:}%",
                daily_weather.precipitation, daily_weather.precipitation_probability
            )
            .leak(),
            E6Color::Black,
        );
        let mut humidity = IconValue16::new(
            &Icon16::Humidity,
            format!("{}%", daily_weather.humidity).leak(),
            E6Color::Black,
        );
        let mut uv_index_max = IconValue16::new(
            &Icon16::Uv,
            format!("{}", daily_weather.uv_index_max).leak(),
            E6Color::Black,
        );

        date.translate_mut((0, 18).into()).align_to_mut(
            &frame,
            horizontal::Center,
            vertical::NoAlignment,
        );
        weather_icon_background.translate_mut((4, 80).into());
        weather_icon.translate_mut((23, 20).into());
        wind_arrow.translate_mut((90, 34).into());
        temperature_small
            .translate_mut((15, 114).into())
            .align_to_mut(&frame, horizontal::Center, vertical::NoAlignment);
        temperature_small_secondary
            .translate_mut((35, 136).into())
            .align_to_mut(&frame, horizontal::Center, vertical::NoAlignment);
        let details_offset = 147;
        apparent_temperature.translate_mut((4, details_offset).into());
        wind_speed.translate_mut((4, details_offset + 18).into());
        precipitation_probability.translate_mut((4, details_offset + 18 * 2).into());
        humidity.translate_mut((4, details_offset + 18 * 3).into());
        uv_index_max.translate_mut((4, details_offset + 18 * 4).into());

        Self {
            frame,
            date,
            weather_icon_background,
            weather_icon,
            wind_arrow,
            temperature_small,
            temperature_small_secondary,
            apparent_temperature,
            wind_speed,
            precipitation_probability,
            humidity,
            uv_index_max,
        }
    }
}

impl Drawable for DailyWeatherWidget<'_> {
    type Color = E6Color;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.frame.draw(target)?;
        self.date.draw(target)?;
        self.weather_icon_background.draw(target)?;
        self.weather_icon.draw(target)?;
        self.wind_arrow.draw(target)?;
        self.temperature_small.draw(target)?;
        self.temperature_small_secondary.draw(target)?;
        self.apparent_temperature.draw(target)?;
        self.wind_speed.draw(target)?;
        self.precipitation_probability.draw(target)?;
        self.humidity.draw(target)?;
        self.uv_index_max.draw(target)?;

        Ok(())
    }
}

#[derive(ViewGroup, Clone)]
pub struct CurrentWeatherWidget<'a> {
    frog_icon: Icon<'a, Frog130x180>,
    date_time: Text<'a, BinaryFontStyleType>,
    temperature: Text<'a, BinaryFontStyleType>,
    apparent_temperature_label: Text<'a, BinaryFontStyleType>,
    apparent_temperature: Text<'a, BinaryFontStyleType>,
    weather_details: Text<'a, BinaryFontStyleType>,
}

impl<'a> CurrentWeatherWidget<'a> {
    pub fn new(current_weather: &CurrentWeather) -> Self {
        let date_time = current_weather
            .time
            .format("%A, %d %B %Y, %_H:%M")
            .to_string();
        let frog_icon: Frog130x180 = current_weather.weather_icon.into();

        let mut frog_icon = Icon::new_owned(frog_icon);
        let mut date_time = Text::new(date_time.leak(), styles::date_time_style(), E6Color::Black);
        let mut temperature = Text::new(
            format!("{:+}C°", current_weather.temperature.round() as u32).leak(),
            BitmapFontStyle::new(&TEMPERATURE_FONT_50, BinaryColor::On),
            styles::temperature_color(current_weather.temperature),
        );
        let mut apparent_temperature_label = Text::new(
            "Feels like ",
            styles::current_weather_details_style(),
            E6Color::Black,
        );
        let mut apparent_temperature = Text::new(
            format!(
                "{:+}C°",
                current_weather.apparent_temperature.round() as u32
            )
            .leak(),
            BitmapFontStyle::new(&TEMPERATURE_FONT_20, BinaryColor::On),
            styles::temperature_color(current_weather.apparent_temperature),
        );
        let mut weather_details = Text::new(
            format!(
                "Wind: {:}km/h ({:}km/h)\nHumidity: {:}%\nPrecipitation: {:}mm\nCloud cover: {:}% UV: {:}",
                current_weather.wind_speed,
                current_weather.wind_gusts,
                current_weather.humidity,
                current_weather.precipitation,
                current_weather.cloud_cover,
                current_weather.uv_index,
            )
            .leak(),
            styles::current_weather_details_style(),
            E6Color::Black,
        );

        frog_icon.translate_mut((210, 32).into());
        date_time.translate_mut((5, 17).into());
        temperature.translate_mut((5, 82).into());
        apparent_temperature_label.translate_mut((5, 120).into());
        apparent_temperature
            .translate_mut((apparent_temperature_label.size().width as i32 + 12, 120).into());
        weather_details.translate_mut((5, 144).into());

        Self {
            frog_icon,
            date_time,
            temperature,
            apparent_temperature_label,
            apparent_temperature,
            weather_details,
        }
    }
}

impl Drawable for CurrentWeatherWidget<'_> {
    type Color = E6Color;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.frog_icon.draw(target)?;
        self.date_time.draw(target)?;
        self.temperature.draw(target)?;
        self.apparent_temperature_label.draw(target)?;
        self.apparent_temperature.draw(target)?;
        self.weather_details.draw(target)?;
        Ok(())
    }
}

impl Widget for CurrentWeatherWidget<'_> {}
