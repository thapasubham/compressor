use std::sync::Arc;
use std::sync::atomic::Ordering;

use nice_plug::editor::dpi::LogicalSize;
use nice_plug::prelude::{FloatParam, Param};
use nice_plug_iced::iced::widget::{column, container, row, slider, text};
use nice_plug_iced::iced::{Center, Element, Fill, Length, PollSubNotifier, Renderer, Task, Theme};
use nice_plug_iced::{
    IcedEditor, IcedEditorState, IcedNiceContext, IcedNiceSettings, PersistentState,
    create_iced_editor,
};

use crate::plugin::CompressorParams;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    Threshold,
    Ratio,
    Attack,
    Release,
    Makeup,
}

impl ParamKind {
    const ALL: [ParamKind; 5] = [
        ParamKind::Threshold,
        ParamKind::Ratio,
        ParamKind::Attack,
        ParamKind::Release,
        ParamKind::Makeup,
    ];

    fn param<'a>(self, params: &'a CompressorParams) -> &'a FloatParam {
        match self {
            ParamKind::Threshold => &params.threshold,
            ParamKind::Ratio => &params.ratio,
            ParamKind::Attack => &params.attack,
            ParamKind::Release => &params.release,
            ParamKind::Makeup => &params.makeup,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Changed(ParamKind, f32),
    Released(ParamKind),
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
    }

    Task::none()
}

fn param_row(kind: ParamKind, param: &FloatParam) -> Element<'_, Message> {
    let normalized = param.modulated_normalized_value();
    let value_text = param.normalized_value_to_string(normalized, true);

    row![
        text(param.name()).width(Length::Fixed(100.0)),
        slider(0.0f32..=1.0, normalized, move |v| Message::Changed(kind, v))
            .step(0.001f32)
            .on_release(Message::Released(kind))
            .width(Fill),
        text(value_text).width(Length::Fixed(90.0)),
    ]
    .spacing(16)
    .align_y(Center)
    .into()
}

fn view(state: &State) -> Element<'_, Message, Theme, Renderer> {
    let sliders = ParamKind::ALL
        .into_iter()
        .fold(column![].spacing(18), |col, kind| {
            col.push(param_row(kind, kind.param(&state.params)))
        });

    let gain_reduction_db = state.params.gain_reduction.load(Ordering::Relaxed);
    let meter = row![
        text("Gain Reduction").width(Length::Fixed(100.0)),
        text(format!("{gain_reduction_db:.1} dB")),
    ]
    .spacing(16)
    .align_y(Center);

    container(
        column![text("Compressor").size(24), sliders, meter]
            .spacing(24)
            .width(Fill),
    )
    .padding(24)
    .into()
}

pub fn create(params: Arc<CompressorParams>) -> Option<IcedEditor> {
    let editor_state = IcedEditorState::from_size(LogicalSize::new(480.0, 360.0), 1.0);

    create_iced_editor(
        editor_state,
        params,
        PollSubNotifier::default(),
        IcedNiceSettings::default().with_tile("Compressor"),
        |persistent_state, nice_ctx| {
            Ok(nice_plug_iced::application(persistent_state, nice_ctx, boot, update, view).run())
        },
    )
}
