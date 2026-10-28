//! ⭐ A vista **Calendário** sobre uma [`Source`] — a de UM Kanban (W7) e a do projecto (W9).
//! (Em construção.)

use ph2d_editor_core::doc_surface::KeyInput;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

use crate::common::{Ctx, PointerAt, Reply};
use crate::item::Source;

/// O estado do calendário (da VISTA).
#[derive(Debug, Default)]
pub struct CalendarState {}

#[allow(clippy::too_many_arguments)]
pub fn paint(
    _out: &mut VectorScene,
    _ts: &mut TextSystem,
    _theme: Theme,
    _area: Rect,
    _st: &mut CalendarState,
    _src: &dyn Source,
    _cx: &Ctx<'_>,
) {
}

pub fn pointer(_st: &mut CalendarState, _src: &dyn Source, _cx: &Ctx<'_>, _p: PointerAt) -> Reply {
    Reply::default()
}

pub fn wheel(_st: &mut CalendarState, _at: (f32, f32), _dy: f32, _shift: bool) -> Reply {
    Reply::default()
}

pub fn key(
    _st: &mut CalendarState,
    _src: &dyn Source,
    _cx: &Ctx<'_>,
    _input: &KeyInput<'_>,
) -> Reply {
    Reply::default()
}
