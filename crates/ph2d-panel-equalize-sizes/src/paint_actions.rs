//! **AS DUAS LINHAS DE AÇÃO do painel Equalize Sizes** — `Reset to Defaults` e o par
//! `Cancel / Apply`.
//!
//! # Por que um ficheiro e não uma função irmã
//!
//! ⚠️ **Dois tetos de LOC, e eles medem grandezas diferentes.** O `paint_body_sections` estava a
//! **253** linhas contra um tecto de 200; extrair esta cauda *dentro do mesmo ficheiro* curou-o e
//! empurrou o **ficheiro** para 608 contra um tecto de 600. É literalmente o caminho que a memória
//! `feedback_a_fn_cap_and_a_file_cap_measure_different_things` descreve, percorrido em directo.
//!
//! *O corte que cura os dois é para o irmão.*
//!
//! O corte é por responsabilidade e não por número de linhas: no `paint.rs` ficam os **parâmetros**
//! da operação; aqui ficam os **verbos** que a confirmam ou desfazem. Eles não partilham estado
//! nenhum com as secções — só o `y` que já vinha a descer.

use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::widget::{Button, ButtonKind, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{Spacing, Theme};

use crate::ids;

/// **As duas linhas de ação do painel** — `Reset to Defaults` e o par `Cancel / Apply`.
///
/// ⚠️ Saiu do [`paint_body_sections`] em 2026-08-20 por medição: ele estava a **253** linhas contra
/// um tecto de 200, e a regra registada deste projeto é **cortar, nunca alargar a allowlist**
/// (`feedback_loc_cap_split_not_allowlist_and_fmt_reexpands`).
///
/// O corte é por responsabilidade e não por linha: acima ficam os **parâmetros** da operação;
/// aqui ficam os **verbos** que a confirmam ou desfazem. Elas não partilham estado nenhum com as
/// secções — só o `y` que já vinha a descer.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_action_rows(
    scene: &mut ph2d_vector::VectorScene,
    text_system: &mut ph2d_text::TextSystem,
    theme: Theme,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    inner_x: f32,
    inner_w: f32,
    row_h: f32,
    row_gap: f32,
    y_in: f32,
) -> f32 {
    let mut y = y_in;
    // ── Reset (ghost, full width) row ──────────────────────────────
    let btn_gap = Spacing::Sm.px();
    let reset_rect = Rect::new(inner_x, y, inner_w, row_h);
    let reset_state = store.button_visual(ids::EQS_RESET);
    let reset = Button::new(ids::EQS_RESET, "Reset to Defaults")
        .kind(ButtonKind::Default)
        .visual(reset_state);
    paint_button(&reset, reset_rect, scene, text_system, theme);
    hit_index.register(ids::EQS_RESET, reset_rect);
    y += row_h + row_gap;

    // ── Cancel + Apply row ─────────────────────────────────────────
    let half_btn = ((inner_w - btn_gap) * 0.5).max(0.0);
    let cancel_rect = Rect::new(inner_x, y, half_btn, row_h);
    let cancel_state = store.button_visual(ids::EQS_CANCEL);
    let cancel = Button::new(ids::EQS_CANCEL, "Cancel")
        .kind(ButtonKind::Default)
        .visual(cancel_state);
    paint_button(&cancel, cancel_rect, scene, text_system, theme);
    hit_index.register(ids::EQS_CANCEL, cancel_rect);
    let apply_rect = Rect::new(inner_x + half_btn + btn_gap, y, half_btn, row_h);
    let apply_state = store.button_visual(ids::EQS_APPLY);
    let apply = Button::new(ids::EQS_APPLY, "Apply")
        .kind(ButtonKind::Accent)
        .visual(apply_state);
    paint_button(&apply, apply_rect, scene, text_system, theme);
    hit_index.register(ids::EQS_APPLY, apply_rect);
    y += row_h;
    y
}
