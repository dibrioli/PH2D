//! A [`super::paint_fields_row`] com as **casas que as caixas mostram em repouso**
//! ([`crate::widget::NumberInput::casas`]) e com o **fim** que a caixa cede — a MESMA porta
//! (`paint_fields_row_inner`), não uma segunda conta da linha.

use super::{Rotulo, Seccao, paint_fields_row_inner};
use crate::interaction::{HitIndex, WidgetStore};
use crate::widget::Unit;
use crate::zones::Rect;
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
        0.0,
    )
    .0
}

/// A linha de UM campo com as colunas da linha INTEIRA (o valor arranca na coluna da secção), e
/// `fim_w` tirado só do fim da caixa. Devolve o `y` seguinte e o retângulo que sobra (o nome da
/// camada do Ray Sensor pinta-se nele).
#[allow(clippy::too_many_arguments)]
pub fn paint_fields_row_com_fim<'a>(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    (x, w, y): (f32, f32, f32),
    label: impl Into<Rotulo<'a>>,
    id: NodeId,
    step: f64,
    seccao: Seccao,
    fim_w: f32,
) -> (f32, Rect) {
    let (y_seguinte, caixa) = paint_fields_row_inner(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        label.into(),
        &[id],
        step,
        None,
        None,
        seccao,
        fim_w,
    );
    let c = caixa.expect("uma linha de UM campo pinta sempre esse campo");
    (y_seguinte, Rect::new(c.x + c.w, c.y, fim_w, c.h))
}
