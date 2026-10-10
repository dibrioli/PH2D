//! ⭐ O interruptor **«Autoplay»** do grupo de clipes (doc ObjectsAndComponents 01 §14, item 1 — o
//! *Autoplay* do Godot): o clipe em edição toca sozinho quando a cena corre (o Play da barra de cima).
//! Liga e desliga em qualquer clipe; ligar um desliga os irmãos; por omissão, nenhum (dono, 10/10).
//! Existe em toda animação — a da Cena também (o clipe dela toca quando o Arrange está vazio).

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, PaintCtx};
use ph2d_timeline::{TimelineIntent, TimelineViewSnapshot};
use ph2d_tokens::{Spacing, Theme, TypeToken};

use crate::ids::TIMELINE_CLIP_AUTOPLAY;

/// A chave do rótulo — da família dos interruptores da barra (`transport_labels`).
pub(crate) const LABEL_KEY: &str = "panel.timeline.autoplay";

/// O clipe em edição está marcado?
fn ligado(snap: &TimelineViewSnapshot) -> bool {
    snap.animation.entry == Some(snap.active_clip)
}

/// A largura que o interruptor acrescenta ao grupo — a mesma conta do pintor.
pub(crate) fn extra(_snap: &TimelineViewSnapshot, label_col: f32) -> f32 {
    let pad = Spacing::Xs.px();
    pad + label_col + pad + TypeToken::Xl3.px() + pad + Spacing::Sm.px() * 0.5
}

/// Pinta o interruptor em `(x, y)`, ligado quando o clipe em edição é o do Autoplay.
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    at: (f32, f32),
    snap: &TimelineViewSnapshot,
    label_col: f32,
) {
    crate::transport::toggle(
        ctx,
        theme,
        at.0,
        at.1,
        TIMELINE_CLIP_AUTOPLAY,
        ph2d_i18n::tr(LABEL_KEY),
        ligado(snap),
        label_col,
    );
}

pub(crate) fn owns(ev: &WidgetEvent) -> bool {
    matches!(*ev, WidgetEvent::Toggled(id) if id == TIMELINE_CLIP_AUTOPLAY)
}

/// O clique alterna o Autoplay do clipe em edição (um passo do undo da timeline).
pub(crate) fn apply_event() -> EventOutcome {
    let snap = crate::state::current_snapshot();
    crate::state::push_intent(TimelineIntent::SetAutoplay {
        index: snap.active_clip,
        on: !ligado(&snap),
    });
    EventOutcome::Consumed
}
