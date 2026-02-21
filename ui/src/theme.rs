use xilem::core::MessageResult;
use xilem::masonry::core::{ArcStr, DefaultProperties};
use xilem::masonry::layout::Dim;
use xilem::masonry::properties::{Dimensions, FocusedBorderColor};
use xilem::masonry::theme::ZYNC_600;
use xilem::masonry::widgets::TextInput;
use xilem::palette::css::{BLACK, WHITE};
use xilem::style::{
    ActiveBackground, Background, BorderColor, BorderWidth, CornerRadius, HoveredBorderColor,
    Padding,
};
use xilem::view::{Button, Label, PointerButton, Prose, button, label, prose};
use xilem::{Color, FontWeight, TextAlign};

use crate::class::Class;

pub const DARK_OVERLAY: Color = BLACK.with_alpha(0.25);

pub const BACKGROUND_COLOR: Color = Color::from_rgb8(0x0a, 0x0a, 0x0a);
pub const SURFACE_COLOR: Color = Color::from_rgb8(0x14, 0x14, 0x14);
pub const SURFACE_BORDER_COLOR: Color = Color::from_rgb8(0x1e, 0x1e, 0x1e);

pub const ACCENT_COLOR: Color = Color::from_rgb8(0x00, 0x92, 0xb8);
pub const ACTIVE_ACCENT_COLOR: Color = Color::from_rgb8(0x00, 0xb8, 0xdb);

pub const SUCCESS_COLOR: Color = Color::from_rgb8(0x00, 0xbc, 0x7d);
pub const WARNING_COLOR: Color = Color::from_rgb8(0xfd, 0x9a, 0x00);
pub const DANGER_COLOR: Color = Color::from_rgb8(0xfb, 0x2c, 0x36);

pub fn apply_theme(def_props: &mut DefaultProperties) {
    def_props.insert::<TextInput, Background>(Background::Color(
        SURFACE_COLOR.map_lightness(|l| l * 0.95),
    ));
}

pub const SURFACE: (Background, BorderWidth, BorderColor) = (
    Background::Color(SURFACE_COLOR),
    BorderWidth::all(1.),
    BorderColor::new(SURFACE_BORDER_COLOR),
);

pub const CONTAINER: (Padding, CornerRadius, Background, BorderWidth, BorderColor) = (
    Padding::all(25.),
    CornerRadius::all(15.),
    SURFACE.0,
    SURFACE.1,
    SURFACE.2,
);

pub const ROW: (Padding, CornerRadius, Background) = (
    Padding::all(5.),
    CornerRadius::all(10.),
    Background::Color(SURFACE_COLOR),
);

pub const ROW_OVERLAY: (Padding, CornerRadius, Background) =
    (ROW.0, ROW.1, Background::Color(DARK_OVERLAY));

pub const BORDERED_ROW: (Padding, CornerRadius, Background, BorderWidth, BorderColor) =
    (ROW.0, ROW.1, SURFACE.0, SURFACE.1, SURFACE.2);

pub const ACTION_BTN: (
    Dimensions,
    Padding,
    Background,
    HoveredBorderColor,
    ActiveBackground,
) = (
    Dimensions::new(Dim::Stretch, Dim::Auto),
    Padding::all(7.5),
    Background::Color(ACCENT_COLOR),
    HoveredBorderColor(BorderColor::new(WHITE)),
    ActiveBackground(Background::Color(ACTIVE_ACCENT_COLOR)),
);

pub fn constant_border_color(
    color: Option<Color>,
) -> (BorderColor, HoveredBorderColor, FocusedBorderColor) {
    let color = color.unwrap_or(ZYNC_600);
    (
        BorderColor::new(color),
        HoveredBorderColor(BorderColor::new(color)),
        FocusedBorderColor(BorderColor::new(color)),
    )
}

pub fn header<State, Action>(content: impl Into<ArcStr>) -> Prose<State, Action> {
    prose(content)
        .text_size(24.)
        .text_alignment(TextAlign::Center)
}

pub fn action_button<State: 'static, Action>(
    text: impl Into<ArcStr>,
    callback: impl Fn(&mut State) -> Action + Send + Sync + 'static,
) -> Button<
    State,
    Action,
    impl Fn(&mut State, Option<PointerButton>) -> MessageResult<Action> + Send + Sync + 'static,
    Label,
> {
    button(label(text).weight(FontWeight::BLACK), callback)
}

pub trait ApplyClass<State, Action, C> {
    fn apply(self, class: C) -> C::Styled
    where
        Self: Sized,
        C: Class<State, Action, Self>,
    {
        class.styled(self)
    }

    fn apply_fn<F, I>(self, f: F, input: I) -> C::Styled
    where
        Self: Sized,
        C: Class<State, Action, Self>,
        F: FnOnce(I) -> C,
    {
        f(input).styled(self)
    }
}

impl<State, Action, C, T> ApplyClass<State, Action, C> for T where C: Class<State, Action, T> {}
