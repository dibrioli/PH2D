//! A aritmética de CAMINHO que o traço partilha: distância, interpolação linear e a base de Hermite
//! (que é como uma Catmull-Rom é avaliada).
//!
//! Ela vive num módulo próprio porque tem **três** consumidores — o caminhador (`stroke.rs`), o
//! achatador de curva (`curve.rs`) e o estabilizador (`stabilize.rs`) — e nenhum deles é o dono do
//! fato; deixá-la em qualquer um faria dos outros dois clientes de um detalhe alheio.

#[inline]
pub(super) fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    (dx * dx + dy * dy).sqrt()
}

#[inline]
pub(super) fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Cubic Hermite at `t∈[0,1]`: endpoints `p0`,`p1` with tangents `m0`,`m1`. With Catmull-Rom
/// tangents this evaluates the spline segment between two input points.
#[inline]
pub(super) fn hermite(p0: [f32; 2], m0: [f32; 2], p1: [f32; 2], m1: [f32; 2], t: f32) -> [f32; 2] {
    let t2 = t * t;
    let t3 = t2 * t;
    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + t;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;
    [
        h00 * p0[0] + h10 * m0[0] + h01 * p1[0] + h11 * m1[0],
        h00 * p0[1] + h10 * m0[1] + h01 * p1[1] + h11 * m1[1],
    ]
}
