//! Painter sidebar panel state.
//!
//! Espelha o padrão `BgRemovalPanelState` (ADR-0029 §4.3 + ADR-0040 TG-B):
//! - `PainterSidebarPanelState` é struct vazia — sem state autoritativo
//!   próprio. PainterTool no shell ToolRegistry mantém o canon.
//! - Shell publica [`PainterUiSnapshot`] via [`set_current_painter_snapshot`]
//!   ANTES de paint; o paint lê e renderiza sliders posicionados.
//! - Eventos saem via `EditorAction::ToolPanelEvent` (ADR-0040 TG-B canal
//!   genérico) — shell roteia pra `PainterTool::handle_panel_event` que
//!   despacha via `apply_ui_edit::PainterUiEdit`.

use ph2d_tool_painter::PainterUiSnapshot;
use std::cell::{Cell, RefCell};

thread_local! {
    /// Snapshot normalizado publicado pelo host antes de cada `paint`.
    /// `None` até primeiro push (panel paints defaults).
    static CURRENT_SNAPSHOT: RefCell<Option<PainterUiSnapshot>> = const { RefCell::new(None) };

    /// Última altura de conteúdo scrollable medida (set por paint, lido
    /// pelo orchestrator content_h publish). Paridade com BgRemoval.
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
    /// Última altura visível do body (panel rect minus title + paddings).
    static LAST_VISIBLE_H: Cell<f32> = const { Cell::new(0.0) };
}

/// State per-instance retained do `PainterSidebarPanel`. Vazio
/// intencionalmente — params canônicos vivem no PainterTool shell-side;
/// o panel renderiza snapshot per-frame. `Default` exigido pelo bound
/// `Panel::State: Default`.
#[derive(Clone, Debug, Default)]
pub struct PainterSidebarPanelState;

/// Publica o snapshot atual normalizado. Chamado pelo shell uma vez por
/// frame quando o `painter` tool é ativo; pass `None` pra limpar.
pub fn set_current_painter_snapshot(snapshot: Option<PainterUiSnapshot>) {
    CURRENT_SNAPSHOT.with(|c| *c.borrow_mut() = snapshot);
}

/// Lê o snapshot publicado pelo host neste frame, fallback pra default
/// quando host não pushou ainda.
pub(crate) fn current_snapshot() -> PainterUiSnapshot {
    CURRENT_SNAPSHOT.with(|c| c.borrow().clone().unwrap_or_default())
}

pub fn last_content_h() -> f32 {
    LAST_CONTENT_H.with(|c| c.get())
}

pub fn last_visible_h() -> f32 {
    LAST_VISIBLE_H.with(|c| c.get())
}

pub(crate) fn set_last_content_h(v: f32) {
    LAST_CONTENT_H.with(|c| c.set(v));
}

pub(crate) fn set_last_visible_h(v: f32) {
    LAST_VISIBLE_H.with(|c| c.set(v));
}
