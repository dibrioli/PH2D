//! **SONDA — uma moldura de altura FIXA cujos filhos não cabem: eles TRANSBORDAM, ou encolhem?**
//!
//! Não é gate: é uma pergunta ao motor antes de desenhar a rolagem (o item 3 do estudo dos
//! contêineres). Ela decide o desenho inteiro, e as duas respostas levam a lugares opostos:
//!
//! - **transbordam** ⇒ o conteúdo excedente já está resolvido, o `clip` da moldura já o recorta, e
//!   rolar é deslocar os filhos por um número — o `world_target` da fatia é a linha inteira;
//! - **encolhem** ⇒ não há excedente nenhum a rolar, e a rolagem exigiria primeiro dizer ao motor
//!   *"não aperte estes filhos"* — um controlo novo antes da feature.
//!
//! ⚠️ Ela pergunta pela **porta do produto** ([`crate::solve`]), e não ao `taffy` cru: o que decide
//! é o que o NOSSO motor faz com a árvore que a NOSSA fatia monta (a lição do doc 28 §5.40 do
//! Painter — *quando o número vira decisão de produto, ele TEM de sair da porta do produto*).
//!
//! Rodar:
//! ```text
//! cargo test -p ph2d-vec-layout --release overflow_probe -- --ignored --nocapture
//! ```

use crate::{Dir, FrameStyle, ItemStyle, Len, Node, solve};

/// Uma coluna de altura `h` com `n` filhos de altura `kid`, sem vão nem recuo.
fn column(h: Len, n: usize, kid: f64, max_h: Option<f64>) -> Vec<Node> {
    let mut v = vec![Node {
        parent: None,
        frame: Some(FrameStyle {
            dir: Dir::Column,
            ..Default::default()
        }),
        item: ItemStyle::default(),
        size: [Len::Fixed(50.0), h],
        min: [None; 2],
        max: [None, max_h],
    }];
    v.extend((0..n).map(|_| Node {
        parent: Some(0),
        size: [Len::Fixed(50.0), Len::Fixed(kid)],
        ..Default::default()
    }));
    v
}

/// A extensão vertical do CONTEÚDO — o que a moldura precisaria de medir para tudo caber.
fn content_h(solved: &[[f64; 4]]) -> f64 {
    solved[1..]
        .iter()
        .fold(0.0_f64, |acc, s| acc.max(s[1] + s[3]))
}

/// **A pergunta.** 5 filhos de 40 numa moldura de 100: o conteúdo mede 200.
#[test]
#[ignore = "sonda de estudo, nao gate"]
fn overflow_probe_children_that_do_not_fit_a_fixed_frame() {
    let solved = solve(&column(Len::Fixed(100.0), 5, 40.0, None)).expect("resolve");
    println!("raiz  = {:?}", solved[0]);
    for (i, s) in solved[1..].iter().enumerate() {
        println!("kid {i} = {s:?}");
    }
    println!(
        "ALTURA DA MOLDURA {:.1} · CONTEUDO {:.1} · excedente {:.1}",
        solved[0][3],
        content_h(&solved),
        content_h(&solved) - solved[0][3]
    );
}

/// **O par que o estudo nomeia: `Hug` + `Max`** — cresce com o conteúdo *até* um teto. É onde a
/// rolagem começa a fazer falta, e a sonda mede se o excedente é derivável ali também.
#[test]
#[ignore = "sonda de estudo, nao gate"]
fn overflow_probe_hug_with_a_ceiling() {
    let solved = solve(&column(Len::Hug, 5, 40.0, Some(100.0))).expect("resolve");
    println!("raiz  = {:?}", solved[0]);
    for (i, s) in solved[1..].iter().enumerate() {
        println!("kid {i} = {s:?}");
    }
    println!(
        "ALTURA DA MOLDURA {:.1} · CONTEUDO {:.1} · excedente {:.1}",
        solved[0][3],
        content_h(&solved),
        content_h(&solved) - solved[0][3]
    );
}

/// **O CONTROLE:** os mesmos cinco filhos numa moldura que os comporta. Sem ele, *"o conteúdo mede
/// 200"* não distingue *transbordou* de *o motor sempre reporta a soma*.
#[test]
#[ignore = "sonda de estudo, nao gate"]
fn overflow_probe_control_a_frame_that_fits() {
    let solved = solve(&column(Len::Fixed(400.0), 5, 40.0, None)).expect("resolve");
    println!("raiz  = {:?}", solved[0]);
    println!(
        "ALTURA DA MOLDURA {:.1} · CONTEUDO {:.1} · excedente {:.1}",
        solved[0][3],
        content_h(&solved),
        content_h(&solved) - solved[0][3]
    );
}
