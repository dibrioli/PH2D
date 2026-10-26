//! A costura da P6 no painel PIXEL: o registo das opções de selecção, dos campos da moldura solta
//! e dos botões dos menus Select/Edit/Sprite, e o pedido de cada um (filho de [`crate::seam`]).

use crate::ids;
use crate::state::with_current;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::pixel_panel::{PixelOptions, PixelRequest};
use ph2d_editor_core::widget::{ButtonState, TextInputState};

pub(crate) fn populate(store: &mut WidgetStore) {
    let segmentos = ids::SELECT_MODES
        .iter()
        .map(|(_, id)| *id)
        .chain(ids::RESAMPLES.iter().map(|(_, id)| *id));
    let botoes = ids::SELECT_ACTIONS
        .iter()
        .chain(&ids::EDIT_ACTIONS)
        .chain(&ids::SPRITE_ACTIONS)
        .map(|(id, ..)| *id);
    for id in segmentos.chain(botoes) {
        let state = ButtonState::Normal;
        store.register(id, InteractiveState::Button { state });
    }
    for (_, id, (min, max)) in ids::XFORM_FIELDS {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: "0".to_owned(),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        store.set_number_range(id, min, max, 1.0);
    }
}

/// As opções com o segmento `id` (modo de juntar, amostragem da moldura), se é um destes.
pub(crate) fn com_segmento(mut o: PixelOptions, id: NodeId) -> Option<PixelOptions> {
    if let Some((m, _)) = ids::SELECT_MODES.iter().find(|(_, s)| *s == id) {
        o.select_mode = *m;
        return Some(o);
    }
    let (r, _) = ids::RESAMPLES.iter().find(|(_, s)| *s == id)?;
    o.resample = *r;
    Some(o)
}

/// O pedido de um campo da moldura: o valor inteiro, só com células levantadas.
pub(crate) fn campo_da_moldura(id: NodeId, v: Option<f64>) -> Option<PixelRequest> {
    let (field, _, (min, max)) = ids::XFORM_FIELDS.iter().find(|(_, n, _)| *n == id)?;
    let v = v.filter(|v| v.is_finite())?;
    #[expect(
        clippy::cast_possible_truncation,
        reason = "preso à faixa do campo (±65535) antes da conversão"
    )]
    let value = ph2d_editor_core::math::safe_clamp_f64(v.round(), *min, *max) as i32;
    with_current(|i| i.transform.is_some()).then_some(PixelRequest::TransformField {
        field: *field,
        value,
    })
}
