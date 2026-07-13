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
use ph2d_editor_core::ids;
use ph2d_editor_core::tool::{PanelEvent, Tool};

use crate::params::{
    EraseMode, FillMode, FlipMode, FlipStyleSnapshot, GAP_MAX_PX, GROW_MAX, GROW_MIN,
    PRECISION_MAX, PRECISION_MIN, ReshapeKind, slider_to_px, slider_to_unit,
};

/// Largura default do traço (px de tela) — uma linha média.
pub const DEFAULT_WIDTH_PX: f64 = 6.0;
/// Dureza default da borda `0..=1` (borda dura).
pub const DEFAULT_HARDNESS: f32 = 1.0;
/// Opacidade default do traço `0..=1`.
pub const DEFAULT_OPACITY: f32 = 1.0;
/// Intensidade default do active smoothing `0..=1`.
pub const DEFAULT_SMOOTHING: f32 = 0.5;
/// Precision default do balde (px de buffer por px de tela) — 1.6 (Enio 2026-07-12,
/// smoke da âncora no eixo): acima da resolução da tela, o resíduo de quantização do
/// contorno cai para sub-pixel sem encarecer o clique.
pub const DEFAULT_PRECISION: f64 = 1.6;
/// Cor default do traço (sRGB8) — quase-branco, como o Vector.
pub const DEFAULT_STROKE: [u8; 4] = [240, 240, 245, 255];
/// Cor default do PREENCHIMENTO (um ocre claro — visível sobre a linha clara e sobre
/// o fundo escuro). Distinta da cor do traço de propósito: `docs/Flip/06`.
pub const DEFAULT_FILL: [u8; 4] = [230, 190, 120, 255];

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
    // ── O balde (W4). Cor PRÓPRIA: colorir usa outra paleta que desenhar, e obrigar
    //    a trocar a cor do traço para pintar uma região seria hostil.
    fill_color: [u8; 4],
    fill_mode: FillMode,
    gap_px: f64,
    grow: f64,
    precision: f64,
    // ── O Reshape (W5). Sem raio/força próprios: usa o Size + o Strength do
    //    pincel (ver `FlipStyleSnapshot::reshape`).
    reshape: ReshapeKind,
}

impl Default for FlipTool {
    fn default() -> Self {
        Self {
            stroke: DEFAULT_STROKE,
            width_px: DEFAULT_WIDTH_PX,
            hardness: DEFAULT_HARDNESS,
            opacity: DEFAULT_OPACITY,
            smoothing: DEFAULT_SMOOTHING,
            // Default = Select (gizmo transforma o objeto; arbitragem ADR-0112).
            // O painel docado (T2.15) tem a linha de modos Select/Draw/Erase.
            mode: FlipMode::Select,
            erase: EraseMode::Soft,
            fill_color: DEFAULT_FILL,
            fill_mode: FillMode::Paint,
            gap_px: 0.0,
            // Grow 0: com a âncora no EIXO da linha (BUGS #14), o default já é exato em
            // qualquer espessura e zoom — o Grow é só o ajuste estilístico.
            grow: 0.0,
            precision: DEFAULT_PRECISION,
            reshape: ReshapeKind::Smooth,
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
    /// Pincel de escultura atual (só relevante em `FlipMode::Reshape`).
    #[must_use]
    pub fn reshape_kind(&self) -> ReshapeKind {
        self.reshape
    }

    /// Define a cor do traço (read-back do picker).
    pub fn set_stroke_rgba(&mut self, rgba: [u8; 4]) {
        self.stroke = rgba;
    }
    /// Define o modo de canvas (pill / botões do painel).
    pub fn set_mode(&mut self, mode: FlipMode) {
        self.mode = mode;
    }
    /// Define o modo de borracha (botões Soft/Hard/Stroke do painel).
    pub fn set_erase_mode(&mut self, mode: EraseMode) {
        self.erase = mode;
    }
    /// Define a largura do traço em px de tela.
    pub fn set_width_px(&mut self, px: f64) {
        self.width_px = px;
    }
    /// Define a dureza da borda `0..=1` (clampada).
    pub fn set_hardness(&mut self, h: f32) {
        self.hardness = h.clamp(0.0, 1.0);
    }
    /// Define a opacidade do traço `0..=1` (clampada).
    pub fn set_opacity(&mut self, o: f32) {
        self.opacity = o.clamp(0.0, 1.0);
    }
    /// Define a intensidade do active smoothing `0..=1` (clampada).
    pub fn set_smoothing(&mut self, s: f32) {
        self.smoothing = s.clamp(0.0, 1.0);
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
            fill_color: self.fill_color,
            fill_mode: self.fill_mode,
            reshape: self.reshape,
            gap_px: self.gap_px,
            grow: self.grow,
            precision: self.precision,
        }
    }

    /// A cor do PREENCHIMENTO (o picker OKLCH escreve aqui quando a swatch de Fill
    /// está no alvo).
    #[must_use]
    pub fn fill_rgba(&self) -> [u8; 4] {
        self.fill_color
    }

    pub fn set_fill_rgba(&mut self, c: [u8; 4]) {
        self.fill_color = c;
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

    fn handle_panel_event(&mut self, event: PanelEvent) {
        // Os controles do painel docado (`ids::FLIP_*`) chegam aqui via
        // `ToolPanelEvent`. Só os que editam o ESTILO da tool (modo + brush)
        // são tratados aqui; as ops de CAMADA (add/delete/reorder/visibility/
        // blend/opacity) são edições de DOCUMENTO e ficam com o drain do shell
        // (mesmo padrão do Vector: Boolean/Arrange caem no drain, não na tool).
        match event {
            // Linha de modos Select/Draw/Erase (gizmo só no Select — o shell lê
            // `mode()` pra rotear input + publicar o `GizmoView`).
            PanelEvent::Click(id) if id == ids::FLIP_MODE_SELECT => self.mode = FlipMode::Select,
            PanelEvent::Click(id) if id == ids::FLIP_MODE_DRAW => self.mode = FlipMode::Draw,
            PanelEvent::Click(id) if id == ids::FLIP_MODE_ERASE => self.mode = FlipMode::Erase,
            PanelEvent::Click(id) if id == ids::FLIP_MODE_FILL => self.mode = FlipMode::Fill,
            PanelEvent::Click(id) if id == ids::FLIP_MODE_RESHAPE => self.mode = FlipMode::Reshape,
            // Os oito pincéis de escultura (W5). A tabela `FLIP_RESHAPE_KIND_IDS` está
            // na MESMA ordem que `ReshapeKind::ALL` — o zip é o decodificador, e o
            // seam test dirige os oito ids para provar que as duas listas não derivam.
            PanelEvent::Click(id) if ids::FLIP_RESHAPE_KIND_IDS.contains(&id) => {
                if let Some((_, kind)) = ids::FLIP_RESHAPE_KIND_IDS
                    .iter()
                    .zip(ReshapeKind::ALL)
                    .find(|(kid, _)| **kid == id)
                {
                    self.reshape = kind;
                }
            }
            // Modo do balde (Paint / Paint-Behind / Unpaint).
            PanelEvent::Click(id) if id == ids::FLIP_FILL_PAINT => self.fill_mode = FillMode::Paint,
            PanelEvent::Click(id) if id == ids::FLIP_FILL_BEHIND => {
                self.fill_mode = FillMode::PaintBehind;
            }
            PanelEvent::Click(id) if id == ids::FLIP_FILL_UNPAINT => {
                self.fill_mode = FillMode::Unpaint;
            }
            // Sliders do balde. Cada um é um mapa afim `track → valor` — o mesmo que o
            // painel usa no `link_slider_number_mapped`, senão o knob e o valor
            // divergem no 1º arrasto.
            PanelEvent::SetValue(id, v) if id == ids::FLIP_GAP => {
                self.gap_px = v.clamp(0.0, 1.0) * GAP_MAX_PX;
            }
            PanelEvent::SetValue(id, v) if id == ids::FLIP_GROW => {
                self.grow = GROW_MIN + v.clamp(0.0, 1.0) * (GROW_MAX - GROW_MIN);
            }
            PanelEvent::SetValue(id, v) if id == ids::FLIP_PRECISION => {
                self.precision =
                    PRECISION_MIN + v.clamp(0.0, 1.0) * (PRECISION_MAX - PRECISION_MIN);
            }
            // Sub-modo da borracha.
            PanelEvent::Click(id) if id == ids::FLIP_ERASE_SOFT => self.erase = EraseMode::Soft,
            PanelEvent::Click(id) if id == ids::FLIP_ERASE_HARD => self.erase = EraseMode::Hard,
            PanelEvent::Click(id) if id == ids::FLIP_ERASE_STROKE => self.erase = EraseMode::Stroke,
            // Sliders de brush (track `0..1` → valor; o mapa afim é o mesmo do painel).
            PanelEvent::SetValue(id, v) if id == ids::FLIP_SIZE => {
                self.width_px = slider_to_px(v as f32);
            }
            PanelEvent::SetValue(id, v) if id == ids::FLIP_HARDNESS => {
                self.hardness = slider_to_unit(v as f32);
            }
            PanelEvent::SetValue(id, v) if id == ids::FLIP_OPACITY => {
                self.opacity = slider_to_unit(v as f32);
            }
            PanelEvent::SetValue(id, v) if id == ids::FLIP_SMOOTHING => {
                self.smoothing = slider_to_unit(v as f32);
            }
            _ => {}
        }
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
        assert_eq!(t.mode(), FlipMode::Select); // gizmo por default (ADR-0112)
        assert_eq!(t.hardness(), 1.0);
    }

    #[test]
    fn default_snapshot_matches_fresh_tool() {
        // The panel paints `FlipStyleSnapshot::default()` before the shell's 1st
        // push; it must equal the fresh tool's snapshot so nothing "jumps".
        assert_eq!(FlipStyleSnapshot::default(), FlipTool::new().ui_snapshot());
    }

    #[test]
    fn panel_events_drive_mode_and_brush() {
        let mut t = FlipTool::new();
        t.handle_panel_event(PanelEvent::Click(ids::FLIP_MODE_DRAW));
        assert_eq!(t.mode(), FlipMode::Draw);
        t.handle_panel_event(PanelEvent::Click(ids::FLIP_MODE_ERASE));
        assert_eq!(t.mode(), FlipMode::Erase);
        t.handle_panel_event(PanelEvent::Click(ids::FLIP_ERASE_HARD));
        assert_eq!(t.erase_mode(), EraseMode::Hard);
        // Size slider at full track → WIDTH_MAX_PX.
        t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_SIZE, 1.0));
        assert_eq!(t.width_px(), crate::params::WIDTH_MAX_PX);
        t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_HARDNESS, 0.25));
        assert!((t.hardness() - 0.25).abs() < 1e-6);
        // A layer-op id (document edit) is ignored by the tool.
        t.handle_panel_event(PanelEvent::Click(ids::FLIP_LAYER_ADD));
        assert_eq!(t.mode(), FlipMode::Erase, "layer op didn't touch the tool");
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
