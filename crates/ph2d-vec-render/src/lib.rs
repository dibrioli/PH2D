#![forbid(unsafe_code)]
//! ph2d-vec-render — pipeline de render da cena vetorial nova (ADR-0108, Fase 0).
//!
//! Converte o modelo editor-first (`ph2d-vec-scene`) em chamadas Vello, emitindo
//! no **`VectorScene` fundacional compartilhado do frame** (`ph2d-vector`) — sem
//! abrir passe de GPU novo, só anexando comandos de encode à cena que o compositor
//! já rasteriza. Toda a stack Linebender chega pelas re-exports de `ph2d-vector`
//! (gate-proof + skew-proof).
//!
//! Fase 0: draw estático da cena inteira. **Dirty-tracking** (só re-encodar a
//! sub-árvore que mudou — a alavanca de escala do ADR-0108) é o próximo passo.

use ph2d_vec_scene::{
    GradientPoint, LineCap, LineJoin, Paint, Rgba8, StrokeSpec, VecPath, VecPathId, VecScene,
};
use ph2d_vector::{
    Affine, BezPath, Brush, Cap, Circle, Color, ColorStop, Fill, Gradient, ImageQuality, Join,
    Point, Rect, Stroke, VectorScene,
};
use std::sync::Arc;

/// Constrói o `BezPath` (world-space) de um path editável: `move_to` na 1ª âncora,
/// depois uma cúbica por segmento usando `out_handle(i)` e `in_handle(i+1)`;
/// fecha com uma cúbica final se `closed`.
pub fn build_bezpath(path: &VecPath) -> BezPath {
    let mut bp = BezPath::new();
    let verts = &path.verts;
    let Some(first) = verts.first() else {
        return bp;
    };
    bp.move_to(pt(first.anchor));
    for pair in verts.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        bp.curve_to(pt(a.out_handle), pt(b.in_handle), pt(b.anchor));
    }
    if path.closed && verts.len() >= 2 {
        let last = verts.last().unwrap();
        bp.curve_to(pt(last.out_handle), pt(first.in_handle), pt(first.anchor));
        bp.close_path();
    }
    bp
}

/// Desenha toda a `scene` no `target` (o `VectorScene` do frame) sob `transform`
/// (o world→screen da câmera). Fill primeiro, stroke por cima.
pub fn dispatch(scene: &VecScene, transform: Affine, target: &mut VectorScene) {
    for path in scene.paths() {
        let bp = build_bezpath(path);
        if let Some(fill) = &path.fill {
            if let Paint::MultiPoint { points } = fill {
                fill_multipoint(target, &bp, path, points, transform);
            } else {
                target.fill_path(&bp, &fill_brush(fill, path), transform);
            }
        }
        if let Some(s) = path.stroke {
            target.inner_mut().stroke(
                &kurbo_stroke(&s),
                transform,
                &Brush::Solid(color(s.color)),
                None,
                &bp,
            );
        }
    }
}

/// Desenha os **gizmos de edição** por cima da cena (screen-space, tamanho
/// constante em px): quadradinho em cada âncora; e — só no path `selected` — as
/// linhas âncora→handle + bolinhas nos handles dos vértices suaves. Cores
/// hardcoded de scaffold (Fase 1); migram p/ tokens no chrome do cutover (Fase R).
pub fn draw_overlays(
    scene: &VecScene,
    selected: Option<VecPathId>,
    selected_verts: &[usize],
    transform: Affine,
    target: &mut VectorScene,
) {
    for path in scene.paths() {
        let is_sel = Some(path.id) == selected;
        if is_sel {
            for v in &path.verts {
                let a = transform * Point::new(v.anchor[0], v.anchor[1]);
                // Draw a handle only when it is offset from the anchor — cusps
                // (Corner), Smooth and Symmetric all show their non-degenerate
                // handles; a straight corner (handle == anchor) shows nothing.
                for h in [v.in_handle, v.out_handle] {
                    let (dx, dy) = (h[0] - v.anchor[0], h[1] - v.anchor[1]);
                    if dx * dx + dy * dy <= 1e-18 {
                        continue;
                    }
                    let hp = transform * Point::new(h[0], h[1]);
                    let mut line = BezPath::new();
                    line.move_to(a);
                    line.line_to(hp);
                    target.inner_mut().stroke(
                        &Stroke::new(1.0),
                        Affine::IDENTITY,
                        &Brush::Solid(Color::from_rgba8(120, 190, 230, 200)),
                        None,
                        &line,
                    );
                    target.inner_mut().fill(
                        Fill::NonZero,
                        Affine::IDENTITY,
                        &Brush::Solid(Color::from_rgba8(120, 190, 230, 255)),
                        None,
                        &Circle::new(hp, 3.5),
                    );
                }
            }
        }
        for (i, v) in path.verts.iter().enumerate() {
            let a = transform * Point::new(v.anchor[0], v.anchor[1]);
            // A vertex in the multi-selection (selected path only) is drawn bigger
            // + cyan; other anchors of the selected path are orange; other paths gray.
            let picked = is_sel && selected_verts.contains(&i);
            let s = if picked { 4.5 } else { 3.5 };
            let col = if picked {
                Color::from_rgba8(90, 200, 235, 255) // ciano = selecionado (grupo)
            } else if is_sel {
                Color::from_rgba8(250, 180, 90, 255) // laranja = path selecionado
            } else {
                Color::from_rgba8(230, 230, 235, 220)
            };
            target.fill_rect(Rect::new(a.x - s, a.y - s, a.x + s, a.y + s), col);
        }
    }
}

/// A draggable handle of a path's gradient fill (on-canvas editing, Cavalry-style).
/// MultiPoint exposes one handle per point; Linear its two endpoints; Radial its
/// center + an edge handle (drags the radius). The screen hit-test / camera work
/// lives in the shell; [`hit_gradient_handle`] / [`drag_gradient_handle`] are the
/// pure world-space geometry helpers it drives.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GradHandle {
    /// A multi-point gradient point by index.
    Point(usize),
    /// A linear gradient's start endpoint.
    LinearStart,
    /// A linear gradient's end endpoint.
    LinearEnd,
    /// A radial gradient's center.
    RadialCenter,
    /// A radial gradient's edge (drags the radius).
    RadialEdge,
}

impl GradHandle {
    /// The MultiPoint point index this handle addresses (`None` for linear/radial).
    #[must_use]
    pub fn point(self) -> Option<usize> {
        match self {
            GradHandle::Point(i) => Some(i),
            _ => None,
        }
    }
}

/// The world-space anchor of `handle` on `path`'s fill (`None` if the handle
/// doesn't match the fill kind / index). The radial EDGE sits at `center + (r, 0)`.
fn handle_world_pos(path: &VecPath, handle: GradHandle) -> Option<[f64; 2]> {
    match (&path.fill, handle) {
        (Some(Paint::MultiPoint { points }), GradHandle::Point(i)) => {
            points.get(i).map(|gp| gp.pos)
        }
        (Some(Paint::Linear { start, .. }), GradHandle::LinearStart) => Some(*start),
        (Some(Paint::Linear { end, .. }), GradHandle::LinearEnd) => Some(*end),
        (Some(Paint::Radial { center, .. }), GradHandle::RadialCenter) => Some(*center),
        (Some(Paint::Radial { center, radius, .. }), GradHandle::RadialEdge) => {
            Some([center[0] + radius, center[1]])
        }
        _ => None,
    }
}

/// Hit-test all of `path`'s gradient handles at world point `(wx, wy)` within
/// `world_thresh` world-units, returning the closest match (`None` if none / no
/// gradient fill). Squared distance only.
#[must_use]
pub fn hit_gradient_handle(
    path: &VecPath,
    wx: f64,
    wy: f64,
    world_thresh: f64,
) -> Option<GradHandle> {
    let t2 = world_thresh * world_thresh;
    let mut best: Option<(GradHandle, f64)> = None;
    let consider = |best: &mut Option<(GradHandle, f64)>, h: GradHandle| {
        if let Some(p) = handle_world_pos(path, h) {
            let d2 = (wx - p[0]).powi(2) + (wy - p[1]).powi(2);
            if d2 <= t2 && best.is_none_or(|(_, b)| d2 < b) {
                *best = Some((h, d2));
            }
        }
    };
    match &path.fill {
        Some(Paint::MultiPoint { points }) => {
            for i in 0..points.len() {
                consider(&mut best, GradHandle::Point(i));
            }
        }
        Some(Paint::Linear { .. }) => {
            consider(&mut best, GradHandle::LinearStart);
            consider(&mut best, GradHandle::LinearEnd);
        }
        Some(Paint::Radial { .. }) => {
            consider(&mut best, GradHandle::RadialCenter);
            consider(&mut best, GradHandle::RadialEdge);
        }
        _ => {}
    }
    best.map(|(h, _)| h)
}

/// Move `handle` to world `(wx, wy)` on `path`'s fill (the radial edge sets the
/// radius = distance to the center). Returns `true` iff it applied. No-op if the
/// handle doesn't match the fill kind / index.
pub fn drag_gradient_handle(path: &mut VecPath, handle: GradHandle, wx: f64, wy: f64) -> bool {
    match (&mut path.fill, handle) {
        (Some(Paint::MultiPoint { points }), GradHandle::Point(i)) => {
            if let Some(gp) = points.get_mut(i) {
                gp.pos = [wx, wy];
                return true;
            }
        }
        (Some(Paint::Linear { start, .. }), GradHandle::LinearStart) => {
            *start = [wx, wy];
            return true;
        }
        (Some(Paint::Linear { end, .. }), GradHandle::LinearEnd) => {
            *end = [wx, wy];
            return true;
        }
        (Some(Paint::Radial { center, .. }), GradHandle::RadialCenter) => {
            *center = [wx, wy];
            return true;
        }
        (Some(Paint::Radial { center, radius, .. }), GradHandle::RadialEdge) => {
            *radius = ((wx - center[0]).powi(2) + (wy - center[1]).powi(2))
                .sqrt()
                .max(1e-6);
            return true;
        }
        _ => {}
    }
    false
}

/// Desenha os **handles do gradiente** do path `selected` em screen-space (bolinhas
/// roxas estilo Cavalry): um por ponto (MultiPoint), os dois extremos + a linha da
/// rampa (Linear), ou o centro + o anel de raio + o handle de borda (Radial). O
/// handle `active` ganha um anel branco. No-op se não houver gradiente selecionado.
pub fn draw_gradient_handles(
    scene: &VecScene,
    selected: Option<VecPathId>,
    active: Option<GradHandle>,
    transform: Affine,
    target: &mut VectorScene,
) {
    let Some(sel) = selected else { return };
    let Some(path) = scene.paths().iter().find(|p| p.id == sel) else {
        return;
    };
    let purple = Color::from_rgba8(180, 120, 235, 255); // Cavalry purple
    let white = Color::from_rgba8(255, 255, 255, 255);
    // A ringed colour-swatch dot; a white ring marks the active handle.
    let dot = |target: &mut VectorScene, c: Point, fill: Color, is_active: bool| {
        target.inner_mut().fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(fill),
            None,
            &Circle::new(c, 5.5),
        );
        let ring = if is_active { white } else { purple };
        target.inner_mut().stroke(
            &Stroke::new(if is_active { 2.0 } else { 1.5 }),
            Affine::IDENTITY,
            &Brush::Solid(ring),
            None,
            &Circle::new(c, 5.5),
        );
    };
    match &path.fill {
        Some(Paint::MultiPoint { points }) => {
            for (i, gp) in points.iter().enumerate() {
                let c = transform * Point::new(gp.pos[0], gp.pos[1]);
                dot(
                    target,
                    c,
                    color(gp.color),
                    active == Some(GradHandle::Point(i)),
                );
            }
        }
        Some(Paint::Linear { start, end, stops }) => {
            let a = transform * Point::new(start[0], start[1]);
            let b = transform * Point::new(end[0], end[1]);
            let mut line = BezPath::new();
            line.move_to(a);
            line.line_to(b);
            target.inner_mut().stroke(
                &Stroke::new(1.5),
                Affine::IDENTITY,
                &Brush::Solid(purple),
                None,
                &line,
            );
            let c0 = stops.first().map_or(purple, |s| color(s.color));
            let c1 = stops.last().map_or(purple, |s| color(s.color));
            dot(target, a, c0, active == Some(GradHandle::LinearStart));
            dot(target, b, c1, active == Some(GradHandle::LinearEnd));
        }
        Some(Paint::Radial {
            center,
            radius,
            stops,
        }) => {
            let c = transform * Point::new(center[0], center[1]);
            let e = transform * Point::new(center[0] + radius, center[1]);
            let sr = ((e.x - c.x).powi(2) + (e.y - c.y).powi(2)).sqrt();
            target.inner_mut().stroke(
                &Stroke::new(1.5),
                Affine::IDENTITY,
                &Brush::Solid(purple),
                None,
                &Circle::new(c, sr),
            );
            let cc = stops.first().map_or(purple, |s| color(s.color));
            let ce = stops.last().map_or(purple, |s| color(s.color));
            dot(target, c, cc, active == Some(GradHandle::RadialCenter));
            dot(target, e, ce, active == Some(GradHandle::RadialEdge));
        }
        _ => {}
    }
}

/// Desenha a caixa de **marquee** (box-select) em **screen-space** (o shell
/// passa cantos de tela): preenchimento translúcido + contorno. Chamada só
/// enquanto o Shift+arrasto está ativo.
pub fn draw_marquee(min: [f64; 2], max: [f64; 2], target: &mut VectorScene) {
    let (x0, x1) = (min[0].min(max[0]), min[0].max(max[0]));
    let (y0, y1) = (min[1].min(max[1]), min[1].max(max[1]));
    let rect = Rect::new(x0, y0, x1, y1);
    target.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(Color::from_rgba8(90, 200, 235, 40)),
        None,
        &rect,
    );
    let mut outline = BezPath::new();
    outline.move_to(Point::new(x0, y0));
    outline.line_to(Point::new(x1, y0));
    outline.line_to(Point::new(x1, y1));
    outline.line_to(Point::new(x0, y1));
    outline.close_path();
    target.inner_mut().stroke(
        &Stroke::new(1.0),
        Affine::IDENTITY,
        &Brush::Solid(Color::from_rgba8(90, 200, 235, 200)),
        None,
        &outline,
    );
}

/// `StrokeSpec` → `kurbo::Stroke` (ponta/junção + dash). Larguras/dashes ficam em
/// world-units; o `transform` do render escala p/ screen.
fn kurbo_stroke(s: &StrokeSpec) -> Stroke {
    let cap = match s.cap {
        LineCap::Butt => Cap::Butt,
        LineCap::Round => Cap::Round,
        LineCap::Square => Cap::Square,
    };
    let join = match s.join {
        LineJoin::Miter => Join::Miter,
        LineJoin::Round => Join::Round,
        LineJoin::Bevel => Join::Bevel,
    };
    let stroke = Stroke::new(s.width).with_caps(cap).with_join(join);
    // `dash` carries width MULTIPLES `(dash, gap)` (width-aware — a thicker line
    // gets proportionally longer dash + gap, so the cap projection never swallows
    // the gap). A zero-length gap collapses to a solid look; clamp it off zero so
    // kurbo never emits a degenerate dash element.
    match s.dash {
        Some((d, g)) if d > 0.0 => {
            let dash_len = d * s.width;
            let gap_len = (g * s.width).max(f64::EPSILON);
            stroke.with_dashes(0.0, [dash_len, gap_len])
        }
        _ => stroke,
    }
}

#[inline]
fn pt(p: [f64; 2]) -> Point {
    Point::new(p[0], p[1])
}

#[inline]
fn color(c: Rgba8) -> Color {
    Color::from_rgba8(c.r, c.g, c.b, c.a)
}

/// Peniko color stops from our gradient stops (`(offset f32, Color)` → `ColorStop`).
fn stops_of(stops: &[ph2d_vec_scene::GradientStop]) -> Vec<ColorStop> {
    stops
        .iter()
        .map(|s| ColorStop::from((s.offset as f32, color(s.color))))
        .collect()
}

/// Build the Vello fill brush for a path's [`Paint`]. Linear/Radial map to native
/// peniko gradients using the paint's OWN world-space geometry (start/end,
/// center/radius) — which transforms rigidly with the path, so the gradient never
/// "breathes" under rotation. The frame's world→screen transform maps them.
/// MultiPoint is a solid placeholder until its IDW image-brush rasterizer lands
/// (gradient group increment 3).
fn fill_brush(paint: &Paint, _path: &VecPath) -> Brush {
    match paint {
        Paint::Solid(c) => Brush::Solid(color(*c)),
        Paint::Linear { stops, start, end } => {
            let a = Point::new(start[0], start[1]);
            let b = Point::new(end[0], end[1]);
            Brush::Gradient(Gradient::new_linear(a, b).with_stops(stops_of(stops).as_slice()))
        }
        Paint::Radial {
            stops,
            center,
            radius,
        } => {
            let c = Point::new(center[0], center[1]);
            let r = (*radius as f32).max(f32::MIN_POSITIVE);
            Brush::Gradient(Gradient::new_radial(c, r).with_stops(stops_of(stops).as_slice()))
        }
        // MultiPoint is handled by `fill_multipoint` (image-clip path), never here.
        Paint::MultiPoint { .. } => Brush::Solid(color(paint.primary_color())),
    }
}

/// Raster resolution of the multi-point IDW blend (upscaled bilinearly by the image
/// brush, so a small buffer stays smooth).
const IDW_RES: u32 = 64;

/// World-space bbox of a path's CONTROL POINTS (anchor + both handles) — the
/// convex hull of every Bézier segment, so it FULLY contains the drawn curve
/// (the anchor-only bbox misses the parts that bulge past the anchors, leaving
/// the multi-point image unfilled there). Unit rect if empty.
fn control_point_bounds(path: &VecPath) -> ([f64; 2], [f64; 2]) {
    let mut lo = [f64::INFINITY; 2];
    let mut hi = [f64::NEG_INFINITY; 2];
    for v in &path.verts {
        for p in [v.anchor, v.in_handle, v.out_handle] {
            lo[0] = lo[0].min(p[0]);
            lo[1] = lo[1].min(p[1]);
            hi[0] = hi[0].max(p[0]);
            hi[1] = hi[1].max(p[1]);
        }
    }
    if lo[0] > hi[0] {
        ([0.0, 0.0], [1.0, 1.0])
    } else {
        (lo, hi)
    }
}

/// Deterministic per-texel white noise in `[0, 1)` from a 3-way integer key
/// (`px`, `py`, point index) via a splitmix64 finalizer — transcendental-free
/// (HR-5) and stable across frames/machines, so the jitter grain never crawls.
#[inline]
fn hash_noise(px: u32, py: u32, i: u32) -> f64 {
    let mut z = (u64::from(px) << 42)
        ^ (u64::from(py) << 21)
        ^ u64::from(i).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    // Top 53 bits → [0, 1).
    (z >> 11) as f64 / (1u64 << 53) as f64
}

/// Rasterize the Cavalry-style multi-point gradient into a straight-alpha RGBA8
/// buffer of `res × res`, covering the world bbox `(lo,hi)`. Each texel's colour is
/// the inverse-distance-weighted blend of the points:
/// `c(p) = Σ wᵢcᵢ / Σ wᵢ`, `wᵢ = influenceᵢ / (dist²(p,posᵢ) + ε)` — exact colour at
/// each point, smooth between. Squared distance only (no transcendentals). A point's
/// `jitter` (0..1) multiplies its weight by a deterministic per-texel noise factor
/// `1 ± jitter` (grain near that point); `jitter = 0` leaves the blend byte-identical.
fn rasterize_idw(points: &[GradientPoint], lo: [f64; 2], hi: [f64; 2], res: u32) -> Arc<Vec<u8>> {
    const EPS: f64 = 1e-6;
    let n = res as usize;
    let mut buf = vec![0u8; n * n * 4];
    let (w, h) = (hi[0] - lo[0], hi[1] - lo[1]);
    for py in 0..n {
        for px in 0..n {
            // Texel center → world point.
            let wx = lo[0] + (px as f64 + 0.5) / res as f64 * w;
            let wy = lo[1] + (py as f64 + 0.5) / res as f64 * h;
            let (mut sr, mut sg, mut sb, mut sa, mut sw) = (0.0, 0.0, 0.0, 0.0, 0.0);
            for (i, gp) in points.iter().enumerate() {
                let (dx, dy) = (wx - gp.pos[0], wy - gp.pos[1]);
                let mut wgt = gp.influence.max(0.0) / (dx * dx + dy * dy + EPS);
                let jit = gp.jitter.clamp(0.0, 1.0);
                if jit > 0.0 {
                    let noise = hash_noise(px as u32, py as u32, i as u32);
                    // Factor ∈ [1−jit, 1+jit), floored at 0 so the weight stays ≥ 0.
                    wgt *= (1.0 + jit * (noise - 0.5) * 2.0).max(0.0);
                }
                sr += wgt * f64::from(gp.color.r);
                sg += wgt * f64::from(gp.color.g);
                sb += wgt * f64::from(gp.color.b);
                sa += wgt * f64::from(gp.color.a);
                sw += wgt;
            }
            let inv = if sw > 0.0 { 1.0 / sw } else { 0.0 };
            let o = (py * n + px) * 4;
            buf[o] = (sr * inv).round().clamp(0.0, 255.0) as u8;
            buf[o + 1] = (sg * inv).round().clamp(0.0, 255.0) as u8;
            buf[o + 2] = (sb * inv).round().clamp(0.0, 255.0) as u8;
            buf[o + 3] = (sa * inv).round().clamp(0.0, 255.0) as u8;
        }
    }
    Arc::new(buf)
}

/// Fill `bp` with a multi-point gradient: rasterize the IDW blend over the path's
/// world bbox, clip to the (screen-space) path, and blit the image scaled to the
/// bbox. Reuses the tested clip + image-blit path (mirror of the BgRemoval overlay).
fn fill_multipoint(
    target: &mut VectorScene,
    bp: &BezPath,
    path: &VecPath,
    points: &[GradientPoint],
    transform: Affine,
) {
    if points.is_empty() {
        return;
    }
    let (lo, hi) = control_point_bounds(path);
    let (w, h) = (hi[0] - lo[0], hi[1] - lo[1]);
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let img = rasterize_idw(points, lo, hi, IDW_RES);
    // Clip to the path in screen space.
    let mut screen_bp = bp.clone();
    screen_bp.apply_affine(transform);
    target.push_clip(&screen_bp);
    // Image pixels (0..res) → world bbox → screen.
    let px_to_world = Affine::translate((lo[0], lo[1]))
        * Affine::scale_non_uniform(w / f64::from(IDW_RES), h / f64::from(IDW_RES));
    target.draw_image_rgba_transformed(
        &img,
        IDW_RES,
        IDW_RES,
        transform * px_to_world,
        ImageQuality::High,
    );
    target.pop_layer();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_path_yields_empty_bezpath() {
        let p = VecPath {
            id: 0,
            verts: vec![],
            closed: false,
            fill: None,
            stroke: None,
        };
        assert!(build_bezpath(&p).elements().is_empty());
    }

    #[test]
    fn demo_scene_builds_nonempty_paths() {
        let scene = VecScene::demo();
        for path in scene.paths() {
            assert!(!build_bezpath(path).elements().is_empty());
        }
    }

    #[test]
    fn idw_raster_is_exact_at_points_and_blends_between() {
        let red = Rgba8::new(255, 0, 0, 255);
        let blue = Rgba8::new(0, 0, 255, 255);
        let pts = [
            GradientPoint::new([0.0, 0.5], red, 1.0),
            GradientPoint::new([1.0, 0.5], blue, 1.0),
        ];
        let res = 64u32;
        let buf = rasterize_idw(&pts, [0.0, 0.0], [1.0, 1.0], res);
        let texel = |px: u32, py: u32| {
            let o = ((py * res + px) * 4) as usize;
            [buf[o], buf[o + 1], buf[o + 2], buf[o + 3]]
        };
        // Near the left point → almost pure red; near the right → almost pure blue.
        let left = texel(0, res / 2);
        let right = texel(res - 1, res / 2);
        assert!(left[0] > 240 && left[2] < 15, "left ≈ red, got {left:?}");
        assert!(
            right[2] > 240 && right[0] < 15,
            "right ≈ blue, got {right:?}"
        );
        // Middle column blends both channels (neither is ~0 or ~255).
        let mid = texel(res / 2, res / 2);
        assert!(
            mid[0] > 40 && mid[0] < 215 && mid[2] > 40 && mid[2] < 215,
            "mid blends, got {mid:?}"
        );
        // Empty points → all-zero buffer (no divide-by-zero).
        let empty = rasterize_idw(&[], [0.0, 0.0], [1.0, 1.0], 4);
        assert!(empty.iter().all(|&b| b == 0));
    }

    #[test]
    fn jitter_perturbs_the_blend_but_zero_is_identical() {
        let red = Rgba8::new(255, 0, 0, 255);
        let blue = Rgba8::new(0, 0, 255, 255);
        let res = 32u32;
        let base = [
            GradientPoint::new([0.0, 0.5], red, 1.0),
            GradientPoint::new([1.0, 0.5], blue, 1.0),
        ];
        // jitter = 0 everywhere → byte-identical to the plain blend (default preserved).
        let plain = rasterize_idw(&base, [0.0, 0.0], [1.0, 1.0], res);
        let zero_jit = [
            GradientPoint::with_jitter([0.0, 0.5], red, 1.0, 0.0),
            GradientPoint::with_jitter([1.0, 0.5], blue, 1.0, 0.0),
        ];
        let same = rasterize_idw(&zero_jit, [0.0, 0.0], [1.0, 1.0], res);
        assert_eq!(*plain, *same, "jitter=0 must be byte-identical");
        // A non-zero jitter changes at least some texels (grain), but never the
        // exact colour at a point (its weight dominates → factor is irrelevant there).
        let jit = [
            GradientPoint::with_jitter([0.0, 0.5], red, 1.0, 0.8),
            GradientPoint::with_jitter([1.0, 0.5], blue, 1.0, 0.8),
        ];
        let noisy = rasterize_idw(&jit, [0.0, 0.0], [1.0, 1.0], res);
        assert_ne!(*plain, *noisy, "jitter>0 must perturb the blend");
        // Deterministic: same inputs → same buffer (no frame crawl).
        let noisy2 = rasterize_idw(&jit, [0.0, 0.0], [1.0, 1.0], res);
        assert_eq!(*noisy, *noisy2, "jitter must be deterministic");
    }

    /// Spike de escala (ADR-0108 §5) — custo de re-encode NAIVE por frame (CPU,
    /// sem dirty-tracking), a fração dominante do custo em escala (achado Rive).
    /// `cargo test -p ph2d-vec-render --release -- --ignored --nocapture`
    #[test]
    #[ignore = "spike manual de medição; rode em --release --nocapture"]
    fn encode_cost_by_n() {
        use std::time::Instant;
        let affine = Affine::IDENTITY;
        println!("\n=== re-encode NAIVE por frame (CPU, sem dirty-tracking) ===");
        for &n in &[1_000usize, 5_000, 10_000, 20_000, 50_000] {
            let scene = VecScene::demo_grid(n);
            let mut target = VectorScene::new();
            target.reset();
            dispatch(&scene, affine, &mut target); // warm
            let iters = 30;
            let t = Instant::now();
            for _ in 0..iters {
                target.reset();
                dispatch(&scene, affine, &mut target);
            }
            let ms = t.elapsed().as_secs_f64() * 1000.0 / iters as f64;
            println!(
                "N={:>6}  encode={:>7.3} ms/frame   (teto encode-bound: {:>6.0} fps)",
                n,
                ms,
                1000.0 / ms
            );
        }
    }
}
