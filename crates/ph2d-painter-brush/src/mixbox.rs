//! Mixbox — pigment mixing subtrativo (Sochorová+Jamriška SIGGRAPH 2021).
//! ADR-0044 §2.5 + Proposta 2.
//!
//! **T1.3 stub.** Apenas interface API + comment com referência ao paper +
//! repo. Implementação WGSL real + LUT 3KB + 12 SH coefs em T-color+T1.X+.
//!
//! **NÃO FUNCIONA em --features det-painter** (ADR-0044 §2.5.1) — fallback
//! Linear; port Q-fixed-point é follow-up registrado.

/// Linear interpolation entre duas cores sRGB via Mixbox pigment mixing.
/// Stub T1.3: retorna o input `b` (não-funcional). Substitui em T-color+T1.X+.
///
/// Refs:
/// - Paper: Sochorová+Jamriška 2021 ACM TOG 40(6).
/// - Open-source impl: <https://github.com/scrtwpns/mixbox> (CC0/MIT, ~700 LOC C).
pub fn mixbox_lerp_srgb8_stub(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    // T1.3 stub: fallback linear lerp em sRGB. T-color implementa Kubelka-Munk
    // subtrativo real via 7 pigment primaries (Hansa Yellow, Cadmium Red,
    // Cobalt Blue, Phthalo Green, Burnt Sienna, Titanium White, Carbon Black).
    let t = t.clamp(0.0, 1.0);
    [
        ((a[0] as f32) * (1.0 - t) + (b[0] as f32) * t) as u8,
        ((a[1] as f32) * (1.0 - t) + (b[1] as f32) * t) as u8,
        ((a[2] as f32) * (1.0 - t) + (b[2] as f32) * t) as u8,
    ]
}

/// Mixbox linear lerp em Linear sRGB (canonical Mixbox space). Stub T1.3.
pub fn mixbox_lerp_linear_stub(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] * (1.0 - t) + b[0] * t,
        a[1] * (1.0 - t) + b[1] * t,
        a[2] * (1.0 - t) + b[2] * t,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_lerp_at_t0_returns_a() {
        let a = [10, 20, 30];
        let b = [200, 100, 50];
        assert_eq!(mixbox_lerp_srgb8_stub(a, b, 0.0), a);
    }

    #[test]
    fn stub_lerp_at_t1_returns_b() {
        let a = [10, 20, 30];
        let b = [200, 100, 50];
        assert_eq!(mixbox_lerp_srgb8_stub(a, b, 1.0), b);
    }

    #[test]
    fn stub_lerp_linear_clamps_t() {
        let a = [0.0, 0.5, 1.0];
        let b = [1.0, 1.0, 1.0];
        // t > 1.0 deve clamp para 1.0 → retorna b.
        assert_eq!(mixbox_lerp_linear_stub(a, b, 2.0), b);
        // t < 0.0 deve clamp para 0.0 → retorna a.
        assert_eq!(mixbox_lerp_linear_stub(a, b, -1.0), a);
    }
}
