//! **AS TRÊS LETRAS QUE PLANTAM UM OBJECTO** — `J` · `K` · `M`: que elas existem no alfabeto
//! da tartaruga, e QUANTO cada marca já abriu.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 no default da workspace), e o
//! corte é por RESPONSABILIDADE: o irmão [`super::tests`] mede o nó inteiro (defaults,
//! gerações fraccionárias, a costura com o painel), e este mede **as marcas**.

use super::*;
use ph2d_nodegraph::attr::Column;

/// A coluna escalar `name` do esqueleto. ⚠️ Gémea da do irmão `lib_tests`: ela é três linhas
/// e viaja com quem a usa — partilhá-la obrigaria a um módulo de teste comum entre dois
/// `#[path]`, que é mais encanamento do que a duplicação que evita.
fn scal(s: &ph2d_nodegraph::attr::Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => panic!("coluna {name}"),
    }
}

/// ⭐⭐ **AS TRÊS LETRAS SÃO MESMO ÂNCORAS** — o contrato entre [`LEAF_PARAMS`] e o alfabeto.
///
/// ⚠️ **O emparelhamento por índice é compile-enforced** (dois arrays de `3`), mas isso não diz
/// que as letras EXISTEM: um `LEAF_SYMBOLS` com um `b'Q'` compilaria, o painel ofereceria o
/// campo, e o artista escreveria um nome que nunca planta nada. *Uma lista bem-formada não é
/// uma lista verdadeira.*
///
/// A régua é o produto: uma gramática que emite a letra tem de devolver um elemento com aquele
/// `sym`, e ele NÃO pode ter osso (uma âncora é uma marca, não um segmento).
#[test]
fn every_leaf_letter_is_an_anchor_the_turtle_actually_emits() {
    for (i, letter) in LEAF_SYMBOLS.iter().enumerate() {
        let src = format!("F[{}]", *letter as char);
        let s = probe_build(&src, "F -> F", 1.0, &[]);
        let syms = scal(&s, "sym");
        let lens = scal(&s, "len");
        let at = syms
            .iter()
            .position(|v| *v as i32 as u8 == *letter)
            .unwrap_or_else(|| {
                panic!(
                    "a letra {} ({}) do param `{}` não produz elemento nenhum: {syms:?}",
                    *letter as char, letter, LEAF_PARAMS[i]
                )
            });
        assert_eq!(
            lens[at], 0.0,
            "a marca {} tem de ser uma ÂNCORA (comprimento zero), e veio com osso",
            *letter as char
        );
    }
}

/// ⭐⭐⭐ **O PESO DE ABERTURA DE UMA MARCA É UM CRUZA-FADE, E ELE SOMA `1`.**
///
/// A lei que o report do Enio de 2026-08-30 pediu (*"as folhas não crescem, elas aparecem"* +
/// *"aparecem em cada segmento"*). Ver [`crate::turtle::mark_grow`] para o mecanismo.
///
/// ⚠️ **A afirmação forte é a CONSERVAÇÃO**, não os casos: `peso(Y) + peso(Y−1) = 1` em todo
/// instante é o que torna a virada de geração contínua. Um gate que só verificasse os extremos
/// passaria com um degrau no meio.
#[test]
fn a_marks_opening_weight_crossfades_between_two_generations() {
    use crate::turtle::mark_grow;
    for k in 0..=20 {
        let f = k as f32 / 20.0;
        let (y, w) = (5u16, (5u16, f));
        assert!(
            (mark_grow(b'J', y, w) + mark_grow(b'J', y - 1, w) - 1.0).abs() < 1e-6,
            "a soma dos dois pesos tem de ser 1 (f={f})"
        );
        assert_eq!(mark_grow(b'J', y - 2, w), 0.0, "a marca velha ja' fechou");
    }
    // Geração INTEIRA: só a última se vê, e cheia.
    let whole = (5u16, 1.0f32);
    assert_eq!(mark_grow(b'J', 5, whole), 1.0);
    assert_eq!(mark_grow(b'J', 4, whole), 0.0);
    // ⚠️ **As TRÊS letras**, não só o `J` — uma lei escrita para uma letra deixaria as outras
    // duas a aparecer em cada segmento, que é o defeito original em dois terços do alfabeto.
    for sym in *crate::LEAF_SYMBOLS {
        assert_eq!(
            mark_grow(sym, 5, whole),
            1.0,
            "a letra {} nao abre",
            sym as char
        );
    }
    // ⛔ **E um OSSO nunca fecha** — devolver-lhe o peso da marca apagaria a planta.
    for sym in *b"FGfg" {
        assert_eq!(
            mark_grow(sym, 1, whole),
            1.0,
            "o osso {} envelheceu e desapareceu",
            sym as char
        );
    }
}

/// ⚠️ **E a coluna existe na CORRENTE**, não só na função — o consumidor é a membrana do shell,
/// e uma lei sem coluna não alcança ninguém.
#[test]
fn the_skeleton_publishes_the_mark_opening_weight() {
    let p = &PRESETS[0];
    let over = [(param::ANGLE, p.angle), (param::STEP, p.step)];
    let count_open = |g: f32| -> usize {
        let s = probe_build(p.axiom, p.rules, g, &over);
        let (sym, grow) = (
            match s.get("sym") {
                Some(Column::Scalar(v)) => v.clone(),
                _ => panic!("sem sym"),
            },
            match s.get("mark_grow") {
                Some(Column::Scalar(v)) => v.clone(),
                _ => panic!("a coluna `mark_grow` nao viaja na corrente"),
            },
        );
        (0..sym.len())
            .filter(|&i| sym[i] as i32 as u8 == b'J' && grow[i] > 0.5)
            .count()
    };
    // A árvore de fábrica tem `2^g` pontas e `2(2^g − 1)` marcas acumuladas: a geração
    // inteira mostra as pontas, não o acumulado.
    assert_eq!(
        count_open(4.0),
        16,
        "g=4: as pontas sao 16, as marcas sao 30"
    );
    assert_eq!(
        count_open(5.0),
        32,
        "g=5: as pontas sao 32, as marcas sao 62"
    );
}
