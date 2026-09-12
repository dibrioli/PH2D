#![forbid(unsafe_code)]
//! ph2d-vec-edit — pen STYLE (sibling of `lib.rs` for the HR-18 file-LOC cap).
//! `PenStyle` is the pen's stroke/fill/cap/join/dash config; it is `pub` and re-exported by
//! `lib.rs`, so `crate::PenStyle` keeps resolving for `shape.rs` and the tests. No `PenTool` coupling.
//!
//! ⛔ **A `History` que morava aqui MORREU em 2026-09-12** (`line/render-loop`, A9 da auditoria de
//! arquitectura): o snapshot da `VecScene` que ela empilhava era escrito por ~40 portas e **lido por
//! nenhuma** — o Ctrl+Z é a fila GLOBAL (`ProjectUndo`, na shell), que regista um passo por DIFF do
//! estado. Cada gesto vetorial pagava uma cópia da cena inteira para uma pilha que ninguém desfazia.

use ph2d_vec_scene::{LineCap, LineJoin, Marker, Rgba8, StrokeAlign, StrokeSpec};

/// Cor do traço do Pen (claro, sobre o canvas escuro).
const PEN_STROKE: Rgba8 = Rgba8::new(240, 240, 245, 255);

/// Preenchimento leve aplicado ao fechar o path.
const PEN_FILL: Rgba8 = Rgba8::new(90, 150, 230, 120);

/// Estilo aplicado a paths **recém-criados** pelo Pen. O shell sincroniza a partir
/// da tool (`ph2d-tool-vector`): traço, largura em px, e o fill usado ao fechar.
/// Default = as cores de scaffold da Fase 1.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PenStyle {
    /// Cor do traço dos paths desenhados.
    pub stroke: Rgba8,
    /// Largura do traço em **pixels de tela** (o shell multiplica por `px_to_world`).
    pub stroke_w_px: f64,
    /// Preenchimento aplicado ao FECHAR um path.
    pub fill: Rgba8,
    /// Ponta / junção do traço.
    pub cap: LineCap,
    pub join: LineJoin,
    /// De que lado da linha a faixa cai — ver `ph2d_vec_scene::StrokeAlign`.
    pub align: StrokeAlign,
    /// Dash/vão como **múltiplos da largura**: `Some((dash, gap))` ⇒ traço e vão
    /// de `dash·width`/`gap·width`; `None` = contínuo. O render multiplica pela
    /// largura do path.
    pub dash: Option<(f64, f64)>,
    /// **Pontas** (arrowheads) do começo e do fim. Propriedade do traço, como o
    /// cap: um caminho novo já nasce com a ponta que a tool tem armada. Sem ponta
    /// (o default) nada muda — o `has_markers()` do render corta antes.
    pub marker_start: Marker,
    pub marker_end: Marker,
    /// **Tamanho da ponta**, como múltiplo do que a largura já dita (`1.0` = default), e
    /// **arredondamento das quinas dela** (`0` = afiada). Ver `StrokeSpec`.
    pub marker_scale: f64,
    pub marker_round: f64,
}

impl Default for PenStyle {
    fn default() -> Self {
        Self {
            stroke: PEN_STROKE,
            stroke_w_px: 3.0,
            fill: PEN_FILL,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
            align: StrokeAlign::Centre,
            dash: None,
            marker_start: Marker::None,
            marker_end: Marker::None,
            marker_scale: 1.0,
            marker_round: 0.0,
        }
    }
}

impl PenStyle {
    /// `StrokeSpec` do traço para `width` world-units (aplica cap/join/dash + pontas).
    #[must_use]
    pub fn stroke_spec(&self, width: f64) -> StrokeSpec {
        StrokeSpec {
            paint: ph2d_vec_scene::StrokePaint::Solid(self.stroke),
            width,
            cap: self.cap,
            join: self.join,
            align: self.align,
            dash: self.dash,
            marker_start: self.marker_start,
            marker_end: self.marker_end,
            marker_scale: self.marker_scale,
            marker_round: self.marker_round,
        }
    }
}
