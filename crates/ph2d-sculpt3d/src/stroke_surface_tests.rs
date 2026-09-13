//! **A SUPERFÍCIE não depende da BASE TANGENTE em que foi ajustada** — o gate que o doc do
//! [`super::tangent_frame`] prometia.
//!
//! ⛔ **Ele era NOMEADO e NÃO EXISTIA** (medido 2026-09-13: nenhum commit no histórico do git o
//! escreveu). É a segunda nota deste ficheiro a prometer um gate ausente: o da escala por raio
//! (`the_fit_is_the_same_surface_at_any_brush_size`) nasceu da outra, com a mutação que ele
//! mata a sobreviver à suíte inteira até lá.
//!
//! # O que se mede
//!
//! A troca de semente em `|n.x| ≥ 0,9` faz a base saltar, e o doc diz *«os coeficientes saltam,
//! a superfície não»*. As bases possíveis diferem por uma ROTAÇÃO em torno da normal (todas são
//! directas, `tu × tv = n`), logo a propriedade é: **ajustar as MESMAS amostras em qualquer
//! base rodada devolve a mesma altura**. A razão é álgebra — as quádricas completas
//! `{1, u, v, u², uv, v²}` são fechadas sob rotação —, e é por isso que o gate morre quando o
//! ajuste perde um termo.
//!
//! ⚠️ **O controlo é que os COEFICIENTES mudam** entre as bases: sem ele uma fixtura simétrica
//! demais (uma parábola de revolução) passaria sem re-parametrizar nada.

use super::*;

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn unit(a: [f32; 3]) -> [f32; 3] {
    let l = dot(a, a).sqrt();
    [a[0] / l, a[1] / l, a[2] / l]
}

/// A base rodada de `theta` em torno da normal — directa como a original.
fn rodada((tu, tv): ([f32; 3], [f32; 3]), theta: f32) -> ([f32; 3], [f32; 3]) {
    let (c, s) = (theta.cos(), theta.sin());
    let comb = |x: f32, y: f32| {
        [
            x * tu[0] + y * tv[0],
            x * tu[1] + y * tv[1],
            x * tu[2] + y * tv[2],
        ]
    };
    (comb(c, s), comb(-s, c))
}

const RAIO: f32 = 0.4;

/// A superfície da fixtura, escrita numa base PRÓPRIA que não é nenhuma das que o ajuste usa:
/// curvaturas diferentes nos dois eixos e um termo cruzado, para que rotação nenhuma a deixe
/// igual a si mesma.
fn altura(x: f32, y: f32) -> f32 {
    0.01 + 0.02 * x - 0.015 * y + 0.08 * x * x - 0.05 * x * y + 0.03 * y * y
}

/// Os pontos de uma pegada circular, como OFFSETS de mundo ao ponto do plano, com a altura.
fn pegada(n: [f32; 3]) -> Vec<([f32; 3], f32)> {
    let (e1, e2) = rodada(tangent_frame(n), 0.6);
    let mut out = Vec::new();
    for i in -6i8..=6 {
        for j in -6i8..=6 {
            let (x, y) = (f32::from(i) / 6.0, f32::from(j) / 6.0);
            if x * x + y * y > 1.0 {
                continue;
            }
            let h = altura(x, y);
            let d = [
                RAIO * (x * e1[0] + y * e2[0]) + h * n[0],
                RAIO * (x * e1[1] + y * e2[1]) + h * n[1],
                RAIO * (x * e1[2] + y * e2[2]) + h * n[2],
            ];
            out.push((d, h));
        }
    }
    out
}

/// O ajuste do PRODUTO sobre as amostras, na base dada — as coordenadas são as mesmas que o
/// `stroke_plane` monta: a projecção na base, dividida pelo raio.
fn ajuste(amostras: &[([f32; 3], f32)], base: ([f32; 3], [f32; 3])) -> Quadric {
    let inv_r = 1.0 / RAIO;
    fit(
        amostras
            .iter()
            .map(|&(d, h)| (dot(d, base.0) * inv_r, dot(d, base.1) * inv_r, h, 1.0)),
        base,
        inv_r,
    )
    .expect("a pegada tem pontos de sobra e não é degenerada")
}

/// ⚠️ **A barra saiu de um vale MEDIDO** (2026-09-13):
///
/// | lado | pior desvio |
/// |---|---:|
/// | produto, `n` antes da troca de semente | `2,98e-8` |
/// | produto, `n` depois da troca de semente | `2,24e-8` |
/// | mutante: o monómio `uv` vira `u²` no `height_at` | `8,53e-2` |
///
/// `1e-5` fica ~2,5 ordens de grandeza acima do produto (a aritmética de `f32`) e ~4 abaixo do
/// mutante.
const TOL: f32 = 1e-5;

#[test]
fn the_surface_is_the_same_in_any_tangent_frame() {
    // Uma normal de cada lado da troca de semente — `|n.x| < 0,9` e `≥ 0,9`.
    let normais = [unit([0.3, -0.5, 0.81]), unit([0.95, 0.2, 0.24])];
    assert!(
        normais[0][0].abs() < 0.9 && normais[1][0].abs() >= 0.9,
        "controlo: as duas normais têm de cair em lados opostos da troca de semente"
    );
    for n in normais {
        let amostras = pegada(n);
        assert!(
            amostras.len() > 60,
            "controlo: a pegada tem só {} pontos",
            amostras.len()
        );
        let base = tangent_frame(n);
        let mut pior = 0.0f32;
        let mut primeiro: Option<Quadric> = None;
        let mut maior_salto = 0.0f32;
        for theta in [
            0.0f32,
            0.37,
            1.0,
            std::f32::consts::FRAC_PI_2,
            2.5,
            std::f32::consts::PI,
        ] {
            let q = ajuste(&amostras, rodada(base, theta));
            if let Some(p) = &primeiro {
                for (a, b) in q.c.iter().zip(p.c) {
                    maior_salto = maior_salto.max((a - b).abs());
                }
            }
            for &(d, h) in &amostras {
                pior = pior.max((q.height_at(d) - h).abs());
            }
            primeiro.get_or_insert(q);
        }
        assert!(
            pior < TOL,
            "a superfície mudou com a base tangente: pior desvio {pior:.3e} (barra {TOL:.0e}) \
             com n = {n:?}"
        );
        assert!(
            maior_salto > 1e-3,
            "controlo: os coeficientes não mudaram entre as bases ({maior_salto:.3e}) — a \
             fixtura não re-parametriza nada e o gate não mede a pergunta"
        );
        eprintln!(
            "[surface-frame] n = {n:?}: pior desvio {pior:.3e}, maior salto de coeficiente \
             {maior_salto:.3e}"
        );
    }
}
