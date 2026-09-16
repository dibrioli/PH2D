//! ⏱️⭐⭐⭐ **OS CENTROS DE ROTAÇÃO OPTIMIZADOS, na mesa** — o candidato cuja RECUSA perdeu o chão.
//!
//! # Porque ele volta à mesa (§0.0)
//!
//! A mesa de 2026-09-14 mediu os *optimized centers of rotation* (Le & Hodgins, SIGGRAPH 2016) em
//! **`8,8 %`** de dobra e escreveu, ela própria, por que o número não valia:
//!
//! > *«não se pode julgar uma mistura melhor por cima de pesos degenerados — `1 155` dos `2 401`
//! > vértices (`48 %`) têm peso exactamente `0` ou `1` … metade da nossa arte não mistura nada»*
//!
//! ⭐ **Essa premissa dissolveu-se:** com o padrão-ouro e a folga da junta, a arte rígida passou de
//! `37,6 %` para **`6,6 %`**. *A recusa não foi revogada — o chão dela mudou.*
//!
//! # A lei, escrita
//!
//! O colapso do LBS a `90°` não é dos PESOS: é da MISTURA. Misturar linearmente duas rotações
//! encolhe o resultado (a média de dois versores não é um versor), e é isso que estrangula a arte na
//! parte de dentro da dobra. A cura publicada não toca nos pesos — ela troca o **ponto em torno do
//! qual** cada vértice roda:
//!
//! ```text
//!     p*(v) = Σ_p  s(w_v, w_p) · a_p · p   /   Σ_p  s(w_v, w_p) · a_p
//!     v'    = R(w_v) · (v − p*(v))  +  LBS(p*(v))
//! ```
//!
//! - `s` é a **semelhança entre dois vectores de peso** — dois pontos com a mesma mistura de ossos
//!   partilham centro;
//! - `R(w_v)` é a rotação **misturada como rotação** (em 2D: a soma dos versores, renormalizada),
//!   ⛔ nunca a matriz linear misturada, que é precisamente o que encolhe;
//! - `LBS(p*)` é o LBS de sempre, avaliado **no centro** — é ele que põe o vértice no sítio.
//!
//! ⭐⭐ **`p*` só depende do REPOUSO e dos pesos** ⇒ é trabalho do *bind*, e viaja por vértice como os
//! pesos. Isto encaixa no que a [`crate`] já guarda.
//!
//! ⚠️ **`O(n²)` no bind**, e é o preço declarado: `2 401` vértices ⇒ `5,8 M` pares em `46 ms`.
//!
//! # ⛔⛔⛔ O VEREDITO: RECUSADO, e por uma razão DIFERENTE da de 2026-09-14
//!
//! | lei | `60°` | `90°` | `120°` | `150°` |
//! |---|---:|---:|---:|---:|
//! | **padrão-ouro + LBS (o que shipa)** | **`0,28 %`** | **`8,27 %`** | **`11,00 %`** | **`9,51 %`** |
//! | padrão-ouro + centros de rotação | `1,45 %` | `11,70 %` | `13,80 %` | `12,57 %` |
//!
//! ⚠️ **A recusa antiga dizia *«os pesos são degenerados, não dá para julgar»*. Essa dissolveu-se —
//! e o candidato perde na mesma, por uma causa que só agora é visível:**
//!
//! ⭐⭐⭐ **O nosso meio é uma FOLHA PLANA com os ossos a correr pelo MEIO dela, então o campo de
//! pesos é SIMÉTRICO em `y`.** Medido: `1 176` pares espelhados `(x, +y)` / `(x, −y)` com uma
//! diferença de peso de **`6,7e-4`**. A semelhança do artigo é uma função **dos pesos e de mais
//! nada** ⇒ ela não pode distinguir os dois lados, e o centro de ambos cai **no eixo**:
//!
//! - `|y|` médio do centro: **`0,0003`**, numa arte que vai de `−2,4` a `+2,4`;
//! - `2 074` dos `2 243` vértices ficam a mais de `0,5` do próprio centro, o pior a `2,53`.
//!
//! ⇒ a metade de cima e a de baixo passam a rodar em torno do **mesmo ponto**, e o resultado é pior
//! que o LBS. É a limitação que o próprio artigo nomeia (partes simétricas que partilham pesos
//! colapsam num centro comum) — aqui ela não é um caso patológico, **é a forma normal da nossa
//! arte**.
//!
//! ⛔⛔ **E nenhum `σ` cura isto**, o que é o que torna esta recusa FINAL em vez de uma afinação: a
//! semelhança é uma função dos vectores de peso, e dois pontos com o mesmo vector de peso são
//! indistinguíveis para qualquer função deles.
//!
//! ⚠️ **É a MESMA frase que fechou a porta da difusão de calor** em 2026-09-14: *«o método não é
//! mau — ele não é do nosso meio»*. Duas técnicas de topo do campo, recusadas pela mesma
//! propriedade da nossa geometria. ⭐ *Isso é um facto sobre o MEIO, e vale mais do que as duas
//! recusas somadas: uma terceira candidata que dependa de distinguir pontos pelos PESOS vai falhar
//! aqui também — e agora sabe-se antes de a construir.*

use super::bancada::interpola;
use super::bancada_oraculo::{ORA_ARCO, ora_bump, ora_malha, ora_poses, ora_reguas_de};
use super::{Handle, Options, bounded_biharmonic};
use ph2d_affine::Xform;
use ph2d_poly2d::Mesh2d;

/// A área de cada vértice — um terço da área dos triângulos que o tocam.
fn areas(m: &Mesh2d) -> Vec<f64> {
    let mut a = vec![0.0; m.rest.len()];
    for t in &m.tris {
        let (x, y, z) = (
            m.rest[t[0] as usize],
            m.rest[t[1] as usize],
            m.rest[t[2] as usize],
        );
        let s = ((y[0] - x[0]) * (z[1] - x[1]) - (y[1] - x[1]) * (z[0] - x[0])).abs() / 2.0;
        for i in t {
            a[*i as usize] += s / 3.0;
        }
    }
    a
}

/// ⭐ **A SEMELHANÇA entre dois vectores de peso.**
///
/// ⚠️ Ela não é uma distância entre vectores: é uma soma sobre **pares de ossos**, e o que ela mede
/// é se os dois pontos repartem os mesmos dois ossos na mesma PROPORÇÃO. `σ = 0,1` é o valor
/// publicado.
fn semelhanca(a: &[f64], b: &[f64], sigma2: f64) -> f64 {
    let mut s = 0.0;
    for j in 0..a.len() {
        for k in 0..a.len() {
            if j == k {
                continue;
            }
            let d = a[j] * b[k] - a[k] * b[j];
            s += a[j] * a[k] * b[j] * b[k] * (-(d * d) / sigma2).exp();
        }
    }
    s
}

/// Os centros de rotação, um por vértice. `None` onde nenhum vizinho é semelhante (o vértice fica
/// com o LBS de sempre — ⛔ nunca um centro inventado).
fn centros(m: &Mesh2d, pesos: &[Vec<f64>], sigma: f64) -> Vec<Option<[f64; 2]>> {
    let a = areas(m);
    let sigma2 = sigma * sigma;
    m.rest
        .iter()
        .enumerate()
        .map(|(v, _)| {
            let (mut num, mut den) = ([0.0, 0.0], 0.0);
            for (p, q) in m.rest.iter().enumerate() {
                let s = semelhanca(&pesos[v], &pesos[p], sigma2) * a[p];
                if s <= 0.0 {
                    continue;
                }
                num[0] += s * q[0];
                num[1] += s * q[1];
                den += s;
            }
            (den > 1e-12).then(|| [num[0] / den, num[1] / den])
        })
        .collect()
}

/// A rotação MISTURADA COMO ROTAÇÃO — em 2D, a soma dos versores renormalizada.
///
/// ⛔ **É esta linha que separa a lei nova da velha.** Misturar a matriz linear encolhe o resultado
/// (a média de dois versores tem norma `< 1`), e é esse encolhimento que estrangula a arte na parte
/// de dentro de uma dobra forte.
fn rotacao_misturada(poses: &[Xform], w: &[f64]) -> (f64, f64) {
    let (mut c, mut s) = (0.0, 0.0);
    for (j, &wj) in w.iter().enumerate() {
        if wj == 0.0 {
            continue;
        }
        let Xform([a, b, ..]) = poses[j];
        let n = a.hypot(b);
        if n > 0.0 {
            c += wj * a / n;
            s += wj * b / n;
        }
    }
    let n = c.hypot(s);
    if n > 1e-12 {
        (c / n, s / n)
    } else {
        (1.0, 0.0)
    }
}

/// Onde a lei dos CENTROS põe um ponto.
fn pos_cor(p: [f64; 2], w: &[f64], poses: &[Xform], centro: Option<[f64; 2]>) -> [f64; 2] {
    let lbs = |q: [f64; 2]| {
        let mut o = [0.0, 0.0];
        for (j, &wj) in w.iter().enumerate() {
            if wj == 0.0 {
                continue;
            }
            let r = poses[j].apply(q);
            o[0] += wj * r[0];
            o[1] += wj * r[1];
        }
        o
    };
    let Some(cen) = centro else { return lbs(p) };
    let (c, s) = rotacao_misturada(poses, w);
    let d = [p[0] - cen[0], p[1] - cen[1]];
    let base = lbs(cen);
    [
        (d[0] * c - d[1] * s) + base[0],
        (d[0] * s + d[1] * c) + base[1],
    ]
}

/// ⏱️⭐⭐⭐ **A MESA: o padrão-ouro com e sem os centros de rotação** (`--ignored`).
#[test]
#[ignore = "bancada: a mesa dos centros de rotacao, sem barra"]
fn bancada_centros_de_rotacao() {
    let m = ora_malha(48);
    let l = ORA_ARCO / 2.0;
    let handles = vec![
        Handle {
            a: [0.0, 0.0],
            b: [l, 0.0],
        },
        Handle {
            a: [l, 0.0],
            b: [2.0 * l, 0.0],
        },
    ];
    let w = bounded_biharmonic(&m, &handles, Options::default()).expect("resolve");
    let t = std::time::Instant::now();
    let cen = centros(&m, &w.por_vertice, 0.1);
    let com_centro = cen.iter().filter(|c| c.is_some()).count();
    println!(
        "centros: {com_centro} de {} vertices, em {:?} ({} pares)",
        m.rest.len(),
        t.elapsed(),
        m.rest.len() * m.rest.len()
    );

    // ⭐⭐⭐ **A HIPÓTESE, medida em vez de afirmada:** numa FOLHA PLANA com os ossos a correr pelo
    // meio dela, o campo de pesos é **simétrico em `y`** — o ponto `(x, +2)` e o `(x, −2)` têm o
    // MESMO vector de peso. A semelhança não sabe distinguir dois pontos que só diferem no lado, e
    // o centro de ambos cai **no eixo**. ⇒ a metade de cima e a de baixo passam a rodar em torno do
    // mesmo ponto, que é exactamente a limitação que o próprio artigo nomeia.
    let mut longe = 0usize;
    let (mut pior, mut soma_y) = (0.0_f64, 0.0);
    for (v, q) in m.rest.iter().enumerate() {
        if let Some(c) = cen[v] {
            let d = (c[0] - q[0]).hypot(c[1] - q[1]);
            pior = pior.max(d);
            soma_y += c[1].abs();
            if d > 0.5 {
                longe += 1;
            }
        }
    }
    println!(
        "distancia ao centro: pior {pior:.2} (a arte tem meia-altura 2,4) | {longe} vertices a mais \
         de 0,5 | |y| MEDIO do centro {:.4} (o eixo e' y = 0)",
        soma_y / com_centro as f64
    );

    // ⛔⛔⛔ **E NENHUM `σ` cura isto, porque a simetria é EXACTA.** Se o vector de peso de `(x, +y)`
    // for bit-a-bit o de `(x, −y)`, nenhuma função DELES os distingue — a semelhança é uma função
    // dos pesos e de mais nada. *Esta linha é o que torna a recusa final em vez de uma afinação.*
    let mut pior_simetria = 0.0_f64;
    let mut pares = 0usize;
    for (v, q) in m.rest.iter().enumerate() {
        if q[1] <= 1e-9 {
            continue;
        }
        let espelho = m
            .rest
            .iter()
            .position(|r| (r[0] - q[0]).abs() < 1e-9 && (r[1] + q[1]).abs() < 1e-9);
        if let Some(e) = espelho {
            pares += 1;
            for (a, b) in w.por_vertice[v].iter().zip(&w.por_vertice[e]) {
                pior_simetria = pior_simetria.max((a - b).abs());
            }
        }
    }
    println!(
        "simetria em y: {pares} pares espelhados, pior diferenca de peso {pior_simetria:.3e} \
         -- nenhum sigma distingue dois pontos com o MESMO vector de peso"
    );

    println!(
        "\nlei                                 60°            90°           120°           150°"
    );
    let malha = ora_malha(48);
    // A lei de hoje: o padrão-ouro com LBS.
    print!("{:<32}", "padrao-ouro + LBS (hoje)");
    for g in [60.0_f64, 90.0, 120.0, 150.0] {
        let poses = ora_poses(g.to_radians());
        let (d, seg) = ora_reguas_de(&malha, &|p| {
            let ws = interpola(&m, &w.por_vertice, p).unwrap_or_else(|| ora_bump(p, l));
            pos_cor(p, &ws, &poses, None)
        });
        print!("  {d:>5.2}% /{seg:>6.1}°");
    }
    println!();
    // A lei candidata: o padrão-ouro com os centros.
    print!("{:<32}", "padrao-ouro + CENTROS");
    for g in [60.0_f64, 90.0, 120.0, 150.0] {
        let poses = ora_poses(g.to_radians());
        let (d, seg) = ora_reguas_de(&malha, &|p| {
            // O centro do vértice mais próximo — a interpolação do centro é a wave seguinte.
            let (mut perto, mut pd) = (0usize, f64::INFINITY);
            for (i, q) in m.rest.iter().enumerate() {
                let dd = (q[0] - p[0]).hypot(q[1] - p[1]);
                if dd < pd {
                    (perto, pd) = (i, dd);
                }
            }
            let ws = interpola(&m, &w.por_vertice, p).unwrap_or_else(|| ora_bump(p, l));
            pos_cor(p, &ws, &poses, cen[perto])
        });
        print!("  {d:>5.2}% /{seg:>6.1}°");
    }
    println!();
}
