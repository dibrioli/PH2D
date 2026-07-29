//! **A drenagem do arrasto de `CurvePoint` que é NOSSO** — irmão do [`super`] pelo teto de 600 LOC
//! do painel.
//!
//! ⚠️ **O stash é um canal GLOBAL**, e o braço de `ValueChanged` deste painel roda para TODO id, de
//! todo painel do registry (o `HeroScreen::apply_event` pergunta a todos, visível ou não, e para no
//! primeiro `Consumed`). Drenar antes de perguntar de quem é o gesto **rouba** o arrasto de outro
//! painel — e o `take` é irreversível, então o dono não tem o que drenar.
//!
//! Medido 2026-07-29: este braço comia os arrastos dos punhos do trilho de rampa do painel de VETOR
//! (e devolvia `Consumed`, então o painel dono nem era perguntado), que é por que aqueles punhos não
//! se moviam — sem erro, sem warning, com os gates isolados dos dois painéis verdes. A pergunta
//! virou parte da chamada (`WidgetStore::take_curve_point_drag_if`); o gate de comportamento é
//! `tests/seam_curve_drag_ownership.rs`.

use super::*;

/// Drena o arrasto de curva/gradiente **deste** painel, se o gesto pendente for de um editor dele.
/// Devolve `true` quando consumiu — e `false` deixa o stash INTACTO para o painel dono.
pub(super) fn drain_own_curve_drag(host: &mut dyn PanelHostInternal) -> bool {
    // W4 §3 — a Curves control-point 2-D drag stashed `(parent, ch, idx, x, y)` → drain it,
    // but ONLY if the gesture is ours.
    //
    // ⚠️ The stash is a GLOBAL channel and this arm runs for EVERY `ValueChanged`, so draining
    // before asking whose gesture it is STEALS another panel's drag — and `take` is irreversible.
    // Measured 2026-07-29: this arm ate the vector panel's Gradient Map ramp-handle drags (and
    // returned `Consumed`, so the owning panel was never even asked), which is why those handles
    // would not move. See `tests/seam_curve_drag_ownership.rs`.
    let mine = |parent: ph2d_a11y::NodeId| {
        state::current_layers().is_some_and(|stack| {
            stack.all_ids().any(|l| {
                painter_curve_editor_id(l.0) == parent || painter_gradient_editor_id(l.0) == parent
            })
        })
    };
    if let Some((parent, ch, idx, x, y)) = host.store_mut().take_curve_point_drag_if(mine) {
        if let Some(stack) = state::current_layers() {
            if let Some(layer) = stack
                .all_ids()
                .find(|l| painter_curve_editor_id(l.0) == parent)
            {
                // Remember the touched point so the "−" button knows what to drop.
                state::set_selected_curve_point(Some((layer.0, ch, usize::from(idx))));
                host.bus_mut()
                    .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
                        core_ids::PAINTER_CURVE_EDIT,
                        format!("{}:{ch}:{idx}:{x}:{y}", layer.0),
                    )));
            } else if let Some(layer) = stack
                .all_ids()
                .find(|l| painter_gradient_editor_id(l.0) == parent)
            {
                // Gradient Map stop drag — `x` is the new offset; selecting the
                // dragged stop drives its color sliders + the "−" button.
                state::set_selected_gradient_stop(layer.0, usize::from(idx));
                host.bus_mut()
                    .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
                        core_ids::PAINTER_GRADIENT_EDIT,
                        format!("{}:{idx}:{x}", layer.0),
                    )));
            }
        }
        return true;
    }
    false
}
