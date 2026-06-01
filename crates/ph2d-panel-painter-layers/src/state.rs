//! Painter layers panel state.
//!
//! Espelha o padrão `PainterSidebarPanelState` (ADR-0029 §4.3 + ADR-0040
//! TG-B):
//! - `PainterLayersPanelState` é struct vazia — sem state autoritativo
//!   próprio. O `LayerStack` canônico vive no `PainterTool` (shell-side
//!   ToolRegistry); o panel renderiza um snapshot per-frame.
//! - O shell publica o snapshot das layers via [`set_current_layers`]
//!   ANTES de paint; o paint lê via [`current_layers`] e renderiza as rows.
//! - Eventos sairão via `EditorAction::ToolPanelEvent` (canal genérico
//!   ADR-0040 TG-B) — shell roteia pra `PainterTool::handle_panel_event`.
//!
//! **SCAFFOLD:** o snapshot é o próprio `LayerStack` (já tem `Clone`+`serde`
//! na fundação T3.1). Quando o Implementador preencher as layer rows, ele
//! lê `current_layers()` em `paint`. Se um snapshot mais enxuto (só os
//! campos exibidos: id/name/visible/opacity/blend/thumb-handle) for melhor
//! pra perf, troque o tipo aqui — `set_current_layers`/`current_layers` são
//! os únicos pontos de contato com o shell.

use ph2d_editor_core::zones::Rect;
use ph2d_tool_painter::LayerStack;
use std::cell::{Cell, RefCell};

thread_local! {
    /// Snapshot do `LayerStack` publicado pelo host antes de cada `paint`.
    /// `None` até o primeiro push (panel pinta o placeholder "No layers").
    static CURRENT_LAYERS: RefCell<Option<LayerStack>> = const { RefCell::new(None) };

    /// Última altura de conteúdo scrollable medida (set por paint, lido
    /// pelo orchestrator content_h publish). Paridade com o sidebar.
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
    /// Última altura visível do body (panel rect minus title + paddings).
    static LAST_VISIBLE_H: Cell<f32> = const { Cell::new(0.0) };

    /// The single open blend-mode dropdown to render as a deferred popover on
    /// top of the rows (after the body clip pops): `(layer_id, chip_rect,
    /// current_mode_u8)`. Set during row paint, drained at the end of `paint`.
    /// Enforces one-open-at-a-time. `(u64, Rect, u8)` is `Copy`.
    static PENDING_BLEND_DD: Cell<Option<(u64, Rect, u8)>> = const { Cell::new(None) };
}

/// Stash the open blend dropdown for the deferred popover pass.
pub(crate) fn set_pending_blend_dd(v: Option<(u64, Rect, u8)>) {
    PENDING_BLEND_DD.with(|c| c.set(v));
}

/// Peek the pending blend dropdown (non-consuming) — used to enforce a single
/// open popover when more than one dropdown's store state says "open".
pub(crate) fn pending_blend_dd() -> Option<(u64, Rect, u8)> {
    PENDING_BLEND_DD.with(|c| c.get())
}

/// Take (and clear) the pending blend dropdown for the deferred popover paint.
pub(crate) fn take_pending_blend_dd() -> Option<(u64, Rect, u8)> {
    PENDING_BLEND_DD.with(|c| c.take())
}

/// State per-instance retido do `PainterLayersPanel`. Vazio
/// intencionalmente — o `LayerStack` canônico vive no PainterTool
/// shell-side; o panel renderiza o snapshot per-frame. `Default` exigido
/// pelo bound `Panel::State: Default`.
#[derive(Clone, Debug, Default)]
pub struct PainterLayersPanelState;

/// Publica o snapshot atual do `LayerStack`. Chamado pelo shell uma vez
/// por frame quando o `painter` tool é ativo; pass `None` pra limpar
/// (panel volta ao placeholder).
pub fn set_current_layers(stack: Option<LayerStack>) {
    CURRENT_LAYERS.with(|c| *c.borrow_mut() = stack);
}

/// Lê o snapshot publicado pelo host neste frame. `None` quando o host
/// ainda não pushou (boot, ou Painter inativo) — `paint` pinta o
/// placeholder "No layers".
pub(crate) fn current_layers() -> Option<LayerStack> {
    CURRENT_LAYERS.with(|c| c.borrow().clone())
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
