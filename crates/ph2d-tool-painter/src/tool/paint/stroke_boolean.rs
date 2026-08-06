//! **Stroke boolean composite** (Enio 2026-07-04): Add/Remove stroke shapes combine like the Selection ops
//! — overlapping regions union (Add) / subtract (Remove); SEPARATE regions stay individual (each traced as
//! its own contour); Overlay shapes never combine. Reuses the Selection rasteriser + `combine_into` +
//! multi-component contour trace, so a group of fillable Add/Remove shapes becomes the STROKED outline of
//! the combined region (the inner arcs where they overlap vanish, as in a real boolean).

use super::selection_shapes::{
    SelectionShape, combine_into, rasterize_ellipse, sel_polygon_vertices,
};
use super::stroke_multi::StrokeOp;
use super::*;
use crate::undo::ShapeEditState;

/// SUPERSAMPLE factor for the boolean mask: rasterise + trace at `SS×` the canvas resolution, then scale the
/// contour back down, so the stroked boolean outline reads as SMOOTH as a direct (analytic) ellipse outline —
/// no visible raster staircase (Enio 2026-07-04 wants a "perfect" Add/Remove appearance). Transcendental-free.
const SS: usize = 3;

/// O wire de [`StrokeOp::Add`] — a Operation que CONTRIBUI região, e por isso a única que define a
/// janela do composite.
const ADD_WIRE: u8 = 1;

/// Build an editing [`CurveState`](crate::undo::CurveState) from a fitted [`CurveModel`] — installs the
/// boolean-composite result as one editable Curve.
fn curve_state_from_model(model: curve_model::CurveModel, seed: u64) -> crate::undo::CurveState {
    let anchor = model.points.first().copied().unwrap_or([0.0, 0.0]);
    crate::undo::CurveState {
        kinds: model.kinds.iter().map(|k| k.wire()).collect(),
        points: model.points,
        handles: model.handles,
        selected: None,
        added_point: false,
        closed: model.closed,
        editing: true,
        freehand: true,
        seed,
        anchor,
        stabilized: anchor,
    }
}

/// Whether a stroke shape is a CLOSED fillable region that can take part in the boolean composite (Ellipse,
/// Polygon, closed Curve, closed Line). Open shapes (open Line / open Curve) always paint their own outline.
pub(super) fn is_boolean_fillable(st: &ShapeEditState) -> bool {
    matches!(st, ShapeEditState::Ellipse(_) | ShapeEditState::Polygon(_))
        || matches!(st, ShapeEditState::Curve(c) if c.closed)
        || matches!(st, ShapeEditState::Line(l) if l.closed)
}

impl PainterTool {
    /// Convert a boolean-fillable stroke shape to a [`SelectionShape`] (to reuse the selection rasteriser),
    /// with the live Offset `off` folded into its geometry so the composite tracks the Offset slider. `None`
    /// for shapes that don't fill (open Curve / Line).
    fn stroke_state_to_fill_shape(&self, st: &ShapeEditState, off: f32) -> Option<SelectionShape> {
        match st {
            ShapeEditState::Ellipse(e) => Some(SelectionShape::Ellipse {
                center: e.center,
                u: e.u,
                rx: (e.rx + off).max(0.5),
                ry: (e.ry + off).max(0.5),
            }),
            ShapeEditState::Polygon(p) => {
                // Use the SAME perimeter the gizmo + fill draw (`polygon_perimeter`, first vertex at the top)
                // as a Freehand polygon — NOT `SelectionShape::Polygon`, whose CORNER-seeded vertices (45°
                // phase) made the boolean outline diverge from the gizmo on rotation (Enio 2026-07-04).
                let mut perim = Vec::new();
                ph2d_painter_brush::polygon_perimeter(
                    p.center,
                    p.u,
                    (p.rx + off).max(0.5),
                    (p.ry + off).max(0.5),
                    p.sides,
                    &mut perim,
                );
                if perim.len() < 3 {
                    return None;
                }
                let handles = perim.iter().map(|q| [*q, *q]).collect(); // sharp corners
                let model = curve_model::CurveModel::from_curve(
                    perim,
                    handles,
                    true,
                    curve_handle::HandleKind::Free,
                );
                Some(SelectionShape::Freehand {
                    model,
                    u: [1.0, 0.0],
                })
            }
            ShapeEditState::Curve(c) if c.closed => {
                let (pts, handles, _) =
                    curve_offset::offset_curve_refined(&c.points, &c.handles, off, true);
                let model = curve_model::CurveModel::from_curve(
                    pts,
                    handles,
                    true,
                    curve_handle::HandleKind::Aligned,
                );
                Some(SelectionShape::Freehand {
                    model,
                    u: [1.0, 0.0],
                })
            }
            ShapeEditState::Line(l) if l.closed => {
                // Expand the corner Fillet/Chamfer + offset the parallel polyline, then fill it as a sharp
                // Freehand polygon (no Bézier handles → the rasteriser fills the raw polygon).
                let mods: Vec<line_corner::CornerMod> = l
                    .corner_mods
                    .iter()
                    .copied()
                    .map(line_corner::CornerMod::from_wire)
                    .collect();
                let expanded =
                    line_corner::expand(&l.points, true, &mods, self.paint.shape_grab_tol_px);
                let path = line_offset::offset_polyline(&expanded, true, off);
                if path.len() < 3 {
                    return None;
                }
                let handles = path.iter().map(|p| [*p, *p]).collect(); // sharp corners
                let model = curve_model::CurveModel::from_curve(
                    path,
                    handles,
                    true,
                    curve_handle::HandleKind::Free,
                );
                Some(SelectionShape::Freehand {
                    model,
                    u: [1.0, 0.0],
                })
            }
            _ => None,
        }
    }

    /// Convert one stroke shape STATE to its own editable **dense** [`CurveState`](crate::undo::CurveState) —
    /// the PRISTINE geometry (the live Offset is NOT baked in; it persists as a drawing transform, so every
    /// anchor stays exactly on the shape and no offset self-cross can land in the control points — Enio
    /// 2026-07-05). Ellipse → its exact 4-arc, Polygon/closed-Line → sharp corners, existing Curve → itself.
    /// `None` for an OPEN Line. The per-shape analogue of the Selection `shape_to_closed_curve`; every result
    /// densifies to the Convert anchor spacing.
    pub(super) fn stroke_state_to_curve_state(
        &mut self,
        st: &ShapeEditState,
    ) -> Option<crate::undo::CurveState> {
        use super::curve::{CONVERT_ANCHOR_SPACING_PX, MAX_CONVERT_POINTS};
        let seed = self.paint.seed;
        self.paint.seed = self.paint.seed.wrapping_add(1);
        let densify = |p: Vec<[f32; 2]>, h: Vec<[[f32; 2]; 2]>| {
            curve_geom::densify_closed_curve(&p, &h, CONVERT_ANCHOR_SPACING_PX, MAX_CONVERT_POINTS)
        };
        let model = match st {
            ShapeEditState::Ellipse(e) => {
                let (p, h) =
                    super::selection_edit::ellipse_to_closed_curve(e.center, e.u, e.rx, e.ry);
                let (p, h) = densify(p, h);
                curve_model::CurveModel::from_curve(p, h, true, curve_handle::HandleKind::Aligned)
            }
            ShapeEditState::Polygon(pg) => {
                let mut perim = Vec::new();
                ph2d_painter_brush::polygon_perimeter(
                    pg.center, pg.u, pg.rx, pg.ry, pg.sides, &mut perim,
                );
                if perim.len() < 3 {
                    return None;
                }
                let handles = perim.iter().map(|q| [*q, *q]).collect();
                let (p, h) = densify(perim, handles);
                curve_model::CurveModel::from_curve(p, h, true, curve_handle::HandleKind::Free)
            }
            ShapeEditState::Curve(c) => {
                // Keep the pristine curve verbatim (its own anchors + handles + kinds) — no offset bake.
                let kinds: Vec<curve_handle::HandleKind> = c
                    .kinds
                    .iter()
                    .map(|k| {
                        curve_handle::HandleKind::from_wire(*k)
                            .unwrap_or(curve_handle::HandleKind::Aligned)
                    })
                    .collect();
                curve_model::CurveModel {
                    points: c.points.clone(),
                    handles: c.handles.clone(),
                    kinds,
                    selected: None,
                    closed: c.closed,
                }
            }
            ShapeEditState::Line(l) if l.closed => {
                let mods: Vec<line_corner::CornerMod> = l
                    .corner_mods
                    .iter()
                    .copied()
                    .map(line_corner::CornerMod::from_wire)
                    .collect();
                let expanded =
                    line_corner::expand(&l.points, true, &mods, self.paint.shape_grab_tol_px);
                // Pristine expanded polyline (offset stays a drawing transform).
                let path = line_offset::offset_polyline(&expanded, true, 0.0);
                if path.len() < 3 {
                    return None;
                }
                let handles = path.iter().map(|p| [*p, *p]).collect();
                let (p, h) = densify(path, handles);
                curve_model::CurveModel::from_curve(p, h, true, curve_handle::HandleKind::Free)
            }
            ShapeEditState::Line(_) => return None, // an OPEN line is not a closed curve — keep as-is
        };
        Some(curve_state_from_model(model, seed))
    }

    /// Boolean-composite the Add/Remove fillable `shapes` into contour polylines (image px): rasterize each
    /// at `SS×` resolution, union (Add) / subtract (Remove) in DRAW ORDER, trace EVERY connected component
    /// (separate regions → separate contours; overlapping → merged), then scale the contours back to canvas
    /// px. The supersample + the tracer's smoothing make the stroked outline as regular as a direct ellipse.
    pub(super) fn stroke_boolean_contours(
        &self,
        shapes: &[(ShapeEditState, StrokeOp)],
        off: f32,
    ) -> Vec<Vec<[f32; 2]>> {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        if w == 0 || h == 0 {
            return Vec::new();
        }
        // ⚠️ **A JANELA é a das formas, não a do canvas** — medido em 2026-08-06, e a diferença é de
        // duas ordens de grandeza. Este composite roda a CADA move de ponteiro (o `restamp_shapes_preview`
        // o chama sempre que há uma forma com Operation), e ele alocava e percorria um buffer
        // supersampleado do **canvas inteiro**: a `SS = 3` isso é 12288² = **151 MB** a 4096², zerado uma
        // vez para o `crisp`, outra por forma para o `region`, mais duas dentro do traçado — e o
        // `scanline_fill` percorria as 12288 linhas para desenhar uma figura de 400 px.
        //
        // Medido pela porta do artista, com a MESMA figura de 200 px nas três telas (um move, mediana):
        //
        // | tela | Overlay | 1 Add |
        // |------|---------|-------|
        // | 1024 |  1,30   |  21,7 |
        // | 2048 |  1,40   |  77,8 |
        // | 4096 |  1,50   | 284,4 |
        //
        // A coluna Overlay é **plana** (a figura não mudou); a de Add cresce **4× por 4× de área**. O
        // custo era do BUFFER, não do desenho — marcar `Add` numa forma custava 190× um move normal por
        // uma razão que não tem nada a ver com o que o artista pediu.
        //
        // ⚠️ A janela sai das formas que **ADICIONAM**: o resultado de um boolean está contido na união
        // dos Add, então uma forma de Remove longe dali não pode mudar um texel — incluí-la só faria o
        // buffer crescer de novo. Sem nenhum Add não há região nenhuma, e a saída é vazia.
        let mut fills: Vec<(SelectionShape, u8)> = Vec::with_capacity(shapes.len());
        let mut bb: Option<[f32; 4]> = None;
        for (st, op) in shapes {
            let Some(sel) = self.stroke_state_to_fill_shape(st, off) else {
                continue;
            };
            let wire = op.to_wire();
            if wire == ADD_WIRE
                && let Some(r) = self.fill_shape_bounds(&sel)
            {
                bb = Some(bb.map_or(r, |a| {
                    [
                        a[0].min(r[0]),
                        a[1].min(r[1]),
                        a[2].max(r[2]),
                        a[3].max(r[3]),
                    ]
                }));
            }
            fills.push((sel, wire));
        }
        let Some(bb) = bb else {
            return Vec::new();
        };
        let s = SS as f32;
        // ⚠️ **A janela NÃO precisa de borda de zeros, e isto foi lido no traçador, não suposto.** Eu
        // escrevi que uma folga era o que mantinha o traçado idêntico — *"uma componente colada na
        // borda do buffer não é percorrida como uma que tem zeros em volta"* — e armei a mutação que
        // deveria sangrar. Ela **passou**, nos dez casos do gate, inclusive numa figura no meio do
        // canvas cuja caixa apertada a encosta na coluna 0 da janela. O motivo está no
        // `selection_trace::inside`: ele confere os limites e devolve `false` fora do buffer, ou seja
        // **fora lê como FUNDO** — exatamente o que uma borda de zeros daria. A folga foi removida em
        // vez de ficar com uma justificativa falsa ao lado.
        //
        // O que de fato mantém a identidade é o `clamp` ao canvas: onde a figura sai da tela a janela
        // recorta no MESMO lugar em que o buffer de tela cheia recortava.
        let cap = |v: f32, hi: usize| -> usize {
            #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
            let hi_f = hi as f32;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let out = v.clamp(0.0, hi_f) as usize;
            out.min(hi)
        };
        let (ssw, ssh) = (w * SS, h * SS);
        let x0 = cap((bb[0] * s).floor(), ssw);
        let y0 = cap((bb[1] * s).floor(), ssh);
        let x1 = cap((bb[2] * s).ceil(), ssw);
        let y1 = cap((bb[3] * s).ceil(), ssh);
        if x1 <= x0 || y1 <= y0 {
            return Vec::new();
        }
        let (sw, sh) = (x1 - x0, y1 - y0);
        #[allow(clippy::cast_precision_loss)]
        let origin = [x0 as f32, y0 as f32];
        let mut crisp = vec![0u8; sw * sh];
        // ⚠️ **UM `region` para todas as formas.** Ele era alocado dentro do laço, uma vez por forma —
        // com a janela do canvas isso eram 151 MB por forma, por move.
        let mut region = vec![0u8; sw * sh];
        for (sel, wire) in &fills {
            region.fill(0);
            self.rasterize_fill_ss(sel, sw, sh, origin, &mut region);
            combine_into(&mut crisp, &region, *wire);
        }
        super::selection_trace::trace_all_contours(&crisp, sw, sh)
            .into_iter()
            .map(|c| {
                c.iter()
                    .map(|p| [(p[0] + origin[0]) / s, (p[1] + origin[1]) / s])
                    .collect()
            })
            .collect()
    }

    /// **A rota da TELA CHEIA, congelada** — o código que este módulo rodava até 2026-08-06, verbatim
    /// no que importa: o buffer é o canvas supersampleado inteiro e o `origin` é a origem dele.
    ///
    /// ⚠️ Ela existe como oráculo e **só como oráculo**. Um `pub(super)` sem `cfg` seria uma segunda
    /// resposta a *"que contornos este conjunto de formas produz?"*, esperando alguém chamá-la — a
    /// lição que o `warp_axis` e o `serial_side` já pagaram nesta linha. Com o `cfg` o gate compara a
    /// janela nova contra **o que shipava**, e não contra uma reimplementação escrita para o teste.
    #[cfg(test)]
    pub(super) fn stroke_boolean_contours_whole_canvas(
        &self,
        shapes: &[(ShapeEditState, StrokeOp)],
        off: f32,
    ) -> Vec<Vec<[f32; 2]>> {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        if w == 0 || h == 0 {
            return Vec::new();
        }
        let (sw, sh) = (w * SS, h * SS);
        let mut crisp = vec![0u8; sw * sh];
        let mut any = false;
        for (st, op) in shapes {
            let Some(sel) = self.stroke_state_to_fill_shape(st, off) else {
                continue;
            };
            let mut region = vec![0u8; sw * sh];
            self.rasterize_fill_ss(&sel, sw, sh, [0.0, 0.0], &mut region);
            combine_into(&mut crisp, &region, op.to_wire());
            any = true;
        }
        if !any {
            return Vec::new();
        }
        let s = SS as f32;
        super::selection_trace::trace_all_contours(&crisp, sw, sh)
            .into_iter()
            .map(|c| c.iter().map(|p| [p[0] / s, p[1] / s]).collect())
            .collect()
    }

    /// A caixa de uma forma de preenchimento, em px de imagem — **um SUPERCONJUNTO**, nunca apertada.
    ///
    /// ⚠️ Para a Freehand a caixa sai dos pontos **e das alças**, não da espinha achatada: uma Bézier
    /// vive dentro do casco convexo dos próprios controles, então essa caixa a contém sem que o
    /// `flatten_spine` precise rodar duas vezes (uma para medir, outra para desenhar). Superconjunto é a
    /// direção segura: uma caixa grande demais custa alguns texels, uma pequena demais **corta a figura
    /// em silêncio**.
    fn fill_shape_bounds(&self, sel: &SelectionShape) -> Option<[f32; 4]> {
        let hull = |pts: &[[f32; 2]]| -> Option<[f32; 4]> {
            pts.iter().fold(None, |acc: Option<[f32; 4]>, p| {
                Some(acc.map_or([p[0], p[1], p[0], p[1]], |a| {
                    [
                        a[0].min(p[0]),
                        a[1].min(p[1]),
                        a[2].max(p[0]),
                        a[3].max(p[1]),
                    ]
                }))
            })
        };
        match sel {
            // A caixa exata de uma elipse girada: o extremo de `c + rx·cos t·u + ry·sin t·v` é
            // `hypot(rx·u, ry·v)` em cada eixo, com `v = perp(u)`.
            SelectionShape::Ellipse { center, u, rx, ry } => {
                let hx = ((rx * u[0]).powi(2) + (ry * u[1]).powi(2)).sqrt();
                let hy = ((rx * u[1]).powi(2) + (ry * u[0]).powi(2)).sqrt();
                Some([
                    center[0] - hx,
                    center[1] - hy,
                    center[0] + hx,
                    center[1] + hy,
                ])
            }
            SelectionShape::Polygon {
                center,
                u,
                rx,
                ry,
                sides,
            } => hull(&sel_polygon_vertices(*center, *u, *rx, *ry, *sides)),
            SelectionShape::Freehand { model, .. } => {
                let mut pts: Vec<[f32; 2]> = model.points.clone();
                for h in &model.handles {
                    pts.push(h[0]);
                    pts.push(h[1]);
                }
                hull(&pts)
            }
            // O rasterizador não desenha um Raster aqui, então ele não tem caixa a contribuir.
            SelectionShape::Raster { .. } => None,
        }
    }

    /// **Merge Curves** (the panel's Merge button): fold EVERY fillable shape the artist sees — active +
    /// parked, **Overlay counted as Add** (union) so the whole visible fill collapses, exactly like the
    /// Selection Merge — into one/few high-precision dense curves by tracing the composed mask. Add/Remove
    /// still union/subtract; separate regions each become their own dense curve. NON-fillable shapes (open
    /// Line / open Curve) are preserved untouched. One undo step. `false` when nothing fillable (Enio
    /// 2026-07-05). Supersedes the former boolean-only auto-merge inside Convert.
    pub(crate) fn merge_open_shapes_to_curves(&mut self) -> bool {
        let off = self.shape_offset_px();
        // Overlay → Add for the union (a plain-overlay shape still contributes its region to the merged fill).
        let as_union = |op: StrokeOp| {
            if op == StrokeOp::Overlay {
                StrokeOp::Add
            } else {
                op
            }
        };
        let mut shapes: Vec<(ShapeEditState, StrokeOp)> = self
            .paint
            .parked_shapes
            .iter()
            .filter(|s| is_boolean_fillable(&s.state))
            .map(|s| (s.state.clone(), as_union(s.op)))
            .collect();
        // A non-fillable ACTIVE shape (open Line/Curve) must survive the merge — re-park it afterwards.
        let mut keep_active: Option<ShapeEditState> = None;
        if let Some(active) = self.capture_shape() {
            if is_boolean_fillable(&active) {
                shapes.push((*active, as_union(self.paint.active_op)));
            } else {
                keep_active = Some(*active);
            }
        }
        if shapes.is_empty() {
            return false; // nothing fillable to merge
        }
        let contours = self.stroke_boolean_contours(&shapes, off);
        let mut states: Vec<ShapeEditState> = Vec::new();
        for c in &contours {
            if let Some(m) = super::selection_edit::to_closed_curve_smooth(c) {
                let seed = self.paint.seed;
                self.paint.seed = self.paint.seed.wrapping_add(1);
                states.push(ShapeEditState::Curve(curve_state_from_model(m, seed)));
            }
        }
        if states.is_empty() {
            return false;
        }
        let before = self.capture_shape_model();
        // Drop every fillable shape (they are now the merged curve); keep only NON-fillable parked ones.
        self.paint
            .parked_shapes
            .retain(|s| !is_boolean_fillable(&s.state));
        self.paint.curve = None;
        self.paint.ellipse = None;
        self.paint.polygon = None;
        self.paint.line = None;
        self.paint.shape_offset_base_px = 0.0;
        self.paint.shape_offset_norm = 0.5;
        self.paint.active_op = StrokeOp::Overlay; // the merged curve is a plain overlay
        // Re-park a preserved non-fillable active shape as its own Overlay entry (no shape is ever lost).
        if let Some(state) = keep_active {
            self.paint
                .parked_shapes
                .push(super::stroke_multi::StrokeShape {
                    state,
                    op: StrokeOp::Overlay,
                });
        }
        let first = states.remove(0);
        for s in states {
            self.paint
                .parked_shapes
                .push(super::stroke_multi::StrokeShape {
                    state: s,
                    op: StrokeOp::Overlay,
                });
        }
        self.install_shape_editor(first);
        self.refill_open_shape();
        let after = self.capture_shape_model();
        self.undo.record_structural(before, after);
        true
    }

    /// Rasterise one fill shape scaled by `SS` into the supersampled `crisp` via `op` (transcendental-free:
    /// exact ellipse inside-test / baked-step polygon / flattened freehand → scanline fill).
    /// Rasteriza UMA forma na janela local, com `origin` (em texels supersampleados) subtraído.
    ///
    /// ⚠️ **A janela é uma TELA VIRTUAL**, o mesmo truque da banda do `stamp_banded`: o rasterizador
    /// recebe as coordenadas deslocadas e a largura/altura da janela, e o recorte cai do `clamp` que
    /// ele já faz contra o tamanho da tela. Nenhuma segunda resposta a *"que pixels esta forma cobre?"*.
    fn rasterize_fill_ss(
        &self,
        sel: &SelectionShape,
        sw: usize,
        sh: usize,
        origin: [f32; 2],
        region: &mut [u8],
    ) {
        let s = SS as f32;
        let at = |p: [f32; 2]| [p[0] * s - origin[0], p[1] * s - origin[1]];
        match sel {
            SelectionShape::Ellipse { center, u, rx, ry } => {
                rasterize_ellipse(at(*center), *u, rx * s, ry * s, sw, sh, region);
            }
            SelectionShape::Polygon {
                center,
                u,
                rx,
                ry,
                sides,
            } => {
                let verts: Vec<[f32; 2]> = sel_polygon_vertices(*center, *u, *rx, *ry, *sides)
                    .iter()
                    .map(|p| at(*p))
                    .collect();
                scanline_fill(&verts, sw, sh, region);
            }
            SelectionShape::Freehand { model, .. } => {
                let spine = self.freehand_spine(&model.points, &model.handles);
                let verts: Vec<[f32; 2]> = spine.iter().map(|p| at(*p)).collect();
                scanline_fill(&verts, sw, sh, region);
            }
            SelectionShape::Raster { .. } => {}
        }
    }
}

/// Even-odd scanline polygon fill of the closed polyline `pts` into `cov` (0/255) at `w`×`h`. Mirrors the
/// selection `raster_lasso` but writes into a caller buffer at an arbitrary (supersampled) resolution.
fn scanline_fill(pts: &[[f32; 2]], w: usize, h: usize, cov: &mut [u8]) {
    if pts.len() < 3 {
        return;
    }
    for yy in 0..h {
        let yc = yy as f32 + 0.5;
        let mut xs: Vec<f32> = Vec::new();
        for i in 0..pts.len() {
            let p = pts[i];
            let q = pts[(i + 1) % pts.len()];
            if (p[1] <= yc && yc < q[1]) || (q[1] <= yc && yc < p[1]) {
                xs.push(p[0] + (yc - p[1]) / (q[1] - p[1]) * (q[0] - p[0]));
            }
        }
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mut i = 0;
        while i + 1 < xs.len() {
            let xa = xs[i].max(0.0).round() as usize;
            let xb = (xs[i + 1].min(w as f32).round() as usize).min(w);
            for xx in xa..xb {
                cov[yy * w + xx] = 255;
            }
            i += 2;
        }
    }
}
