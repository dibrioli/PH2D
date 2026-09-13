//! ⭐⭐ **A CAIXA ENVOLVENTE de uma geometria, para colidir** — doc 109 §5, report do dono
//! (2026-09-13): *«o collider não é gerado conforme a forma da Shape»*.
//!
//! Irmão do [`super`] pelo tecto de LOC e por ASSUNTO: lá mora *como a forma vira geometria*, aqui
//! *que espaço essa geometria ocupa para colidir*.
//!
//! Uma medida só serve as duas formas de colisor que o cartão oferece: a **caixa** é ela, e o
//! **círculo** toca os lados maiores dela (a escolha é do nó, `ph2d_node_motion_shape`).
//!
//! ⚠️ **Pelo MEIO da caixa e não pela origem da peça**: uma estrela de cinco pontas vai a `y = 1` em
//! cima e a `y ≈ −0,81` em baixo, e um colisor centrado na origem sairia deslocado da arte. O centro
//! viaja no stream e o solver põe-no no sítio certo, girado e escalado com a peça.
//!
//! ⚠️ **Por amostragem de Bernstein**, e não pelos pontos de controlo: os de uma cúbica ficam FORA
//! da curva, e uma caixa feita deles seria maior que a arte. [`AMOSTRAS`] por segmento dão, num
//! quarto de círculo, um erro de corda de `1 − cos(π/4 / 64) ≈ 7,5e-5` do raio — abaixo de tudo o
//! que um contacto consegue mostrar.
//!
//! ⚠️ **O contorno é o do PREENCHIMENTO** (`build_fill_bezpath`): as linhas de construção abertas (a
//! aresta interior do cubo isométrico) não ocupam espaço, e o traço também não entra — um colisor
//! que crescesse com a largura da linha é outra pergunta, e ela fica nomeada no doc 109.

use ph2d_vec_scene::VecPath;
use ph2d_vector::PathEl;

/// Amostras por segmento de curva — ver o cabeçalho para o erro que elas compram.
const AMOSTRAS: u32 = 64;

/// **A caixa envolvente do contorno de preenchimento**, na unidade da geometria (a forma é
/// construída em raio 1).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ColliderFit {
    /// O meio da caixa.
    pub center: [f32; 2],
    /// As meias extensões. `[0, 0]` numa geometria vazia — um colisor que não colide, nunca
    /// inventado.
    pub half: [f32; 2],
}

/// Mede a caixa envolvente de uma geometria.
#[expect(
    clippy::cast_possible_truncation,
    reason = "uma extensão de geometria em raio 1 cabe num f32 sem perda que um contacto veja"
)]
pub fn measure(path: &VecPath) -> ColliderFit {
    let mut lo = [f64::INFINITY; 2];
    let mut hi = [f64::NEG_INFINITY; 2];
    for p in polilinhas(path).iter().flatten() {
        lo = [lo[0].min(p[0]), lo[1].min(p[1])];
        hi = [hi[0].max(p[0]), hi[1].max(p[1])];
    }
    if lo[0] > hi[0] || lo[1] > hi[1] {
        return ColliderFit::default();
    }
    ColliderFit {
        center: [
            ((lo[0] + hi[0]) * 0.5) as f32,
            ((lo[1] + hi[1]) * 0.5) as f32,
        ],
        half: [
            ((hi[0] - lo[0]) * 0.5) as f32,
            ((hi[1] - lo[1]) * 0.5) as f32,
        ],
    }
}

/// Cada sub-caminho do preenchimento como uma polilinha.
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

#[cfg(test)]
#[path = "motion_shape_collider_tests.rs"]
mod tests;
