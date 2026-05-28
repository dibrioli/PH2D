//! Integration test: Q16.16 / Q8.8 ULP-zero roundtrip pra coords + pressure
//! dentro da janela útil (ADR-0046 §2.3 + §2.8).
//!
//! Behavior gate `painter_replay_q1616_roundtrip` (ADR-0046 §2.12) —
//! `q1616_to_f32 ∘ f32_to_q1616` DEVE ser identidade pra valores
//! representáveis exatos (multiples de 1/65536). Sem isso, replay
//! determinístico diverge entre ARM/x86.

use ph2d_painter_stroke::{f32_to_q88, f32_to_q1616_saturating, q88_to_f32, q1616_to_f32};

#[test]
fn q1616_roundtrip_full_canvas_range() {
    // Cap operacional: ±16384 px (canvas máximo ADR-0046 §2.7).
    // Step 0.5 px varre tanto inteiros quanto half-pixels (representáveis
    // exatos em Q16.16: 32768 e 1, respectivamente).
    let mut x = -16384.0f32;
    while x <= 16384.0 {
        let q = f32_to_q1616_saturating(x);
        let back = q1616_to_f32(q);
        assert_eq!(
            back, x,
            "Q16.16 roundtrip drifted at x={} (q={}, back={})",
            x, q, back
        );
        x += 0.5;
    }
}

#[test]
fn q1616_subpixel_eighths_roundtrip() {
    // 1/8 px = 8192 em Q16.16 (exato).
    for n in -1024..=1024i32 {
        let v = n as f32 / 8.0;
        let q = f32_to_q1616_saturating(v);
        let back = q1616_to_f32(q);
        assert_eq!(back, v, "1/8-px roundtrip drift at v={}", v);
    }
}

#[test]
fn q88_pressure_range_roundtrip() {
    // Pressure ∈ [0, 1]. Step 1/256 cobre toda Q8.8 resolution.
    for i in 0..=256u32 {
        let v = i as f32 / 256.0;
        let q = f32_to_q88(v);
        let back = q88_to_f32(q);
        assert_eq!(back, v, "Q8.8 pressure roundtrip drift at p={}", v);
    }
}

#[test]
fn q88_tilt_range_roundtrip() {
    // Tilt ∈ [0, π/2). π/2 ≈ 1.5708. Step exato 1/256.
    let mut t = 0.0f32;
    while t < std::f32::consts::FRAC_PI_2 {
        let q = f32_to_q88(t);
        let back = q88_to_f32(q);
        // Audit T1.8 L3-G10: Q8.8 representa [0, 256) (u16 / 256, NÃO [0, 1)).
        // Tilt [0, π/2 ≈ 1.57) cabe folgado. Tolerância 1/256 (Q8.8 LSB).
        assert!(
            (back - t).abs() <= 1.0 / 256.0,
            "Q8.8 tilt drift > 1 LSB at t={} (q={}, back={})",
            t,
            q,
            back
        );
        t += 0.01;
    }
}

#[test]
fn q1616_zero_negative_inf_clamps_to_zero() {
    assert_eq!(f32_to_q1616_saturating(0.0), 0);
    assert_eq!(f32_to_q1616_saturating(f32::NAN), 0);
    assert_eq!(f32_to_q1616_saturating(f32::INFINITY), 0);
    assert_eq!(f32_to_q1616_saturating(f32::NEG_INFINITY), 0);
}

#[test]
fn q88_nan_and_infinities_clamp_to_zero() {
    // Q8.8 é representação de quantidades fisicamente bounded (pressure ∈ [0,1],
    // tilt ∈ [0, π/2)). NaN ou ±INF como input = sinal de bug upstream — helper
    // escolhe "no contribution" (0) em vez de propagar valor sem sentido pro
    // hot path. Acoplamento com `is_finite()` short-circuit antes do clamp:
    // f32::INFINITY → 0 (não u16::MAX). Sane fallback documentado.
    assert_eq!(f32_to_q88(f32::NAN), 0);
    assert_eq!(f32_to_q88(f32::INFINITY), 0);
    assert_eq!(f32_to_q88(f32::NEG_INFINITY), 0);
}
