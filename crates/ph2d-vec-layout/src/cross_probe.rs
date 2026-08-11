//! **SONDA — o eixo TRANSVERSAL: onde as FAIXAS de um wrap sentam, e o que o `Stretch` faz.**
//!
//! Não é gate: são perguntas ao motor antes de mexer numa linha, e **duas delas derrubaram uma
//! afirmação minha** — ficam no repo por isso.
//!
//! 1. **`RowWrap` com folga** — o handoff regista, desde 2026-08-02, que *"numa moldura `Wrap` com
//!    folga o `taffy` distribui as faixas (a 2ª pousou em 54,5 em vez de 19)"*, e o item ficou
//!    aberto como *"`align_content` não é exposto"*. A wave da GRADE construiu metade da cura sem
//!    tocar nesta metade. **Confirmado e pior**: com `align = Start` a 2ª faixa pousava em **50**
//!    onde encostar pede 20 — e o `Center`/`End` posicionavam o filho dentro de uma faixa inchada.
//! 2. **`Align::Stretch`** — a wave da grade mediu que ele não estica uma folha e eu generalizei
//!    para *"chip morto"*. ⚠️ **A generalização era falsa** (§2b): sobre uma **moldura filha** que
//!    abraça o eixo transversal ele leva-a de **12,0 a 60,0**. O que ele alcança é o que é
//!    auto-dimensionado, e uma folha nunca é.
//! 3. **`align_content` alcança uma faixa única?** ⚠️ Eu escrevi *"a spec diz que não"* e ia
//!    shipar isso; medido, depende da FLAG: em `NoWrap` é inerte, em `Wrap` **ele posiciona e
//!    vence o `align_items`**. Uma mutação foi quem levantou a dúvida, não a leitura.
//! 4. **O que esta wave move numa cena já aprovada** (`=69`) — em números, não em adjectivos.
//!
//! ⚠️ Todas perguntam pela **porta do produto** ([`crate::solve`]) — excepto a §3, que fala ao
//! `taffy` cru **de propósito**: ela pergunta o que a DEP faz com as duas propriedades em
//! desacordo, e a nossa porta nunca as põe em desacordo (é o que o espelho garante).
//!
//! Rodar:
//! ```text
//! cargo test -p ph2d-vec-layout --release cross_probe -- --ignored --nocapture
//! ```

use crate::{Align, Dir, FrameStyle, ItemStyle, Len, Node, solve};

/// Uma moldura `dir` de `w × h` com `n` filhos de `kid_w × kid_h`, vão `gap`, alinhamento `align`.
fn frame(
    dir: Dir,
    w: f64,
    h: f64,
    n: usize,
    kid: [Len; 2],
    gap: [f64; 2],
    align: Align,
) -> Vec<Node> {
    let mut v = vec![Node {
        parent: None,
        frame: Some(FrameStyle {
            dir,
            gap,
            align,
            ..Default::default()
        }),
        item: ItemStyle::default(),
        size: [Len::Fixed(w), Len::Fixed(h)],
        min: [None; 2],
        max: [None; 2],
    }];
    v.extend((0..n).map(|_| Node {
        parent: Some(0),
        size: kid,
        ..Default::default()
    }));
    v
}

/// O topo de cada filho, na ordem em que foram descritos.
fn tops(solved: &[[f64; 4]]) -> Vec<f64> {
    solved[1..].iter().map(|s| s[1]).collect()
}

/// A altura de cada filho.
fn heights(solved: &[[f64; 4]]) -> Vec<f64> {
    solved[1..].iter().map(|s| s[3]).collect()
}

/// **Pergunta 1 — onde as faixas de um wrap com folga sentam, por `align`.**
///
/// Seis filhos de 30 de largura numa moldura de 100: cabem TRÊS por faixa (`30·3 = 90 ≤ 100`, e o
/// quarto daria 120). Duas faixas de 20 de altura numa moldura de 100 ⇒ **60 de folga**. Se o
/// motor as encostasse no topo, os topos seriam `0` e `20`.
#[test]
#[ignore = "sonda: cargo test -p ph2d-vec-layout --release cross_probe -- --ignored --nocapture"]
fn where_the_wrap_bands_sit() {
    println!("\n=== 1. RowWrap 100x100, 6 filhos 30x20, DUAS faixas, 60 de folga ===");
    println!("   (encostado no topo daria topos [0,0,0, 20,20,20])");
    for align in [Align::Start, Align::Center, Align::End, Align::Stretch] {
        let nodes = frame(
            Dir::RowWrap,
            100.0,
            100.0,
            6,
            [Len::Fixed(30.0), Len::Fixed(20.0)],
            [0.0, 0.0],
            align,
        );
        let s = solve(&nodes).expect("resolve");
        println!(
            "   align={align:?}  topos={:?}  alturas={:?}",
            tops(&s)
                .iter()
                .map(|v| (v * 10.0).round() / 10.0)
                .collect::<Vec<_>>(),
            heights(&s)
                .iter()
                .map(|v| (v * 10.0).round() / 10.0)
                .collect::<Vec<_>>()
        );
    }
}

/// **O CONTROLE da pergunta 1** — a mesma cena sem folga nenhuma (moldura de 40 = duas faixas de
/// 20). Sem sobra não há o que distribuir, então as quatro respostas TÊM de coincidir; se
/// divergirem, o que a pergunta 1 mede não é distribuição de sobra.
#[test]
#[ignore = "sonda: cargo test -p ph2d-vec-layout --release cross_probe -- --ignored --nocapture"]
fn the_control_no_slack_no_difference() {
    println!("\n=== 1b. CONTROLE — RowWrap 100x40 (zero folga): as quatro TEM de coincidir ===");
    for align in [Align::Start, Align::Center, Align::End, Align::Stretch] {
        let nodes = frame(
            Dir::RowWrap,
            100.0,
            40.0,
            6,
            [Len::Fixed(30.0), Len::Fixed(20.0)],
            [0.0, 0.0],
            align,
        );
        let s = solve(&nodes).expect("resolve");
        println!("   align={align:?}  topos={:?}", tops(&s));
    }
}

/// **Pergunta 2 — o `Stretch` estica alguma coisa?**
///
/// Três fixtures no MESMO `Row`, mudando só o tamanho do filho no eixo transversal:
/// `Fixed(20)` (o default do documento) · `Hug` (o `auto` do CSS, que é a condição que o
/// flexbox exige para esticar) · e um filho `Hug` **com conteúdo**, porque um `Hug` vazio mede
/// zero e não distingue *"esticou"* de *"não tinha tamanho"*.
#[test]
#[ignore = "sonda: cargo test -p ph2d-vec-layout --release cross_probe -- --ignored --nocapture"]
fn what_stretch_stretches() {
    println!("\n=== 2. Row 100x60: o filho ESTICA no eixo transversal? ===");
    for (name, kid) in [
        ("Fixed(20)", [Len::Fixed(30.0), Len::Fixed(20.0)]),
        ("Hug", [Len::Fixed(30.0), Len::Hug]),
    ] {
        for align in [Align::Start, Align::Stretch] {
            let nodes = frame(Dir::Row, 100.0, 60.0, 2, kid, [0.0, 0.0], align);
            let s = solve(&nodes).expect("resolve");
            println!(
                "   filho={name:<10} align={align:?}  alturas={:?}  topos={:?}",
                heights(&s),
                tops(&s)
            );
        }
    }
    println!("   (se a altura nao mudar entre Start e Stretch, o chip nao move um pixel)");
}

/// **Pergunta 2b — o ÚNICO caso em que o flexbox PODE esticar.**
///
/// O `size_of` da fatia só devolve [`Len::Hug`] para um nó que **FLUI** (tem `VecLayout`), e o
/// motor recusa um `Hug` sem fluxo. Logo o único filho cujo tamanho transversal é `auto` é uma
/// **moldura filha** — e é aí, e só aí, que o `align: Stretch` tem o que esticar.
///
/// A moldura filha leva um neto de 12 de altura: sem ele o `Hug` mede zero e *"esticou"* e *"não
/// tinha tamanho"* ficam indistinguíveis.
#[test]
#[ignore = "sonda: cargo test -p ph2d-vec-layout --release cross_probe -- --ignored --nocapture"]
fn what_stretch_stretches_on_a_child_frame() {
    println!("\n=== 2b. Row 100x60 com uma MOLDURA filha de altura Hug (neto de 12) ===");
    for align in [Align::Start, Align::Stretch] {
        let nodes = vec![
            Node {
                parent: None,
                frame: Some(FrameStyle {
                    dir: Dir::Row,
                    align,
                    ..Default::default()
                }),
                item: ItemStyle::default(),
                size: [Len::Fixed(100.0), Len::Fixed(60.0)],
                min: [None; 2],
                max: [None; 2],
            },
            Node {
                parent: Some(0),
                frame: Some(FrameStyle {
                    dir: Dir::Row,
                    ..Default::default()
                }),
                item: ItemStyle::default(),
                size: [Len::Fixed(30.0), Len::Hug],
                min: [None; 2],
                max: [None; 2],
            },
            Node {
                parent: Some(1),
                size: [Len::Fixed(20.0), Len::Fixed(12.0)],
                ..Default::default()
            },
        ];
        let s = solve(&nodes).expect("resolve");
        println!(
            "   align={align:?}  moldura filha h={:.1}  (Hug do neto = 12,0)",
            s[1][3]
        );
    }
    println!("   (se a altura da moldura filha nao passar de 12, o Stretch nao alcanca NADA)");
}

/// **Pergunta 3 — `align_content` alcança um contentor de UMA faixa?**
///
/// A mutação que trocou `Center`/`End` **só no `align_content`** fez um gate de faixa única
/// falhar, o que contradiz *"a spec diz que ele não tem efeito em linha única"*. Esta sonda mede
/// as duas propriedades **em desacordo** para ver qual delas de facto posiciona.
#[test]
#[ignore = "sonda: cargo test -p ph2d-vec-layout --release cross_probe -- --ignored --nocapture"]
fn does_content_alignment_reach_a_single_band() {
    use taffy::prelude::*;
    println!("\n=== 3. Row 100x100, UM filho 30x20: align_items x align_content em DESACORDO ===");
    for (wrap, items, content, name) in [
        (
            FlexWrap::NoWrap,
            AlignItems::FLEX_START,
            None,
            "NoWrap items=Start  content=<default>",
        ),
        (
            FlexWrap::NoWrap,
            AlignItems::FLEX_START,
            Some(AlignContent::CENTER),
            "NoWrap items=Start  content=Center",
        ),
        (
            FlexWrap::NoWrap,
            AlignItems::FLEX_START,
            Some(AlignContent::FLEX_END),
            "NoWrap items=Start  content=End",
        ),
        (
            FlexWrap::NoWrap,
            AlignItems::CENTER,
            Some(AlignContent::FLEX_START),
            "NoWrap items=Center content=Start",
        ),
        (
            FlexWrap::Wrap,
            AlignItems::FLEX_START,
            None,
            "Wrap   items=Start  content=<default>",
        ),
        (
            FlexWrap::Wrap,
            AlignItems::FLEX_START,
            Some(AlignContent::FLEX_START),
            "Wrap   items=Start  content=Start",
        ),
        (
            FlexWrap::Wrap,
            AlignItems::FLEX_START,
            Some(AlignContent::CENTER),
            "Wrap   items=Start  content=Center",
        ),
        (
            FlexWrap::Wrap,
            AlignItems::FLEX_START,
            Some(AlignContent::FLEX_END),
            "Wrap   items=Start  content=End",
        ),
        (
            FlexWrap::Wrap,
            AlignItems::CENTER,
            Some(AlignContent::FLEX_START),
            "Wrap   items=Center content=Start",
        ),
    ] {
        let mut t: TaffyTree<()> = TaffyTree::new();
        let kid = t
            .new_leaf(Style {
                size: Size {
                    width: length(30.0),
                    height: length(20.0),
                },
                ..Default::default()
            })
            .unwrap();
        let root = t
            .new_with_children(
                Style {
                    display: Display::Flex,
                    size: Size {
                        width: length(100.0),
                        height: length(100.0),
                    },
                    flex_wrap: wrap,
                    align_items: Some(items),
                    align_content: content,
                    ..Default::default()
                },
                &[kid],
            )
            .unwrap();
        t.compute_layout(root, Size::MAX_CONTENT).unwrap();
        let l = t.layout(kid).unwrap();
        println!(
            "   {name:<38} -> topo {:>5.1}  altura {:>5.1}",
            l.location.y, l.size.height
        );
    }
    println!("   (se o topo mudar com o content, ele ALCANCA a faixa unica)");
}

/// **SONDA — o que esta wave move na cena `=69` (a moldura de CONTROLE do smoke da grade).**
///
/// Ela é um `RowWrap` com folga, então é exactamente o caso que o espelho muda. O smoke foi
/// aprovado com o desenho ANTERIOR; esta sonda diz, em números, o que o artista vai ver diferente.
#[test]
#[ignore = "sonda: cargo test -p ph2d-vec-layout --release cross_probe -- --ignored --nocapture"]
fn what_this_wave_moves_in_the_grid_smoke_control() {
    const KID_W: [f64; 6] = [0.85, 0.85, 0.35, 0.35, 0.35, 0.35];
    let (kid_h, half_w, half_h, gap) = (0.35, 2.5, 1.3, 0.25);
    let mut v = vec![Node {
        parent: None,
        frame: Some(FrameStyle {
            dir: Dir::RowWrap,
            gap: [gap, gap],
            ..Default::default()
        }),
        item: ItemStyle::default(),
        size: [Len::Fixed(half_w * 2.0), Len::Fixed(half_h * 2.0)],
        min: [None; 2],
        max: [None; 2],
    }];
    v.extend(KID_W.iter().map(|w| Node {
        parent: Some(0),
        size: [Len::Fixed(w * 2.0), Len::Fixed(kid_h * 2.0)],
        ..Default::default()
    }));
    let s = solve(&v).expect("resolve");
    println!(
        "\n=== 4. cena =69, CONTROLE RowWrap {:.2}x{:.2}, filho h={:.2} ===",
        half_w * 2.0,
        half_h * 2.0,
        kid_h * 2.0
    );
    for (i, r) in s[1..].iter().enumerate() {
        println!("   filho {i}: y={:.3} h={:.3}", r[1], r[3]);
    }
}
