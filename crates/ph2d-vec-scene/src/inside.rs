//! **O ponto está dentro da forma?** — módulo irmão de [`crate::path_ops`] (teto de LOC).
//!
//! Amostra cada contorno FECHADO num polígono e aplica a [`FillRule`] do path (even-odd =
//! paridade de cruzamentos; non-zero = winding number). O buraco de um compound é um contorno a
//! mais, então ele sai `false` de graça. Transcendental-free.

use crate::{FillRule, VecPath};

/// Amostras por segmento de cúbica ao achatar o contorno.
const CURVE_SAMPLES: usize = 16;

/// Um ponto da cúbica em `t` (cópia local — o de `path_ops` é privado, e um módulo irmão não
/// justifica alargar a superfície dele).
fn cubic(p0: [f64; 2], p1: [f64; 2], p2: [f64; 2], p3: [f64; 2], t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [
        w0 * p0[0] + w1 * p1[0] + w2 * p2[0] + w3 * p3[0],
        w0 * p0[1] + w1 * p1[1] + w2 * p2[1] + w3 * p3[1],
    ]
}

/// O mesmo teste, para um [`VecPath`] **que não está na cena** — a ponta de seta, que é
/// construída sob demanda (`stroke_head`) e nunca vive no documento.
///
/// É o par natural do método acima: um hit-test que só soubesse perguntar por `id` não
/// conseguiria perguntar pela cabeça da seta — e a cabeça é justamente a parte gorda, a que o
/// olho mira e o mouse persegue.
#[must_use]
pub fn contains_point(path: &VecPath, p: [f64; 2]) -> bool {
    // A geometria COZIDA (quinas já arredondadas) — é a forma que está na tela, e é ela
    // que o dedo tem de pegar. Sem raio nenhum isto é a própria fonte, emprestada.
    let path = &*path.cooked();
    {
        let even_odd = path.fill_rule == FillRule::EvenOdd;
        let mut crossings = 0i32;
        let mut winding = 0i32;
        let mut any = false;
        for k in 0..path.contour_count() {
            let Some((verts, closed)) = path.contour(k) else {
                continue;
            };
            if !closed || verts.len() < 2 {
                continue;
            }
            any = true;
            let mut poly: Vec<[f64; 2]> = Vec::with_capacity(verts.len() * CURVE_SAMPLES);
            for i in 0..verts.len() {
                let a = &verts[i];
                let b = &verts[(i + 1) % verts.len()];
                for j in 0..CURVE_SAMPLES {
                    let t = j as f64 / CURVE_SAMPLES as f64;
                    poly.push(cubic(a.anchor, a.out_handle, b.in_handle, b.anchor, t));
                }
            }
            let n = poly.len();
            if n < 3 {
                continue;
            }
            let mut j = n - 1;
            for i in 0..n {
                let (a, b) = (poly[j], poly[i]);
                // O guard garante a[1] != b[1] (sem divisão por zero).
                if (a[1] > p[1]) != (b[1] > p[1]) {
                    let t = (p[1] - a[1]) / (b[1] - a[1]);
                    let x = a[0] + t * (b[0] - a[0]);
                    if p[0] < x {
                        crossings += 1;
                        winding += if b[1] > a[1] { 1 } else { -1 };
                    }
                }
                j = i;
            }
        }
        if !any {
            return false;
        }
        if even_odd {
            crossings % 2 != 0
        } else {
            winding != 0
        }
    }
}
