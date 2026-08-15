//! **Stroke MULTI-SHAPE** — multiple simultaneously-editable stroke shapes on one canvas (Enio 2026-07-04),
//! mirroring the Selection subsystem: the shapes are the parametric source of truth and the canvas PIXELS
//! are a DERIVED cache, re-stamped from ALL shapes onto one pristine baseline every recompose (the invariant
//! that makes overlapping vector edits clean: nothing is baked until the final Apply).
//!
//! To keep the ~100 `self.paint.{curve,ellipse,polygon,line}` sites untouched, exactly ONE shape is the
//! **active** live editor (the existing single slots) and the rest are **parked** here as plain geometry
//! ([`StrokeShape`]) + their Operation ([`StrokeOp`]). Switching active serializes the live editor into the
//! parked set and deserializes the target back out (cheap — geometry only). Undo clones the whole set.

use super::*;

// A geometria de polilinha mudou-se para o roteador (`super::stroke_router`) junto com o hit-test que
// a usa; re-exportada aqui porque os editores já a leem por este caminho — mover o CÓDIGO não é
// motivo para mover o endereço de quem o chama.
use super::stroke_router::dist2;
pub(super) use super::stroke_router::{min_dist2_to_polyline, point_near_polyline};

/// The boolean Operation a stroke shape participates in. Mirrors the Selection ops, except the former
/// "New" (replace) is replaced by **Overlay** — no boolean at all, the shape simply paints independently.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum StrokeOp {
    /// No boolean: the shape paints on its own; offset acts on it individually.
    #[default]
    Overlay,
    /// Boolean UNION with the overlapping Add/Remove shapes; offset acts on the combined region.
    Add,
    /// Boolean SUBTRACT from the overlapping Add/Remove shapes; offset acts on the combined region.
    Remove,
}

impl StrokeOp {
    /// Next op in the +/−/o cycle (the gizmo's centre square quick-click walks this): Overlay→Add→Remove→…
    pub(crate) fn cycle(self) -> Self {
        match self {
            StrokeOp::Overlay => StrokeOp::Add,
            StrokeOp::Add => StrokeOp::Remove,
            StrokeOp::Remove => StrokeOp::Overlay,
        }
    }

    /// The single-char glyph drawn in the gizmo's type square: `+` Add · `-` Remove · `o` Overlay.
    pub(crate) fn glyph(self) -> &'static str {
        match self {
            StrokeOp::Overlay => "o",
            StrokeOp::Add => "+",
            StrokeOp::Remove => "-",
        }
    }

    /// Wire `u8` for undo/panel round-trips (`0`=Overlay `1`=Add `2`=Remove).
    pub(crate) fn to_wire(self) -> u8 {
        match self {
            StrokeOp::Overlay => 0,
            StrokeOp::Add => 1,
            StrokeOp::Remove => 2,
        }
    }

    /// Inverse of [`Self::to_wire`] (any unknown value → Overlay, the neutral default).
    pub(crate) fn from_wire(w: u8) -> Self {
        match w {
            1 => StrokeOp::Add,
            2 => StrokeOp::Remove,
            _ => StrokeOp::Overlay,
        }
    }
}

/// A PARKED (inactive but still-editable) stroke shape: its frozen geometry + Operation. The live editor
/// lives in `PaintState::{curve,ellipse,polygon,line}`; everything else is a `StrokeShape` in the list.
#[derive(Clone, Debug)]
pub(crate) struct StrokeShape {
    pub state: crate::undo::ShapeEditState,
    pub op: StrokeOp,
}

/// A multi-shape gizmo **op badge** for the shell to draw: the centre square shows the `+`/`−`/`o` glyph of
/// the shape's Operation, and (for a PARKED shape) a faint AABB frame marks it as still-selectable. Image px.
#[derive(Clone, Debug)]
pub struct StrokeOpBadge {
    /// Centre of the shape (where the type-square + glyph sit).
    pub center: [f32; 2],
    /// **O CONTORNO da figura** em px de imagem (`super::stroke_outline`) — a linha que a shell traça.
    ///
    /// ⚠️ Era uma AABB, desenhada como moldura apagada. A caixa só bastava enquanto a TINTA aparecia sob
    /// a mão e dizia onde a figura estava; com o gesto rascunhado ela virou a única coisa na tela, e
    /// quatro círculos viravam quatro retângulos (Enio, 2026-08-07). O contorno é a MESMA linha que o
    /// hit-test alcança — *o que é desenhado é o que é clicável*.
    pub outline: Vec<[f32; 2]>,
    /// `true` quando o contorno fecha — a shell traça fechado (`stroke_box`) ou aberto (`stroke_open`).
    pub closed: bool,
    /// The Operation glyph: `"+"` Add · `"-"` Remove · `"o"` Overlay.
    pub glyph: &'static str,
}

impl PainterTool {
    /// `true` when at least one shape is parked (i.e. more than one shape is currently on canvas).
    pub(crate) fn has_parked_shapes(&self) -> bool {
        !self.paint.parked_shapes.is_empty()
    }

    /// Set the multi-shape **Operation** mode a NEW shape is created with (panel OPERATION card): `0`=Overlay
    /// `1`=Add `2`=Remove. Also retags the ACTIVE shape's op so the change is visible immediately on its
    /// gizmo (matches the Selection Operation feel) + recomposes so any boolean result updates live.
    pub fn set_stroke_op_mode(&mut self, wire: u8) {
        let op = StrokeOp::from_wire(wire);
        self.paint.stroke_op_mode = op;
        if self.is_editing_shape() {
            self.paint.active_op = op;
            self.refill_open_shape(); // the combined boolean result may change → recompose the pixels
        }
    }

    /// The current panel Operation mode as a wire `u8` (for the snapshot / tests).
    pub fn stroke_op_mode(&self) -> u8 {
        self.paint.stroke_op_mode.to_wire()
    }

    /// Serialize the ACTIVE editor (Curve/Ellipse/Polygon/Line) into the parked set with the active op,
    /// then clear the live slots — WITHOUT baking pixels (the shape keeps painting via `recompose`). Bakes
    /// any live Offset into the geometry first so the parked copy is self-contained. No-op when idle.
    pub(crate) fn park_active_shape(&mut self) {
        // Do NOT bake the Offset and do NOT reset the slider: the one Offset slider is GLOBAL — every shape
        // (active + parked) is offset by the live value in `parked_shape_dabs`, so parking keeps the raw
        // geometry (the base) and leaves the slider where it is (Enio 2026-07-04: offset acts on all at once).
        let Some(state) = self.capture_shape() else {
            return;
        };
        self.paint.parked_shapes.push(StrokeShape {
            state: *state,
            op: self.paint.active_op,
        });
        self.paint.curve = None;
        self.paint.ellipse = None;
        self.paint.polygon = None;
        self.paint.line = None;
        // Re-stamp the parked shapes onto the pristine canvas so they stay visible with no active editor.
        self.restamp_shapes_preview(&[]);
    }

    /// **Apaga a figura em mãos** (a tecla Delete) — a última selecionada, e só ela. `true` quando havia
    /// uma.
    ///
    /// Gêmeo exato do [`Self::park_active_shape`] **menos o empurrão**: parquear guarda a geometria na
    /// lista e apagar a joga fora; as duas depois limpam os slots vivos e re-carimbam o conjunto que
    /// sobrou sobre a linha de base pristina. Nada mais precisa saber que uma figura sumiu, porque os
    /// pixels **são cache derivado** do conjunto (a invariante do cabeçalho deste módulo).
    ///
    /// ⚠️ **Não é o Esc.** O `cancel_open_shape` descarta o conjunto INTEIRO (ativa + parqueadas) e não
    /// deixa passo de undo, porque nada foi commitado; este apaga UMA figura, deixa as outras de pé e é
    /// **um passo de undo** — o artista tem de poder trazê-la de volta.
    ///
    /// ⚠️ **Nada fica selecionado depois**, de propósito: promover uma parqueada a ativa escolheria por
    /// ele qual — e a próxima que ele quer é a que ele vai clicar, não a que a lista tinha por último.
    pub fn delete_active_shape(&mut self) -> bool {
        if self.capture_shape().is_none() {
            return false; // nenhuma figura em mãos — a tecla segue para quem mais a quiser
        }
        self.begin_shape_txn(); // apagar é UM passo de undo
        self.paint.curve = None;
        self.paint.ellipse = None;
        self.paint.polygon = None;
        self.paint.line = None;
        // O relevo e a sessão de sculpt descreviam a figura que acabou de sair: deixá-los de pé faria a
        // luz iluminar uma crista sobre tinta que não está mais lá (a mesma frase do Esc, e do
        // `restamp_shapes_preview` sob a mão).
        self.reset_stroke_height();
        self.restamp_reset_sculpt();
        self.restamp_shapes_preview(&[]);
        self.commit_shape_txn();
        true
    }

    /// Pull parked shape `i` back into the live editor (its op becomes the active op), parking whatever is
    /// currently active first. `true` when `i` was in range. The stroke method follows the restored shape.
    pub(crate) fn activate_parked_shape(&mut self, i: usize) -> bool {
        if i >= self.paint.parked_shapes.len() {
            return false;
        }
        // Park the current active shape first (if any), so switching never drops it.
        let had_active = self.capture_shape().is_some();
        if had_active {
            self.park_active_shape();
            // `park_active_shape` pushed the previously-active shape at the end; `i` still points at the
            // originally-targeted entry (it was before the push).
        }
        let StrokeShape { state, op } = self.paint.parked_shapes.remove(i);
        self.paint.active_op = op;
        // Rebuild the live editor from the geometry + point the tool at its method.
        self.install_shape_editor(state);
        // Re-stamp active + remaining parked from the pristine baseline.
        self.refill_open_shape();
        true
    }

    /// Rebuild the live editor slot + stroke method from a captured [`crate::undo::ShapeEditState`]. Shared
    /// by [`Self::activate_parked_shape`] and the undo restore path.
    pub(crate) fn install_shape_editor(&mut self, state: crate::undo::ShapeEditState) {
        use crate::undo::ShapeEditState as S;
        self.paint.curve = None;
        self.paint.ellipse = None;
        self.paint.polygon = None;
        self.paint.line = None;
        match state {
            S::Curve(s) => {
                self.paint.curve = Some(curve::CurveEditor::from_state(s));
                self.paint.brush.stroke_method = StrokeMethod::Arc;
            }
            S::Ellipse(s) => {
                self.paint.ellipse = Some(ellipse::EllipseEditor::from_state(s));
                self.paint.brush.stroke_method = StrokeMethod::Ellipse;
            }
            S::Polygon(s) => {
                self.paint.polygon = Some(polygon::PolygonEditor::from_state(s));
                self.paint.brush.stroke_method = StrokeMethod::Polygon;
            }
            S::Line(s) => {
                self.paint.line = Some(line::LineEditor::from_state(s));
                self.paint.brush.stroke_method = StrokeMethod::Line;
            }
        }
    }

    /// Stamp EVERY shape as one drag-preview batch — the single seam the four `*_refill` paths funnel
    /// through. Overlay + open shapes paint their own outline; Add/Remove CLOSED shapes are BOOLEAN-composited
    /// (union / subtract, separate regions stay individual) and the combined contour is stroked instead. The
    /// ACTIVE shape keeps its exact `own` dabs when it paints an outline (no regression); with no parked
    /// shapes and an Overlay active shape this is byte-identical to the former single-shape stamp.
    pub(super) fn restamp_shapes_preview(&mut self, own: &[Dab]) {
        // ⚠️ **Sob a mão, o GIZMO é o preview** (`super::shape_draft`): descasca e volta. O composite
        // booleano abaixo é `O(área da união × SS²)` — 284 ms dos 308 de um move na cena de quatro
        // círculos de 900 px —, e ele roda **por quadro do arrasto** para produzir um contorno que a
        // mão vai invalidar no quadro seguinte. O guia amarelo já mostra onde a figura está, e ele é
        // desenhado no vector scene, fora daqui.
        if self.draft_stamp() {
            self.paint.shape_stale = true;
            self.peel_drag_preview();
            // O envelope de relevo e a sessão de sculpt descreviam o carimbo que acabou de sair da
            // tela — deixá-los de pé faria a próxima luz iluminar uma figura que não está mais lá.
            self.reset_stroke_height();
            self.restamp_reset_sculpt();
            self.restamp_reset_erase(); // idem, no ramo do rascunho sob a mao
            return;
        }
        // A partir daqui a tela volta a mostrar as figuras abertas.
        self.paint.shape_stale = false;
        // ⚠️ **`Style: Solid` NÃO sai mais daqui** (W7, ordem do Enio 2026-08-15: *"Solid deve usar o
        // pincel com o falloff e espessura do traço como no modo flip"*). Até aqui esta função
        // retornava com a mancha e sem contorno — era assim que a espessura do pincel deixava de
        // entrar. Agora a figura é a região **MAIS** o traço, como um `FlipStroke` com `fill`: a
        // lista de dabs segue o caminho normal e o `stamp_drag_preview` escreve a mancha dentro da
        // MESMA transação (`super::solid_deposit` diz por que ela não pode ser uma segunda).
        let off = self.shape_offset_px();
        // The ACTIVE shape's state + op (captured), so it can join the boolean composite or keep its `own`.
        let active = self.capture_shape().map(|s| (*s, self.paint.active_op));
        let active_is_bool = active.as_ref().is_some_and(|(s, op)| {
            *op != StrokeOp::Overlay && stroke_boolean::is_boolean_fillable(s)
        });

        // 1) Outlines: the active shape's own dabs (unless it joins the boolean), + parked Overlay / open.
        let mut dabs = Vec::new();
        if !active_is_bool {
            dabs.extend_from_slice(own);
        }
        let mut scratch = Vec::new();
        let parked: Vec<StrokeShape> = self.paint.parked_shapes.clone();
        for s in &parked {
            if s.op == StrokeOp::Overlay || !stroke_boolean::is_boolean_fillable(&s.state) {
                scratch.clear();
                self.parked_shape_dabs(&s.state, &mut scratch);
                dabs.extend_from_slice(&scratch);
            }
        }

        // 2) Boolean composite over the Add/Remove CLOSED shapes (active + parked), stroked per contour.
        let mut bool_shapes: Vec<(crate::undo::ShapeEditState, StrokeOp)> = parked
            .iter()
            .filter(|s| s.op != StrokeOp::Overlay && stroke_boolean::is_boolean_fillable(&s.state))
            .map(|s| (s.state.clone(), s.op))
            .collect();
        if active_is_bool {
            bool_shapes.push(active.unwrap());
        }
        if !bool_shapes.is_empty() {
            for contour in self.stroke_boolean_contours(&bool_shapes, off) {
                if contour.len() >= 2 {
                    let mut stroke =
                        Stroke::new(self.stroke_spec(), self.paint.dynamics, self.paint.seed);
                    scratch.clear();
                    stroke.fill_polyline_preview(&contour, &mut scratch);
                    dabs.extend_from_slice(&scratch);
                }
            }
        }
        // #3 (doc 13): with Watercolor on, the shape's preview/bake runs the OPTICAL wash (frozen base +
        // rim / warp) instead of the plain deposit — the shape editors' one seam into the render-path.
        if self.watercolor_render_active() {
            self.stamp_drag_preview_watercolor(&dabs);
        } else {
            self.stamp_drag_preview(&dabs);
        }
    }

    /// Fill `out` with the dabs for one parked shape, applying the LIVE Offset slider so the one global
    /// Offset acts on EVERY shape simultaneously and in real time (Enio 2026-07-04), not just the active one.
    /// Mirrors each `*_refill`'s geometry→(offset)→spine→`fill_*_preview` core, minus the stamp.
    fn parked_shape_dabs(&self, st: &crate::undo::ShapeEditState, out: &mut Vec<Dab>) {
        use crate::undo::ShapeEditState as S;
        let off = self.shape_offset_px();
        let trim = self.paint.offset_trim;
        let mut stroke;
        match st {
            S::Curve(c) => {
                // DRAWING-ONLY offset (Enio 2026-07-05): the PERFECT CAD offset spine, control points pristine.
                let spine = self.offset_curve_spine(&c.points, &c.handles, c.closed, off);
                stroke = Stroke::new(self.stroke_spec(), self.paint.dynamics, c.seed);
                stroke.fill_polyline_preview(&spine, out);
            }
            S::Ellipse(e) => {
                let (erx, ery) = self.ellipse_offset_radii(e.rx, e.ry);
                stroke = Stroke::new(self.stroke_spec(), self.paint.dynamics, e.seed);
                stroke.fill_ellipse_preview(e.center, e.u, erx, ery, out);
            }
            S::Polygon(p) => {
                let (erx, ery) = ((p.rx + off).max(0.5), (p.ry + off).max(0.5));
                stroke = Stroke::new(self.stroke_spec(), self.paint.dynamics, p.seed);
                stroke.fill_polygon_preview(p.center, p.u, erx, ery, p.sides, out);
            }
            S::Line(l) => {
                let mods: Vec<line_corner::CornerMod> = l
                    .corner_mods
                    .iter()
                    .copied()
                    .map(line_corner::CornerMod::from_wire)
                    .collect();
                let mut path =
                    line_corner::expand(&l.points, l.closed, &mods, self.paint.shape_grab_tol_px);
                path = line_offset::offset_polyline(&path, l.closed, off);
                if trim && off != 0.0 {
                    path = if l.closed {
                        curve_trim::trim_self_intersections_closed(&path)
                    } else {
                        curve_trim::trim_self_intersections(&path)
                    };
                }
                if path.len() >= 2 {
                    stroke = Stroke::new(self.stroke_spec(), self.paint.dynamics, l.seed);
                    stroke.fill_polyline_preview(&path, out);
                }
            }
        }
    }

    /// Capture the parked shapes for a [`crate::undo::ModelSnapshot`] (geometry + wire op).
    pub(crate) fn parked_shapes_snapshot(&self) -> Vec<crate::undo::ParkedShapeState> {
        self.paint
            .parked_shapes
            .iter()
            .map(|s| crate::undo::ParkedShapeState {
                state: s.state.clone(),
                op: s.op.to_wire(),
            })
            .collect()
    }

    /// Reinstate the parked shapes from a restored snapshot (inverse of [`Self::parked_shapes_snapshot`]).
    pub(crate) fn restore_parked_shapes(&mut self, parked: Vec<crate::undo::ParkedShapeState>) {
        self.paint.parked_shapes = parked
            .into_iter()
            .map(|p| StrokeShape {
                state: p.state,
                op: StrokeOp::from_wire(p.op),
            })
            .collect();
    }

    /// Drop every parked shape (Apply / cancel / teardown clear the whole multi-shape set).
    pub(crate) fn clear_parked_shapes(&mut self) {
        self.paint.parked_shapes.clear();
    }

    /// Route a canvas pointer through the shape editors WITH multi-shape switching: on a Down that lands on
    /// a PARKED shape re-activate it (parking the current one first); on a Down in EMPTY space with a
    /// complete active shape, park it so the following `*_down` begins a fresh shape. Then dispatch to the
    /// active editor by method (the classic single-shape path). Line point-placement is never interrupted.
    pub(super) fn route_shape_pointer_multi(&mut self, mut ev: CanvasPointer) -> bool {
        // Seamless Tiling (edit-in-tile, Enio 2026-07-11): the shape's wash + editable overlay are drawn in
        // the wrapped neighbour tiles too, so a grab/drag on a tile COPY must edit the ORIGINAL. Fix ONE tile
        // offset at the grab Down — the copy that lands the pointer on the active shape's bbox — and subtract
        // it from EVERY pointer of the gesture. A fixed per-gesture offset (not a per-sample `rem_euclid`)
        // keeps the drag continuous across the seam (no size jump on Ellipse/Polygon) AND edits geometry drawn
        // beyond the sprite. Suppressed while a Free Hand path is DRAWN (raw capture stays continuous).
        let drawing_freehand = self
            .paint
            .curve
            .as_ref()
            .is_some_and(|c| c.is_drawing_freehand());
        if !drawing_freehand {
            if ev.phase == PointerPhase::Down {
                self.paint.shape_edit_wrap = self.shape_edit_tile_offset(ev.pos);
            }
            ev.pos[0] -= self.paint.shape_edit_wrap[0];
            ev.pos[1] -= self.paint.shape_edit_wrap[1];
        }
        let slop = self.paint.shape_grab_tol_px;
        match ev.phase {
            PointerPhase::Down => {
                self.maybe_switch_or_new_shape(ev.pos);
                // Arm an op-cycle tap iff the Down lands on the active shape's centre square.
                self.paint.op_tap = self
                    .on_active_centre_square(ev.pos, slop * 2.0)
                    .then_some(ev.pos);
            }
            PointerPhase::Move => {
                // A drag past the slop is a MOVE, not an op tap — disarm.
                if let Some(p) = self.paint.op_tap
                    && dist2(ev.pos, p) > slop * slop
                {
                    self.paint.op_tap = None;
                }
            }
            _ => {}
        }
        let consumed = match self.paint.brush.stroke_method {
            StrokeMethod::Arc | StrokeMethod::FreeHand => self.curve_pointer(ev),
            StrokeMethod::Ellipse => self.ellipse_pointer(ev),
            StrokeMethod::Polygon => self.polygon_pointer(ev),
            StrokeMethod::Line => self.line_pointer(ev),
            _ => false,
        };
        // Pen-up on an un-dragged centre-square tap → cycle the active shape's Operation (+ → − → o).
        if ev.phase == PointerPhase::Up
            && let Some(p) = self.paint.op_tap.take()
            && dist2(ev.pos, p) <= slop * slop
        {
            self.cycle_active_op();
        }
        consumed
    }

    /// `true` when `pos` is within `tol` of the ACTIVE shape's centre (its gizmo type-square). Used both to
    /// keep a centre click from being read as "empty space" and to arm the op-cycle tap.
    pub(super) fn on_active_centre_square(&self, pos: [f32; 2], tol: f32) -> bool {
        let Some(state) = self.capture_shape() else {
            return false;
        };
        let Some(bb) = self.shape_state_bbox(&state) else {
            return false;
        };
        let c = [(bb[0] + bb[2]) * 0.5, (bb[1] + bb[3]) * 0.5];
        dist2(pos, c) <= tol * tol
    }

    /// The seamless-Tiling **edit-in-tile** offset for a grab at `pos` ([`PaintState::shape_edit_wrap`]): the
    /// tile offset (px — whole sprite periods per tiled axis) that lands `pos` on the ACTIVE shape's bbox, so
    /// a grab on a WRAPPED overlay copy edits the original. `[0, 0]` when off-tiling, no active shape, or the
    /// snapped point misses the shape's grab region (a new-shape / empty click stays where it was clicked).
    fn shape_edit_tile_offset(&self, pos: [f32; 2]) -> [f32; 2] {
        if !(self.paint.tiling[0] || self.paint.tiling[1]) {
            return [0.0, 0.0];
        }
        let (fw, fh) = self.source_size;
        let (fw, fh) = (fw as f32, fh as f32);
        let tol = self.paint.shape_grab_tol_px.max(4.0);
        // Snap `p` to the nearest whole-sprite tile of the `[lo, hi]` span centre.
        let snap = |p: f32, lo: f32, hi: f32, period: f32| -> f32 {
            if period > 0.0 {
                ((p - (lo + hi) * 0.5) / period).round() * period
            } else {
                0.0
            }
        };
        // Consider the ACTIVE shape + every PARKED shape (`bb = [minx, miny, maxx, maxy]`): use the tile
        // offset of the FIRST whose grab region the snapped pointer lands in — so a grab OR a shape-switch
        // click works from ANY tile (multi-shape). No hit ⇒ empty click (new shape) → no wrap.
        let active = self.capture_shape().and_then(|s| self.shape_state_bbox(&s));
        let parked = self
            .paint
            .parked_shapes
            .iter()
            .filter_map(|s| self.shape_state_bbox(&s.state));
        for bb in active.into_iter().chain(parked) {
            let o = [
                if self.paint.tiling[0] {
                    snap(pos[0], bb[0], bb[2], fw)
                } else {
                    0.0
                },
                if self.paint.tiling[1] {
                    snap(pos[1], bb[1], bb[3], fh)
                } else {
                    0.0
                },
            ];
            let s = [pos[0] - o[0], pos[1] - o[1]];
            if s[0] >= bb[0] - tol
                && s[0] <= bb[2] + tol
                && s[1] >= bb[1] - tol
                && s[1] <= bb[3] + tol
            {
                return o;
            }
        }
        [0.0, 0.0]
    }

    /// Cycle the ACTIVE shape's Operation (+ → − → o), keep the panel mode in sync, and recompose (a boolean
    /// result may change). The `+`/`−`/`o` glyph in its gizmo updates live. One COALESCED undo entry — a run
    /// of consecutive taps rolls back to the pre-run op in one Ctrl+Z (Enio 2026-07-05; the tap previously
    /// recorded NO undo at all — `active_op` wasn't captured in the snapshot).
    pub(crate) fn cycle_active_op(&mut self) {
        if !self.is_editing_shape() {
            return;
        }
        self.flush_shape_txn();
        let before = self.capture_shape_model();
        self.paint.active_op = self.paint.active_op.cycle();
        self.paint.stroke_op_mode = self.paint.active_op;
        self.refill_open_shape();
        let after = self.capture_shape_model();
        self.undo.record_structural_coalesced(
            crate::undo::CoalesceKind::OpCycleStroke,
            before,
            after,
        );
    }

    /// The ACTIVE shape's Operation as its wire `u8` — the undo-snapshot accessor (the field is private to
    /// the `paint` module).
    pub(crate) fn active_op_wire(&self) -> u8 {
        self.paint.active_op.to_wire()
    }

    /// Reinstate the ACTIVE shape's Operation from a snapshot's wire value (undo/redo restore).
    pub(crate) fn set_active_op_wire(&mut self, wire: u8) {
        self.paint.active_op = StrokeOp::from_wire(wire);
    }

    /// The `+`/`−`/`o` glyph for the ACTIVE shape's Operation — the shell draws it INSIDE the shape's gizmo
    /// centre-move square (enlarged), so there is no separate badge square (Enio 2026-07-04). `None` when no
    /// shape is being edited.
    pub fn active_op_glyph(&self) -> Option<&'static str> {
        self.is_editing_shape()
            .then(|| self.paint.active_op.glyph())
    }

    /// The op badges the shell draws for the PARKED shapes only — each the shape's **CONTORNO** + its
    /// `+`/`−`/`o` glyph, so it reads as a still-editable shape. The ACTIVE shape's glyph is drawn INSIDE
    /// its gizmo centre square instead (`active_op_glyph`), so there is never a second/duplicate square
    /// (Enio 2026-07-04). Empty in the classic single-shape / no-parked case. Image px.
    ///
    /// ⚠️ O contorno vem de [`Self::shape_state_outline`], a MESMA porta que
    /// [`Self::hit_parked_shape`] pergunta — desenho e clique não podem discordar sobre onde a figura
    /// está. E ele **não constrói dabs**: a versão anterior pedia a AABB ao `shape_state_bbox`, que monta
    /// a lista de dabs inteira de cada figura parqueada a cada quadro para guardar quatro floats.
    pub fn stroke_op_badges(&self) -> Vec<StrokeOpBadge> {
        let parked: Vec<(crate::undo::ShapeEditState, StrokeOp)> = self
            .paint
            .parked_shapes
            .iter()
            .map(|s| (s.state.clone(), s.op))
            .collect();
        let mut out = Vec::new();
        for (st, op) in &parked {
            let Some(o) = self.shape_state_outline(st) else {
                continue;
            };
            let Some(bb) = o.bbox() else { continue };
            out.push(StrokeOpBadge {
                center: [(bb[0] + bb[2]) * 0.5, (bb[1] + bb[3]) * 0.5],
                outline: o.points,
                closed: o.closed,
                glyph: op.glyph(),
            });
        }
        out
    }

    /// AABB over a shape's stamped dab centres (reuses [`Self::parked_shape_dabs`]).
    pub(super) fn shape_state_bbox(&self, st: &crate::undo::ShapeEditState) -> Option<[f32; 4]> {
        let mut dabs = Vec::new();
        self.parked_shape_dabs(st, &mut dabs);
        let mut bb = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
        for d in &dabs {
            bb[0] = bb[0].min(d.center[0]);
            bb[1] = bb[1].min(d.center[1]);
            bb[2] = bb[2].max(d.center[0]);
            bb[3] = bb[3].max(d.center[1]);
        }
        (bb[0] <= bb[2]).then_some(bb)
    }

    /// Prepare the pixel baseline for a NEW shape session's idle-create branch. Always forces a full
    /// recompose (drops the composite cache), but drops the drag-preview RESTORE record ONLY when no shape
    /// is parked. With parked shapes present the baseline must persist — dropping it would leave the parked
    /// stamp on canvas as a false "pristine" base — so the new shape's first fill peels + re-stamps the
    /// whole set from the real pristine baseline instead.
    pub(super) fn begin_shape_session_base(&mut self) {
        // A freshly-created shape adopts the current panel Operation mode.
        self.paint.active_op = self.paint.stroke_op_mode;
        if !self.has_parked_shapes() {
            // Truly fresh session: drop the baseline + re-centre the Offset. With shapes already parked the
            // baseline persists AND the Offset slider stays put — it is GLOBAL across the whole set, so a new
            // shape must not reset it (else the parked shapes would jump back to zero offset).
            self.paint.drag_preview = None;
            self.paint.shape_offset_base_px = 0.0;
            self.paint.shape_offset_norm = 0.5;
        }
        self.reseed_preview_base();
    }
}

/// Minimum distance (px), regardless of grab tolerance, within which a click on a Curve counts as an
/// insert-on-curve EDIT rather than a "start a new shape". Keeps the new-shape gesture from firing on a
/// click that visually reads as "on the curve".
pub(super) const NEW_SHAPE_INSERT_BAND_PX: f32 = 20.0;
