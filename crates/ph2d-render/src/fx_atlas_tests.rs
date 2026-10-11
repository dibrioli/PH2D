//! Os gates do empacotador do atlas de FX.
//!
//! O que se prova aqui é o que uma foto **não** consegue mostrar: duas células a sobrepor-se
//! pintam uma forma dentro da outra, e o resultado ainda parece arte.

use super::{Batch, Placement, pack};

/// Todo par de células de um mesmo lote é disjunto, e nenhuma sai da textura.
fn assert_sane(batches: &[Batch], sizes: &[(u32, u32)], max_side: u32) {
    let mut seen: Vec<usize> = Vec::new();
    for b in batches {
        for c in &b.cells {
            let (w, h) = sizes[c.index];
            assert!(
                c.org[0] + w <= b.w && c.org[1] + h <= b.h,
                "célula {c:?} ({w}x{h}) sai da textura {}x{} do lote",
                b.w,
                b.h
            );
            seen.push(c.index);
        }
        // Um lote nunca é maior que o teto, salvo quando UMA forma sozinha já o excede.
        let lone = b.cells.len() == 1;
        assert!(
            lone || (b.w <= max_side && b.h <= max_side),
            "lote {}x{} passou do teto {max_side}",
            b.w,
            b.h
        );
        for (i, a) in b.cells.iter().enumerate() {
            for d in &b.cells[i + 1..] {
                assert!(
                    disjoint(*a, *d, sizes),
                    "células sobrepostas: {a:?} e {d:?}"
                );
            }
        }
    }
    seen.sort_unstable();
    let all: Vec<usize> = (0..sizes.len()).collect();
    assert_eq!(
        seen, all,
        "alguma forma não foi colocada, ou foi duas vezes"
    );
}

fn disjoint(a: Placement, b: Placement, sizes: &[(u32, u32)]) -> bool {
    let (aw, ah) = sizes[a.index];
    let (bw, bh) = sizes[b.index];
    a.org[0] + aw <= b.org[0]
        || b.org[0] + bw <= a.org[0]
        || a.org[1] + ah <= b.org[1]
        || b.org[1] + bh <= a.org[1]
}

/// **A propriedade inteira**: toda forma entra exactamente uma vez, dentro da textura, sem
/// sobrepor nenhuma outra. Fixtures que contêm o fenómeno: alturas iguais (o caso fácil), alturas
/// TODAS diferentes (o que exercita a ordenação) e uma lista que estoura a largura do teto.
#[test]
fn every_shape_lands_once_inside_its_texture_and_no_two_cells_overlap() {
    let uniform: Vec<(u32, u32)> = (0..9).map(|_| (256, 256)).collect();
    let ladder: Vec<(u32, u32)> = (1..=12).map(|i| (40 + i * 7, 30 + i * 11)).collect();
    let wide: Vec<(u32, u32)> = (0..6).map(|i| (300 + i * 13, 200)).collect();
    for (name, sizes, max) in [
        ("uniforme", uniform, 1024u32),
        ("escada", ladder, 512),
        ("larga", wide, 700),
    ] {
        let b = pack(&sizes, max);
        assert!(!b.is_empty(), "{name}: nenhum lote");
        assert_sane(&b, &sizes, max);
    }
}

/// **Uma cena típica cabe num render.** É a afirmação que a wave inteira compra: se o empacotador
/// devolvesse um lote por forma, o custo fixo continuaria a multiplicar e nada teria mudado.
#[test]
fn a_typical_scene_packs_into_a_single_render() {
    let sizes: Vec<(u32, u32)> = (0..32)
        .map(|i| (232 + (i % 4) * 16, 232 + (i % 3) * 24))
        .collect();
    let b = pack(&sizes, 8192);
    assert_eq!(b.len(), 1, "32 formas de ~256 px deviam caber num atlas só");
    assert_sane(&b, &sizes, 8192);
    // E a textura é APERTADA: a área do atlas não passa do dobro da área da arte (o desperdício
    // de prateleira). Sem isto, "um render" poderia significar "um render de 8192², todo frame".
    let art: u64 = sizes
        .iter()
        .map(|&(w, h)| u64::from(w) * u64::from(h))
        .sum();
    let tex = u64::from(b[0].w) * u64::from(b[0].h);
    assert!(
        tex < art * 2,
        "o atlas {}x{} ({tex} px) desperdiça demais para {art} px de arte",
        b[0].w,
        b[0].h
    );
}

/// Passado o teto, o empacotador **divide em lotes** em vez de deixar formas de fora — e cada lote
/// continua são.
#[test]
fn shapes_that_do_not_fit_one_texture_are_split_into_batches_not_dropped() {
    let sizes: Vec<(u32, u32)> = (0..40).map(|_| (500, 500)).collect();
    let b = pack(&sizes, 1024);
    assert!(
        b.len() > 1,
        "40 formas de 500 px não cabem num atlas de 1024"
    );
    assert_sane(&b, &sizes, 1024);
}

/// Uma forma **maior que o teto** ganha o lote dela em vez de desaparecer. (O chamador já a
/// limitou a `MAX_FX_SIDE`; o que este gate proíbe é o outro modo de falha.)
#[test]
fn a_shape_larger_than_the_ceiling_still_gets_placed() {
    let sizes = [(64, 64), (9000, 40), (64, 64)];
    let b = pack(&sizes, 8192);
    assert_sane(&b, &sizes, 8192);
}

/// **O mesmo frame dá o mesmo atlas.** O memo do `fx_live` compara a pilha resolvida; um
/// empacotamento que oscilasse faria a origem da célula mudar sem nada ter mudado.
#[test]
fn the_packing_is_deterministic() {
    let sizes: Vec<(u32, u32)> = (0..20)
        .map(|i| (100 + (i * 37) % 90, 80 + (i * 53) % 70))
        .collect();
    assert_eq!(pack(&sizes, 512), pack(&sizes, 512));
}

/// Lista vazia = nenhum lote (e portanto nenhum render). O caminho comum de uma cena SEM FX.
#[test]
fn an_empty_scene_asks_for_no_render() {
    assert!(pack(&[], 8192).is_empty());
}
