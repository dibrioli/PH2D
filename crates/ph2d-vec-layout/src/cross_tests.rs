//! **O EIXO TRANSVERSAL** — onde o BLOCO de faixas senta, e o que o `Stretch` alcança.
//!
//! Irmão do [`crate::grid_tests`]. Aquele pergunta *o que uma grade dá que um wrap não dá*; este
//! pergunta a metade que os DOIS partilham: um contentor de várias faixas tem duas perguntas de
//! alinhamento (*onde o filho senta na faixa* × *onde o bloco de faixas senta na moldura*) e o
//! artista tem **um** controlo para as duas.
//!
//! ⚠️ O gate que carrega o assunto é o [`a_grid_and_a_wrap_agree_on_what_align_means`]: o defeito
//! não era *"o wrap espalha"* isolado — era **dois contentores a responder ao contrário sob o
//! mesmo controlo**, e isso só ficou observável quando a grade nasceu ao lado do wrap.

use crate::{Align, Dir, FrameStyle, ItemStyle, Len, Node, solve};

/// Meia-largura dos filhos e a moldura: **três** cabem por faixa (`3×30 = 90 ≤ 100`; o quarto daria
/// 120), então há **duas** faixas de 20 de altura numa moldura de 100 ⇒ **60 de folga**.
const KID: [f64; 2] = [30.0, 20.0];
const FRAME_W: f64 = 100.0;
/// Alta o bastante para haver sobra entre as faixas — é a sobra que o `align_content` reparte.
const FRAME_H_SLACK: f64 = 100.0;
/// Exactamente duas faixas: **zero** sobra. É o CONTROLE.
const FRAME_H_TIGHT: f64 = 40.0;

fn frame(dir: Dir, h: f64, n: usize, align: Align) -> Vec<Node> {
    let mut v = vec![Node {
        parent: None,
        frame: Some(FrameStyle {
            dir,
            align,
            ..Default::default()
        }),
        item: ItemStyle::default(),
        size: [Len::Fixed(FRAME_W), Len::Fixed(h)],
        min: [None; 2],
        max: [None; 2],
    }];
    v.extend((0..n).map(|_| Node {
        parent: Some(0),
        size: [Len::Fixed(KID[0]), Len::Fixed(KID[1])],
        ..Default::default()
    }));
    v
}

fn tops(solved: &[[f64; 4]]) -> Vec<f64> {
    solved[1..].iter().map(|s| s[1]).collect()
}

/// **O wrap ENCOSTA o bloco de faixas onde o `align` manda.**
///
/// Red-first: sem o espelho o `taffy` herda `align-content: stretch`, INFLA as duas faixas para
/// 50 cada e a segunda pousa em **50** — o número medido pela sonda antes da cura.
#[test]
fn the_wrap_packs_its_bands_where_the_align_says() {
    let band2 = |align| {
        let s = solve(&frame(Dir::RowWrap, FRAME_H_SLACK, 6, align)).expect("resolve");
        let t = tops(&s);
        // Os três primeiros são a 1ª faixa, os três últimos a 2ª.
        (t[0], t[3])
    };
    // Encostado: faixa 1 no topo, faixa 2 uma altura de filho abaixo.
    assert_eq!(band2(Align::Start), (0.0, KID[1]), "Start tem de ENCOSTAR");
    // Centrado: o bloco mede 2·20 = 40 numa moldura de 100 ⇒ sobra 60, metade acima.
    let block = KID[1] * 2.0;
    let pad = (FRAME_H_SLACK - block) / 2.0;
    assert_eq!(
        band2(Align::Center),
        (pad, pad + KID[1]),
        "Center centra o BLOCO"
    );
    // No fim: o bloco encosta em baixo.
    let bot = FRAME_H_SLACK - block;
    assert_eq!(
        band2(Align::End),
        (bot, bot + KID[1]),
        "End encosta em BAIXO"
    );
    // ⚠️ E o `Stretch` INFLA as faixas em vez de as encostar: com duas, cada uma fica com metade
    // da moldura, e o filho (que é Fixed, logo inesticável) senta no topo da sua. É o único dos
    // quatro que reparte a sobra em vez de a empurrar para um lado — sem esta linha, colapsá-lo
    // em `Start` passaria despercebido.
    let half = FRAME_H_SLACK / 2.0;
    assert_eq!(
        band2(Align::Stretch),
        (0.0, half),
        "Stretch INFLA as faixas"
    );
}

/// **O CONTROLE: sem sobra, os quatro alinhamentos coincidem.**
///
/// Se esta afirmação cair, o que o gate acima mede não é distribuição de sobra — é outra coisa,
/// e o número dele deixaria de significar o que ele diz.
#[test]
fn with_no_slack_between_bands_every_align_agrees() {
    let mut seen = Vec::new();
    for align in [Align::Start, Align::Center, Align::End, Align::Stretch] {
        let s = solve(&frame(Dir::RowWrap, FRAME_H_TIGHT, 6, align)).expect("resolve");
        seen.push(tops(&s));
    }
    for (i, t) in seen.iter().enumerate().skip(1) {
        assert_eq!(t, &seen[0], "align #{i} divergiu num contentor SEM sobra");
    }
    assert_eq!(seen[0], vec![0.0, 0.0, 0.0, KID[1], KID[1], KID[1]]);
}

/// **A GRADE e o WRAP concordam sobre o que o `align` significa** — o gate do assunto.
///
/// Mesmos filhos, mesma moldura, mesmo controlo, e três por faixa nos dois casos (a grade por
/// contagem, o wrap por largura). O bloco de faixas TEM de pousar no mesmo sítio; se divergir, o
/// artista aprende o controlo num contentor e ele mente no outro.
#[test]
fn a_grid_and_a_wrap_agree_on_what_align_means() {
    for align in [Align::Start, Align::Center, Align::End] {
        let wrap = solve(&frame(Dir::RowWrap, FRAME_H_SLACK, 6, align)).expect("wrap");
        let grid = solve(&frame(Dir::Grid { columns: 3 }, FRAME_H_SLACK, 6, align)).expect("grid");
        assert_eq!(
            tops(&wrap),
            tops(&grid),
            "grade e wrap discordam sob align={align:?}"
        );
    }
}

/// **Uma faixa só pousa onde sempre pousou.**
///
/// ⚠️ E **não** porque o `align_content` seja inerte ali — essa era a minha frase, e a medição
/// derrubou-a. Num `Row`/`Column` (`NoWrap`) ele é de facto inerte; num `RowWrap` de faixa única
/// ele **posiciona a faixa e VENCE o `align_items`** (a faixa mede o filho, logo não sobra folga
/// dentro dela). Medido em `cross_probe` §3: com `items=Start`, `content=Center` põe o filho em
/// **40,0** e `content=End` em **80,0**.
///
/// A resposta só não muda porque o espelho entrega **o mesmo valor às duas** — é isso que este
/// gate afirma, e é por isso que a mutação que troca `Center`/`End` só no `align_content` o
/// derruba.
#[test]
fn a_single_band_lands_where_it_always_did() {
    // Row/Column com 2 filhos, e um wrap cujos 2 filhos cabem folgados numa faixa.
    for dir in [Dir::Row, Dir::Column, Dir::RowWrap] {
        let mut seen = Vec::new();
        for align in [Align::Start, Align::Center, Align::End, Align::Stretch] {
            let s = solve(&frame(dir, FRAME_H_SLACK, 2, align)).expect("resolve");
            seen.push((align, tops(&s)));
        }
        // Numa faixa só, o que muda entre os alinhamentos é o `align_items` — que já existia.
        // O que se afirma aqui é que cada resposta é a MESMA de antes desta wave, e a forma
        // barata de o dizer é: o filho ocupa a altura dele e o alinhamento move-o dentro da
        // moldura, nunca reparte faixas.
        for (align, t) in &seen {
            let expect = match (dir, align) {
                // Column: os filhos empilham no eixo principal; o transversal é a largura.
                (Dir::Column, _) => vec![0.0, KID[1]],
                (_, Align::Start | Align::Stretch) => vec![0.0, 0.0],
                (_, Align::Center) => {
                    let c = (FRAME_H_SLACK - KID[1]) / 2.0;
                    vec![c, c]
                }
                (_, Align::End) => {
                    let e = FRAME_H_SLACK - KID[1];
                    vec![e, e]
                }
            };
            assert_eq!(t, &expect, "dir={dir:?} align={align:?} mudou de resposta");
        }
    }
}

/// **O `Stretch` alcança o que é auto-dimensionado, e uma FOLHA nunca é.**
///
/// ⚠️ Isto pina os DOIS lados de uma frase que é fácil escrever pela metade. A wave da grade
/// mediu *"o Stretch não estica"* sobre uma folha e generalizou — e a generalização era falsa:
/// numa moldura filha que ABRAÇA o eixo transversal ele estica, e é para isso que serve.
#[test]
fn stretch_reaches_only_what_is_auto_sized() {
    // (a) Folha de tamanho explícito: `Start` e `Stretch` dão o mesmo.
    for align in [Align::Start, Align::Stretch] {
        let s = solve(&frame(Dir::Row, 60.0, 2, align)).expect("resolve");
        assert_eq!(s[1][3], KID[1], "uma folha Fixed nao pode ser esticada");
    }
    // (b) Moldura filha que abraça: o neto mede 12, e o `Stretch` leva-a à moldura inteira.
    let grandchild_h = 12.0;
    let parent_h = 60.0;
    let build = |align| {
        vec![
            Node {
                parent: None,
                frame: Some(FrameStyle {
                    dir: Dir::Row,
                    align,
                    ..Default::default()
                }),
                item: ItemStyle::default(),
                size: [Len::Fixed(FRAME_W), Len::Fixed(parent_h)],
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
                size: [Len::Fixed(KID[0]), Len::Hug],
                min: [None; 2],
                max: [None; 2],
            },
            Node {
                parent: Some(1),
                size: [Len::Fixed(20.0), Len::Fixed(grandchild_h)],
                ..Default::default()
            },
        ]
    };
    let hugging = solve(&build(Align::Start)).expect("start");
    let stretched = solve(&build(Align::Stretch)).expect("stretch");
    assert_eq!(hugging[1][3], grandchild_h, "Start deixa a moldura abracar");
    assert_eq!(stretched[1][3], parent_h, "Stretch enche a moldura");
}
