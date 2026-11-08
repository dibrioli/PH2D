//! A [`super::paint_fields_row`] com as **casas que as caixas mostram em repouso**
//! ([`crate::widget::NumberInput::casas`]) — a MESMA porta (`paint_fields_row_inner`), não uma
//! segunda conta da linha.

use super::{Rotulo, Seccao, paint_fields_row_inner};
use crate::interaction::{HitIndex, WidgetStore};
use crate::widget::Unit;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// Para quem espelha o valor com casas fixas e não quer que a caixa as troque pelas de
/// [`crate::widget::format_number`] (`0.700` ao lado de `1`, cartão Pen do Painter, 25/10).
#[allow(clippy::too_many_arguments)]
pub fn paint_fields_row_com_casas<'a>(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: impl Into<Rotulo<'a>>,
    field_ids: &[NodeId],
    step: f64,
    unit: Option<Unit>,
    casas: Option<usize>,
    seccao: Seccao,
) -> f32 {
    paint_fields_row_inner(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        label.into(),
        field_ids,
        step,
        unit,
        casas,
        seccao,
    )
    .0
}
