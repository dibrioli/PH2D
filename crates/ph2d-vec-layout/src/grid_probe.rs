//! **SONDA — o que a GRADE de facto faz, e de que recurso ela é feita.**
//!
//! Não é gate: é uma pergunta ao motor, e o §0 do `CLAUDE.md` manda fazê-la ANTES de escrever
//! qualquer teto. Três perguntas, e as três decidem coisas diferentes:
//!
//! 1. **onde caem as células** — o oráculo aritmético que o gate vai usar;
//! 2. **o `Hug` numa grade** — colunas `1fr` precisam de uma largura para dividir, e `Hug` oferece
//!    `MaxContent`. Se `1fr` colapsar para zero ali, a moldura DESAPARECE, que é exactamente o modo
//!    de falha que o `HugWithoutFlow` existe para impedir noutro sítio;
//! 3. **quanto custa uma grade LARGA** — se o custo explodir com a contagem de colunas, há um teto
//!    a escrever com a medição ao lado; se for linear e barato, escrever um teto seria o palpite
//!    que a §0 proíbe.
//!
//! ⚠️ Tudo passa pela porta do PRODUTO ([`super::solve`]) e nunca por uma árvore `taffy` montada à
//! mão: uma sonda que re-implementa o laço fica cega à porta e continua a imprimir números depois
//! de o produto deixar de os pagar.
//!
//! Rodar:
//! ```text
//! cargo test -p ph2d-vec-layout --release grid_probe -- --ignored --nocapture
//! ```

use super::{Dir, FrameStyle, ItemStyle, Len, Node, solve};

/// Uma moldura em grade com `kids` filhos de 10×6.
fn scene(size: [Len; 2], columns: u16, kids: usize, gap: [f64; 2]) -> Vec<Node> {
    let mut out = vec![Node {
        parent: None,
        frame: Some(FrameStyle {
            dir: Dir::Grid { columns },
            gap,
            ..FrameStyle::default()
        }),
        size,
        ..Node::default()
    }];
    out.extend((0..kids).map(|_| Node {
        parent: Some(0),
        size: [Len::Fixed(10.0), Len::Fixed(6.0)],
        item: ItemStyle::default(),
        ..Node::default()
    }));
    out
}

/// **Onde caem as sete células de uma grade de três colunas.**
#[test]
#[ignore = "sonda de estudo, nao gate"]
fn grid_probe_where_the_cells_land() {
    // 3 colunas de 10 com vão 4 = 38 de largura; 3 linhas de 6 com vão 2 = 22 de altura.
    let out = solve(&scene(
        [Len::Fixed(38.0), Len::Fixed(22.0)],
        3,
        7,
        [4.0, 2.0],
    ))
    .expect("a grade resolve");
    println!("raiz  {:?}", out[0]);
    for (i, r) in out.iter().skip(1).enumerate() {
        println!(
            "filho {i}: x={:6.2} y={:6.2} w={:6.2} h={:6.2}",
            r[0], r[1], r[2], r[3]
        );
    }
}

/// **A grade que ABRAÇA o conteúdo** — a pergunta 2.
#[test]
#[ignore = "sonda de estudo, nao gate"]
fn grid_probe_does_hug_survive_fr_tracks() {
    for (name, size) in [
        ("Hug nos DOIS eixos", [Len::Hug, Len::Hug]),
        ("Hug so na largura", [Len::Hug, Len::Fixed(22.0)]),
        ("Hug so na altura", [Len::Fixed(38.0), Len::Hug]),
    ] {
        let out = solve(&scene(size, 3, 7, [4.0, 2.0])).expect("a grade resolve");
        println!("{name:20}: raiz {:?}  1o filho {:?}", out[0], out[1]);
    }
}

/// **De que recurso é feita uma grade LARGA** — a pergunta 3, e a que decide se há teto.
#[test]
#[ignore = "sonda de estudo, nao gate"]
fn grid_probe_what_a_wide_grid_costs() {
    println!("colunas  filhos     ms/solve");
    for columns in [3u16, 16, 64, 256, 1024, 4096, 16384, u16::MAX] {
        for kids in [12usize, usize::from(columns)] {
            let tree = scene(
                [Len::Fixed(4000.0), Len::Fixed(400.0)],
                columns,
                kids,
                [0.0, 0.0],
            );
            // ⚠️ Acima do teto medido a fatia é RECUSADA, e a sonda diz isso em vez de estourar:
            // uma sonda que ninguém consegue rodar até ao fim deixa de responder à pergunta.
            if solve(&tree).is_err() {
                println!("{columns:7}  {kids:6}  {:>11}", "RECUSADA");
                continue;
            }
            let mut ms: Vec<f64> = (0..7)
                .map(|_| {
                    let t = std::time::Instant::now();
                    let _ = solve(&tree).expect("resolve");
                    t.elapsed().as_secs_f64() * 1e3
                })
                .collect();
            ms.sort_by(f64::total_cmp);
            println!("{columns:7}  {kids:6}  {:11.4}", ms[3]);
        }
    }
}
