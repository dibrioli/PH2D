//! ⭐ **O gesto do campo Figma** (`docs/CartoesUI/02_especificacao.md` §3): o Down na pega.
//! O registo e o multiplicador vivem em `state::card_field_ops`; o desenho em `widget::card_field`.

use super::super::{InteractiveState, WidgetEvent, WidgetStore, drag};
use bumpalo::collections::Vec as BumpVec;
use ph2d_a11y::NodeId;

/// O Down na pega: duplo clique repõe o valor de fábrica (um passo de Desfazer); senão semeia o
/// arrasto, que só vira scrub depois do limiar (então um clique no ícone não muda nada).
pub(super) fn down_on_handle(
    store: &mut WidgetStore,
    id: NodeId,
    x: f32,
    y: f32,
    is_double_click: bool,
    events: &mut BumpVec<'_, WidgetEvent>,
) {
    let Some(spec) = store.card_field(id) else {
        return;
    };
    let Some(InteractiveState::NumberInput { value, .. }) = store.get(id) else {
        return;
    };
    let value = *value;
    if is_double_click {
        store.end_number_input_drag();
        if let Some(fabrica) = spec.default {
            let (v, _) = super::apply_chip_value_with_mirror(store, id, fabrica);
            if let Some(InteractiveState::NumberInput { last_committed, .. }) = store.get_mut(id) {
                *last_committed = v;
            }
            store.clear_number_input_mixed(id);
            events.push(WidgetEvent::ValueChanged(id));
            // O campo com barra ligada (as opacidades do Vector): a barra também avisa o painel dono.
            super::push_mirrored_slider_event(store, id, false, events);
        }
        return;
    }
    store.begin_number_input_drag(drag::NumberInputDragState {
        id,
        start_x: x,
        start_y: y,
        start_value: value,
        last_x: x,
        last_y: y,
        step: spec.scrub,
        crossed_threshold: false,
        axis_horizontal: true,
        caret_offset_at_down: 0,
        accum: value,
    });
}

/// Esc a meio do scrub de um campo de cartão: o valor de partida volta (o campo não tem foco, então
/// a reversão do buffer não o alcançaria). `false` = não havia scrub de cartão.
pub(super) fn escape_scrub(store: &mut WidgetStore, events: &mut BumpVec<'_, WidgetEvent>) -> bool {
    let Some(d) = store.number_input_drag() else {
        return false;
    };
    if store.card_field(d.id).is_none() {
        return false;
    }
    store.end_number_input_drag();
    if d.crossed_threshold {
        super::apply_chip_value_with_mirror(store, d.id, d.start_value);
        events.push(WidgetEvent::ValueChanged(d.id));
    }
    true
}

/// ↑ / ↓ num campo de cartão com foco: ± um passo, Shift ×10, Alt ×0,1, preso ao intervalo.
pub(super) fn arrow(
    store: &mut WidgetStore,
    up: bool,
    events: &mut BumpVec<'_, WidgetEvent>,
) -> bool {
    let Some(id) = store.focus_id() else {
        return false;
    };
    let Some(spec) = store.card_field(id) else {
        return false;
    };
    let Some(InteractiveState::NumberInput { value, .. }) = store.get(id) else {
        return false;
    };
    let delta = spec.step * store.card_step_mul() * if up { 1.0 } else { -1.0 };
    let mut v = *value + delta;
    if let Some((lo, hi)) = store.number_scrub_interval(id) {
        v = v.clamp(lo, hi); // CLAMP-OK: (lo, hi) já normalizado por `number_scrub_interval`
    }
    let (v, _) = super::apply_chip_value_with_mirror(store, id, v);
    if let Some(InteractiveState::NumberInput {
        buffer,
        caret,
        last_committed,
        selection_anchor,
        ..
    }) = store.get_mut(id)
    {
        *last_committed = v;
        *buffer = format!("{v:.*}", crate::widget::card_field::decimals(spec.step));
        *caret = buffer.len();
        *selection_anchor = Some(0);
    }
    store.clear_number_input_mixed(id);
    events.push(WidgetEvent::ValueChanged(id));
    // O campo com barra ligada (as opacidades do Vector): a barra também avisa o painel dono.
    super::push_mirrored_slider_event(store, id, false, events);
    true
}
