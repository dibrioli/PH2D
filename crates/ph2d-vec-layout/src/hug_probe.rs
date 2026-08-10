//! **SONDA — o `hug` do Figma sai de graça, ou precisa de uma measure function?**
//!
//! Não é gate: é uma pergunta ao motor, para um estudo (`docs/Vector Module/Estudos/`). O
//! cabeçalho da crate afirma *"não há measure function"* e conclui daí que o tamanho de um nó é
//! sempre o que ele traz. As duas frases não são a mesma: uma measure function é precisa para uma
//! **FOLHA** cujo tamanho depende do espaço oferecido (texto que reflui); um **CONTENTOR** que se
//! ajusta ao conteúdo é intrinsic sizing puro, e o flexbox resolve-o sem perguntar nada a ninguém.
//!
//! Rodar:
//! ```text
//! cargo test -p ph2d-vec-layout --release hug_probe -- --ignored --nocapture
//! ```

use taffy::prelude::*;
use taffy::{Rect as TaffyRect, Size as TaffySize};

/// Uma moldura de tamanho **AUTO** com três filhos de 10, vão 4, recuo 2 — a árvore que o
/// `solve()` monta hoje, com a única diferença de a raiz não declarar largura.
///
/// A aritmética que o resultado tem de obedecer: `2 + 10 + 4 + 10 + 4 + 10 + 2 = 42`.
#[test]
#[ignore = "sonda de estudo, nao gate"]
fn hug_probe_a_frame_sized_auto_wraps_its_children() {
    let mut tree: TaffyTree<()> = TaffyTree::new();
    tree.disable_rounding();

    let kids: Vec<NodeId> = (0..3)
        .map(|_| {
            tree.new_leaf(Style {
                size: TaffySize {
                    width: length(10.0),
                    height: length(6.0),
                },
                ..Default::default()
            })
            .expect("folha")
        })
        .collect();

    let root = tree
        .new_with_children(
            Style {
                // ⚠️ A única mudança contra o que a crate emite hoje: `auto` em vez de `length`.
                size: TaffySize {
                    width: Dimension::AUTO,
                    height: Dimension::AUTO,
                },
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                gap: TaffySize {
                    width: length(4.0),
                    height: length(0.0),
                },
                padding: TaffyRect {
                    top: length(2.0),
                    right: length(2.0),
                    bottom: length(2.0),
                    left: length(2.0),
                },
                ..Default::default()
            },
            &kids,
        )
        .expect("moldura");

    // Espaço disponível INDEFINIDO: é o que "abraça o conteúdo" significa — ninguém oferece
    // largura, o nó decide-a a partir do que tem dentro.
    tree.compute_layout(
        root,
        TaffySize {
            width: AvailableSpace::MaxContent,
            height: AvailableSpace::MaxContent,
        },
    )
    .expect("resolve");

    let r = tree.layout(root).expect("layout");
    println!(
        "[hug] moldura AUTO com 3x10, gap 4, pad 2 -> {:.3} x {:.3} (esperado 42,000 x 10,000)",
        r.size.width, r.size.height
    );
    assert!(
        (r.size.width - 42.0).abs() < 1e-3,
        "a moldura devia abracar 42 e mede {:.3}",
        r.size.width
    );
    assert!(
        (r.size.height - 10.0).abs() < 1e-3,
        "a altura devia abracar 10 (6 + 2 + 2) e mede {:.3}",
        r.size.height
    );
}

/// E o caso que decide se `hug` serve para UI de verdade: **hug num eixo, fixo no outro** — a
/// barra de ferramentas que ocupa a largura toda e tem a altura do conteúdo.
#[test]
#[ignore = "sonda de estudo, nao gate"]
fn hug_probe_one_axis_hugs_while_the_other_is_fixed() {
    let mut tree: TaffyTree<()> = TaffyTree::new();
    tree.disable_rounding();

    let kid = tree
        .new_leaf(Style {
            size: TaffySize {
                width: length(10.0),
                height: length(6.0),
            },
            ..Default::default()
        })
        .expect("folha");

    let root = tree
        .new_with_children(
            Style {
                size: TaffySize {
                    width: length(100.0),
                    height: Dimension::AUTO,
                },
                display: Display::Flex,
                padding: TaffyRect {
                    top: length(3.0),
                    right: length(0.0),
                    bottom: length(3.0),
                    left: length(0.0),
                },
                ..Default::default()
            },
            &[kid],
        )
        .expect("moldura");

    tree.compute_layout(
        root,
        TaffySize {
            width: AvailableSpace::Definite(100.0),
            height: AvailableSpace::MaxContent,
        },
    )
    .expect("resolve");

    let r = tree.layout(root).expect("layout");
    println!(
        "[hug] largura FIXA 100 + altura AUTO sobre filho de 6, pad 3+3 -> {:.3} x {:.3} \
         (esperado 100,000 x 12,000)",
        r.size.width, r.size.height
    );
    assert!(
        (r.size.width - 100.0).abs() < 1e-3 && (r.size.height - 12.0).abs() < 1e-3,
        "os dois eixos sao independentes: {:.3} x {:.3}",
        r.size.width,
        r.size.height
    );
}

/// **O contrapeso, e é ele que impede a conclusão fácil:** `min`/`max` do Figma existem no motor
/// como `min_size`/`max_size`. Se eles funcionarem sem nada novo, a wave do sizing é a mesma para
/// os três controles que o Figma oferece (Fixed · Hug · Fill) mais os dois limites.
#[test]
#[ignore = "sonda de estudo, nao gate"]
fn hug_probe_min_and_max_clamp_the_hug() {
    let mut tree: TaffyTree<()> = TaffyTree::new();
    tree.disable_rounding();

    let kid = tree
        .new_leaf(Style {
            size: TaffySize {
                width: length(10.0),
                height: length(6.0),
            },
            ..Default::default()
        })
        .expect("folha");

    let root = tree
        .new_with_children(
            Style {
                size: TaffySize {
                    width: Dimension::AUTO,
                    height: Dimension::AUTO,
                },
                min_size: TaffySize {
                    width: length(30.0),
                    height: auto(),
                },
                display: Display::Flex,
                ..Default::default()
            },
            &[kid],
        )
        .expect("moldura");

    tree.compute_layout(
        root,
        TaffySize {
            width: AvailableSpace::MaxContent,
            height: AvailableSpace::MaxContent,
        },
    )
    .expect("resolve");

    let r = tree.layout(root).expect("layout");
    println!(
        "[hug] AUTO sobre filho de 10 com min 30 -> {:.3} (esperado 30,000)",
        r.size.width
    );
    assert!(
        (r.size.width - 30.0).abs() < 1e-3,
        "o minimo devia segurar em 30 e mede {:.3}",
        r.size.width
    );
}
