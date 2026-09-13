//! ⭐⭐ **Os dois RAIOS DE COLISÃO de uma geometria** — doc 109, ordem do dono (2026-09-13):
//! *«colidem sozinhas»*.
//!
//! Irmão do [`super`] pelo tecto de LOC e por ASSUNTO: lá mora *como a forma vira geometria*, aqui
//! *que tamanho essa geometria ocupa para colidir*.
//!
//! - **`around`** — o menor círculo **centrado na origem da peça** que contém o contorno: `max |p|`.
//! - **`inside`** — o maior círculo centrado na origem que cabe no contorno EXTERIOR: a menor
//!   distância da origem a ele. ⚠️ `0` quando a origem nem está dentro desse contorno (um
//!   crescente): ali não existe círculo inscrito à volta de `P`, e inventar um seria mentir.
//!
//! ⚠️ **Centrados na ORIGEM e não no centro do contorno**, porque o solver põe o disco em `P`, e
//! `P` é a origem da peça — um círculo mínimo deslocado não seria o que colide.
//!
//! ⚠️ **Por amostragem de Bernstein**, e não por `kurbo::flatten`/`nearest`, que o `ph2d_vector` não
//! reexporta: [`AMOSTRAS`] por segmento dão, num quarto de círculo, um erro de corda de
//! `1 − cos(π/4 / 64) ≈ 7,5e-5` do raio — abaixo de tudo o que um contacto consegue mostrar.
//!
//! ⚠️ **O contorno é o do PREENCHIMENTO** (`build_fill_bezpath`): as linhas de construção abertas (a
//! aresta interior do cubo isométrico) não ocupam espaço, e o traço também não entra — um colisor
//! que crescesse com a largura da linha é outra pergunta, e ela fica nomeada no doc 109.

use ph2d_vec_scene::VecPath;
use ph2d_vector::PathEl;

/// Amostras por segmento de curva — ver o cabeçalho para o erro que elas compram.
const AMOSTRAS: u32 = 64;

/// `[around, inside]` na unidade da geometria (a forma é construída em raio 1).
#[expect(
    clippy::cast_possible_truncation,
    reason = "um raio de geometria em raio 1 cabe num f32 sem perda que um contacto veja"
)]
pub(super) fn measure(path: &VecPath) -> [f32; 2] {
    let contornos = polilinhas(path);
    let around = contornos
        .iter()
        .flatten()
        .map(|p| p[0].hypot(p[1]))
        .fold(0.0_f64, f64::max);
    let inside = contornos
        .iter()
        .max_by(|a, b| area(a).abs().total_cmp(&area(b).abs()))
        .filter(|exterior| contem_a_origem(exterior))
        .map_or(0.0, |exterior| distancia_a_origem(exterior));
    [around as f32, inside as f32]
}

/// Cada sub-caminho do preenchimento como uma polilinha fechada implícita.
fn polilinhas(path: &VecPath) -> Vec<Vec<[f64; 2]>> {
    let mut out: Vec<Vec<[f64; 2]>> = Vec::new();
    let mut ultimo = [0.0, 0.0];
    let junta = |out: &mut Vec<Vec<[f64; 2]>>, p: [f64; 2]| match out.last_mut() {
        Some(c) => c.push(p),
        None => out.push(vec![p]),
    };
    for el in ph2d_vec_render::build_fill_bezpath(path).elements() {
        match *el {
            PathEl::MoveTo(p) => {
                ultimo = [p.x, p.y];
                out.push(vec![ultimo]);
            }
            PathEl::LineTo(p) => {
                ultimo = [p.x, p.y];
                junta(&mut out, ultimo);
            }
            PathEl::QuadTo(c, p) => {
                let (a, c, b) = (ultimo, [c.x, c.y], [p.x, p.y]);
                for k in 1..=AMOSTRAS {
                    let t = f64::from(k) / f64::from(AMOSTRAS);
                    let u = 1.0 - t;
                    let q = |i: usize| u * u * a[i] + 2.0 * u * t * c[i] + t * t * b[i];
                    junta(&mut out, [q(0), q(1)]);
                }
                ultimo = b;
            }
            PathEl::CurveTo(c1, c2, p) => {
                let (a, c1, c2, b) = (ultimo, [c1.x, c1.y], [c2.x, c2.y], [p.x, p.y]);
                for k in 1..=AMOSTRAS {
                    let t = f64::from(k) / f64::from(AMOSTRAS);
                    let u = 1.0 - t;
                    let q = |i: usize| {
                        u * u * u * a[i]
                            + 3.0 * u * u * t * c1[i]
                            + 3.0 * u * t * t * c2[i]
                            + t * t * t * b[i]
                    };
                    junta(&mut out, [q(0), q(1)]);
                }
                ultimo = b;
            }
            PathEl::ClosePath => {}
        }
    }
    out
}

/// Os lados da polilinha, com o que a fecha.
fn lados(c: &[[f64; 2]]) -> impl Iterator<Item = ([f64; 2], [f64; 2])> + '_ {
    (0..c.len()).map(move |i| (c[i], c[(i + 1) % c.len()]))
}

/// A área com sinal (shoelace).
fn area(c: &[[f64; 2]]) -> f64 {
    lados(c)
        .map(|(a, b)| a[0] * b[1] - b[0] * a[1])
        .sum::<f64>()
        * 0.5
}

/// A origem está dentro da polilinha? (par-ímpar, raio para `+x`)
fn contem_a_origem(c: &[[f64; 2]]) -> bool {
    lados(c)
        .filter(|(a, b)| {
            (a[1] > 0.0) != (b[1] > 0.0) && {
                let x = a[0] + (0.0 - a[1]) * (b[0] - a[0]) / (b[1] - a[1]);
                x > 0.0
            }
        })
        .count()
        % 2
        == 1
}

/// A menor distância da origem a um lado da polilinha.
fn distancia_a_origem(c: &[[f64; 2]]) -> f64 {
    lados(c)
        .map(|(a, b)| {
            let d = [b[0] - a[0], b[1] - a[1]];
            let len2 = d[0] * d[0] + d[1] * d[1];
            let t = if len2 > 0.0 {
                (-(a[0] * d[0] + a[1] * d[1]) / len2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            (a[0] + t * d[0]).hypot(a[1] + t * d[1])
        })
        .fold(f64::INFINITY, f64::min)
}

#[cfg(test)]
#[path = "motion_shape_collider_tests.rs"]
mod tests;
