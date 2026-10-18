//! **As sementes de um gradiente** — as cores e a geometria que um preenchimento ganha ao virar
//! gradiente (média, contraste, mistura, a rampa que cobre a caixa). Puras: saíram da shell
//! (`input_dispatch/despacho_vetor_gradiente.rs`, P46), que fica com as portas `apply_vec_*`.

/// Component-wise average of two colours (alpha too).
pub fn avg_color(a: ph2d_vec_scene::Rgba8, b: ph2d_vec_scene::Rgba8) -> ph2d_vec_scene::Rgba8 {
    let m = |x: u8, y: u8| ((u16::from(x) + u16::from(y)) / 2) as u8;
    ph2d_vec_scene::Rgba8::new(m(a.r, b.r), m(a.g, b.g), m(a.b, b.b), m(a.a, b.a))
}

/// Multi-point set to use when switching to a freeform fill: reuse existing points,
/// else seed 3 spread points `[fill, contrast, average]` across the bbox `(lo,hi)`.
pub fn gradient_points_from(
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
pub fn contrast_color(c: ph2d_vec_scene::Rgba8) -> ph2d_vec_scene::Rgba8 {
    let lum = 0.2126 * f64::from(c.r) + 0.7152 * f64::from(c.g) + 0.0722 * f64::from(c.b);
    if lum > 128.0 {
        ph2d_vec_scene::Rgba8::new(0, 0, 0, c.a)
    } else {
        ph2d_vec_scene::Rgba8::new(255, 255, 255, c.a)
    }
}

/// Gradient stops to use when switching to a gradient: reuse the existing gradient's
/// stops (Linear↔Radial keep them), else seed a 2-stop ramp `[fill → contrast]`.
pub fn gradient_stops_from(
    fill: &Option<ph2d_vec_scene::Paint>,
) -> Vec<ph2d_vec_scene::GradientStop> {
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
pub fn linear_span(lo: [f64; 2], hi: [f64; 2], degrees: f64) -> ([f64; 2], [f64; 2]) {
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

/// Component-wise linear blend of two colours at `t ∈ [0,1]`.
pub fn lerp_color(
    a: ph2d_vec_scene::Rgba8,
    b: ph2d_vec_scene::Rgba8,
    t: f64,
) -> ph2d_vec_scene::Rgba8 {
    let m = |x: u8, y: u8| (f64::from(x) + (f64::from(y) - f64::from(x)) * t).round() as u8;
    ph2d_vec_scene::Rgba8::new(m(a.r, b.r), m(a.g, b.g), m(a.b, b.b), m(a.a, b.a))
}
