//! Flip-tool UI vocabulary — o modo de canvas (Select/Draw/Erase), o modo de
//! borracha, e os mapeamentos slider↔valor do brush, compartilhados pelo painel
//! docado (`ph2d-panel-flip`) e pela tool (`handle_panel_event`).
//!
//! Espelha `ph2d_tool_vector::params`: a tool é dona do estilo autoritativo,
//! projeta-o num [`FlipStyleSnapshot`] por frame (o shell publica → o painel lê),
//! e os dois lados concordam no mapa afim do slider (drag e tool em lock-step).

/// O gesto de canvas que a tool Flip executa. Espelha a arbitragem do Vector
/// (ADR-0112): **gizmo só no `Select`** (os modos de desenho não publicam
/// `GizmoView`, senão as alças comeriam o clique). O pill alterna.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FlipMode {
    /// Seta preta: seleciona e TRANSFORMA o objeto pelo gizmo. Não desenha.
    #[default]
    Select,
    /// Lápis: cada arrasto no canvas cria um traço novo no desenho ativo.
    Draw,
    /// Borracha: remove cobertura/traço (ver [`EraseMode`]).
    Erase,
}

/// Como a borracha age (GP `erase.cc`): `Soft` reduz opacidade (default, mais
/// "pintura"), `Hard` corta a cobertura, `Stroke` apaga o traço inteiro tocado.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EraseMode {
    #[default]
    Soft,
    Hard,
    Stroke,
}

/// Largura do traço em pixels de tela (a faixa inclusiva que o slider Size cobre).
/// O traço do Flip vai de fino (linha) a grosso (marca), daí o teto alto.
pub const WIDTH_MIN_PX: f64 = 1.0;
pub const WIDTH_MAX_PX: f64 = 64.0;

/// Slider normalizado `0..=1` → largura px `MIN..=MAX`.
#[must_use]
pub fn slider_to_px(track: f32) -> f64 {
    WIDTH_MIN_PX + f64::from(track.clamp(0.0, 1.0)) * (WIDTH_MAX_PX - WIDTH_MIN_PX)
}

/// Largura px → slider normalizado `0..=1` (inverso de [`slider_to_px`]), pra
/// semear o knob a partir da largura autoritativa da tool.
#[must_use]
pub fn px_to_slider(px: f64) -> f32 {
    (((px - WIDTH_MIN_PX) / (WIDTH_MAX_PX - WIDTH_MIN_PX)) as f32).clamp(0.0, 1.0)
}

/// Slider `0..=1` → fração `0..=1` (hardness, opacity, smoothing). Identidade
/// clampada — o mapa afim trivial mantém painel e tool em lock-step.
#[must_use]
pub fn slider_to_unit(track: f32) -> f32 {
    track.clamp(0.0, 1.0)
}

/// O snapshot que o painel docado pinta (a tool projeta o estilo por frame).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlipStyleSnapshot {
    /// Cor do traço (sRGB8) — a mesma que o picker OKLCH devolve.
    pub stroke: [u8; 4],
    /// Largura em px de tela.
    pub width_px: f64,
    /// Dureza da borda `0..=1` (1 = borda dura).
    pub hardness: f32,
    /// Opacidade do traço `0..=1`.
    pub opacity: f32,
    /// Intensidade do active smoothing `0..=1` (o "assentar" da cauda).
    pub smoothing: f32,
    /// Modo de canvas atual (o painel destaca o botão ativo).
    pub mode: FlipMode,
    /// Modo de borracha atual (só relevante em `FlipMode::Erase`).
    pub erase: EraseMode,
}
