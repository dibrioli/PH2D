//! **As funções livres do vetor: tinta e gradiente** — movidas VERBATIM do índice ([`super`], `line/input-dispatch`,
//! 2026-09-13), com os caminhos `crate::input_dispatch::..` preservados por re-exportação.

use super::*;

/// Component-wise average of two colours (alpha too).
fn avg_color(a: ph2d_vec_scene::Rgba8, b: ph2d_vec_scene::Rgba8) -> ph2d_vec_scene::Rgba8 {
    let m = |x: u8, y: u8| ((u16::from(x) + u16::from(y)) / 2) as u8;
    ph2d_vec_scene::Rgba8::new(m(a.r, b.r), m(a.g, b.g), m(a.b, b.b), m(a.a, b.a))
}

/// Multi-point set to use when switching to a freeform fill: reuse existing points,
/// else seed 3 spread points `[fill, contrast, average]` across the bbox `(lo,hi)`.
fn gradient_points_from(
    fill: &Option<ph2d_vec_scene::Paint>,
    lo: [f64; 2],
    hi: [f64; 2],
) -> Vec<ph2d_vec_scene::GradientPoint> {
    use ph2d_vec_scene::{GradientPoint, Paint};
    if let Some(Paint::MultiPoint { points }) = fill
        && !points.is_empty()
    {
        return points.clone();
    }
    let base = fill
        .as_ref()
        .map_or(ph2d_vec_scene::Rgba8::new(255, 255, 255, 255), |p| {
            p.primary_color()
        });
    let contrast = contrast_color(base);
    let (w, h) = (hi[0] - lo[0], hi[1] - lo[1]);
    let at = |fx: f64, fy: f64| [lo[0] + w * fx, lo[1] + h * fy];
    vec![
        GradientPoint::new(at(0.25, 0.25), base, 1.0),
        GradientPoint::new(at(0.75, 0.75), contrast, 1.0),
        GradientPoint::new(at(0.75, 0.25), avg_color(base, contrast), 1.0),
    ]
}

/// A luminance-opposite (black/white, alpha preserved) — the second stop seeded
/// when a solid fill first becomes a gradient, so the ramp is visibly a gradient.
fn contrast_color(c: ph2d_vec_scene::Rgba8) -> ph2d_vec_scene::Rgba8 {
    let lum = 0.2126 * f64::from(c.r) + 0.7152 * f64::from(c.g) + 0.0722 * f64::from(c.b);
    if lum > 128.0 {
        ph2d_vec_scene::Rgba8::new(0, 0, 0, c.a)
    } else {
        ph2d_vec_scene::Rgba8::new(255, 255, 255, c.a)
    }
}

/// Gradient stops to use when switching to a gradient: reuse the existing gradient's
/// stops (Linear↔Radial keep them), else seed a 2-stop ramp `[fill → contrast]`.
fn gradient_stops_from(fill: &Option<ph2d_vec_scene::Paint>) -> Vec<ph2d_vec_scene::GradientStop> {
    use ph2d_vec_scene::{GradientStop, Paint};
    match fill {
        Some(Paint::Linear { stops, .. }) | Some(Paint::Radial { stops, .. })
            if stops.len() >= 2 =>
        {
            stops.clone()
        }
        _ => {
            let base = fill
                .as_ref()
                .map_or(ph2d_vec_scene::Rgba8::new(255, 255, 255, 255), |p| {
                    p.primary_color()
                });
            vec![
                GradientStop::new(0.0, base),
                GradientStop::new(1.0, contrast_color(base)),
            ]
        }
    }
}

/// Linear ramp endpoints spanning the bbox `(lo,hi)` along `degrees` (0° = →),
/// centered on the bbox — the world-space geometry a linear gradient stores.
fn linear_span(lo: [f64; 2], hi: [f64; 2], degrees: f64) -> ([f64; 2], [f64; 2]) {
    let (cx, cy) = ((lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5);
    let (w, h) = (hi[0] - lo[0], hi[1] - lo[1]);
    let r = degrees.to_radians();
    let (dx, dy) = (r.cos(), r.sin());
    let reach = 0.5 * ((w * dx).abs() + (h * dy).abs());
    (
        [cx - dx * reach, cy - dy * reach],
        [cx + dx * reach, cy + dy * reach],
    )
}

/// Switch the SELECTED path's fill kind (Solid/Linear/Radial), preserving colour(s)
/// and existing gradient geometry when the kind is unchanged; when entering a
/// gradient from Solid/other, the geometry is seeded to fit the path's bbox. One
/// undo step iff it changed.
/// ⚠️ **`pattern` é a FONTE já resolvida** para o caso `Pattern`, e vem de fora de propósito: ela
/// pode exigir um diálogo de ficheiro, que congela o laço e por isso pertence à shell (a porta
/// `ph2d_app_host::modal`), não a esta função pura de documento.
///
/// ⚠️ **`None` com `kind == Pattern` é DESISTÊNCIA e não muda nada** — o artista fechou o diálogo,
/// e apagar o gradiente dele por isso seria o pior dos dois mundos.
pub(crate) fn apply_vec_set_fill_kind(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    kind: VecFillKind,
    pattern: Option<(ph2d_vec_scene::PatternSource, [f64; 2], [f64; 2])>,
) {
    use ph2d_vec_scene::Paint;
    let Some(sel) = pen.selected() else {
        eprintln!("[ph2d-vec] fill-kind: nenhum path selecionado");
        return;
    };
    let Some(cur) = scene
        .paths()
        .iter()
        .find(|p| p.id == sel)
        .map(|p| p.fill.clone())
    else {
        return;
    };
    let (lo, hi) = scene.path_bbox(sel).unwrap_or(([0.0, 0.0], [1.0, 1.0]));
    let (cx, cy) = ((lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5);
    let new_fill = match kind {
        VecFillKind::Solid => Paint::Solid(
            cur.as_ref()
                .map_or(ph2d_vec_scene::Rgba8::new(255, 255, 255, 255), |p| {
                    p.primary_color()
                }),
        ),
        // Already this kind → keep its geometry; else seed to fit the bbox.
        VecFillKind::Linear => match &cur {
            Some(p @ Paint::Linear { .. }) => p.clone(),
            _ => {
                let (start, end) = linear_span(lo, hi, 0.0);
                Paint::Linear {
                    stops: gradient_stops_from(&cur),
                    start,
                    end,
                }
            }
        },
        VecFillKind::Radial => match &cur {
            Some(p @ Paint::Radial { .. }) => p.clone(),
            _ => Paint::Radial {
                stops: gradient_stops_from(&cur),
                center: [cx, cy],
                radius: 0.5 * (hi[0] - lo[0]).hypot(hi[1] - lo[1]),
            },
        },
        VecFillKind::MultiPoint => match &cur {
            Some(p @ Paint::MultiPoint { .. }) => p.clone(),
            _ => Paint::MultiPoint {
                points: gradient_points_from(&cur, lo, hi),
            },
        },
        VecFillKind::Pattern => match (&cur, pattern) {
            // Já é padrão: preserva a lei inteira (trocar de chip e voltar não perde a arte, nem o
            // reticulado, nem a colocação).
            (Some(p @ Paint::Pattern(_)), _) => p.clone(),
            (_, Some((source, size, origin))) => {
                let mut f = ph2d_vec_scene::PatternFill::new(
                    source,
                    size,
                    crate::texture_pattern_pick::fallback_of(cur.as_ref()),
                );
                // ⛔ O canto é o da FORMA, não a origem do mundo (ver `default_placement`).
                f.origin = origin;
                Paint::Pattern(Box::new(f))
            }
            // ⚠️ Desistiu do diálogo: NÃO mexe no preenchimento.
            (_, None) => return,
        },
    };
    if cur.as_ref() == Some(&new_fill) {
        return;
    }
    if let Some(path) = scene.path_mut(sel) {
        path.fill = Some(new_fill);
    }
}

/// Set the SELECTED path's Linear-gradient angle (degrees; from the Angle slider's
/// `track·360`) by re-fitting the ramp endpoints across the bbox at that angle.
/// No-op unless the fill is Linear. One undo step iff it changed.
pub(crate) fn apply_vec_set_grad_angle(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    degrees: f64,
) {
    use ph2d_vec_scene::Paint;
    let Some(sel) = pen.selected() else {
        return;
    };
    let is_linear = scene
        .paths()
        .iter()
        .find(|p| p.id == sel)
        .is_some_and(|p| matches!(p.fill, Some(Paint::Linear { .. })));
    if !is_linear {
        return;
    }
    let (lo, hi) = scene.path_bbox(sel).unwrap_or(([0.0, 0.0], [1.0, 1.0]));
    let (start, end) = linear_span(lo, hi, degrees);
    if let Some(Paint::Linear {
        start: s, end: e, ..
    }) = scene.path_mut(sel).and_then(|p| p.fill.as_mut())
    {
        if *s == start && *e == end {
            return;
        }
        *s = start;
        *e = end;
    }
}

/// Add a multi-point gradient point at the selected path's bbox center (colour =
/// the first existing point). No-op unless the fill is MultiPoint. One undo step.
pub(crate) fn apply_vec_grad_add_point(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
) {
    use ph2d_vec_scene::{GradientPoint, Paint};
    let Some(sel) = pen.selected() else {
        return;
    };
    let Some((lo, hi)) = scene.path_bbox(sel) else {
        return;
    };
    let center = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
    if let Some(Paint::MultiPoint { points }) = scene.path_mut(sel).and_then(|p| p.fill.as_mut()) {
        let col = points
            .first()
            .map_or(ph2d_vec_scene::Rgba8::new(255, 255, 255, 255), |p| p.color);
        points.push(GradientPoint::new(center, col, 1.0));
    }
}

/// Remove a multi-point gradient point (`selected`, else the last), keeping at
/// least one. Returns the new selection (`None`). One undo step iff it removed.
pub(crate) fn apply_vec_grad_remove_point(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    selected: Option<usize>,
) -> Option<usize> {
    use ph2d_vec_scene::Paint;
    let Some(sel) = pen.selected() else {
        return selected;
    };
    if let Some(Paint::MultiPoint { points }) = scene.path_mut(sel).and_then(|p| p.fill.as_mut())
        && points.len() > 1
    {
        let idx = selected
            .filter(|&i| i < points.len())
            .unwrap_or(points.len() - 1);
        points.remove(idx);
        return None;
    }
    selected
}

/// Set the SELECTED multi-point gradient point's influence (`value` from the
/// Influence slider's `track·4`). No-op unless the fill is MultiPoint and `point`
/// is valid. One undo step iff it changed.
pub(crate) fn apply_vec_grad_influence(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    point: Option<usize>,
    value: f64,
) {
    use ph2d_vec_scene::Paint;
    let Some(sel) = pen.selected() else {
        return;
    };
    let Some(i) = point else {
        return;
    };
    if let Some(Paint::MultiPoint { points }) = scene.path_mut(sel).and_then(|p| p.fill.as_mut())
        && let Some(gp) = points.get_mut(i)
        && (gp.influence - value).abs() > 1e-9
    {
        gp.influence = value;
    }
}

/// Set the SELECTED multi-point gradient point's jitter (`value` 0..1, from the
/// Jitter slider's track). No-op unless the fill is MultiPoint and `point` is valid.
/// One undo step iff it changed.
pub(crate) fn apply_vec_grad_jitter(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    point: Option<usize>,
    value: f64,
) {
    use ph2d_vec_scene::Paint;
    let Some(sel) = pen.selected() else {
        return;
    };
    let Some(i) = point else {
        return;
    };
    if let Some(Paint::MultiPoint { points }) = scene.path_mut(sel).and_then(|p| p.fill.as_mut())
        && let Some(gp) = points.get_mut(i)
        && (gp.jitter - value).abs() > 1e-9
    {
        gp.jitter = value;
    }
}

/// Component-wise linear blend of two colours at `t ∈ [0,1]`.
fn lerp_color(a: ph2d_vec_scene::Rgba8, b: ph2d_vec_scene::Rgba8, t: f64) -> ph2d_vec_scene::Rgba8 {
    let m = |x: u8, y: u8| (f64::from(x) + (f64::from(y) - f64::from(x)) * t).round() as u8;
    ph2d_vec_scene::Rgba8::new(m(a.r, b.r), m(a.g, b.g), m(a.b, b.b), m(a.a, b.a))
}

/// The Linear/Radial gradient stops of the selected path (`None` for other fills).
fn selected_ramp_stops<'a>(
    scene: &'a ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
) -> Option<&'a [ph2d_vec_scene::GradientStop]> {
    use ph2d_vec_scene::Paint;
    let sel = pen.selected()?;
    match &scene.paths().iter().find(|p| p.id == sel)?.fill {
        Some(Paint::Linear { stops, .. }) | Some(Paint::Radial { stops, .. }) => Some(stops),
        _ => None,
    }
}

/// Add an interior ramp stop to the SELECTED Linear/Radial gradient, at the midpoint
/// of the widest gap (colour = the blend there). Returns the new stop's index (to
/// select), or `None` if the fill isn't a ramp. One undo step.
pub(crate) fn apply_vec_grad_add_stop(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
) -> Option<usize> {
    use ph2d_vec_scene::{GradientStop, Paint};
    let sel = pen.selected()?;
    // Interior stops may cross, so the Vec isn't sorted — find the widest gap on a
    // sorted (offset, colour) view; the new stop's colour is the blend across it.
    let stops = selected_ramp_stops(scene, pen)?;
    if stops.len() < 2 {
        return None;
    }
    let mut sorted: Vec<(f64, ph2d_vec_scene::Rgba8)> =
        stops.iter().map(|s| (s.offset, s.color)).collect();
    sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut best = (0usize, f64::NEG_INFINITY);
    for k in 0..sorted.len() - 1 {
        let gap = sorted[k + 1].0 - sorted[k].0;
        if gap > best.1 {
            best = (k, gap);
        }
    }
    let k = best.0;
    let off = (sorted[k].0 + sorted[k + 1].0) * 0.5;
    let col = lerp_color(sorted[k].1, sorted[k + 1].1, 0.5);
    if let Some(Paint::Linear { stops, .. }) | Some(Paint::Radial { stops, .. }) =
        scene.path_mut(sel).and_then(|p| p.fill.as_mut())
    {
        // Insert as an INTERIOR stop (just before the last end stop) so the two ends
        // stay at index 0 / last; return its index to select it.
        let idx = stops.len() - 1;
        stops.insert(idx, GradientStop::new(off, col));
        return Some(idx);
    }
    None
}

/// Remove the SELECTED interior ramp stop (`selected` index) from the Linear/Radial
/// gradient, keeping the two end stops (≥2 total). Returns the new selection
/// (`None`). One undo step iff it removed.
pub(crate) fn apply_vec_grad_remove_stop(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    selected: Option<usize>,
) -> Option<usize> {
    use ph2d_vec_scene::Paint;
    let Some(sel) = pen.selected() else {
        return selected;
    };
    if let Some(Paint::Linear { stops, .. }) | Some(Paint::Radial { stops, .. }) =
        scene.path_mut(sel).and_then(|p| p.fill.as_mut())
        && let Some(i) = selected
        && i > 0
        && i + 1 < stops.len()
    {
        stops.remove(i);
        return None;
    }
    selected
}
