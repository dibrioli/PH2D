//! **O QUE UM SHAPE ABERTO É, dentro de um snapshot** — filho de [`super`] (`#[path]`, então os
//! caminhos `crate::undo::…` seguem valendo), split pelo cap de LOC e pela linha de corte natural: o
//! pai é *o que a HISTÓRIA faz*, aqui mora *a forma dos editores que um estado carrega*.
//!
//! Nada aqui sabe o que é um delta, um cursor ou uma entrada — é plain data que o
//! [`ModelSnapshot`](super::ModelSnapshot) transporta para que o undo restaure o overlay VIVO em
//! lock-step com os pixels (um shape reinstalado sobre pixels de outra era re-carimba a figura errada).

/// Plain-data snapshot of an open on-canvas shape editor, stored in a [`ModelSnapshot`] so a structural
/// undo/redo reinstates the editable overlay with the pixels. Geometry only — the transient grab/gizmo
/// fields reset to idle on restore. Curve handle kinds are kept as their wire `u8` so this module stays
/// free of the editor types.
#[derive(Clone, Debug, PartialEq)]
pub enum ShapeEditState {
    Curve(CurveState),
    Ellipse(EllipseState),
    Polygon(PolygonState),
    Line(LineState),
}

/// Editable Curve / Free Hand state (see `tool::paint::curve::CurveEditor`).
#[derive(Clone, Debug, PartialEq)]
pub struct CurveState {
    pub points: Vec<[f32; 2]>,
    pub handles: Vec<[[f32; 2]; 2]>,
    pub kinds: Vec<u8>,
    pub selected: Option<usize>,
    pub added_point: bool,
    pub closed: bool,
    pub editing: bool,
    pub freehand: bool,
    pub seed: u64,
    pub anchor: [f32; 2],
    pub stabilized: [f32; 2],
}

/// Editable Ellipse state (see `tool::paint::ellipse::EllipseEditor`).
#[derive(Clone, Debug, PartialEq)]
pub struct EllipseState {
    pub center: [f32; 2],
    pub u: [f32; 2],
    pub rx: f32,
    pub ry: f32,
    pub editing: bool,
    pub seed: u64,
}

/// Editable Polygon state (see `tool::paint::polygon::PolygonEditor`).
#[derive(Clone, Debug, PartialEq)]
pub struct PolygonState {
    pub center: [f32; 2],
    pub u: [f32; 2],
    pub rx: f32,
    pub ry: f32,
    pub sides: u32,
    pub editing: bool,
    pub seed: u64,
}

/// Editable Line (polyline) state (see `tool::paint::line::LineEditor`). Plain corner points, no handles;
/// per-corner Fillet/Chamfer carried as `(tag, amount)` wire pairs (`0` sharp / `1` fillet / `2` chamfer).
#[derive(Clone, Debug, PartialEq)]
pub struct LineState {
    pub points: Vec<[f32; 2]>,
    pub closed: bool,
    pub editing: bool,
    pub corner_mods: Vec<(u8, f32)>,
    pub seed: u64,
}

impl ShapeEditState {
    /// Geometry equality IGNORING the curve's `selected` index — selecting a point is not an undoable
    /// change (the no-op check in `commit_shape_txn` drops it), though selection IS restored on undo.
    #[must_use]
    pub fn geom_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Curve(a), Self::Curve(b)) => {
                a.points == b.points
                    && a.handles == b.handles
                    && a.kinds == b.kinds
                    && a.added_point == b.added_point
                    && a.closed == b.closed
                    && a.editing == b.editing
                    && a.freehand == b.freehand
                    && a.seed == b.seed
                    && a.anchor == b.anchor
                    && a.stabilized == b.stabilized
            }
            _ => self == other,
        }
    }
}

/// A PARKED (inactive but still-editable) stroke shape captured for undo: its geometry plus the wire `u8`
/// of its Operation (`0`=Overlay `1`=Add `2`=Remove — see `tool::paint::stroke_multi::StrokeOp`). Stroke
/// multi-shape keeps a list of these alongside the one live editor (`shape`); a structural undo/redo
/// restores the whole list so every simultaneously-editable shape rolls back in lock-step with the pixels.
/// Kept as a wire `u8` so this module stays free of the `paint` editor types (mirrors `CurveState.kinds`).
#[derive(Clone, Debug, PartialEq)]
pub struct ParkedShapeState {
    pub state: ShapeEditState,
    pub op: u8,
}

/// The in-progress drag-preview's saved pixels (a small bbox), carried in a [`ModelSnapshot`] so a restore
/// can peel the live preview back to the pristine baseline before re-stamping it (no double paint). `None`
/// for a snapshot taken with no live preview (layer ops, a committed shape).
#[derive(Clone, Debug, PartialEq)]
pub struct PreviewPatch {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub pixels: Vec<u8>,
}

/// **O teto do histórico quando ainda não há documento** — ver [`history_budget_bytes`], que é quem
/// manda assim que o canvas existe.
///
/// ⚠️ *Contagem é multiplicador, não teto* (ADR-0117): o `max_depth = 300` que isto substitui não
/// limitava nada — ele multiplicava por 300 o custo de um passo que ninguém media.
pub const DEFAULT_MAX_BYTES: usize = 256 * 1024 * 1024;

/// **O orçamento do histórico é função do DOCUMENTO**, no molde exato do Audio Editor (ADR-0117, cuja
/// linha no HR-13 diz `2×clipe + 256`): `2 × documento + 256 MB`.
///
/// ⚠️ **Uma constante absoluta foi escrita primeiro e a MEDIÇÃO a derrubou.** Ela prometia, no lugar
/// desta doc, *"> 300 traços a 2048² e a 4096² — o cap não morde"*; medido (`measure_undo_capacity`),
/// 512 MB fixos compram **204 traços a 1024², 62 a 2048² e 17 a 4096²**. Um teto absoluto racionaria o
/// artista justamente na tela em que ele tem menos margem, e escreveria a promessa errada nos dois
/// extremos. O orçamento acompanha o documento porque **é dele que o passo é uma fração**.
///
/// Medido, com um traço que atravessa a tela inteira (o pior caso — traços reais são mais curtos):
///
/// | tela | orçamento | passo (traço) | **traços** | camada inteira | ops |
/// |---|---|---|---|---|---|
/// | 1024² | 288 MB | 2,51 MB | **114** | 32 MB | 9 |
/// | 2048² | 384 MB | 8,19 MB | **46** | 128 MB | 3 |
/// | 4096² | 768 MB | 28,55 MB | **26** | 512 MB | 1 |
///
/// **Cena pesada ganha janela mais CURTA, não conta maior** — é a frase do W1.5 da física sobre o ring
/// de checkpoints, e é o que um cap em bytes significa. Contra o modelo antigo (um documento por
/// endpoint) o mesmo orçamento comprava **9 · 3 · 1** passos: o delta multiplica a profundidade do undo
/// por ~13× em toda tela, que é o número que o gate afirma.
#[must_use]
pub const fn history_budget_bytes(width: u32, height: u32) -> usize {
    // Os quatro planos canvas-shaped de uma camada tocada: rgba + heights(f32) + covers + mats([u8;7]).
    let doc = (width as usize) * (height as usize) * 16;
    2 * doc + 256 * 1024 * 1024
}

/// Guarda de sanidade sobre o NÚMERO de passos, muito acima de qualquer sessão real.
///
/// Não é o cap — o cap é [`DEFAULT_MAX_BYTES`]. Ele existe porque uma entrada pode custar ~zero byte
/// (renomear uma camada), e sem ele uma sessão longa acumularia entradas indefinidamente por não pesar
/// nada. Duas perguntas diferentes, dois limites; o que decide memória é o de bytes.
pub const MAX_HISTORY_STEPS: usize = 1000;
