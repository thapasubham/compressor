use nice_plug_iced::iced::mouse;
use nice_plug_iced::iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Program, Stroke};
use nice_plug_iced::iced::{
    Color, Degrees, Element, Length, Point, Radians, Rectangle, Renderer, Theme, Vector,
};

const INK: Color = Color::from_rgb(0x11 as f32 / 255.0, 0x11 as f32 / 255.0, 0x11 as f32 / 255.0);
const HAIRLINE: Color = Color::from_rgb(0xE0 as f32 / 255.0, 0xE0 as f32 / 255.0, 0xE0 as f32 / 255.0);
const MUTED: Color = Color::from_rgb(0x6B as f32 / 255.0, 0x6B as f32 / 255.0, 0x6B as f32 / 255.0);

/// Pixels of vertical drag needed to sweep the knob from 0.0 to 1.0.
const DRAG_RANGE_PX: f32 = 200.0;

/// The dial sweeps 270 degrees, leaving a 90 degree gap at the bottom, with
/// 0.0 pointing to the lower-left and 1.0 to the lower-right.
const SWEEP_START: f32 = -135.0;
const SWEEP_END: f32 = 135.0;

/// A minimal, geometric rotary control: a faint track arc with a solid
/// progress arc on top, matching a measurement-instrument aesthetic rather
/// than a skeuomorphic pointer knob. Dragging vertically anywhere in the
/// window (not just within the small dial itself) changes the value, the
/// same way a DAW's own knobs behave.
pub struct Knob<'a, Message> {
    normalized: f32,
    diameter: f32,
    on_change: Box<dyn Fn(f32) -> Message + 'a>,
    on_release: Message,
}

impl<'a, Message: Clone> Knob<'a, Message> {
    pub fn new(
        normalized: f32,
        diameter: f32,
        on_change: impl Fn(f32) -> Message + 'a,
        on_release: Message,
    ) -> Self {
        Self {
            normalized: normalized.clamp(0.0, 1.0),
            diameter,
            on_change: Box::new(on_change),
            on_release,
        }
    }

    pub fn view(self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        let diameter = self.diameter;
        Canvas::new(self)
            .width(Length::Fixed(diameter))
            .height(Length::Fixed(diameter))
            .into()
    }
}

#[derive(Default)]
pub struct DragState {
    /// The pointer's absolute position and the knob's value when the drag began.
    drag: Option<(Point, f32)>,
}

impl<'a, Message: Clone> Program<Message, Theme, Renderer> for Knob<'a, Message> {
    type State = DragState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                // Only start a drag when the click actually lands on the dial.
                cursor.position_in(bounds)?;
                let start = cursor.position()?;
                state.drag = Some((start, self.normalized));
                Some(canvas::Action::request_redraw().and_capture())
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                // Once dragging, track the cursor across the whole window, not
                // just while it happens to stay over the small dial.
                let (start, start_value) = state.drag?;
                let position = cursor.position()?;
                let delta = (start.y - position.y) / DRAG_RANGE_PX;
                let new_value = (start_value + delta).clamp(0.0, 1.0);
                Some(canvas::Action::publish((self.on_change)(new_value)).and_capture())
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.drag.take().is_some() {
                    Some(canvas::Action::publish(self.on_release.clone()).and_capture())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let mut frame = Frame::new(renderer, bounds.size());
        let center = frame.center();
        let radius = (self.diameter / 2.0) - 5.0;

        let start_angle = Radians::from(Degrees(SWEEP_START - 90.0));
        let end_angle = Radians::from(Degrees(SWEEP_END - 90.0));
        let value_angle = Radians::from(Degrees(
            SWEEP_START + self.normalized * (SWEEP_END - SWEEP_START) - 90.0,
        ));

        let track = Path::new(|builder| {
            builder.arc(canvas::path::Arc {
                center,
                radius,
                start_angle,
                end_angle,
            });
        });
        frame.stroke(
            &track,
            Stroke::default()
                .with_color(HAIRLINE)
                .with_width(2.5)
                .with_line_cap(canvas::LineCap::Round),
        );

        let progress = Path::new(|builder| {
            builder.arc(canvas::path::Arc {
                center,
                radius,
                start_angle,
                end_angle: value_angle,
            });
        });
        frame.stroke(
            &progress,
            Stroke::default()
                .with_color(INK)
                .with_width(2.5)
                .with_line_cap(canvas::LineCap::Round),
        );

        // A pointer from the hub out to the value's position on the arc.
        let pointer_inner = point_on_circle(center, radius * 0.3, value_angle.0);
        let pointer_outer = point_on_circle(center, radius - 6.0, value_angle.0);
        frame.stroke(
            &Path::line(pointer_inner, pointer_outer),
            Stroke::default()
                .with_color(INK)
                .with_width(2.0)
                .with_line_cap(canvas::LineCap::Round),
        );
        frame.fill(&Path::circle(center, 2.0), INK);

        let center_angle = Radians::from(Degrees(-90.0));
        let tick_inner = point_on_circle(center, radius - 4.0, center_angle.0);
        let tick_outer = point_on_circle(center, radius + 2.0, center_angle.0);
        frame.stroke(
            &Path::line(tick_inner, tick_outer),
            Stroke::default().with_color(MUTED).with_width(1.0),
        );

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.drag.is_some() {
            mouse::Interaction::Grabbing
        } else if cursor.position_over(bounds).is_some() {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::default()
        }
    }
}

fn point_on_circle(center: Point, radius: f32, angle: f32) -> Point {
    center + Vector::new(radius * angle.cos(), radius * angle.sin())
}
