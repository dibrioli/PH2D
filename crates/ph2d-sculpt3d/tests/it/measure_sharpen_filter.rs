//! **AS DUAS LEIS DA W9c JÁ SÃO EXPRIMÍVEIS?** — a pergunta que a §8 do plano
//! manda fazer ANTES de escrever um kernel novo, agora sobre os dois últimos
//! tipos de filtro: o `SHARPEN` e o `ENHANCE_DETAILS` da referência.
//!
//! ⚠️ **A tabela do plano trata os dois como um item só (*"Sharpen · Enhance
//! Details"*, custo 2), e a leitura da referência os separa:** eles não são
//! variações um do outro, são leis de complexidade **muito** diferente.
//!
//! | | o que a referência faz | passes |
//! |---|---|---|
//! | `ENHANCE_DETAILS` | `t = detail_directions × −strength`, onde `detail_directions` é o deslocamento laplaciano da pose congelada | **1** |
//! | `SHARPEN` | um `sharpen_factor` por vértice (curvatura normalizada, curvada por `1−(1−f)²` e **alisada N vezes**), e depois um gather cujos dois termos são pesados por `f²` e `(1−f)` | **2** |
//!
//! ⚠️ **E o `ENHANCE_DETAILS` é, ALGEBRICAMENTE, o nosso `Smooth` com o sinal
//! trocado** — o que esta sonda existe para confirmar com um número em vez de
//! com uma conta no papel. A W9a já tinha escrito a metade interna disto no
//! `stroke_target_ring.rs` (*"`sharpen(w)` **é** `smooth(−w)`, e num arrasto o
//! sinal já existe"*), mas essa frase é sobre **duas funções nossas**; a
//! afirmação que a W9c precisa é outra — *a lei da REFERÊNCIA coincide com a
//! nossa* —, e uma não implica a outra.
//!
//! ⚠️ **A diferença que sobra NÃO é de lei, é de TETO:** o nosso `Smooth` clampa
//! em `(−1, 1)` porque a referência clampa o `SMOOTH` dela ali
//! (`clamp_factors(factors, -1.0f, 1.0f)`), e o `ENHANCE_DETAILS` dela **não
//! passa pelo `clamp_factors`**. A sonda mede o que se perde no teto: em `−1` a
//! lei reflete a média através do próprio vértice, e além disso ela continua a
//! afastar.
//!
//! Ela **imprime e não afirma**. O `SHARPEN` fica de fora desta sonda de
//! propósito: ele não tem candidato a composição na árvore, então não há o que
//! medir — há o que construir.
//!
//! # O VEREDITO que ela produziu (2026-08-18)
//!
//! **A lei já era exprimível; o TETO não.** Desvio `1,2e-7` a `2,4e-7` (um a
//! dois ULP de `f32`) em toda a faixa `0,10..1,00`, com o CONTROLE em força zero
//! a dar `0,000e0` — e o resíduo é de **EXPRESSÃO**, não de modelo (o
//! `target_sharpen` escreve a forma da referência, o `target_smooth` a mesma lei
//! por outra conta). Nas forças `1,5 / 2,0 / 3,0` o nosso `Smooth` fica preso em
//! **0,072617** enquanto a referência alcança **0,108926 / 0,145235 /
//! 0,217852**.
//!
//! ⇒ A [`FilterKind::EnhanceDetails`] nasceu **fina** — o kernel é o
//! [`SculptStroke::target_sharpen`], que já existia como o do [`Verb::Sharpen`],
//! e o conteúdo INTEIRO dela é a `range()` sem clamp. Os dois gates que a
//! defendem estão no `stroke_filter_tests.rs`
//! (`..._is_the_smooth_filter_dragged_backwards` e `..._has_no_ceiling_...`).
//!
//! Rodar: `cargo test -p ph2d-sculpt3d --test measure_sharpen_filter --release
//! -- --ignored --nocapture`

use ph2d_mesh::{Mesh, shapes::uv_sphere_noisy};
use ph2d_sculpt3d::{Brush, FilterKind, SculptStroke, Verb};

/// Uma malha com DETALHE, porque as duas leis medem o laplaciano e sobre uma
/// esfera lisa ele é quase o mesmo em todo vértice.
///
/// ⚠️ **A fixture TEM de conter o fenómeno:** numa esfera perfeita o
/// deslocamento laplaciano é uniforme e minúsculo, então *realçar detalhe* e
/// *não fazer nada* ficam indistinguíveis. O `uv_sphere_noisy` é a fixture da
/// casa para isto — é literalmente o que estas duas leis existem para tratar.
fn wrinkled_sphere() -> Mesh {
    uv_sphere_noisy(24, 36, 1.0, 0.04)
}

/// A lei da referência, `calc_enhance_details_filter` + `apply_translations`:
/// `t = detail_directions × −strength`, com `detail_directions` = o
/// deslocamento que um passo de smooth daria sobre a pose CONGELADA.
///
/// ⚠️ **Escrita à mão de propósito** — chamar a nossa seria o oráculo-espelho:
/// ela devolveria a nossa resposta com outro nome, e o gate ficaria verde por
/// construção.
fn reference_enhance_details(pre: &[[f32; 3]], mesh: &Mesh, strength: f32) -> Vec<[f32; 3]> {
    (0..pre.len())
        .map(|i| {
            let p = pre[i];
            let avg = neighbour_average(pre, mesh, i as u32);
            [
                p[0] - strength * (avg[0] - p[0]),
                p[1] - strength * (avg[1] - p[1]),
                p[2] - strength * (avg[2] - p[2]),
            ]
        })
        .collect()
}

/// A média do anel de vizinhos, sobre a pose PASSADA (nunca a viva).
fn neighbour_average(pos: &[[f32; 3]], mesh: &Mesh, v: u32) -> [f32; 3] {
    let mut sum = [0.0f64; 3];
    let mut n = 0u32;
    for &nb in mesh.adjacency().vert_verts.neighbours(v as usize) {
        let q = pos[nb as usize];
        sum[0] += f64::from(q[0]);
        sum[1] += f64::from(q[1]);
        sum[2] += f64::from(q[2]);
        n += 1;
    }
    if n == 0 {
        return pos[v as usize];
    }
    let inv = 1.0 / f64::from(n);
    [
        (sum[0] * inv) as f32,
        (sum[1] * inv) as f32,
        (sum[2] * inv) as f32,
    ]
}

/// O que o NOSSO filtro produz: `FilterKind::Smooth` com a força NEGATIVA, que
/// é o que um arrasto para a esquerda entrega.
fn ours_negative_smooth(mesh: &Mesh, strength: f32) -> Vec<[f32; 3]> {
    let mut m = mesh.clone();
    let mut s = SculptStroke::default();
    let brush = Brush {
        verb: Verb::Smooth,
        ..Brush::default()
    };
    s.filter_begin(&m);
    s.filter(&mut m, &brush, FilterKind::Smooth, -strength);
    m.positions().to_vec()
}

fn max_dev(a: &[[f32; 3]], b: &[[f32; 3]]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(p, q)| {
            let d = [p[0] - q[0], p[1] - q[1], p[2] - q[2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        })
        .fold(0.0f32, f32::max)
}

/// Quanto a malha andou — a régua contra a qual um desvio é grande ou pequeno.
fn excursion(pre: &[[f32; 3]], post: &[[f32; 3]]) -> f32 {
    max_dev(pre, post)
}

#[test]
#[ignore = "sonda: imprime, não afirma"]
fn measure_enhance_details_against_negative_smooth() {
    let mesh = wrinkled_sphere();
    let pre = mesh.positions().to_vec();

    println!("\n=== ENHANCE DETAILS: a referência contra o nosso Smooth NEGATIVO ===");
    println!(
        "  malha: {} vértices, com ruga de alta frequência",
        pre.len()
    );
    println!("\n  força | excursão | desvio máx | desvio/excursão");
    for s in [0.1f32, 0.25, 0.5, 0.8, 1.0] {
        let r = reference_enhance_details(&pre, &mesh, s);
        let o = ours_negative_smooth(&mesh, s);
        let exc = excursion(&pre, &o);
        let dev = max_dev(&r, &o);
        let rel = if exc > 0.0 { dev / exc } else { 0.0 };
        println!("  {s:>5.2} | {exc:>8.6} | {dev:>10.3e} | {rel:>15.3e}");
    }

    println!(
        "\n  ⚠️ CONTROLE: força ZERO, onde as duas TÊM de coincidir (se esta linha\n   \
         não for ~0, a sonda está medindo outra coisa)"
    );
    let r = reference_enhance_details(&pre, &mesh, 0.0);
    let o = ours_negative_smooth(&mesh, 0.0);
    println!("        0,00 | {:>8.6} | {:>10.3e} |", 0.0, max_dev(&r, &o));

    println!("\n=== O TETO: o que o clamp de (−1, 1) do nosso Smooth deixa de fora ===");
    println!("  (a referência NÃO clampa o ENHANCE_DETAILS -- ver `clamp_factors`)");
    println!("\n  força pedida | excursão nossa | excursão da referência");
    for s in [1.0f32, 1.5, 2.0, 3.0] {
        let r = reference_enhance_details(&pre, &mesh, s);
        let o = ours_negative_smooth(&mesh, s);
        println!(
            "  {s:>12.2} | {:>14.6} | {:>22.6}",
            excursion(&pre, &o),
            excursion(&pre, &r)
        );
    }
}
