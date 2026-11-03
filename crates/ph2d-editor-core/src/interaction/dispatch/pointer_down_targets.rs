//! O que um `Down` pergunta ao alvo antes de agir: é um alvo de COR (reabre o seletor em vez de
//! o fechar)? caiu no ✕ de limpar de uma Combobox?

use crate::interaction::{InteractiveState, WidgetStore};
use crate::zones::Rect;

/// True iff a `Down` on `id` should RE-OPEN the colour picker (a colour-target
/// widget) rather than dismiss it. Mix of legacy numeric ranges + the hashed
/// Painter colour thumb.
pub(super) fn is_color_target_id(id: ph2d_a11y::NodeId) -> bool {
    let v = id.0;
    // W2.T2.3: the Painter color thumb is a hashed id (outside the raw
    // legacy range), so it can't be matched arithmetically — clicking it
    // re-opens the picker rather than dismissing it.
    (360..=369).contains(&v) || v == 328 || id == crate::ids::PAINTER_COLOR_THUMB
}

/// Detect a Down landing on the Combobox's inline clear-✕ icon. When
/// the widget at `id` is a Combobox with a non-empty query and the
/// click coordinates fall inside `Combobox::clear_button_rect(host)`,
/// wipe the query/caret/selection and return true. Returns false in
/// every other case (non-Combobox, empty query, click outside the X).
/// The caller emits `WidgetEvent::TextChanged(id)` on a true return.
pub(super) fn clear_combobox_if_button_hit(
    store: &mut WidgetStore,
    id: ph2d_a11y::NodeId,
    host: Rect,
    click_x: f32,
    click_y: f32,
) -> bool {
    use crate::widget::{Combobox, ComboboxOption};
    // Build a throw-away Combobox snapshot just to reuse the
    // widget-side `clear_button_rect` math — keeps geometry in
    // exactly one place (the widget). Cost: one empty Vec alloc per
    // Down event, dwarfed by the bumpalo event arena cost.
    let (query, state) = match store.get(id) {
        Some(InteractiveState::Combobox { query, state, .. }) => (query.clone(), *state),
        _ => return false,
    };
    let probe = Combobox::new(id, "", Vec::<ComboboxOption>::new())
        .query(query)
        .state(state);
    let Some(btn_rect) = probe.clear_button_rect(host) else {
        return false;
    };
    if !btn_rect.contains(click_x, click_y) {
        return false;
    }
    if let Some(InteractiveState::Combobox {
        query,
        caret,
        selection_anchor,
        ..
    }) = store.get_mut(id)
    {
        query.clear();
        *caret = 0;
        *selection_anchor = None;
        return true;
    }
    false
}
