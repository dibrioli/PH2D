//! **QUANTO ESPAÇO uma malha fechada encerra** — o teorema da divergência.
//!
//! ⚠️ **Ele existia como helper privado num `_tests.rs`** (`extract_tests.rs`),
//! e subiu para a crate quando ganhou um segundo consumidor: o remesh precisa
//! dele para saber se o campo de distância *encontrou* o interior que a malha
//! declara ter. Duas cópias da mesma integral divergiriam no dia em que uma
//! aprendesse a tratar quads e a outra não — e a divergência apareceria como
//! uma recusa que dispara na peça errada.

use crate::mesh::Mesh;

/// O volume COM SINAL de uma malha fechada — positivo quando ela é enrolada
/// para FORA.
///
/// ⚠️ **O sinal é o que separa *"a casca fechou"* de *"a casca fechou do lado
/// certo"***: inverter a peça inteira mantém toda aresta com duas faces em
/// sentidos opostos e só troca o sinal disto. Quem só quer o TAMANHO usa
/// `.abs()` — é o caso do remesh, cujo flood fill é geométrico e não sabe o que
/// é winding.
///
/// ⚠️ **Numa malha ABERTA o número não significa nada** (a integral pressupõe
/// fronteira fechada). Isso não é uma limitação a contornar: o único chamador de
/// produto o consulta DEPOIS do `fill_holes`.
///
/// Um quad é somado como o leque de triângulos que ele é, então a conta não
/// depende de a malha ter sido triangulada.
#[must_use]
pub fn signed_volume(mesh: &Mesh) -> f32 {
    let p = mesh.positions();
    let mut v = 0.0f32;
    for f in mesh.faces() {
        let idx = f.verts();
        for k in 1..idx.len() - 1 {
            let (a, b, c) = (
                p[idx[0] as usize],
                p[idx[k] as usize],
                p[idx[k + 1] as usize],
            );
            let cr = [
                b[1] * c[2] - b[2] * c[1],
                b[2] * c[0] - b[0] * c[2],
                b[0] * c[1] - b[1] * c[0],
            ];
            v += a[0] * cr[0] + a[1] * cr[1] + a[2] * cr[2];
        }
    }
    v / 6.0
}
