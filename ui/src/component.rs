pub mod error;
pub mod form;
pub mod list;

pub use error::ErrorView;
pub use form::Form;
pub use list::AsyncList;
use parley::LineHeight;
use parley::layout::{Alignment, AlignmentOptions};
use xilem::core::frozen;
use xilem::masonry::core::{ArcStr, render_text};
use xilem::masonry::layout::{AsUnit, Dim};
use xilem::masonry::parley::{FontFamily, FontStack, GenericFamily, StyleProperty};
use xilem::palette::css::WHITE;
use xilem::style::{Padding, Style};
use xilem::vello::kurbo::{Affine, Circle, Point, Stroke};
use xilem::vello::peniko::Fill;
use xilem::view::{
    CrossAxisAlignment, Prose, button, canvas, flex_col, flex_row, label, prose, sized_box,
};
use xilem::{FontWeight, TextAlign, WidgetView};

use crate::theme::{ACCENT_COLOR, ACTION_BTN, ApplyClass, SURFACE_COLOR};

pub fn logo<State, Action>() -> impl WidgetView<State, Action>
where
    State: 'static + Send + Sync,
    Action: 'static + Send + Sync,
{
    frozen(|| {
        flex_col((
            label("Kreqo")
                .weight(FontWeight::BOLD)
                .text_size(22.)
                .transform(Affine::translate((-25., 7.))),
            label("Learn")
                .weight(FontWeight::EXTRA_BLACK)
                .text_size(28.)
                .color(ACCENT_COLOR)
                .transform(Affine::translate((10., 0.))),
        ))
        .gap(0.px())
    })
}

pub fn user_profile_overview(username: &mut String) -> impl WidgetView<String> + use<> {
    let profile_circle = canvas(move |state: &mut String, ctx, scene, size| {
        let (fcx, lcx) = ctx.text_contexts();
        let letter = &state[..1].to_uppercase();

        let half_size = size.to_vec2() / 2.;
        let circle = Circle::new(Point::new(half_size.x, half_size.y), half_size.x);
        scene.fill(Fill::NonZero, Affine::IDENTITY, ACCENT_COLOR, None, &circle);
        scene.stroke(&Stroke::default(), Affine::IDENTITY, WHITE, None, &circle);

        let mut text_layout_builder = lcx.ranged_builder(fcx, letter, 1., true);
        text_layout_builder.push_default(StyleProperty::FontStack(FontStack::Single(
            FontFamily::Generic(GenericFamily::SansSerif),
        )));
        text_layout_builder.push_default(StyleProperty::FontSize(size.height as f32 * 0.75));
        text_layout_builder.push_default(StyleProperty::FontWeight(FontWeight::SEMI_BOLD));
        text_layout_builder.push_default(StyleProperty::LineHeight(LineHeight::Absolute(
            size.height as f32,
        )));
        let mut text_layout = text_layout_builder.build(letter);
        text_layout.break_all_lines(None);
        text_layout.align(
            Some(size.width as f32),
            Alignment::Center,
            AlignmentOptions::default(),
        );
        render_text(scene, Affine::IDENTITY, &text_layout, &[WHITE.into()], true);
    });
    flex_row((
        sized_box(profile_circle).dims(Dim::Fixed(35.px())),
        prose(username.to_string()).text_size(18.),
    ))
}

pub fn header<State, Action>(content: impl Into<ArcStr>) -> Prose<State, Action> {
    prose(content)
        .weight(FontWeight::BOLD)
        .text_size(24.)
        .text_alignment(TextAlign::Center)
}

pub fn form_input_label<State, Action>(text: impl Into<ArcStr>) -> impl WidgetView<State, Action>
where
    State: 'static + Send + Sync,
    Action: 'static + Send + Sync,
{
    flex_row(
        label(text)
            .text_size(13.)
            .padding(3.)
            .background(SURFACE_COLOR)
            .transform(Affine::translate((0., -9.))),
    )
    .cross_axis_alignment(CrossAxisAlignment::Start)
    .padding(Padding::horizontal(19.))
}

pub fn action_button<State, Action>(
    text: impl Into<ArcStr>,
    callback: impl Fn(&mut State) -> Action + Send + Sync + 'static,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
{
    button(label(text).weight(FontWeight::BLACK), callback).class(ACTION_BTN)
}
