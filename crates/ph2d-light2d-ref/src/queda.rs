//! **A queda autoral com a distância** — porte da janela de B. Karis, «Real Shading in Unreal
//! Engine 4» (SIGGRAPH 2013), eq. (9): `saturate(1 − (d/R)⁴)²`. O `1/(d²+1)` da mesma equação é o
//! inverso do quadrado do 3D e fica de fora: em 2D a lei física é o ângulo subtendido, que o
//! gabarito integra; a queda é só a JANELA autoral que leva a luz a zero em `R`.
//!
//! ⚠️ Tem gêmea WGSL na W2 (gate G12). Fica ISOLADA de propósito: mudar aqui é mudar lá.

/// Fator de queda à distância `t ≥ 0` do acerto, para uma luz de `alcance > 0`
/// (`f64::INFINITY` = física pura, fator 1).
#[must_use]
pub fn queda(t: f64, alcance: f64) -> f64 {
    debug_assert!(
        alcance > 0.0,
        "alcance tem de ser > 0 (a coleta apaga o resto): {alcance}"
    );
    if alcance == f64::INFINITY {
        return 1.0;
    }
    let x = t / alcance;
    let x2 = x * x;
    let s = (1.0 - x2 * x2).clamp(0.0, 1.0);
    s * s
}

/// **A tabela do G12** — os 1 000 pontos `(t, alcance)` onde a `queda` é medida, contra a fórmula
/// publicada (lado Rust) e contra a gêmea WGSL (`ph2d-light2d-gpu`). Porta ÚNICA: uma cópia da
/// tabela num dos lados mediria outra coisa sem ninguém notar.
///
/// Seis alcances (`0,5 · 1 · 3,7 · 25 · 1 000 · ∞`); em cada um, `t` de 0 a `1,5·R` (atravessa o
/// joelho em `R` e a cauda nula; `R = ∞` varre 0..1 000 m).
#[must_use]
pub fn pontos_g12() -> Vec<(f64, f64)> {
    let alcances = [0.5_f64, 1.0, 3.7, 25.0, 1e3, f64::INFINITY];
    let mut pontos = Vec::with_capacity(1000);
    for (i, &r) in alcances.iter().enumerate() {
        let por_alcance: u32 = if i + 1 == alcances.len() {
            1000 - 5 * 166
        } else {
            166
        };
        let escala = if r.is_finite() { 1.5 * r } else { 1e3 };
        for k in 0..por_alcance {
            pontos.push((escala * f64::from(k) / f64::from(por_alcance - 1), r));
        }
    }
    pontos
}
