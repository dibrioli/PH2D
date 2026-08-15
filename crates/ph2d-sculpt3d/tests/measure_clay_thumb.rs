//! **O QUE O POLEGAR FAZ** — a sonda que mede a inclinação do corte contra o
//! número de dabs, e que decide as barras dos gates da wave.
//!
//! ⚠️ **Ela dirige `SculptStroke::dab`, a porta do artista.** A §7.11 já pagou
//! duas vezes por medir peça isolada, e o que decide um verbo é o que sai na
//! malha — não o que a constante promete.
//!
//! ⚠️ **E o oráculo é a GEOMETRIA, nunca a constante:** o ângulo é lido de volta
//! dos vértices que o traço achatou, por ajuste de plano. Uma sonda que
//! imprimisse `dabs × 0,8` estaria a citar o código sob teste.
//!
//! Rodar: `cargo test -p ph2d-sculpt3d --release --test measure_clay_thumb -- --ignored --nocapture`

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

fn sphere() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(96, 144, 1.0)
}

/// O polo `+z`; o olho olha para `−z`.
const TIP: [f32; 3] = [0.0, 0.0, 1.0];
const EYE: [f32; 3] = [0.0, 0.0, -1.0];
const R: f32 = 0.35;

/// Quanto o traço anda POR DAB, em raios — o espaçamento do produto é da mesma
/// ordem, e é ele que decide quantos dabs cabem num traço real.
const STEP: f32 = 0.06;

/// Um traço de `dabs` eventos que anda ao longo de `+x`, terminando no `TIP`.
///
/// ⚠️ **Ele TERMINA no polo**, e não começa lá: o que se mede é o corte sob o
/// ÚLTIMO dab, e é ele que carrega a inclinação acumulada.
fn walk(verb: Verb, dabs: usize, strength: f32) -> Mesh {
    let mut mesh = sphere();
    let b = Brush {
        verb,
        radius: R,
        strength,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for k in 0..dabs {
        let back = (dabs - 1 - k) as f32 * STEP * R;
        let d = Dab::pulling([TIP[0] - back, TIP[1], TIP[2]], R, EYE, [0.0; 3]);
        s.dab(&mut mesh, &b, &d, Symmetry::default());
    }
    mesh
}

/// A normal do plano que MELHOR descreve os vértices dentro de `frac · R` do
/// polo — ajuste por mínimos quadrados (o menor autovetor da covariância, por
/// iteração inversa barata sobre a matriz 3×3).
///
/// ⚠️ **Mínimos quadrados de VERDADE, e não a média das normais** — o estimador
/// do produto (`fit_plane`) é a média ponderada, e usá-lo aqui faria a sonda
/// citar o código que ela mede.
fn fitted_normal(mesh: &Mesh, frac: f32) -> [f64; 3] {
    let r2 = (frac * R) * (frac * R);
    let pts: Vec<[f64; 3]> = mesh
        .positions()
        .iter()
        // ⚠️ **O `p[2] > 0` é load-bearing, e sem ele o CONTROLE reprovava:** o
        // filtro em `xy` é um CILINDRO, e um cilindro através de uma esfera
        // apanha as DUAS calotas. Com os dois polos dentro, a maior dispersão da
        // nuvem é em `z`, o menor autovetor cai no plano `xy`, e o ajuste
        // devolvia `±90°` para tudo — inclusive para um Flatten, que tem de
        // medir zero. *Uma sonda cujo controle falha está a medir outra coisa.*
        .filter(|p| {
            let d = [p[0] - TIP[0], p[1] - TIP[1]];
            p[2] > 0.0 && d[0] * d[0] + d[1] * d[1] <= r2
        })
        .map(|p| [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])])
        .collect();
    assert!(pts.len() >= 8, "poucos pontos no ajuste: {}", pts.len());
    let n = pts.len() as f64;
    let mut c = [0.0; 3];
    for p in &pts {
        for i in 0..3 {
            c[i] += p[i] / n;
        }
    }
    let mut m = [[0.0f64; 3]; 3];
    for p in &pts {
        let d = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
        for i in 0..3 {
            for j in 0..3 {
                m[i][j] += d[i] * d[j];
            }
        }
    }
    // O menor autovetor: itera `v ← (tr·I − M) v`, que troca o menor pelo maior.
    let tr = m[0][0] + m[1][1] + m[2][2];
    let mut v = [0.0, 0.0, 1.0];
    for _ in 0..200 {
        let mut w = [0.0f64; 3];
        for i in 0..3 {
            w[i] = tr * v[i] - (m[i][0] * v[0] + m[i][1] * v[1] + m[i][2] * v[2]);
        }
        let len = (w[0] * w[0] + w[1] * w[1] + w[2] * w[2]).sqrt();
        if len < 1e-18 {
            break;
        }
        v = [w[0] / len, w[1] / len, w[2] / len];
    }
    if v[2] < 0.0 { [-v[0], -v[1], -v[2]] } else { v }
}

/// O ângulo, em graus, entre a normal ajustada e o eixo `+z` — e o SINAL diz
/// para que lado o plano tomba (positivo = a normal cai para `+x`, o sentido do
/// traço).
fn tilt_deg(mesh: &Mesh, frac: f32) -> f64 {
    let n = fitted_normal(mesh, frac);
    n[0].atan2(n[2]).to_degrees()
}

#[test]
#[ignore = "sonda"]
fn how_the_thumb_tilts_along_the_stroke() {
    println!("\n== o corte sob o ULTIMO dab, medido nos vertices ==");
    println!(
        "{:>6} | {:>12} | {:>12} | {:>10}",
        "dabs", "inclinacao", "plano (lei)", "desloc max"
    );
    let rest = sphere().positions().to_vec();
    for dabs in [2usize, 5, 10, 20, 40, 76, 120, 200] {
        let mesh = walk(Verb::ClayThumb, dabs, 1.0);
        let t = tilt_deg(&mesh, 0.5);
        // O que a LEI prevê: `(dabs − 1) · passo`, com teto — o primeiro dab não
        // avança (ele é o que não tem direção).
        let want = (((dabs - 1) as f32) * 0.8).min(60.0);
        let shift = rest
            .iter()
            .zip(mesh.positions())
            .map(|(a, b)| {
                let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
            })
            .fold(0.0f32, f32::max);
        println!("{dabs:>6} | {t:>11.2}° | {want:>11.2}° | {shift:>10.4}");
    }
    println!(
        "\n(⚠️ as DUAS colunas medem grandezas DIFERENTES, e a segunda nao e'\n \
         uma previsao da primeira: 'plano (lei)' e' quanto o PLANO de UM dab\n \
         esta' inclinado contra a normal de area DELE; 'inclinacao' e' o corte\n \
         que a SEQUENCIA deixou, sobre uma esfera que ja' curva sozinha. Elas\n \
         coincidem por volta dos 20 dabs e divergem depois — o corte passa a\n \
         lei porque cada dab inclina contra o que os anteriores deixaram.)"
    );
}

#[test]
#[ignore = "sonda"]
fn what_the_thumb_leaves_against_the_flatten() {
    println!("\n== polegar contra Flatten, mesmo traco ==");
    println!(
        "{:>12} | {:>12} | {:>12} | {:>12}",
        "verbo", "inclinacao", "desloc max", "vol assinado"
    );
    let rest = sphere().positions().to_vec();
    for verb in [Verb::Flatten, Verb::ClayThumb] {
        let mesh = walk(verb, 40, 1.0);
        let t = tilt_deg(&mesh, 0.5);
        let (mut shift, mut vol) = (0.0f32, 0.0f64);
        for (a, b) in rest.iter().zip(mesh.positions()) {
            let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            shift = shift.max((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
            // O sinal ao longo da normal da esfera de repouso.
            vol += f64::from(d[0] * a[0] + d[1] * a[1] + d[2] * a[2]);
        }
        println!(
            "{:>12} | {t:>11.2}° | {shift:>12.4} | {vol:>12.4}",
            verb.label()
        );
    }
}

#[test]
#[ignore = "sonda"]
fn the_tilt_per_unit_of_stroke_is_ours_not_the_references() {
    println!("\n== graus por RAIO percorrido — a grandeza que e' NOSSA ==");
    println!(
        "{:>10} | {:>14} | {:>16}",
        "passo/R", "graus/dab", "graus/raio"
    );
    for step in [0.03f32, 0.06, 0.12, 0.25] {
        // 20 dabs andando `step` cada: o comprimento é `19 · step · R`.
        let dabs = 20usize;
        let mut mesh = sphere();
        let b = Brush {
            verb: Verb::ClayThumb,
            radius: R,
            strength: 1.0,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        for k in 0..dabs {
            let back = (dabs - 1 - k) as f32 * step * R;
            let d = Dab::pulling([TIP[0] - back, TIP[1], TIP[2]], R, EYE, [0.0; 3]);
            s.dab(&mut mesh, &b, &d, Symmetry::default());
        }
        let travelled = (dabs - 1) as f32 * step;
        let per_dab = 0.8;
        println!(
            "{step:>10.2} | {per_dab:>13.2}° | {:>15.2}°",
            (dabs - 1) as f32 * per_dab / travelled
        );
    }
    println!(
        "\n(o `0,8°/dab` e' CITAVEL do `clay_thumb.cc`; a coluna da direita\n \
         depende do nosso espacamento e por isso e' NOSSA)"
    );
}
