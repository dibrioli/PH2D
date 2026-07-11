//! [`FlipTool`] — o modelo de BRUSH + modo da ferramenta Flip.
//!
//! A tool é deliberadamente fina: guarda a cor/largura/dureza/opacidade/
//! smoothing do traço e o modo de canvas (Select/Draw/Erase). A UI real é o
//! painel **docado** `ph2d-panel-flip` (W2) — `FloatingPanel`s de tool não são
//! pintados neste app. O documento (`FlipDoc`) e a interação (o traço em curso,
//! pointer→mundo) vivem no shell (`flip_bridge`), que faz downcast por
//! [`Tool::as_any_mut`] pra ler o estilo — mesmo padrão do Vector/Painter.
//!
//! **Cor:** a tool guarda sRGB8 (o que o picker OKLCH devolve); o bridge
//! converte pra `Rgba` linear ao assar o traço no `FlipDoc`.

use ph2d_editor_core::floating_panel::{FloatingPanel, PanelAnchor, ToolId};
use ph2d_editor_core::tool::{PanelEvent, Tool};

use crate::params::{EraseMode, FlipMode, FlipStyleSnapshot};

/// Largura default do traço (px de tela) — uma linha média.
pub const DEFAULT_WIDTH_PX: f64 = 6.0;
/// Cor default do traço (sRGB8) — quase-branco, como o Vector.
pub const DEFAULT_STROKE: [u8; 4] = [240, 240, 245, 255];

/// A ferramenta Flip — só estilo de brush + modo de canvas.
#[derive(Debug, Clone, PartialEq)]
pub struct FlipTool {
    stroke: [u8; 4],
    width_px: f64,
    hardness: f32,
    opacity: f32,
    smoothing: f32,
    mode: FlipMode,
    erase: EraseMode,
}

impl Default for FlipTool {
    fn default() -> Self {
        Self {
            stroke: DEFAULT_STROKE,
            width_px: DEFAULT_WIDTH_PX,
            hardness: 1.0,
            opacity: 1.0,
            smoothing: 0.5,
            // INTERINO (W2): default = Draw pra o pill já desenhar no smoke; sem
            // UI de modo ainda. Volta a Select (gizmo, arbitragem ADR-0112) quando
            // os botões de modo do painel landarem (T2.15).
            mode: FlipMode::Draw,
            erase: EraseMode::Soft,
        }
    }
}

impl FlipTool {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Cor do traço (sRGB8).
    #[must_use]
    pub fn stroke_rgba(&self) -> [u8; 4] {
        self.stroke
    }
    /// Largura do traço em px de tela.
    #[must_use]
    pub fn width_px(&self) -> f64 {
        self.width_px
    }
    /// Dureza da borda `0..=1`.
    #[must_use]
    pub fn hardness(&self) -> f32 {
        self.hardness
    }
    /// Opacidade do traço `0..=1`.
    #[must_use]
    pub fn opacity(&self) -> f32 {
        self.opacity
    }
    /// Intensidade do active smoothing `0..=1`.
    #[must_use]
    pub fn smoothing(&self) -> f32 {
        self.smoothing
    }
    /// Modo de canvas atual (o shell espelha pra rotear input + gizmo).
    #[must_use]
    pub fn mode(&self) -> FlipMode {
        self.mode
    }
    /// Modo de borracha atual (só relevante em `FlipMode::Erase`).
    #[must_use]
    pub fn erase_mode(&self) -> EraseMode {
        self.erase
    }

    /// Define a cor do traço (read-back do picker).
    pub fn set_stroke_rgba(&mut self, rgba: [u8; 4]) {
        self.stroke = rgba;
    }
    /// Define o modo de canvas (pill / botões do painel).
    pub fn set_mode(&mut self, mode: FlipMode) {
        self.mode = mode;
    }

    /// Projeta o estilo no snapshot que o painel docado pinta.
    #[must_use]
    pub fn ui_snapshot(&self) -> FlipStyleSnapshot {
        FlipStyleSnapshot {
            stroke: self.stroke,
            width_px: self.width_px,
            hardness: self.hardness,
            opacity: self.opacity,
            smoothing: self.smoothing,
            mode: self.mode,
            erase: self.erase,
        }
    }
}

impl Tool for FlipTool {
    fn id(&self) -> ToolId {
        ToolId::new("flip")
    }

    fn label(&self) -> &str {
        "Flip"
    }

    fn icon_slug(&self) -> &str {
        "flip"
    }

    fn build_panel(&self) -> FloatingPanel {
        // A UI real é o painel docado `ph2d-panel-flip` (W2); `FloatingPanel`s de
        // tool não são pintados. Uma casca vazia satisfaz o trait (mirror do
        // Vector/Padding).
        let mut panel = FloatingPanel::new(self.id(), "Flip");
        panel.anchor = PanelAnchor::BottomCenter;
        panel
    }

    fn handle_panel_event(&mut self, _event: PanelEvent) {
        // Os controles do painel docado (`ids::FLIP_*`) chegam aqui via
        // `ToolPanelEvent` — wirados no bloco de painel do W2 (T2.13-T2.16).
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_tool_defaults() {
        let t = FlipTool::new();
        assert_eq!(t.stroke_rgba(), DEFAULT_STROKE);
        assert_eq!(t.width_px(), DEFAULT_WIDTH_PX);
        assert_eq!(t.mode(), FlipMode::Draw); // INTERINO W2 (Select volta em T2.15)
        assert_eq!(t.hardness(), 1.0);
    }

    #[test]
    fn set_mode_and_stroke() {
        let mut t = FlipTool::new();
        t.set_mode(FlipMode::Draw);
        assert_eq!(t.mode(), FlipMode::Draw);
        t.set_stroke_rgba([220, 60, 60, 255]);
        assert_eq!(t.stroke_rgba(), [220, 60, 60, 255]);
    }

    #[test]
    fn ui_snapshot_round_trips_style() {
        let mut t = FlipTool::new();
        t.set_stroke_rgba([1, 2, 3, 255]);
        t.set_mode(FlipMode::Erase);
        let s = t.ui_snapshot();
        assert_eq!(s.stroke, [1, 2, 3, 255]);
        assert_eq!(s.mode, FlipMode::Erase);
        assert_eq!(s.width_px, t.width_px());
    }

    #[test]
    fn id_label_icon_stable() {
        let t = FlipTool::new();
        assert_eq!(t.id(), ToolId::new("flip"));
        assert_eq!(t.label(), "Flip");
        assert_eq!(t.icon_slug(), "flip");
    }
}
