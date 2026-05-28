//! Painter sidebar paint — T2.1 skeleton (Day-3 smoke).
//!
//! **Day-3 status:** skeleton apenas. Read snapshot + early-return se panel
//! não-visível. Renderização real (sliders + chips + modifier square +
//! undo/redo buttons) chega no Day-7 smoke após `ctx.layout.painter_sidebar`
//! slot ser declarado em `editor-core::screens::layout::PanelLayout`.
//!
//! Pattern espelhado de `ph2d-panel-bgremoval::paint`:
//! - Visibility gate via `PanelHostInternal::panel_visible`.
//! - Stale-rect cleanup on hide.
//! - Snapshot read pra renderização per-frame.
//! - Sections via `paint_sections.rs` (W2.T2.1 Day-7+).

use crate::PainterSidebarPanel;
use crate::ids as panel_ids;
use crate::state::{self, PainterSidebarPanelState, set_last_content_h, set_last_visible_h};
use ph2d_editor_core::panel::{PaintCtx, Panel};

pub(crate) fn paint(_state: &mut PainterSidebarPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(PainterSidebarPanel::ID) {
        // Stale-rect cleanup pra `panel_at` parar de retornar PAINTER_SIDEBAR_PANEL
        // após tool deactivate (mesma convenção BgRemoval).
        ctx.host
            .store_mut()
            .clear_panel_rect(ph2d_editor_core::ids::PAINTER_SIDEBAR_PANEL);
        set_last_content_h(0.0);
        set_last_visible_h(0.0);
        return;
    }

    // Read snapshot publicado pelo shell. Day-3 skeleton: snapshot é lido
    // mas ainda não usado pra renderização (layout slot pendente).
    let _snapshot = state::current_snapshot();
    let _ = panel_ids::SIZE_SLIDER; // silence dead_code warning até wire Day-7

    // **T2.1 Day-7 carry-over:** quando `ctx.layout.painter_sidebar`
    // existir, esta função vai:
    // 1. `paint_panel_surface` + `paint_panel_corner_dot` (chrome canon)
    // 2. `paint_slider_with_chip_layout` x 2 (size + opacity) via
    //    DIRETRIZ v7.0 §5.2 widget canon
    // 3. Modifier square (T2.4) no centro
    // 4. Undo/Redo buttons (T2.2)
    // 5. Publish content_h + visible_h pro scroll bounds
    //
    // Hoje (Day-3): skeleton compila + Panel trait impl válido.
    set_last_content_h(0.0);
    set_last_visible_h(0.0);
}
