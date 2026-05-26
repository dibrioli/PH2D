//! `PainterTool` — impl Tool + RasterEditTool (skeleton W1 T1.1).
//!
//! T1.1 status: **stub neutro** — `id()` / `label()` / `icon_slug()` /
//! `build_panel()` retornam valores hardcoded; `as_raster_edit_mut()` retorna
//! `Some(self)` com RasterEditTool stub que ecoa source inalterado (zero
//! deferral via exception allowlist; regra perfeição 2026-05-26). T1.4+
//! substitui o stub por brush engine GPU stamp pipeline real.
//!
//! W1 day 1 smoke: `cargo run -p ph2d-host-desktop` abre app sem panic.
//! Pill aparece em T1.2 (chrome handler).

use ph2d_editor_core::floating_panel::{FloatingPanel, ToolId};
use ph2d_editor_core::tool::{RasterEditTool, Tool};

use crate::params::PainterParams;

/// Painter — sucessor do Procreate. Stateful workhorse tool.
///
/// Cascata W0 (ADR-0043..0053) congelou caps e contratos. T1.1 entrega
/// skeleton suficiente para registry + boot sem panic; T1.2+ adiciona pill
/// + chrome takeover; T1.3+ adiciona brush engine real.
#[derive(Default)]
pub struct PainterTool {
    pub params: PainterParams,
    // ADR-0043 §2.2 estrutura canônica:
    // source RasterEditTool buffer (RGBA8 straight alpha).
    source_rgba: Vec<u8>,
    source_size: (u32, u32),
    preview_dirty: bool,
    pending_commit: bool,
    // W1+ T1.4: stroke_state: StrokeBuilder (-> ph2d-painter-stroke)
}

impl Tool for PainterTool {
    fn id(&self) -> ToolId {
        ToolId::new("painter")
    }

    fn label(&self) -> &str {
        "Painter"
    }

    fn icon_slug(&self) -> &str {
        "painter"
    }

    fn build_panel(&self) -> FloatingPanel {
        // T1.1 stub: panel vazio com title só. Sidebar Procreate-style
        // ship em W2 (ph2d-panel-painter trait-driven; ADR-0029 padrão).
        FloatingPanel::new(self.id(), "Painter")
    }

    fn on_activate(&mut self) {
        // T1.2 implementará: pushar `painter_active = true` no HeroScreen
        // para acionar takeover (suprime chrome PH2D normal; ADR-0043 §1.1).
        self.params.takeover_active = true;
        // T1.2 smoke 🟦 Day 3 do plano §4: clicar pill → terminal mostra ativação.
        // PH2D não tem convenção de logging consolidada; `println!` é o canon
        // de smoke (bgremoval e outros tools idem). Migração para log/tracing
        // proper acontece quando ADR de logging cross-projeto ratificar.
        println!("painter activated");
    }

    fn on_deactivate(&mut self) {
        self.params.takeover_active = false;
        println!("painter deactivated");
    }

    fn handle_panel_event(&mut self, _event: ph2d_editor_core::tool::PanelEvent) {
        // T1.3+ (sidebar real, ph2d-panel-painter W2): mapeia PanelEvent
        // (NodeId) → PainterUiEdit semântico via `apply_ui_edit`. Vide
        // ADR-0043 §2.3 + params.rs::PainterUiEdit.
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_raster_edit_mut(&mut self) -> Option<&mut dyn RasterEditTool> {
        Some(self)
    }

    fn is_default(&self) -> bool {
        // Brush proto-tool (editor-core) continua default em W1. Quando
        // Painter substituir o proto (T1.X close W1), is_default() flipa
        // para true e brush proto é deletado.
        false
    }
}

/// `RasterEditTool` skeleton — T1.1 stub. Métodos retornam estado vazio
/// até T1.4+ implementar brush engine GPU stamp pipeline.
///
/// Decisão estrutural (regra "perfeição desde início" 2026-05-26):
/// implementar a trait imediatamente, com semântica neutra (retorna source
/// inalterado / sem pending). Alternativa exception-allowlist foi rejeitada
/// — seria deferral via canal lateral.
impl RasterEditTool for PainterTool {
    fn set_source(&mut self, rgba: Vec<u8>, width: u32, height: u32) {
        self.source_rgba = rgba;
        self.source_size = (width, height);
        self.preview_dirty = true;
    }

    fn current_preview(&mut self) -> Option<(&[u8], u32, u32)> {
        if !std::mem::take(&mut self.preview_dirty) || self.source_rgba.is_empty() {
            return None;
        }
        // T1.1 stub: returns source unchanged (no brush strokes applied yet).
        // T1.4+ ship stamp pipeline GPU para compor preview real.
        Some((&self.source_rgba, self.source_size.0, self.source_size.1))
    }

    fn take_pending_commit(&mut self) -> bool {
        std::mem::take(&mut self.pending_commit)
    }

    fn run_full(&mut self) -> (Vec<u8>, u32, u32) {
        // T1.1 stub: echo source (commit semantics tested per-tool; aqui só
        // proves the dispatch wiring). T1.4+ ship full-res stamp render via
        // GPU compute pipeline (ADR-0044).
        //
        // Guard runtime: detecta bridge ativando o stub prematuramente.
        // ImageEdit bridge não-existe pro Painter em T1.1 (Painter não está
        // em image_edit::dispatch ainda), então chamar `run_full` agora é
        // bug-mate: a layer commitaria source-unchanged silenciosamente
        // (no-op no canvas, transação no-op no undo history). Audit 2026-05-26
        // (M-1/F5) recomendou guard.
        debug_assert!(
            false,
            "PainterTool::run_full is a T1.1 stub (echo source). \
             T1.4+ ships the GPU stamp pipeline. If this fired, a bridge \
             wired Painter into image_edit dispatch prematurely."
        );
        (
            self.source_rgba.clone(),
            self.source_size.0,
            self.source_size.1,
        )
    }

    fn deactivate(&mut self) {
        // Clear preview overlays + drop pending state (ADR-0041 lifecycle).
        self.source_rgba.clear();
        self.source_size = (0, 0);
        self.preview_dirty = false;
        self.pending_commit = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_is_painter() {
        let t = PainterTool::default();
        assert_eq!(t.id(), ToolId::new("painter"));
    }

    #[test]
    fn label_is_painter() {
        let t = PainterTool::default();
        assert_eq!(t.label(), "Painter");
    }

    #[test]
    fn icon_slug_is_painter() {
        let t = PainterTool::default();
        assert_eq!(t.icon_slug(), "painter");
    }

    #[test]
    fn build_panel_returns_panel_with_tool_id() {
        let t = PainterTool::default();
        let p = t.build_panel();
        assert_eq!(p.tool_id, ToolId::new("painter"));
    }

    #[test]
    fn activate_sets_takeover_flag() {
        let mut t = PainterTool::default();
        assert!(!t.params.takeover_active);
        t.on_activate();
        assert!(t.params.takeover_active);
        t.on_deactivate();
        assert!(!t.params.takeover_active);
    }

    #[test]
    fn not_default_in_w1() {
        // Brush proto-tool é default em W1; Painter assume em T-close.
        assert!(!PainterTool::default().is_default());
    }

    #[test]
    fn as_raster_edit_mut_returns_self() {
        // T1.1 ship com RasterEditTool stub impl. As_raster_edit_mut() retorna
        // Some(self) — gate `architecture_image_tool_kind_contract` passa.
        let mut t = PainterTool::default();
        assert!(t.as_raster_edit_mut().is_some());
    }

    #[test]
    fn raster_edit_set_source_and_current_preview_round_trip() {
        // T1.1 stub: validates set_source / current_preview / take_pending_commit /
        // deactivate dispatch wiring. `run_full` é coberto por
        // `run_full_panics_in_t11_stub` (debug_assert intencional).
        let mut t = PainterTool::default();
        let re = t.as_raster_edit_mut().unwrap();
        assert!(re.current_preview().is_none(), "no source yet");
        re.set_source(vec![1, 2, 3, 4], 1, 1);
        assert_eq!(re.current_preview(), Some((&[1u8, 2, 3, 4][..], 1, 1)));
        assert!(re.current_preview().is_none(), "dirty drained");
        assert!(!re.take_pending_commit(), "no commit requested");
        re.deactivate();
        assert!(re.current_preview().is_none(), "deactivate cleared state");
    }

    #[test]
    #[should_panic(expected = "T1.1 stub")]
    fn run_full_panics_in_t11_stub() {
        // Audit 2026-05-26 (M-1/F5): guard runtime detecta bridge prematuro.
        // T1.4+ substituirá o stub por GPU stamp pipeline real (ADR-0044);
        // este test então vira `assert_eq!(re.run_full(), ...)` real.
        let mut t = PainterTool::default();
        let re = t.as_raster_edit_mut().unwrap();
        re.set_source(vec![1, 2, 3, 4], 1, 1);
        let _ = re.run_full(); // debug_assert disparado
    }
}
