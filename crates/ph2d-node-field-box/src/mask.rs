//! A **máscara** do `field.box`, isolada: a curva de aresta, a rampa de planalto e o
//! rectângulo que as compõe.
//!
//! ⚠️ **O corte foi FORÇADO pelo HR-18 e a costura escolhida é esta por uma razão:** as três
//! são aritmética pura — não tocam o registry, o `EvalCtx` nem o `GpuKernel` —, então o
//! `lib.rs` fica com o nó (manifesto, kernel, `eval`, painel) e isto fica com a lei. A ironia
//! ficou registada: o arquivo passou dos 700 ao ganhar o **aviso** de que estava a 6 do teto.

/// An edge curve on a pre-clamped `s ∈ [0,1]` — the SAME set as the other fields
/// (HR-5). `0` Linear · `1` Quad · `2` Smooth · `3` Smoother. Monotone, endpoint-exact.
pub(crate) fn curve(kind: i32, s: f32) -> f32 {
    match kind {
        1 => s * s,                                     // Quad
        2 => s * s * (3.0 - 2.0 * s),                   // Smooth (smoothstep)
        3 => s * s * s * (s * (s * 6.0 - 15.0) + 10.0), // Smoother (smootherstep)
        _ => s,                                         // Linear
    }
}

/// The plateau-and-ramp on ONE axis: `half` is the half-extent (edge at `|d| =
/// half`), `soft` the ramp width clamped so it cannot exceed the half-extent (a
/// wider `soft` just meets in the middle, never overshoots the centre). `1`
/// inside `|d| ≤ half − soft`, ramping to `0` at `|d| = half`; `soft = 0` is a
/// hard edge. `half ≤ 0` degenerates to empty.
pub(crate) fn edge_ramp(d: f32, half: f32, soft: f32) -> f32 {
    let a = d.abs();
    let s = soft.max(0.0).min(half);
    if s > 0.0 {
        ((half - a) / s).clamp(0.0, 1.0)
    } else if a <= half {
        1.0
    } else {
        0.0
    }
}

/// The box mask at offset `(dx, dy)` from the centre, for FULL extents
/// `width`/`height`. The box is the INTERSECTION of the two axis bands (`min`),
/// then the curve shapes the combined ramp. Because every `curve` is monotone,
/// `curve(min(rx, ry))` equals `min(curve(rx), curve(ry))` — one eval, both edges.
pub(crate) fn box_mask(
    dx: f32,
    dy: f32,
    width: f32,
    height: f32,
    soft: f32,
    curve_kind: i32,
) -> f32 {
    let rx = edge_ramp(dx, width * 0.5, soft);
    let ry = edge_ramp(dy, height * 0.5, soft);
    curve(curve_kind, rx.min(ry))
}
