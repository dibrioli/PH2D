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
    EditDomain, EraseMode, FillMode, FlipMode, FlipStyleSnapshot, GAP_MAX_WORLD, GROW_MAX,
    GROW_MIN, PRECISION_MAX, PRECISION_MIN, ReshapeKind, TRAP_MAX_PX, slider_to_px, slider_to_unit,
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

/// Cor default do rabisco do Colorize (C2) — um vermelho distinto do ocre do balde,
/// para o 1º rabisco ser visível sobre a linha e o papel.
pub const DEFAULT_COLORIZE: [u8; 4] = [200, 90, 90, 255];

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
    // ── Borracha: valores PRÓPRIOS + os links (§4.C, Unified Paint Settings do
    //    Blender). Enquanto o link está ligado (default) estes números não são lidos
    //    por ninguém — a borracha usa o `width_px`/`opacity` do pincel, como sempre.
    //    Nascem iguais aos defaults do pincel, então o 1º deslink não pula.
    erase_px: f64,
    erase_strength: f32,
    link_size: bool,
    link_strength: bool,
    // ── O balde (W4). Cor PRÓPRIA: colorir usa outra paleta que desenhar, e obrigar
    //    a trocar a cor do traço para pintar uma região seria hostil.
    fill_color: [u8; 4],
    fill_mode: FillMode,
    gap: f64,
    grow: f64,
    precision: f64,
    /// Raio da trapped ball em px de tela (`0` = desligado). COLORIZE C1.
    trap: f64,
    /// O traço nasce preenchido (material stroke+fill do GP) — ver o snapshot.
    draw_filled: bool,
    // ── O Reshape (W5). Sem raio/força próprios: usa o Size + o Strength do
    //    pincel (ver `FlipStyleSnapshot::reshape`).
    reshape: ReshapeKind,
    // ── O domínio da seleção no Edit (W8): traço ou ponto. A conversão no doc é
    //    do shell; a tool só guarda a escolha.
    edit_domain: EditDomain,
    // ── Colorize (C2). Cor PRÓPRIA que o próximo rabisco semeia; os rabiscos
    //    acumulados moram no shell (transientes), não na tool.
    colorize_color: [u8; 4],
    /// **Bleed** (6º smoke): quão fundo a cor entra pelo VÃO ABERTO (a lente). `0..1`,
    /// `0.5` = o pedágio aprovado no 5º smoke. Ver `FlipStyleSnapshot::colorize_bleed`.
    colorize_bleed: f64,
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
            erase_px: DEFAULT_WIDTH_PX,
            erase_strength: DEFAULT_OPACITY,
            link_size: true,
            link_strength: true,
            fill_color: DEFAULT_FILL,
            fill_mode: FillMode::Paint,
            gap: 0.0,
            // Grow 0: com a âncora no EIXO da linha (BUGS #14), o default já é exato em
            // qualquer espessura e zoom — o Grow é só o ajuste estilístico.
            grow: 0.0,
            precision: DEFAULT_PRECISION,
            trap: 0.0,
            draw_filled: false,
            reshape: ReshapeKind::Smooth,
            edit_domain: EditDomain::Stroke,
            colorize_color: DEFAULT_COLORIZE,
            colorize_bleed: 0.5,
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

    /// **O raio EFETIVO da borracha** (§4.C) — a PORTA ÚNICA da pergunta "que raio a
    /// borracha usa?". Linkada (default) devolve o Size do pincel; deslinkada, o dela.
    /// O snapshot publica o resultado disto, então quem apaga e quem desenha o anel
    /// nunca re-derivam a regra (duas cópias divergem).
    #[must_use]
    pub fn eraser_size_px(&self) -> f64 {
        if self.link_size {
            self.width_px
        } else {
            self.erase_px
        }
    }
    /// A força EFETIVA da borracha (mesma porta, outro eixo).
    #[must_use]
    pub fn eraser_strength(&self) -> f32 {
        if self.link_strength {
            self.opacity
        } else {
            self.erase_strength
        }
    }
    /// O Size da borracha segue o do pincel? (estado do toggle de link.)
    #[must_use]
    pub fn link_size(&self) -> bool {
        self.link_size
    }
    /// A Strength da borracha segue a do pincel?
    #[must_use]
    pub fn link_strength(&self) -> bool {
        self.link_strength
    }
    /// O traço nasce preenchido? (só relevante em `FlipMode::Draw`.)
    #[must_use]
    pub fn draw_filled(&self) -> bool {
        self.draw_filled
    }
    /// Pincel de escultura atual (só relevante em `FlipMode::Reshape`).
    #[must_use]
    pub fn reshape_kind(&self) -> ReshapeKind {
        self.reshape
    }
    /// O domínio da seleção (só relevante em `FlipMode::Edit`) — traço ou ponto (W8).
    #[must_use]
    pub fn edit_domain(&self) -> EditDomain {
        self.edit_domain
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
            // EFETIVOS (link já resolvido) — ver `eraser_size_px`/`eraser_strength`.
            erase_px: self.eraser_size_px(),
            erase_strength: self.eraser_strength(),
            link_size: self.link_size,
            link_strength: self.link_strength,
            fill_color: self.fill_color,
            fill_mode: self.fill_mode,
            draw_filled: self.draw_filled,
            reshape: self.reshape,
            edit_domain: self.edit_domain,
            gap: self.gap,
            grow: self.grow,
            precision: self.precision,
            trap: self.trap,
            colorize_color: self.colorize_color,
            colorize_bleed: self.colorize_bleed,
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

    /// A cor do próximo rabisco do Colorize (o picker OKLCH escreve aqui quando a swatch
    /// de Colorize está no alvo).
    #[must_use]
    pub fn colorize_rgba(&self) -> [u8; 4] {
        self.colorize_color
    }

    pub fn set_colorize_rgba(&mut self, c: [u8; 4]) {
        self.colorize_color = c;
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
            PanelEvent::Click(id) if id == ids::FLIP_MODE_EDIT => self.mode = FlipMode::Edit,
            PanelEvent::Click(id) if id == ids::FLIP_MODE_COLORIZE => {
                self.mode = FlipMode::Colorize;
            }
            PanelEvent::Click(id) if id == ids::FLIP_MODE_TRACE => {
                self.mode = FlipMode::Trace;
            }
            // Shape (modo Draw): o traço carrega o próprio preenchimento?
            PanelEvent::Click(id) if id == ids::FLIP_SHAPE_LINE => self.draw_filled = false,
            PanelEvent::Click(id) if id == ids::FLIP_SHAPE_FILLED => self.draw_filled = true,
            // O domínio da seleção (modo Edit, W8 + §4.B): traço inteiro, ponto ou pedaço.
            PanelEvent::Click(id) if id == ids::FLIP_EDIT_DOM_STROKE => {
                self.edit_domain = EditDomain::Stroke;
            }
            PanelEvent::Click(id) if id == ids::FLIP_EDIT_DOM_POINT => {
                self.edit_domain = EditDomain::Point;
            }
            PanelEvent::Click(id) if id == ids::FLIP_EDIT_DOM_SEGMENT => {
                self.edit_domain = EditDomain::Segment;
            }
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
                self.gap = v.clamp(0.0, 1.0) * GAP_MAX_WORLD;
            }
            PanelEvent::SetValue(id, v) if id == ids::FLIP_GROW => {
                self.grow = GROW_MIN + v.clamp(0.0, 1.0) * (GROW_MAX - GROW_MIN);
            }
            PanelEvent::SetValue(id, v) if id == ids::FLIP_PRECISION => {
                self.precision =
                    PRECISION_MIN + v.clamp(0.0, 1.0) * (PRECISION_MAX - PRECISION_MIN);
            }
            PanelEvent::SetValue(id, v) if id == ids::FLIP_TRAP => {
                self.trap = v.clamp(0.0, 1.0) * TRAP_MAX_PX;
            }
            PanelEvent::SetValue(id, v) if id == ids::FLIP_COLORIZE_BLEED => {
                // O track (0..1) É a fração `colorize_bleed` — o shell a mapeia para o
                // pedágio de aperto do motor (`squeeze_from_bleed`).
                self.colorize_bleed = v.clamp(0.0, 1.0);
            }
            // Sub-modo da borracha.
            PanelEvent::Click(id) if id == ids::FLIP_ERASE_SOFT => self.erase = EraseMode::Soft,
            PanelEvent::Click(id) if id == ids::FLIP_ERASE_HARD => self.erase = EraseMode::Hard,
            PanelEvent::Click(id) if id == ids::FLIP_ERASE_STROKE => self.erase = EraseMode::Stroke,
            // Links da borracha (§4.C): o toggle na LINHA da propriedade. Deslinkar não
            // move número nenhum — só troca QUAL valor a borracha lê (os próprios já
            // nascem iguais aos do pincel, então o 1º deslink não pula).
            PanelEvent::Click(id) if id == ids::FLIP_LINK_SIZE => {
                self.link_size = !self.link_size;
            }
            PanelEvent::Click(id) if id == ids::FLIP_LINK_STRENGTH => {
                self.link_strength = !self.link_strength;
            }
            // Sliders PRÓPRIOS da borracha (só existem na tela com o link desligado).
            PanelEvent::SetValue(id, v) if id == ids::FLIP_ERASE_SIZE => {
                self.erase_px = slider_to_px(v as f32);
            }
            PanelEvent::SetValue(id, v) if id == ids::FLIP_ERASE_STRENGTH => {
                self.erase_strength = slider_to_unit(v as f32);
            }
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

    /// **O slider Bleed do Colorize chega à tool** (6º smoke): o `SetValue` do
    /// `FLIP_COLORIZE_BLEED` grava a fração `colorize_bleed` (`0..1`), que o shell mapeia
    /// para o pedágio de aperto do motor. Trap e Bleed são knobs INDEPENDENTES.
    #[test]
    fn the_colorize_bleed_slider_reaches_the_tool() {
        let mut t = FlipTool::new();
        assert!(
            (t.ui_snapshot().colorize_bleed - 0.5).abs() < 1e-9,
            "o Bleed nasce no meio (o pedágio DEFAULT do 5º smoke)"
        );
        t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_COLORIZE_BLEED, 0.8));
        assert!((t.ui_snapshot().colorize_bleed - 0.8).abs() < 1e-9);
        // Fora de [0,1] é clampado (o slider nunca sai, mas a porta se defende).
        t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_COLORIZE_BLEED, 5.0));
        assert!((t.ui_snapshot().colorize_bleed - 1.0).abs() < 1e-9);
        // O Trap é independente: mexer no Bleed não o move.
        assert_eq!(t.ui_snapshot().trap, 0.0, "o Bleed nao pode tocar o Trap");
    }

    /// **O Trap máximo (6º smoke: 20 → 50) chega à tool** — o range que faltava para selar
    /// o vão que o Enio de fato desenha (~100 px de tela ⇒ slider ~50).
    #[test]
    fn the_trap_slider_reaches_fifty_at_full_track() {
        let mut t = FlipTool::new();
        t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_TRAP, 1.0));
        assert!(
            (t.ui_snapshot().trap - crate::params::TRAP_MAX_PX).abs() < 1e-9,
            "o Trap no fim do slider tem de valer TRAP_MAX_PX"
        );
        assert!(
            t.ui_snapshot().trap >= 50.0,
            "o range aumentado (50) e o que sela o vao grande do 6o smoke"
        );
    }

    // ── §4.C — os links da borracha (Unified Paint Settings do Blender) ───────────

    /// 🔴 **Linkado (o DEFAULT) a borracha É o pincel** — um número só, como sempre foi.
    /// Mexer no Size do pincel move o raio da borracha junto; esse é o comportamento
    /// que o projeto tinha antes do §4.C e que o toggle preserva por default.
    #[test]
    fn linked_by_default_the_eraser_follows_the_brush() {
        let mut t = FlipTool::new();
        assert!(t.link_size() && t.link_strength(), "o default e LINKADO");
        assert_eq!(t.eraser_size_px(), t.width_px());
        assert_eq!(t.eraser_strength(), t.opacity());

        t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_SIZE, 1.0));
        assert_eq!(
            t.eraser_size_px(),
            crate::params::WIDTH_MAX_PX,
            "linkada, a borracha segue o Size do pincel"
        );
        t.set_opacity(0.25);
        assert!((t.eraser_strength() - 0.25).abs() < 1e-6);
    }

    /// 🔴 **Deslinkada, cada uma tem a sua** — e mexer numa NÃO move a outra.
    ///
    /// Mutação que sangra: `eraser_size_px` devolver `width_px` incondicionalmente (ou
    /// o toggle não inverter o flag) — o raio da borracha passa a seguir o pincel e os
    /// dois `assert` de independência caem.
    #[test]
    fn unlinked_the_eraser_and_the_brush_keep_their_own_numbers() {
        let mut t = FlipTool::new();
        // O toggle na linha do Size desliga o link.
        t.handle_panel_event(PanelEvent::Click(ids::FLIP_LINK_SIZE));
        t.handle_panel_event(PanelEvent::Click(ids::FLIP_LINK_STRENGTH));
        assert!(!t.link_size() && !t.link_strength());

        // Deslinkar NÃO move número nenhum: os próprios nascem nos defaults do pincel,
        // então o 1º deslink não pula na cara do artista.
        assert_eq!(t.eraser_size_px(), DEFAULT_WIDTH_PX);
        assert_eq!(t.eraser_strength(), DEFAULT_OPACITY);

        // O slider PRÓPRIO da borracha move só ela.
        t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_ERASE_SIZE, 1.0));
        t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_ERASE_STRENGTH, 0.5));
        assert_eq!(t.eraser_size_px(), crate::params::WIDTH_MAX_PX);
        assert!((t.eraser_strength() - 0.5).abs() < 1e-6);
        assert_eq!(t.width_px(), DEFAULT_WIDTH_PX, "o PINCEL nao se mexeu");
        assert_eq!(
            t.opacity(),
            DEFAULT_OPACITY,
            "a forca do pincel nao se mexeu"
        );

        // E o inverso: mexer no pincel não toca a borracha deslinkada.
        t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_SIZE, 0.0));
        assert_eq!(t.width_px(), crate::params::WIDTH_MIN_PX);
        assert_eq!(
            t.eraser_size_px(),
            crate::params::WIDTH_MAX_PX,
            "a borracha deslinkada ignora o Size do pincel"
        );
    }

    /// **Re-linkar devolve a borracha ao pincel** — e o valor próprio dela SOBREVIVE,
    /// então deslinkar de novo o recupera (o modelo do Blender: cada pincel guarda o
    /// seu; o link só escolhe QUEM responde).
    #[test]
    fn relinking_returns_to_the_brush_and_the_own_value_survives() {
        let mut t = FlipTool::new();
        t.handle_panel_event(PanelEvent::Click(ids::FLIP_LINK_SIZE));
        t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_ERASE_SIZE, 1.0));
        assert_eq!(t.eraser_size_px(), crate::params::WIDTH_MAX_PX);

        t.handle_panel_event(PanelEvent::Click(ids::FLIP_LINK_SIZE)); // re-linka
        assert_eq!(t.eraser_size_px(), t.width_px(), "voltou a seguir o pincel");

        t.handle_panel_event(PanelEvent::Click(ids::FLIP_LINK_SIZE)); // deslinka de novo
        assert_eq!(
            t.eraser_size_px(),
            crate::params::WIDTH_MAX_PX,
            "o valor PROPRIO da borracha sobreviveu ao re-link"
        );
    }

    /// O snapshot publica os valores **EFETIVOS** (link já resolvido) — é o que o anel
    /// do cursor e o `flip_erase` leem, e por isso eles nunca re-derivam a regra.
    #[test]
    fn the_snapshot_publishes_effective_eraser_values() {
        let mut t = FlipTool::new();
        t.handle_panel_event(PanelEvent::Click(ids::FLIP_LINK_SIZE));
        t.handle_panel_event(PanelEvent::SetValue(ids::FLIP_ERASE_SIZE, 1.0));
        let s = t.ui_snapshot();
        assert_eq!(
            s.erase_px,
            crate::params::WIDTH_MAX_PX,
            "efetivo = o proprio"
        );
        assert!(!s.link_size, "o snapshot carrega o estado do toggle");
        assert_eq!(
            s.width_px, DEFAULT_WIDTH_PX,
            "o Size do pincel vai separado"
        );
        // Linkado, o efetivo volta a ser o do pincel.
        t.handle_panel_event(PanelEvent::Click(ids::FLIP_LINK_SIZE));
        assert_eq!(t.ui_snapshot().erase_px, DEFAULT_WIDTH_PX);
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
