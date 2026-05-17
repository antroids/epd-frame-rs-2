use alloc::borrow::Cow;
use crate::display::DEFAULT_FONT_12;
use crate::display::color::{BinaryColorAdapter, E6Color};
use crate::display::image::E6ImageSource;
use embedded_graphics::Drawable;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Dimensions, Point, Size};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::primitives::{
    CornerRadii, PrimitiveStyle, Rectangle, RoundedRectangle, StyledDrawable,
};
use embedded_graphics::text::renderer::{CharacterStyle, TextRenderer};
use embedded_layout::View;
use embedded_layout::layout::linear::LinearLayout;
use embedded_layout::prelude::{Chain, horizontal};
use mplusfonts::style::BitmapFontStyle;

pub(crate) mod weather;

pub(crate) trait Widget: Drawable<Color = E6Color> + View {}

#[derive(Clone)]
pub struct Text<'a, S: Clone> {
    inner: embedded_graphics::text::Text<'a, S>,
    color: E6Color,
}

impl<'a, S: Clone> Text<'a, S> {
    pub fn new(text: &'a str, style: S, color: E6Color) -> Self {
        let inner = embedded_graphics::text::Text::new(text, Default::default(), style);
        Self { inner, color }
    }

    pub fn color(&self) -> E6Color {
        self.color
    }
}

impl<'a, S: TextRenderer + Clone> View for Text<'a, S> {
    fn translate_impl(&mut self, by: Point) {
        self.inner.position += by;
    }

    fn bounds(&self) -> Rectangle {
        self.inner.bounding_box()
    }
}

impl<'a, S: CharacterStyle + TextRenderer<Color = BinaryColor> + Clone> Drawable for Text<'a, S> {
    type Color = E6Color;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        BinaryColorAdapter::draw_transparent(self.color, &self.inner, target)?;
        Ok(())
    }
}

impl<'a, S: CharacterStyle + TextRenderer<Color = BinaryColor> + Clone> Widget for Text<'a, S> {}

#[derive(Clone)]
pub struct Icon<'a, I: Clone> {
    position: Point,
    icon: Cow<'a, I>,
}

impl<'a, I: Clone> Icon<'a, I> {
    pub fn new(icon: &'a I) -> Self {
        Self {
            position: Default::default(),
            icon: Cow::Borrowed(icon),
        }
    }

    pub fn new_owned(icon: I) -> Self {
        Self {
            position: Default::default(),
            icon: Cow::Owned(icon),
        }
    }
}

impl<'a, I: E6ImageSource + Clone> View for Icon<'a, I> {
    fn translate_impl(&mut self, by: Point) {
        self.position += by;
    }

    fn bounds(&self) -> Rectangle {
        Rectangle::new(self.position, self.icon.size())
    }
}

impl<'a, I: E6ImageSource + Clone> Drawable for Icon<'a, I> {
    type Color = E6Color;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.icon.draw(self.position, target)?;
        Ok(())
    }
}

impl<'a, I: E6ImageSource + Clone> Widget for Icon<'a, I> {}



#[derive(Clone)]
pub struct RoundWidgetBorder {
    bounds: Rectangle,
}

impl RoundWidgetBorder {
    pub fn new(bounds: Rectangle) -> Self {
        Self { bounds }
    }
}

impl View for RoundWidgetBorder {
    fn translate_impl(&mut self, by: Point) {
        self.bounds.translate_mut(by);
    }

    fn bounds(&self) -> Rectangle {
        self.bounds
    }
}

impl Drawable for RoundWidgetBorder {
    type Color = E6Color;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let frame_style = PrimitiveStyle::with_stroke(E6Color::Black, 1);
        RoundedRectangle::new(self.bounds, CornerRadii::new((4, 4).into()))
            .draw_styled(&frame_style, target)?;
        Ok(())
    }
}

impl Widget for RoundWidgetBorder {}
