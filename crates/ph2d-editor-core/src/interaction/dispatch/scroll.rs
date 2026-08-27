//! Wheel + scrollbar dispatch helpers.
//!
//! Extracted from [`super`] (Track A8). Two responsibilities:
//!
//! 1. [`dispatch_wheel`] — public entry point for wheel / trackpad
//!    events. Finds the panel under the cursor, adjusts its
//!    `panel_scroll`, and clamps against the painter-published
//!    `content_h` / `visible_h` so wheeling past the end doesn't
//!    produce a 1-frame jump.
//! 2. [`scrollbar_panel_for_id`] — maps a scrollbar thumb's
//!    [`ph2d_a11y::NodeId`] back to the panel it scrolls. Pure
//!    routing table; hosts that add new scrollable panels extend
//!    the match here.

use super::super::{InteractiveState, WidgetEvent, WidgetStore};
use bumpalo::Bump;
use bumpalo::collections::Vec as BumpVec;
use ph2d_a11y::NodeId;

/// Wheel / trackpad scroll. Finds the panel under `(x, y)` via
/// [`WidgetStore::panel_at`] and adjusts that panel's
/// `panel_scroll` by `delta_y`. Caller (painter) is responsible
/// for clamping the offset against the panel's `content_h` —
/// dispatch only deltas, doesn't know content height.
pub fn dispatch_wheel<'frame>(
    store: &mut WidgetStore,
    event: ph2d_host::WheelEvent,
    arena: &'frame Bump,
) -> &'frame [WidgetEvent] {
    let events: BumpVec<'frame, WidgetEvent> = BumpVec::new_in(arena);
    // Motion Nodes M0.T3 — a wheel over a graph surface is an anchored zoom,
    // consumed BEFORE any panel scroll so the graph zooms instead of scrolling
    // the panel underneath. The panel drains + applies the accumulated zoom.
    if let Some(surface) = store.graph_surface_at(event.x, event.y) {
        store.add_graph_zoom(surface, event.delta_y, event.x, event.y);
        return events.into_bump_slice();
    }
    // W2.E6 — a wheel over the timeline's dope-sheet drives its own view,
    // consumed BEFORE any panel scroll:
    //
    //   roda simples  -> ROLA as linhas de propriedade
    //   Ctrl/Cmd+roda -> zoom ancorado do eixo do tempo
    //   Shift+roda    -> pan horizontal do eixo do tempo
    //   eixo-x (trackpad) -> pan, sempre
    //
    // ⚠️ **A roda simples ZOOMAVA, e era a única superfície do app em que a roda sobre o corpo de
    // um painel não rolava o corpo.** Reportado assim mesmo por Enio (2026-08-15): *«a timeline
    // está sem o scroll … para as propriedades animadas»*. O scroll existia — em `Shift+roda`, que
    // ninguém descobre — e a barra existia e era **invisível** (o polegar era `border` sobre uma
    // pista `bg-2`: ΔL 0.11 numa faixa de 10 px). Duas metades do mesmo report, duas causas
    // distintas.
    //
    // ⚠️ **A lei escolhida é a do RESTO DO APP, não a de um editor de nós.** Este painel é uma
    // dope-sheet com uma LISTA de propriedades — a família do After Effects / Premiere, onde a roda
    // percorre a lista —, e não um grafo, que é de onde a convenção antiga foi herdada (o
    // comentário do `apply_wheel` ainda diz *"a mesma sensibilidade que o motion graph usa"*).
    // Vinte e quatro painéis deste app respondem *«a roda rola o corpo»*; a timeline ser a única
    // excepção **é** o report.
    //
    // ⚠️ **O `Alt` não entra nesta tabela de propósito:** o KDE rouba-o (precedente já pago pelo
    // `PH2D_STAGGER_SMOKE`, que usa Ctrl pela mesma razão). O `Ctrl+roda` é o modificador de zoom
    // universal (browser, VS Code, Figma) e o `Shift+roda` o de horizontal — nenhum dos dois é
    // invenção nossa.
    //
    // ⚠️ **O preço, nomeado:** com poucas propriedades as linhas cabem e a roda simples não faz
    // nada, exactamente como em qualquer outro painel cuja lista cabe. Quem zooma o tempo passa a
    // segurar Ctrl.
    if let Some(surface) = store.timeline_surface_at(event.x, event.y) {
        let m = event.modifiers;
        let (zoom, pan, scroll) = if m.ctrl || m.meta {
            (event.delta_y, event.delta_x, 0.0)
        } else if m.shift {
            (0.0, event.delta_y, 0.0)
        } else {
            (0.0, event.delta_x, event.delta_y)
        };
        store.add_timeline_wheel(surface, zoom, pan, scroll, event.x);
        return events.into_bump_slice();
    }
    // An OPEN dropdown popover scrolls first — it floats on top of any panel, and its rect lives in a
    // dedicated slot (not `panel_rects`) so `panel_at` isn't polluted. Its scroll value + heights use
    // the `panel_scroll`/`panel_*_h` tables keyed by the dropdown id.
    if let Some((dd, rect)) = store.dropdown_popover()
        && rect.contains(event.x, event.y)
        && matches!(
            store.get(dd),
            Some(InteractiveState::Dropdown { open: true, .. })
        )
    {
        let mut next = (store.panel_scroll_target(dd) - event.delta_y).max(0.0);
        if let Some(content_h) = store.panel_content_h(dd) {
            let visible_h = store.panel_visible_h(dd).unwrap_or(0.0);
            next = next.min((content_h - visible_h).max(0.0));
        }
        store.set_panel_scroll(dd, next);
        return events.into_bump_slice();
    }
    if let Some(panel) = store.panel_at(event.x, event.y) {
        // ⚠️ O ALVO, nunca o vivo: girar depressa sobre uma posição em voo anda menos do que
        //    o dedo pediu.
        let cur = store.panel_scroll_target(panel);
        // delta_y > 0 from winit means "scroll forward" / content
        // moves up. We store offset as "how far down content
        // pretends to be" — so positive delta increments the
        // offset (showing content further down).
        let mut next = (cur - event.delta_y).max(0.0);
        // Clamp at the upper bound when the painter has published a
        // content_h for this panel. Without this, wheeling past the
        // last element pushes `next` arbitrarily high; the next
        // paint pass clamps it back, producing a 1-frame "jump"
        // (the user's "saltos indesejados se rodamos a roda no fim").
        if let Some(content_h) = store.panel_content_h(panel) {
            // Prefer the painter-published visible_h (exact body
            // height); fall back to `panel.h - 60` only when the
            // painter hasn't seeded one yet (first frame).
            let visible_h = store.panel_visible_h(panel).unwrap_or_else(|| {
                store
                    .panel_rect(panel)
                    .map(|r| (r.h - 60.0).max(0.0))
                    .unwrap_or(0.0)
            });
            let max_scroll = (content_h - visible_h).max(0.0);
            if next > max_scroll {
                next = max_scroll;
            }
        }
        store.set_panel_scroll(panel, next);
    }
    events.into_bump_slice()
}

/// Maps a scrollbar thumb's hit id back to the panel it scrolls.
/// Returns `None` for non-scrollbar ids. Keeps the panel↔scrollbar
/// mapping in one place — hosts that add new scrollable panels
/// extend this match.
pub(crate) fn scrollbar_panel_for_id(id: NodeId) -> Option<NodeId> {
    use crate::ids;
    if id == crate::widget::INSPECTOR_SCROLLBAR_ID {
        Some(ids::INSP_PANEL)
    } else if id == crate::widget::HIERARCHY_SCROLLBAR_ID {
        Some(ids::HIER_PANEL)
    } else if id == crate::widget::GALLERY_SCROLLBAR_ID {
        Some(ids::GAL_PANEL)
    } else if id == crate::widget::GRID_SETTINGS_SCROLLBAR_ID {
        Some(crate::ids::GS_PANEL)
    } else if id == crate::widget::COLOR_EQUALIZATION_SCROLLBAR_ID {
        Some(crate::ids::CEQ_PANEL)
    } else if id == crate::widget::BG_REMOVAL_SCROLLBAR_ID {
        Some(crate::ids::BGR_PANEL)
    } else if id == crate::widget::PADDING_SCROLLBAR_ID {
        Some(crate::ids::PAD_PANEL)
    } else if id == crate::widget::UPSCALE_SCROLLBAR_ID {
        Some(crate::ids::UPS_PANEL)
    } else if id == crate::widget::EQUALIZE_SIZES_SCROLLBAR_ID {
        Some(crate::ids::EQS_PANEL)
    } else if id == crate::widget::PAINTER_LAYERS_SCROLLBAR_ID {
        Some(ids::PAINTER_LAYERS_PANEL)
    } else if id == crate::widget::PAINTER_BRUSH_STUDIO_SCROLLBAR_ID {
        Some(ids::PAINTER_BRUSH_STUDIO_PANEL)
    } else if id == crate::widget::AUDIO_MIXER_SCROLLBAR_ID {
        Some(ids::AUDIO_MIXER_PANEL)
    } else if id == crate::widget::VECTOR_SCROLLBAR_ID {
        Some(ids::VECTOR_PANEL)
    } else if id == crate::widget::AUDIO_EDITOR_SCROLLBAR_ID {
        Some(ids::AUDIO_EDITOR_PANEL)
    } else if id == crate::widget::FLIP_SCROLLBAR_ID {
        Some(ids::FLIP_PANEL)
    } else if id == crate::widget::PHYSICS_SCROLLBAR_ID {
        Some(ids::PHYSICS_PANEL)
    } else if id == crate::widget::WET_TUNING_SCROLLBAR_ID {
        Some(ids::WET_TUNING_PANEL)
    } else if id == crate::widget::MOTION_PARAMS_SCROLLBAR_ID {
        Some(ids::MOTION_PARAMS_PANEL)
    } else if id == crate::widget::TOKENS_SCROLLBAR_ID {
        Some(ids::TOKENS_PANEL)
    } else if id == crate::widget::AUTHORED_SCROLLBAR_ID {
        Some(ids::AUTHORED_PANEL)
    } else if id == crate::widget::SCULPT3D_SCROLLBAR_ID {
        Some(ids::SCULPT3D_PANEL)
    } else if id == crate::widget::MODEL3D_SCROLLBAR_ID {
        Some(ids::MODEL3D_PANEL)
    } else {
        None
    }
}
