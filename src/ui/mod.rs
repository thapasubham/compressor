mod knob;

use std::sync::Arc;
use std::sync::atomic::Ordering;

use nice_plug::editor::dpi::LogicalSize;
use nice_plug::editor::ResizeHint;
use nice_plug::prelude::{BoolParam, FloatParam, Param};
use nice_plug_iced::iced::widget::{button, canvas, column, container, row, rule, text, Space};
use nice_plug_iced::iced::{
    Background, Border, Center, Color, Element, Fill, Font, Length, PollSubNotifier, Renderer,
    Task, Theme,
};
use nice_plug_iced::{
    create_iced_editor, IcedEditor, IcedEditorState, IcedNiceContext, IcedNiceSettings,
    PersistentState,
};

use crate::plugin::CompressorParams;
use knob::Knob;

const CANVAS: Color = Color::from_rgb(1.0, 1.0, 1.0);
const INK: Color = Color::from_rgb(0x11 as f32 / 255.0, 0x11 as f32 / 255.0, 0x11 as f32 / 255.0);
const MUTED: Color = Color::from_rgb(0x6B as f32 / 255.0, 0x6B as f32 / 255.0, 0x6B as f32 / 255.0);
const HAIRLINE: Color = Color::from_rgb(0xE0 as f32 / 255.0, 0xE0 as f32 / 255.0, 0xE0 as f32 / 255.0);
const HOVER: Color = Color::from_rgb(0xED as f32 / 255.0, 0xED as f32 / 255.0, 0xED as f32 / 255.0);

fn serif(weight: nice_plug_iced::iced::font::Weight) -> Font {
    Font {
        family: nice_plug_iced::iced::font::Family::Serif,
        weight,
        ..Font::DEFAULT
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    Threshold,
    Ratio,
    Attack,
    Release,
    Makeup,
    Mix,
}

impl ParamKind {
    fn param<'a>(self, params: &'a CompressorParams) -> &'a FloatParam {
        match self {
            ParamKind::Threshold => &params.threshold,
            ParamKind::Ratio => &params.ratio,
            ParamKind::Attack => &params.attack,
            ParamKind::Release => &params.release,
            ParamKind::Makeup => &params.makeup,
            ParamKind::Mix => &params.mix,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Changed(ParamKind, f32),
    Released(ParamKind),
    BypassToggled,
}

pub struct State {
    params: Arc<CompressorParams>,
    nice_ctx: IcedNiceContext,
    dragging: Option<ParamKind>,
}

fn boot(
    persistent_state: PersistentState<Arc<CompressorParams>>,
    nice_ctx: IcedNiceContext,
) -> (State, Task<Message>) {
    let params = (*persistent_state).clone();
    (
        State {
            params,
            nice_ctx,
            dragging: None,
        },
        Task::none(),
    )
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    let setter = state.nice_ctx.nice_context.param_setter();

    match message {
        Message::Changed(kind, normalized) => {
            let param = kind.param(&state.params);
            if state.dragging != Some(kind) {
                setter.begin_set_parameter(param);
                state.dragging = Some(kind);
            }
            setter.set_parameter_normalized(param, normalized);
        }
        Message::Released(kind) => {
            setter.end_set_parameter(kind.param(&state.params));
            if state.dragging == Some(kind) {
                state.dragging = None;
            }
        }
        Message::BypassToggled => {
            let bypass = &state.params.bypass;
            let new_value = !bypass.value();
            setter.begin_set_parameter(bypass);
            setter.set_parameter_normalized(bypass, if new_value { 1.0 } else { 0.0 });
            setter.end_set_parameter(bypass);
        }
    }

    Task::none()
}

fn section_label<'a>(label: &str) -> Element<'a, Message> {
    text(label.to_string())
        .size(11)
        .color(MUTED)
        .into()
}

fn knob_control<'a>(label: &'static str, kind: ParamKind, param: &FloatParam) -> Element<'a, Message> {
    let normalized = param.modulated_normalized_value();
    let value_text = param.normalized_value_to_string(normalized, true);

    let control = column![
        text(label).size(11).color(MUTED),
        Knob::new(
            normalized,
            56.0,
            move |v| Message::Changed(kind, v),
            Message::Released(kind),
        )
        .view(),
        text(value_text).size(15).font(serif(nice_plug_iced::iced::font::Weight::Medium)).color(INK),
    ]
    .spacing(10)
    .align_x(Center);

    container(control)
        .width(Length::FillPortion(1))
        .align_x(Center)
        .into()
}

fn control_row<'a>(controls: [Element<'a, Message>; 2]) -> Element<'a, Message> {
    row(controls).spacing(24).into()
}

fn divider<'a>() -> Element<'a, Message> {
    rule::horizontal(1)
        .style(|_theme| rule::Style {
            color: HAIRLINE,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
            snap: false,
        })
        .into()
}

struct GainReductionMeter {
    /// Gain reduction in dB, always <= 0.
    reduction_db: f32,
}

impl<Message> canvas::Program<Message> for GainReductionMeter {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: nice_plug_iced::iced::Rectangle,
        _cursor: nice_plug_iced::iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        use canvas::{Frame, Path, Stroke};
        use nice_plug_iced::iced::Point;

        let mut frame = Frame::new(renderer, bounds.size());

        // Scale: -24 dB .. +6 dB, mapped left to right.
        const MIN_DB: f32 = -24.0;
        const MAX_DB: f32 = 6.0;
        let x_for_db = |db: f32| -> f32 {
            let t = (db - MIN_DB) / (MAX_DB - MIN_DB);
            t.clamp(0.0, 1.0) * bounds.width
        };

        let line_y = bounds.height / 2.0;
        let track = Path::line(
            Point::new(0.0, line_y),
            Point::new(bounds.width, line_y),
        );
        frame.stroke(&track, Stroke::default().with_color(HAIRLINE).with_width(1.0));

        for &db in &[-24.0, -12.0, 0.0, 6.0] {
            let x = x_for_db(db);
            frame.stroke(
                &Path::line(Point::new(x, line_y - 5.0), Point::new(x, line_y + 5.0)),
                Stroke::default().with_color(MUTED).with_width(1.0),
            );
        }

        // Zero-dB reference tick is emphasized.
        let zero_x = x_for_db(0.0);
        frame.stroke(
            &Path::line(Point::new(zero_x, line_y - 8.0), Point::new(zero_x, line_y + 8.0)),
            Stroke::default().with_color(INK).with_width(1.5),
        );

        let indicator_x = x_for_db(self.reduction_db);
        let indicator = Path::circle(Point::new(indicator_x, line_y), 4.0);
        frame.fill(&indicator, INK);

        vec![frame.into_geometry()]
    }
}

fn gain_reduction_section<'a>(reduction_db: f32) -> Element<'a, Message> {
    let meter = canvas::Canvas::new(GainReductionMeter { reduction_db })
        .width(Fill)
        .height(Length::Fixed(28.0));

    let scale = row![
        text("-24").size(10).color(MUTED),
        Space::new().width(Fill),
        text("-12").size(10).color(MUTED),
        Space::new().width(Fill),
        text("0").size(10).color(INK),
        Space::new().width(Fill),
        text("+6 dB").size(10).color(MUTED),
    ];

    column![
        row![
            section_label("GAIN REDUCTION"),
            Space::new().width(Fill),
            text(format!("{:.1} dB", reduction_db)).size(12).color(INK),
        ],
        meter,
        scale,
    ]
    .spacing(8)
    .into()
}

fn bypass_button<'a>(bypass: &BoolParam) -> Element<'a, Message> {
    let active = bypass.value();

    button(text(if active { "BYPASSED" } else { "BYPASS" }).size(11))
        .padding([6, 14])
        .style(move |_theme, status| {
            let hovered = matches!(status, button::Status::Hovered);
            button::Style {
                background: Some(Background::Color(if active {
                    INK
                } else if hovered {
                    HOVER
                } else {
                    CANVAS
                })),
                text_color: if active { CANVAS } else { INK },
                border: Border {
                    color: HAIRLINE,
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..button::Style::default()
            }
        })
        .on_press(Message::BypassToggled)
        .into()
}

fn view(state: &State) -> Element<'_, Message, Theme, Renderer> {
    let params = &state.params;

    let header = row![
        text("Compressor")
            .size(26)
            .font(serif(nice_plug_iced::iced::font::Weight::Bold))
            .color(INK),
        Space::new().width(Fill),
        bypass_button(&params.bypass),
    ]
    .align_y(Center);

    let compressor_section = column![
        section_label("COMPRESSOR"),
        control_row([
            knob_control("THRESHOLD", ParamKind::Threshold, &params.threshold),
            knob_control("RATIO", ParamKind::Ratio, &params.ratio),
        ]),
    ]
    .spacing(16);

    let timing_section = column![
        section_label("TIMING"),
        control_row([
            knob_control("ATTACK", ParamKind::Attack, &params.attack),
            knob_control("RELEASE", ParamKind::Release, &params.release),
        ]),
    ]
    .spacing(16);

    let output_section = column![
        section_label("OUTPUT"),
        control_row([
            knob_control("MAKEUP", ParamKind::Makeup, &params.makeup),
            knob_control("MIX", ParamKind::Mix, &params.mix),
        ]),
    ]
    .spacing(16);

    let gain_reduction_db = params.gain_reduction.load(Ordering::Relaxed);

    let content = column![
        header,
        divider(),
        compressor_section,
        divider(),
        timing_section,
        divider(),
        gain_reduction_section(gain_reduction_db),
        divider(),
        output_section,
    ]
    .spacing(24)
    .width(Fill);

    container(content)
        .padding(32)
        .width(Fill)
        .height(Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(CANVAS)),
            text_color: Some(INK),
            ..container::Style::default()
        })
        .into()
}

pub fn create(params: Arc<CompressorParams>) -> Option<IcedEditor> {
    let editor_state = IcedEditorState::from_size(LogicalSize::new(560.0, 620.0), 1.0);

    let resize_hint = ResizeHint::resizable().with_min_max_logical_size(
        Some(LogicalSize::new(420.0, 480.0)),
        Some(LogicalSize::new(900.0, 1000.0)),
    );

    create_iced_editor(
        editor_state,
        params,
        PollSubNotifier::default(),
        IcedNiceSettings::default()
            .with_tile("Compressor")
            .with_resize_hint(resize_hint),
        |persistent_state, nice_ctx| {
            Ok(nice_plug_iced::application(persistent_state, nice_ctx, boot, update, view).run())
        },
    )
}
