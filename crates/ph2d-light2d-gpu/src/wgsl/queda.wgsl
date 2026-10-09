// A GÊMEA de `ph2d_light2d_ref::queda::queda` — a janela de B. Karis, «Real Shading in Unreal
// Engine 4» (SIGGRAPH 2013), eq. (9): saturate(1 − (d/R)⁴)². Gate G12 (mesma tabela de 1 000
// pontos do lado Rust). Texto ÚNICO: quem a usa concatena este ficheiro, nunca o copia.
//
// Recebe o INVERSO do alcance (0 = alcance infinito, física pura): `t·(1/R)` dispensa o ramo do
// infinito, que o WGSL não sabe escrever como literal.
fn queda(t: f32, inv_alcance: f32) -> f32 {
    let x = t * inv_alcance;
    let x2 = x * x;
    let s = clamp(1.0 - x2 * x2, 0.0, 1.0);
    return s * s;
}
