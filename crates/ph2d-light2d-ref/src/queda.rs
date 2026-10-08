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
