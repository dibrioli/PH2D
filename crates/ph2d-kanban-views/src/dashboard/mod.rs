//! ⭐ A vista **Painel** (o Dashboard do Trello). (Em construção.)

use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::{KanbanId, KanbanSet};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

use crate::common::{Ctx, PointerAt, Reply};

/// O estado do painel (da VISTA).
#[derive(Debug, Default)]
pub struct DashboardState {}

#[allow(clippy::too_many_arguments)]
pub fn paint(
    _out: &mut VectorScene,
    _ts: &mut TextSystem,
    _theme: Theme,
    _area: Rect,
    _st: &mut DashboardState,
    _set: &KanbanSet,
    _kanban: KanbanId,
    _cx: &Ctx<'_>,
) {
}

pub fn pointer(
    _st: &mut DashboardState,
    _set: &KanbanSet,
    _kanban: KanbanId,
    _cx: &Ctx<'_>,
    _p: PointerAt,
) -> Reply {
    Reply::default()
}
