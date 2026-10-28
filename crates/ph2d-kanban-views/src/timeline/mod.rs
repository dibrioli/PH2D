//! ⭐ A vista **Linha do tempo** (a Timeline do Trello). (Em construção.)

use ph2d_editor_core::doc_surface::KeyInput;
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::{KanbanId, KanbanSet};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

use crate::common::{Ctx, PointerAt, Reply};

/// O estado da linha do tempo (da VISTA).
#[derive(Debug, Default)]
pub struct TimelineState {}

#[allow(clippy::too_many_arguments)]
pub fn paint(
    _out: &mut VectorScene,
    _ts: &mut TextSystem,
    _theme: Theme,
    _area: Rect,
    _st: &mut TimelineState,
    _set: &KanbanSet,
    _kanban: KanbanId,
    _cx: &Ctx<'_>,
) {
}

pub fn pointer(
    _st: &mut TimelineState,
    _set: &KanbanSet,
    _kanban: KanbanId,
    _cx: &Ctx<'_>,
    _p: PointerAt,
) -> Reply {
    Reply::default()
}

#[allow(clippy::too_many_arguments)]
pub fn wheel(
    _st: &mut TimelineState,
    _set: &KanbanSet,
    _kanban: KanbanId,
    _cx: &Ctx<'_>,
    _at: (f32, f32),
    _dy: f32,
    _shift: bool,
) -> Reply {
    Reply::default()
}

pub fn key(
    _st: &mut TimelineState,
    _set: &KanbanSet,
    _kanban: KanbanId,
    _cx: &Ctx<'_>,
    _input: &KeyInput<'_>,
) -> Reply {
    Reply::default()
}
