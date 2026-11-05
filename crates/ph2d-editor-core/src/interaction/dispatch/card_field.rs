//! ⭐ **O gesto do campo Figma** (`docs/CartoesUI/02_especificacao.md` §3): o Down na pega.
//! O registo e o multiplicador vivem em `state::card_field_ops`; o desenho em `widget::card_field`.

use super::super::{InteractiveState, WidgetEvent, WidgetStore, drag};
use bumpalo::collections::Vec as BumpVec;
use ph2d_a11y::NodeId;

/// O campo passa a escrever, com o valor selecionado: o que se digitar substitui-o.
fn entra_em_edicao(store: &mut WidgetStore, id: NodeId, events: &mut BumpVec<'_, WidgetEvent>) {
    if store.focus_id() == Some(id) {
        return;
    }
    store.set_focus(Some(id));
    super::init_number_buffer(store, id);
    events.push(WidgetEvent::Focus(id));
}

/// O Up de um Down na pega que não chegou a scrub: o clique no ícone não muda o valor e abre a escrita
/// (sem isto, o que se digitava a seguir ia para os atalhos do app). No corpo do campo o foco já lá está.
pub(super) fn click_without_scrub(
    store: &mut WidgetStore,
    id: NodeId,
    events: &mut BumpVec<'_, WidgetEvent>,
) {
    if store.card_field(id).is_some() {
        entra_em_edicao(store, id, events);
    }
}

/// O Down na pega: duplo clique repõe o valor de fábrica (um passo de Desfazer) e deixa o campo a
/// escrever; senão semeia o arrasto, que só vira scrub depois do limiar.
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
        entra_em_edicao(store, id, events);
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
        // O painel que engole o evento do chip (Physics) só ouve a barra: sem isto o Esc mostrava o
        // valor velho e o mundo ficava com o arrastado.
        super::push_mirrored_slider_event(store, d.id, false, events);
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
    // O limite da seta de `PH2D_CARDS=0` (`apply_number_stepper_if_hit`): a faixa manda no passo; a taxa
    // de arrasto só solta o scrub.
    let limite = store
        .number_range(id)
        .map(|(a, b, _)| (a.min(b), a.max(b)))
        .or_else(|| store.number_scrub_interval(id));
    if let Some((lo, hi)) = limite {
        v = v.clamp(lo, hi); // CLAMP-OK: (lo, hi) já normalizado acima ou por `number_scrub_interval`
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
        let casas = crate::widget::card_field::decimals(spec.step);
        *buffer = crate::widget::format_number_com_casas(v, Some(casas));
        *caret = buffer.len();
        *selection_anchor = Some(0);
    }
    store.clear_number_input_mixed(id);
    events.push(WidgetEvent::ValueChanged(id));
    // O campo com barra ligada (as opacidades do Vector): a barra também avisa o painel dono.
    super::push_mirrored_slider_event(store, id, false, events);
    true
}
