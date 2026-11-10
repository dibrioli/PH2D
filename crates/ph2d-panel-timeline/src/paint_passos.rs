//! Passos da `paint` da Timeline (tecto de LOC): seguir o cursor, fechar arrastos sem linha e o pé
//! da fita nos cartões.

use super::*;

/// Page-follow ONLY while playing: the view chases the playhead during playback, but a manual
/// pan/zoom while paused is never yanked back. Nos cartões (doc 15 onda 2) não persegue o cursor
/// DEPOIS do fim do clipe: lá a fita é a zona morta listrada, e a vista enquadrada ao abrir ficava
/// trocada por uma fita vazia.
pub(super) fn segue_o_cursor(
    view_start: f64,
    span: f64,
    cartoes: bool,
    snapshot: &ph2d_timeline::TimelineViewSnapshot,
) -> f64 {
    let alem_do_fim = cartoes
        && snapshot.view_length_seconds > 0.0
        && snapshot.time_seconds > snapshot.view_length_seconds;
    if snapshot.playing
        && span > 0.0
        && !alem_do_fim
        && (snapshot.time_seconds < view_start || snapshot.time_seconds >= view_start + span)
    {
        snapshot.time_seconds.max(0.0)
    } else {
        view_start
    }
}

/// A handle or anchor drag whose row got culled never reached its `resolve_drag`: close the undo
/// bracket here so the next atomic edit is not silently swallowed into it.
pub(super) fn fecha_arrastos_sem_linha(state: &mut TimelinePanelState) {
    if state.handle_drag.is_some_and(|d| d.ending) {
        state.handle_drag = None;
        state::push_intent(ph2d_timeline::TimelineIntent::EndEdit);
    }
    if state.anchor_drag.as_ref().is_some_and(|d| d.ending) {
        state.anchor_drag = None;
        state::push_intent(ph2d_timeline::TimelineIntent::EndEdit);
    }
}

/// O pé da fita nos cartões: a legenda das barras (só na aba que tem chaves) e a moldura da área
/// das trilhas (maquete B1, sempre), que o Auto Key a gravar acende a vermelho.
pub(super) fn pe_da_fita(
    ctx: &mut PaintCtx,
    theme: Theme,
    tab: crate::tab::Tab,
    g: &geom::Geom,
    view: graph::TimeView,
    snapshot: &ph2d_timeline::TimelineViewSnapshot,
) {
    if tab.shows_keys() && !snapshot.tracks.is_empty() {
        crate::legenda::paint(ctx, theme, crate::tracks::lane_rect(g, view));
    }
    let (r, cor) = crate::linhas::moldura(g, snapshot.auto_key);
    // FRAME-RAW-OK: a moldura da fita É a mensagem — acende a vermelho com o Auto Key a gravar;
    // pela porta do tema, a pele plana apagá-la-ia e o sinal de gravação sumiria.
    ph2d_editor_core::paint::stroke_rounded_rect(
        ctx.scene,
        r,
        ph2d_editor_core::panel::cards::banda::metrics::CARD_RADIUS,
        ph2d_tokens::TIMELINE_FRAME_PX,
        crate::folha_tinta::tinta(cor, theme),
    );
}
