//! ⭐ O interruptor **«Autoplay»** do grupo de clipes (doc ObjectsAndComponents 01 §14, item 1 —
//! escolha 4.1 do dono, o *Autoplay* do Godot): o clipe em edição é a ENTRADA da animação dele, o
//! que toca sozinho no Play.
//!
//! Só existe na vista da animação de um OBJECTO (a Cena toca o Arrange e não tem entrada). É um
//! RÁDIO: há sempre uma entrada (por omissão, o 1.º clipe), então no clipe que já o é o interruptor
//! pinta-se ligado e MORTO (sem clique — desligá-lo não teria resposta); nos outros, ligá-lo faz
//! deles a entrada.

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, PaintCtx};
use ph2d_timeline::{TimelineIntent, TimelineViewSnapshot};
use ph2d_tokens::{Spacing, Theme, TypeToken};

use crate::ids::TIMELINE_CLIP_AUTOPLAY;

/// A chave do rótulo — da família dos interruptores da barra (`transport_labels`).
pub(crate) const LABEL_KEY: &str = "panel.timeline.autoplay";

fn visivel(snap: &TimelineViewSnapshot) -> bool {
    snap.animation.entry.is_some()
}

/// A largura que o interruptor acrescenta ao grupo (zero na Cena) — a mesma conta do pintor.
pub(crate) fn extra(snap: &TimelineViewSnapshot, label_col: f32) -> f32 {
    if !visivel(snap) {
        return 0.0;
    }
    let pad = Spacing::Xs.px();
    pad + label_col + pad + TypeToken::Xl3.px() + pad + Spacing::Sm.px() * 0.5
}

/// Pinta o interruptor em `(x, y)`; vivo só quando o clipe em edição ainda não é a entrada.
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    at: (f32, f32),
    snap: &TimelineViewSnapshot,
    label_col: f32,
) {
    let Some(entrada) = snap.animation.entry else {
        return;
    };
    let on = entrada == snap.active_clip;
    crate::transport::toggle_vivo(
        ctx,
        theme,
        at,
        TIMELINE_CLIP_AUTOPLAY,
        ph2d_i18n::tr(LABEL_KEY),
        on,
        label_col,
        !on,
    );
}

pub(crate) fn owns(ev: &WidgetEvent) -> bool {
    matches!(*ev, WidgetEvent::Toggled(id) if id == TIMELINE_CLIP_AUTOPLAY)
}

/// Ligar = este clipe passa a ser a entrada (um passo do undo da timeline).
pub(crate) fn apply_event() -> EventOutcome {
    let snap = crate::state::current_snapshot();
    if snap.animation.entry.is_some_and(|e| e != snap.active_clip) {
        crate::state::push_intent(TimelineIntent::SetEntryClip {
            index: snap.active_clip,
        });
    }
    EventOutcome::Consumed
}
