//! Painter undo/redo controller — **transactional (structural) layer undo**.
//!
//! # What it covers
//!
//! Every structural layer edit — add / delete / duplicate / group / reorder a
//! layer, create a mask or adjustment, move a layer, switch the active layer —
//! is recorded as a *before/after* pair of full model snapshots. Undo rolls the
//! model back to `before`; redo rolls forward to `after`. This is the
//! layers + effects editor's undo history.
//!
//! # Design
//!
//! [`UndoController`] keeps two chronological stacks of [`UndoEntry`] (the
//! `undo` stack the user can step back through, the `redo` stack populated by
//! `undo` and cleared by any new edit — the standard linear-history contract).
//! Each entry carries BOTH endpoints, but **only their metadata**: the nineteen
//! canvas-shaped planes live beside them as a DELTA of the window the step
//! touched ([`crate::undo_delta`]), and the history is bounded in **BYTES**.
//!
//! # Por que bytes, e por que delta (U1 do plano 26 · o molde é o ADR-0117)
//!
//! Guardar os dois endpoints INTEIROS custava **um documento por traço**: 1.627 MB
//! retidos depois de 24 traços a 2048² (`tests/measure_undo_memory.rs`), com o cap
//! por CONTAGEM (`max_depth = 300`) **multiplicando** isso em vez de limitá-lo — a
//! frase literal do ADR-0117 (*"`MAX_HISTORY=64` era um multiplicador, não um
//! teto"*), no Painter, com um zero a mais. Um cap em bytes sem o delta resolveria
//! o teto e **encurtaria o undo de forma visível** (8 passos a 2048², 2 a 4096²),
//! que é regressão de produto; com o delta, o cap deixa de morder.
//!
//! O delta precisa de um lado completo do qual partir, e ele é o [`cursor`](UndoController):
//! o endpoint adjacente ao topo, materializado. **UM documento, constante** — e em
//! regime custa zero, porque compartilha os `Arc`s do tool enquanto ninguém escreve.
//! ⚠️ O estado VIVO do tool não serve para isso: `restore_model` termina em
//! `restore_shape_overlay`, que RE-CARIMBA a figura, então o vivo depois de um undo
//! não é byte-a-byte o snapshot instalado.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::compositor::LayerImage;
use crate::layers::{LayerId as RtLayerId, LayerStack};

/// A full snapshot of the editable layer model, for **transactional (structural)
/// undo** — adding / deleting / duplicating a layer, creating a mask or
/// adjustment, or switching the active layer. A structural edit reshapes the
/// whole model, so the undo entry stores the entire state to roll back to.
///
/// `canvas_rgba` is `Arc`-shared (cheap CoW clone — the tool already wraps the
/// active layer's working buffer in an `Arc`); the active layer id lives INSIDE
/// `layers` (`LayerStack` owns it), so restoring `layers` restores the active
/// target too. `images` is the per-layer pixel store for every NON-active layer.
#[derive(Clone, Debug)]
pub struct ModelSnapshot {
    pub layers: LayerStack,
    pub images: BTreeMap<RtLayerId, Arc<LayerImage>>,
    /// **Impasto** relief per layer — captured with the pixels so undoing a stroke takes back the
    /// thickness it laid down, not just the colour (a snapshot that forgot it would leave the light
    /// pass reporting a ridge over paint that is no longer there). Empty for every layer that was
    /// never sculpted, which is the norm — the map is lazy.
    pub heights: BTreeMap<RtLayerId, Arc<Vec<f32>>>,
    /// Paint coverage per layer, captured with the relief it belongs to (see `PainterTool::covers`).
    pub covers: BTreeMap<RtLayerId, Arc<Vec<u8>>>,
    /// The paint's MATERIAL per layer, captured with the relief and the coverage — the three are one
    /// fact about one stroke and they must roll back together.
    ///
    /// Left out when the material landed (2026-07-13), and the hole was nearly invisible: on bare canvas
    /// an undone stroke's coverage goes to zero, so the light weights its stale material by zero and
    /// nothing shows. Paint the stroke over EXISTING paint, though, and undo restores the lower stroke's
    /// coverage under the upper one's material — the paint underneath comes back wearing the wrong
    /// gloss. `undoing_a_stroke_restores_the_material_underneath_it`.
    pub mats: BTreeMap<RtLayerId, Arc<Vec<ph2d_painter_brush::material::MaterialBytes>>>,
    pub canvas_rgba: Arc<Vec<u8>>,
    /// A forma do canvas quando o snapshot foi tirado — o que dá o **stride** de cada plano ao motor de
    /// delta ([`crate::undo_delta::Strides`]). Um snapshot que não sabe a largura do que carrega só pode
    /// guardar um intervalo LINEAR, e aí um traço VERTICAL reivindica o canvas inteiro em linhas.
    pub canvas_size: (u32, u32),
    pub selection: BTreeSet<RtLayerId>,
    /// The open on-canvas shape editor (Curve / Ellipse / Polygon), captured so a structural undo/redo
    /// restores the live overlay TOGETHER with the pixels — the two can never desync. `None` = no shape
    /// open (a layer op, or a committed/cancelled shape).
    pub shape: Option<Box<ShapeEditState>>,
    /// The **Offset** slider track at capture time, restored alongside the shape so undoing an Offset drag
    /// reinstates its value. Ignored when `shape` is `None`.
    pub offset_norm: f32,
    /// The **accumulated** Offset (px) from prior Apply & Keep presses (see `PaintState::shape_offset_base_px`),
    /// restored with `offset_norm` so undoing an Apply & Keep reinstates the pre-commit Offset exactly.
    pub offset_base_px: f32,
    /// The in-progress drag-preview's saved pixels, if a shape preview was live — so a restore can peel the
    /// preview back to the pristine baseline before re-stamping the editor's geometry (no double paint).
    pub preview_patch: Option<PreviewPatch>,
    /// The PARKED stroke shapes (multi-shape) at capture time — every simultaneously-editable shape that
    /// isn't the one live editor (`shape`). Captured so a structural undo/redo restores the whole editable
    /// set in lock-step with the baked/preview pixels, not just the active one. Empty = single/no shape.
    pub parked_shapes: Vec<ParkedShapeState>,
    /// The ACTIVE stroke shape's boolean Operation (wire `0`=Overlay `1`=Add `2`=Remove) at capture time —
    /// the parked shapes carry theirs inline ([`ParkedShapeState::op`]) but the active one lives outside the
    /// editor state, so without this a centre-square op-cycle tap was NOT undoable (Enio 2026-07-05).
    pub active_op: u8,
    /// The **Mask** brush's transient scratch buffer + its target layer at capture time. A mask stroke
    /// mutates only this scratch (it swaps in/out of `canvas_rgba`, which stays unchanged), so without
    /// capturing it here a mask stroke produced a no-op undo entry and could not be rolled back. Restoring
    /// it alongside the layers keeps the live mask-in-progress in lock-step with the global undo/redo.
    /// Empty `Arc` + `None` = no scratch.
    pub mask_scratch: Arc<Vec<u8>>,
    pub mask_scratch_target: Option<crate::layers::LayerId>,
    /// The **Selection** mask + its active flag at capture time (ADR-0103). A selection edit mutates only
    /// this document-wide coverage buffer (not `canvas_rgba`), so — exactly like `mask_scratch` — it must
    /// be captured here or the edit would be a no-op undo entry. Restoring it in lock-step with the pixels
    /// keeps the selection and the layers on the ONE interleaved timeline. Empty `Arc` + `false` = none.
    pub selection_mask: Arc<Vec<u8>>,
    pub selection_active: bool,
    /// The CRISP (pre-Feather) selection accumulator + the Feather amount, captured so undo/redo restores
    /// the exact effective mask AND keeps the Feather slider re-derivable from the right base.
    pub selection_crisp: Arc<Vec<u8>>,
    pub selection_feather: f32,
    /// The **parametric selection shape list** (ADR-0103 Am.2) at capture time — the source of truth the
    /// mask is a derived cache of. Captured so a structural undo/redo restores the editable SHAPES (curve
    /// anchors + handles, box params) in lock-step with the mask: without it, undo restored only the
    /// rasterized mask and the next `recompose_selection_mask` regenerated the edited shape → the selection
    /// curve point edit "came back" (Enio 2026-07-03). Empty = no parametric selection.
    pub(crate) selection_shapes: Vec<crate::tool::SelectionEntry>,
    /// The **Deform** session at capture time — the cumulative displacement map + the pristine `pre` +
    /// the active flag (Deform Wave 1). Captured so undo/redo rolls the WARP back in lock-step with the
    /// pixels: without it, undoing a deform stroke restored the pixels but dropped the displacement, so
    /// Reconstruct could no longer un-warp the remaining deform (Enio 2026-07-04). Empty `Arc`s + `false`
    /// = no session. `Arc`-shared, so a non-deform snapshot carries an empty (0-byte) map for free.
    pub(crate) deform: WarpSnap,
    /// The **Sculpt** session at capture time (`docs/Painter/18…` §10.4).
    ///
    /// Captured for a reason the Deform did NOT have, and it is the reason the plan demanded it: the
    /// sculpt writes the layer's relief **live**, so a snapshot taken mid-preview captures an
    /// already-carved plane. Drop the session on restore and the shape editor's next re-stamp opens a
    /// FRESH one whose frozen source is that carved plane — and re-runs the kernel on top of it. Undo,
    /// redo, and the ridge has been smoothed twice; do it again and it melts further. The pixels are
    /// perfect the whole time, so nothing else in the system notices.
    ///
    /// Rolling the session back with the relief makes the restored state a state the sculpt can *continue
    /// from*, which is the same thing `deform_disp` buys the warp.
    pub(crate) sculpt: SculptSnap,
}

/// The Deform session as an undo snapshot — `Arc`-shared, so a snapshot taken while nobody is deforming
/// carries empty (0-byte) buffers and costs nothing.
///
/// The frozen impasto planes (`pre_h` / `pre_cover` / `pre_mats`, W4) ride beside the pixels' `pre` for
/// the same reason `disp` does (Enio 2026-07-04): an undo mid-session must leave Reconstruct able to
/// un-warp what remains — of the BODY as much as of the colour. Empty when the session's layer carried
/// no relief.
#[derive(Clone, Debug)]
pub(crate) struct WarpSnap {
    pub(crate) disp: Arc<Vec<[f32; 2]>>,
    pub(crate) pre: Arc<Vec<u8>>,
    pub(crate) pre_h: Arc<Vec<f32>>,
    pub(crate) pre_cover: Arc<Vec<u8>>,
    pub(crate) pre_mats: Arc<Vec<ph2d_painter_brush::material::MaterialBytes>>,
    pub(crate) relief_layer: Option<RtLayerId>,
    pub(crate) active: bool,
}

/// The Sculpt session as an undo snapshot — `Arc`-shared, so a snapshot taken while nobody is sculpting
/// carries two empty (0-byte) buffers and costs nothing.
#[derive(Clone, Debug, Default)]
pub(crate) struct SculptSnap {
    /// The layer's relief as the stroke found it — the frozen source every re-render reads.
    pub(crate) pre: Arc<Vec<f32>>,
    /// The accumulated per-texel touch.
    pub(crate) amount: Arc<Vec<f32>>,
    /// The plane family's per-texel target (`Σ w·plane(i)`) — empty in the Smooth family.
    ///
    /// It rides the snapshot and the blur memo does not, and the asymmetry is the point: the memo is a
    /// function of `pre` alone, so the restore can throw it away and let it rebuild. `plane_sum` is a
    /// function of the **dab list**, which no longer exists — drop it on restore and the next re-stamp
    /// divides by it anyway, pulling the footprint toward height 0. Doc 18 §10.4 states the rule with a scar
    /// behind it: *ao adicionar um plano, adicione-o ao snapshot no mesmo commit.*
    pub(crate) plane_sum: Arc<Vec<f32>>,
    /// The **matter** as the stroke found it — coverage, material and the layer's pixels.
    ///
    /// Here for the same reason `plane_sum` is, and the same rule (§10.4): **Inflate moves paint**, so it
    /// re-renders the coverage / material / colour from these every frame. Restore the relief without them
    /// and the next re-render derives the paint from a canvas the gesture has ALREADY written on — the
    /// smear compounds, once per undo, and the first thing anyone would blame is the kernel.
    pub(crate) pre_cover: Arc<Vec<u8>>,
    /// See [`Self::pre_cover`].
    pub(crate) pre_mats: Arc<Vec<ph2d_painter_brush::material::MaterialBytes>>,
    /// See [`Self::pre_cover`]. RGBA8.
    pub(crate) pre_rgba: Arc<Vec<u8>>,
    /// Which layer the session belongs to — and, the session dying at commit, also *whether there is an
    /// uncommitted gesture at all* (`None` ⇒ no session). The two used to be separate fields.
    pub(crate) layer: Option<RtLayerId>,
    /// The window the stroke touched — what a re-stamp restores, and what a knob edit re-renders.
    pub(crate) bbox: Option<crate::compositor::Region>,
}

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

/// Marker for entries that may COALESCE with an immediately-preceding entry of the same kind: a run of
/// repeated same-kind actions (N progressive-Simplify presses, N boolean-op taps on the same shape)
/// collapses into ONE undo step whose `before` is the state before the FIRST action and whose `after`
/// tracks the latest (Enio 2026-07-05: the curve workflow's undo was too granular — "cada mínima ação
/// entra na sequência"). Any other action (a normal entry) breaks the run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoalesceKind {
    /// A **Simplify** press (stroke curve or selection curves) — progressive presses are one logical
    /// "simplify to taste" action; one undo restores the pre-Simplify curve.
    Simplify,
    /// A centre-square boolean-op cycle tap on the ACTIVE stroke shape.
    OpCycleStroke,
    /// A centre-square Add↔Remove tap on selection shape `i` — taps on DIFFERENT shapes don't coalesce.
    OpCycleSelection(usize),
}

/// One retained history entry: the two endpoints' **metadata** plus the DELTA of every canvas-shaped
/// plane between them.
///
/// ⚠️ `before` e `after` aqui carregam os planos **ESVAZIADOS** — os pixels foram para `planes` no
/// momento do push, e voltam na materialização ([`UndoController::undo`]). Os dois campos são privados e
/// nunca escapam do módulo sem passar por ela; um endpoint meio-preenchido circulando seria um
/// `ModelSnapshot` que mente sobre o próprio conteúdo.
#[derive(Clone, Debug)]
struct UndoEntry {
    before: Box<ModelSnapshot>,
    after: Box<ModelSnapshot>,
    planes: crate::undo_planes::PlaneDeltas,
    /// `Some` for a coalescible action — see [`CoalesceKind`]. Plain entries carry `None` and thereby
    /// BREAK any run in progress.
    kind: Option<CoalesceKind>,
}

impl UndoEntry {
    /// Guarda um par de endpoints COMPLETOS, extraindo o delta e esvaziando-os.
    fn split(
        mut before: ModelSnapshot,
        mut after: ModelSnapshot,
        kind: Option<CoalesceKind>,
    ) -> Self {
        let planes = crate::undo_planes::PlaneDeltas::split(&mut before, &mut after);
        Self {
            before: Box::new(before),
            after: Box::new(after),
            planes,
            kind,
        }
    }

    /// Reconstrói um dos endpoints a partir do cursor (o estado adjacente).
    fn materialize(&self, cursor: &ModelSnapshot, want_before: bool) -> Option<Box<ModelSnapshot>> {
        let mut out = Box::new(if want_before {
            self.before.as_ref().clone()
        } else {
            self.after.as_ref().clone()
        });
        self.planes.side(cursor, &mut out, want_before)?;
        Some(out)
    }

    /// Os bytes que esta entrada retém: os deltas mais os pixels de bbox que o preview carrega.
    fn heap_bytes(&self) -> usize {
        let patch = |m: &ModelSnapshot| m.preview_patch.as_ref().map_or(0, |p| p.pixels.len());
        self.planes.heap_bytes() + patch(&self.before) + patch(&self.after)
    }
}

/// Snapshot-based undo/redo for the editable layer model.
///
/// The caller (the [`PainterTool`](crate::tool::PainterTool)) drives it at two
/// points: just after a structural edit commits ([`Self::record_structural`]),
/// and when the gesture/shell requests [`Self::undo`] / [`Self::redo`] (each
/// returns the [`ModelSnapshot`] the caller must reinstall).
#[derive(Debug)]
pub struct UndoController {
    undo: Vec<UndoEntry>,
    redo: Vec<UndoEntry>,
    /// **A base de todo delta**: o endpoint adjacente ao topo, materializado — o `after` da entrada no
    /// topo do `undo`, que é também o `before` da entrada no topo do `redo`.
    ///
    /// ⚠️ Custa **zero em regime**: enquanto ninguém escreve ele compartilha os mesmos `Arc`s do tool, e
    /// só diverge entre o primeiro dab de um traço e o commit dele. É UM documento no pior caso, contra
    /// os N que o histórico retinha.
    cursor: Option<Box<ModelSnapshot>>,
    max_bytes: usize,
    /// Soma corrida de [`UndoEntry::heap_bytes`] das DUAS pilhas (o redo retém tanto quanto o undo).
    bytes: usize,
}

impl Default for UndoController {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_BYTES)
    }
}

impl UndoController {
    /// New controller with an explicit retained-BYTES ceiling.
    #[must_use]
    pub fn new(max_bytes: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            cursor: None,
            // Um orçamento degenerado ainda tem de permitir UM passo — uma edição de camada inteira é
            // irredutivelmente uma camada por passo, e recusar o único passo possível seria um histórico
            // que não existe.
            max_bytes,
            bytes: 0,
        }
    }

    /// Record a STRUCTURAL transition (add/delete/duplicate layer, mask/adjustment
    /// create, active-layer switch). `before` is the model to roll back to on undo;
    /// `after` is the model to roll forward to on redo. Pushing it clears the redo
    /// branch (standard linear-history semantics).
    pub fn record_structural(&mut self, before: ModelSnapshot, after: ModelSnapshot) {
        // O cursor tem de ser o `after` COMPLETO, então ele é tirado ANTES do split (que o esvazia). É
        // um clone de `Arc`s, não de pixels.
        self.cursor = Some(Box::new(after.clone()));
        let entry = UndoEntry::split(before, after, None);
        self.bytes += entry.heap_bytes();
        self.undo.push(entry);
        self.drop_redo();
        self.cap();
    }

    /// Record a COALESCIBLE structural transition: when the newest undo entry carries the SAME
    /// [`CoalesceKind`] (and no redo branch intervenes), the run extends — the top entry keeps its
    /// original `before` and adopts this `after` — so N repeated same-kind actions undo as ONE step.
    /// Otherwise it pushes a fresh entry (which starts a new run).
    pub fn record_structural_coalesced(
        &mut self,
        kind: CoalesceKind,
        before: ModelSnapshot,
        after: ModelSnapshot,
    ) {
        if self.redo.is_empty()
            && let Some(top) = self.undo.last()
            && top.kind == Some(kind)
            // ⚠️ Estender o run RECOMPÕE o delta: o `before` da entrada está esvaziado, então ele é
            // materializado do cursor ANTIGO (que é o `after` de que o delta partiu) e re-partido contra
            // o `after` novo. Concatenar os dois deltas seria a alternativa, e ela erra quando as duas
            // janelas se sobrepõem — o segundo passo escreveria por cima do primeiro na ordem errada.
            && let Some(cursor) = self.cursor.as_deref()
            && let Some(first_before) = top.materialize(cursor, true)
        {
            let old = self.undo.pop().expect("o topo que acabamos de ler");
            self.bytes -= old.heap_bytes();
            self.cursor = Some(Box::new(after.clone()));
            let entry = UndoEntry::split(*first_before, after, Some(kind));
            self.bytes += entry.heap_bytes();
            self.undo.push(entry);
            self.cap();
            return;
        }
        self.cursor = Some(Box::new(after.clone()));
        let entry = UndoEntry::split(before, after, Some(kind));
        self.bytes += entry.heap_bytes();
        self.undo.push(entry);
        self.drop_redo();
        self.cap();
    }

    /// Undo the most recent structural edit: roll back to its `before` model and
    /// park the entry on the redo stack so a later [`Self::redo`] can roll forward
    /// to `after`. Returns the model to reinstall, or `None` if nothing to undo.
    pub fn undo(&mut self) -> Option<Box<ModelSnapshot>> {
        let entry = self.undo.last()?;
        let cursor = self.cursor.as_deref()?;
        let Some(restore) = entry.materialize(cursor, true) else {
            // O cursor não descreve mais o plano que a entrada deltou (o canvas mudou de forma sob o
            // histórico). Um undo que devolve pixels quase-certos é pior que um que se recusa: descarta.
            eprintln!("[painter-undo] o cursor nao casa com o delta: historico descartado");
            debug_assert!(
                false,
                "cursor incoerente com o delta — ver undo_delta::StoredPlane::side"
            );
            self.clear();
            return None;
        };
        let entry = self.undo.pop().expect("o topo que acabamos de ler");
        self.redo.push(entry);
        // O cursor ANDA com a história: o estado que acabamos de instalar é o adjacente do novo topo.
        self.cursor = Some(restore.clone());
        Some(restore)
    }

    /// Redo the most recently undone structural edit: roll forward to its `after`
    /// model and park the entry back on the undo stack. Returns the model to
    /// reinstall, or `None` if the redo stack is empty.
    pub fn redo(&mut self) -> Option<Box<ModelSnapshot>> {
        let entry = self.redo.last()?;
        let cursor = self.cursor.as_deref()?;
        let Some(restore) = entry.materialize(cursor, false) else {
            eprintln!("[painter-undo] o cursor nao casa com o delta: historico descartado (redo)");
            debug_assert!(
                false,
                "cursor incoerente com o delta — ver undo_delta::StoredPlane::side"
            );
            self.clear();
            return None;
        };
        let entry = self.redo.pop().expect("o topo que acabamos de ler");
        self.undo.push(entry);
        self.cursor = Some(restore.clone());
        Some(restore)
    }

    /// `true` if there is at least one edit to undo (drives the `undo_enabled`
    /// affordance).
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// `true` if there is at least one undone edit to redo (drives the
    /// `redo_enabled` affordance).
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Retained undo depth (for tests / memory introspection).
    #[must_use]
    pub fn undo_depth(&self) -> usize {
        self.undo.len()
    }

    /// Retained redo depth.
    #[must_use]
    pub fn redo_depth(&self) -> usize {
        self.redo.len()
    }

    /// Drop the controller's history (e.g. on `set_source` of a fresh canvas).
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.cursor = None;
        self.bytes = 0;
    }

    /// Re-dimensiona o orçamento (o `set_source` o deriva do documento — [`history_budget_bytes`]) e
    /// aplica o cap na hora.
    pub fn set_max_bytes(&mut self, bytes: usize) {
        self.max_bytes = bytes;
        self.cap();
    }

    /// **Os bytes que o histórico de fato retém** — o número que o cap conta e que o gate de memória
    /// mede. Ele soma as duas pilhas: uma sessão de undos não devolve memória, ela a move de lado.
    #[must_use]
    pub fn retained_bytes(&self) -> usize {
        self.bytes
    }

    /// Descarta a ramificação de redo, devolvendo os bytes dela ao orçamento.
    fn drop_redo(&mut self) {
        for e in self.redo.drain(..) {
            self.bytes -= e.heap_bytes();
        }
    }

    /// Enforce the **BYTE** ceiling (and the sanity limit on step count): drop the oldest entries — the
    /// front of the undo Vec — until the history fits.
    ///
    /// ⚠️ Nunca abaixo de UMA entrada: um passo irredutivelmente grande (a edição de camada inteira do
    /// ADR-0117) tem de continuar desfazível, e um cap que come o único passo possível é um histórico
    /// que não existe.
    fn cap(&mut self) {
        while self.undo.len() > 1
            && (self.bytes > self.max_bytes || self.undo.len() > MAX_HISTORY_STEPS)
        {
            let dropped = self.undo.remove(0);
            self.bytes -= dropped.heap_bytes();
        }
    }
}

#[cfg(test)]
#[path = "undo_tests.rs"]
mod tests;
