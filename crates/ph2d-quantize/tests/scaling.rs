//! **A SONDA DE ESCALA** — como o quantizador cresce com o tamanho do layout.
//!
//! ⚠️ **Ela existe porque o custo do fluxo NÃO é função só do número de arestas.**
//! Caminhos-mais-curtos sucessivos é pseudo-polinomial: o número de aumentos
//! cresce com o **valor** do fluxo, e o valor do fluxo cresce com os alvos. Dobrar
//! a densidade pedida pode custar mais que dobrar o número de patches, e nenhuma
//! contagem de arestas mostraria isso.
//!
//! O layout usado é a **grelha toroidal**: `n × m` patches de 4 lados, `2·n·m`
//! arcos, cada arco partilhado por exatamente dois patches. É o gerador mais
//! simples que produz layouts fechados de tamanho arbitrário.
//!
//! ```text
//! cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-quantize --release \
//!     --test scaling -- --ignored --nocapture
//! ```

use std::time::Instant;

use ph2d_quantize::{ArcSpec, Budget, Layout, PatchSpec, quantize_within};

/// ⚠️ **O teto de resoluções DESTA SONDA**, apertado de propósito: ela existe
/// para desenhar a curva, não para resolver o caso difícil. Uma linha
/// `esgotado` é exatamente a informação que se quer.
const SOLVE_CAP: usize = 256;

/// `n × m` patches de 4 lados sobre um toro. `spread` mistura alvos diferentes
/// arco a arco — ⚠️ é o eixo que separa **tamanho** de **heterogeneidade**.
fn torus_grid(n: usize, m: usize, target: f64, spread: f64) -> Layout {
    // Arcos verticais `v[i][j]` (entre `(i,j)` e `(i+1,j)`) vêm primeiro; depois
    // os horizontais `h[i][j]` (entre `(i,j)` e `(i,j+1)`).
    let vid = |i: usize, j: usize| u32::try_from((i % n) * m + (j % m)).unwrap_or(0);
    let hid = |i: usize, j: usize| u32::try_from(n * m + (i % n) * m + (j % m)).unwrap_or(0);
    let patches = (0..n)
        .flat_map(|i| {
            (0..m).map(move |j| PatchSpec {
                // Lados 0/2 opostos (direita/esquerda) e 1/3 (cima/baixo).
                sides: vec![
                    vec![vid(i, j)],
                    vec![hid(i, j)],
                    vec![vid(i + n - 1, j)],
                    vec![hid(i, j + m - 1)],
                ],
            })
        })
        .collect();
    // Um gerador determinístico de dispersão — nada de `rand` numa sonda.
    let mut seed = 0x1234_5678u64;
    let arcs = (0..2 * n * m)
        .map(|_| {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            let u = ((seed >> 11) as f64) / ((1u64 << 53) as f64);
            ArcSpec::new(target * (1.0 - spread + 2.0 * spread * u))
        })
        .collect();
    Layout::new(arcs, patches).expect("a grelha toroidal e' fechada")
}

#[test]
#[ignore = "sonda de escala -- imprime uma tabela, nao afirma um limite"]
fn how_the_quantizer_grows() {
    println!(
        "{:>7} {:>8} {:>8} {:>8} {:>7} {:>7} {:>9} {:>10}",
        "patches", "arcos", "alvo", "dispersao", "fluxos", "aumentos", "custo", "ms"
    );
    for (n, m, target, spread) in [
        (2usize, 2usize, 8.0f64, 0.0f64),
        (4, 4, 8.0, 0.0),
        (8, 8, 8.0, 0.0),
        (16, 16, 8.0, 0.0),
        (32, 32, 8.0, 0.0),
        (64, 64, 8.0, 0.0),
        // ⚠️ A MESMA grelha com alvos maiores: isola o efeito do VALOR do fluxo.
        // Sem a partida a quente das arestas de leque este eixo dominava; com
        // ela, o relógio não se move (medido 2026-08-20).
        (16, 16, 32.0, 0.0),
        (16, 16, 128.0, 0.0),
        (16, 16, 512.0, 0.0),
        // ⭐ O eixo que interessa: a MESMA grelha com alvos DISPERSOS. Se o
        // relógio saltar aqui e não acima, o que custa é a heterogeneidade.
        (16, 16, 32.0, 0.5),
        (16, 16, 32.0, 0.9),
        (24, 24, 32.0, 0.9),
        (32, 32, 32.0, 0.9),
    ] {
        let layout = torus_grid(n, m, target, spread);
        let t = Instant::now();
        // ⚠️ Orçamento de busca ZERO (mede só o mergulho, que é obrigatório) e um
        // teto de resoluções apertado — uma sonda que fica a moer não é uma sonda.
        // A linha `esgotado` é resultado, não falha.
        match quantize_within(&layout, Budget::new(0, SOLVE_CAP)) {
            Ok((_, r)) => println!(
                "{:>7} {:>8} {:>8.0} {:>8.2} {:>7} {:>7} {:>9.2} {:>10.0}",
                n * m,
                layout.arcs().len(),
                target,
                spread,
                r.solves,
                r.augmentations,
                r.cost,
                t.elapsed().as_secs_f64() * 1000.0
            ),
            Err(e) => println!(
                "{:>7} {:>8} {:>8.0} {:>8.2}   esgotado ({e:?}) em {:.0} ms",
                n * m,
                layout.arcs().len(),
                target,
                spread,
                t.elapsed().as_secs_f64() * 1000.0
            ),
        }
    }
}
