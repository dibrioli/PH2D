//! Os gates da **rasterização dos fios** ([`crate::sketchy_raster`], plano 38 W3).
//!
//! Eles perguntam pelo ALFA que sai — quanto, onde, e quanto mais no cruzamento —, nunca pela
//! fórmula que o produz.

use crate::sketchy_raster::{ThreadInk, threads_alpha, threads_bbox};
use crate::stroke::sketchy::Thread;

const W: usize = 40;
const H: usize = 40;

fn ink(op: f32) -> ThreadInk {
    ThreadInk {
        width_px: 2.0,
        opacity: op,
    }
}

/// Alfa no pixel `(x, y)` da janela ancorada na origem do canvas.
fn at(a: &[u8], x: usize, y: usize) -> u32 {
    u32::from(a[y * W + x])
}

/// Um fio horizontal e um vertical que se cruzam em `(20, 20)`.
fn cross() -> Vec<Thread> {
    vec![[5.0, 20.0, 35.0, 20.0], [20.0, 35.0, 20.0, 5.0]]
}

/// **O CRUZAMENTO É MAIS ESCURO QUE O FIO** — a frase inteira da feature, medida no alfa.
///
/// ⚠️ **Mutação que sangra:** preencher os dois quads numa chamada só de `fill_coverage`. A regra
/// não-zero satura em `1`, o cruzamento sai IGUAL ao fio solto, e a teia fica chapada — que é a
/// razão de este módulo compor `over` por fio (ver o doc dele).
#[test]
fn a_crossing_is_darker_than_a_single_thread() {
    let a = threads_alpha(&cross(), ink(0.3), W, H, [0.0, 0.0]);
    let solo = at(&a, 10, 20);
    let both = at(&a, 20, 20);
    assert!(solo > 0, "controle: o fio solto tem de pintar ({solo})");
    // `over` de dois fios de 0,3: 1 − 0,7² = 0,51 ⇒ ~130 contra ~76. A barra é a ORDEM (metade a
    // mais), não o número exato, que depende do arredondamento em u8.
    assert!(
        both > solo + solo / 2,
        "o cruzamento não acumulou: {both} contra {solo} do fio solto"
    );
}

/// **O SENTIDO DO FIO NÃO MUDA NADA** — a metade que prova que o cancelamento de winding, que eu ia
/// declarar no doc, não existe: inverter as duas pontas devolve o MESMO alfa, ao byte.
///
/// ⚠️ Ele é o CONTROLE da nota do módulo. Sem ele a frase *"a orientação do quad é invariante ao
/// sentido"* seria uma afirmação minha sobre aritmética que ninguém volta a conferir.
#[test]
fn reversing_a_thread_paints_the_same_ink() {
    let fwd = threads_alpha(&[[5.0, 20.0, 35.0, 20.0]], ink(0.4), W, H, [0.0, 0.0]);
    let rev = threads_alpha(&[[35.0, 20.0, 5.0, 20.0]], ink(0.4), W, H, [0.0, 0.0]);
    assert!(fwd.iter().any(|&v| v > 0), "controle: o fio tem de pintar");
    assert_eq!(fwd, rev, "o sentido do fio mudou a tinta");
}

/// **A LARGURA É A QUE O SLIDER DIZ** — o oráculo é o corte transversal do fio, não a constante.
#[test]
fn the_thread_is_as_wide_as_the_slider_says() {
    for width in [1.0f32, 2.0, 4.0] {
        let mut i = ink(1.0);
        i.width_px = width;
        let a = threads_alpha(&[[5.0, 20.0, 35.0, 20.0]], i, W, H, [0.0, 0.0]);
        // A soma da coluna do meio é a espessura em px (cobertura exata por área ⇒ a soma dos
        // parciais das bordas fecha o total).
        let sum: u32 = (0..H).map(|y| at(&a, 20, y)).sum();
        #[allow(clippy::cast_precision_loss)]
        let measured = sum as f32 / 255.0;
        assert!(
            (measured - width).abs() < 0.15,
            "largura {width}: o corte mediu {measured:.2} px"
        );
    }
}

/// **O RASTERIZADOR NÃO TEM OPINIÃO SOBRE A DISTÂNCIA** — um fio curto e um longo pintam com a
/// MESMA tinta.
///
/// ⚠️ Esta é a metade que sobrou de um gate que pinava o oposto: a wave nasceu com uma rampa de
/// opacidade por distância aqui, batizada de `Magnetify`, e o manual do Krita a desmente (o
/// Magnetify escolhe QUE PARES viram fio — a lei mora no motor). Um fio que chegou até aqui **já é**
/// um par escolhido, e pesá-lo de novo pela distância seria a decisão tomada duas vezes.
///
/// **Mutação que sangra:** qualquer re-introdução de peso por comprimento nesta função.
#[test]
fn the_rasteriser_has_no_opinion_about_a_threads_length() {
    // Dois fios paralelos com o MESMO desenho e comprimentos bem diferentes (4 px e 20 px).
    let short: Thread = [4.0, 10.0, 8.0, 10.0];
    let long: Thread = [4.0, 30.0, 24.0, 30.0];
    let a = threads_alpha(&[short, long], ink(0.8), W, H, [0.0, 0.0]);
    let (s, l) = (at(&a, 6, 10), at(&a, 14, 30));
    assert!(s > 0 && l > 0, "controle: os dois fios têm de pintar");
    assert_eq!(
        s, l,
        "o comprimento do fio mudou a tinta dele: curto {s}, longo {l}"
    );
}

/// **O NEUTRO NÃO PINTA** — opacidade zero, largura zero e feixe vazio, as três portas de saída.
#[test]
fn the_neutral_paints_nothing() {
    let empty: Vec<Thread> = Vec::new();
    assert!(
        threads_alpha(&empty, ink(1.0), W, H, [0.0, 0.0])
            .iter()
            .all(|&v| v == 0)
    );
    assert!(
        threads_alpha(&cross(), ink(0.0), W, H, [0.0, 0.0])
            .iter()
            .all(|&v| v == 0)
    );
    let mut thin = ink(1.0);
    thin.width_px = 0.0;
    assert!(
        threads_alpha(&cross(), thin, W, H, [0.0, 0.0])
            .iter()
            .all(|&v| v == 0)
    );
    // ⚠️ E o CONTROLE: a MESMA fixture com tinta pinta, senão os três acima são verdadeiros de graça.
    assert!(
        threads_alpha(&cross(), ink(1.0), W, H, [0.0, 0.0])
            .iter()
            .any(|&v| v > 0)
    );
}

/// **A JANELA É A QUE O FEIXE TOCA** — e a tinta cai no mesmo lugar quando a janela é deslocada,
/// que é o que o depósito faz (ele escreve numa caixa, não no canvas inteiro).
#[test]
fn the_window_is_where_the_ink_lands() {
    let t: Vec<Thread> = vec![[12.0, 14.0, 26.0, 22.0]];
    let bb = threads_bbox(&t, 2.0, W, H).expect("o feixe tem de tocar a tela");
    let [bx, by, bw, bh] = bb;
    assert!(
        bx >= 10 && by >= 12 && bw <= 20 && bh <= 14,
        "caixa larga: {bb:?}"
    );
    // A mesma tinta, pela janela recortada: o pixel `(20,18)` do canvas é `(20-bx, 18-by)` nela.
    let full = threads_alpha(&t, ink(0.7), W, H, [0.0, 0.0]);
    #[allow(clippy::cast_precision_loss)]
    let cut = threads_alpha(&t, ink(0.7), bw, bh, [bx as f32, by as f32]);
    for y in 0..bh {
        for x in 0..bw {
            assert_eq!(
                full[(by + y) * W + bx + x],
                cut[y * bw + x],
                "a janela deslocou a tinta em ({}, {})",
                bx + x,
                by + y
            );
        }
    }
}

/// **O FEIXE FORA DA TELA NÃO TEM CAIXA** — a rejeição-antes-do-clamp do [`crate::solid`], herdada
/// pela porta e não re-escrita aqui.
#[test]
fn a_beam_off_screen_has_no_box() {
    let t: Vec<Thread> = vec![[-200.0, -200.0, -150.0, -150.0]];
    assert!(threads_bbox(&t, 2.0, W, H).is_none());
}
