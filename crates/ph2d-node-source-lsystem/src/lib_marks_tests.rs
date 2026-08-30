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

/// ⭐⭐⭐ **O PESO DE UMA MARCA SÓ SOBE** — a lei que o 2.º smoke de 2026-08-30 impôs.
///
/// Enio: *"a cada segmento a folha cresce e diminui. bem bizarro"*. A 1.ª lei era um
/// cruza-fade (peso `f` à geração nova, `1 − f` à anterior), para que uma planta parada
/// mostrasse só as pontas — e o preço era cada folha **encolher até sumir** quando o ramo
/// seguinte brotava dela.
///
/// ⚠️ **A afirmação forte é a MONOTONIA**, não os casos: uma folha nascida em `b` nunca fica
/// mais pequena do que já esteve, por mais que a planta cresça. Um gate que só olhasse os
/// extremos passaria com um vale no meio — que é exactamente o que a lei antiga tinha.
#[test]
fn a_marks_weight_only_ever_grows() {
    // ⚠️ **A régua corre no PRODUTO, não numa cópia do plano de gerações.** A 1.ª redacção
    // fabricava a sequência `(youngest, fracção)` à mão — e fabricou-a ao contrário, andando
    // para trás no tempo. *Uma fixtura que reimplementa a lei que testa pode estar errada de
    // maneiras que a lei não está.*
    let p = &PRESETS[0];
    let over = [(param::ANGLE, p.angle), (param::STEP, p.step)];
    let mut visto: std::collections::BTreeMap<i64, f32> = Default::default();
    let mut g = 0.5f32;
    while g <= 6.0 {
        let s = probe_build(p.axiom, p.rules, g, &over);
        let (sym, born) = (scal(&s, "sym"), scal(&s, "gen"));
        let grow = match s.get("mark_grow") {
            Some(Column::Scalar(v)) => v.clone(),
            _ => panic!("sem `mark_grow`"),
        };
        for i in 0..sym.len() {
            if !crate::LEAF_SYMBOLS.contains(&(sym[i] as i32 as u8)) {
                continue;
            }
            let e = visto.entry(born[i] as i64).or_insert(0.0);
            assert!(
                grow[i] >= *e - 1e-6,
                "a folha da geracao {} ENCOLHEU a g={g}: {} -> {}",
                born[i],
                *e,
                grow[i]
            );
            *e = grow[i];
        }
        g += 0.25;
    }
    // ⚠️ **O CONTROLE**: a varredura tem de ter visto várias gerações e todas maduras no fim —
    // um laço que não iterasse passaria as afirmações acima em silêncio.
    assert!(visto.len() >= 4, "a varredura viu {} geracoes", visto.len());
    assert!(
        visto.values().all(|w| (*w - 1.0).abs() < 1e-6),
        "no fim toda folha tem de estar madura: {visto:?}"
    );
    // A geração mais nova abre com a fracção; toda a mais velha está cheia.
    use crate::turtle::mark_grow;
    assert_eq!(mark_grow(b'J', 5, (5, 0.25)), 0.25);
    assert_eq!(mark_grow(b'J', 4, (5, 0.25)), 1.0);
    assert_eq!(mark_grow(b'J', 1, (5, 0.25)), 1.0);
    // ⚠️ **As TRÊS letras**, não só o `J` — uma lei escrita para uma letra deixaria as outras
    // duas sem crescimento nenhum.
    for sym in *crate::LEAF_SYMBOLS {
        assert_eq!(
            mark_grow(sym, 5, (5, 0.5)),
            0.5,
            "a letra {} nao abre",
            sym as char
        );
    }
    // ⛔ **E um OSSO está sempre cheio** — devolver-lhe o peso da marca apagaria a planta.
    for sym in *b"FGfg" {
        assert_eq!(
            mark_grow(sym, 5, (5, 0.5)),
            1.0,
            "o osso {} encolheu",
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
    let pesos = |g: f32| -> Vec<f32> {
        let s = probe_build(p.axiom, p.rules, g, &over);
        let sym = scal(&s, "sym");
        let grow = match s.get("mark_grow") {
            Some(Column::Scalar(v)) => v.clone(),
            _ => panic!("a coluna `mark_grow` nao viaja na corrente"),
        };
        (0..sym.len())
            .filter(|&i| sym[i] as i32 as u8 == b'J')
            .map(|i| grow[i])
            .collect()
    };
    // Geração INTEIRA: toda folha está madura.
    let cheias = pesos(5.0);
    assert_eq!(cheias.len(), 31, "uma marca por segmento, a g=5");
    assert!(
        cheias.iter().all(|w| (*w - 1.0).abs() < 1e-6),
        "numa planta parada nenhuma folha esta' a meio: {cheias:?}"
    );
    // A MEIO: só a colheita nova está a abrir, e ela é a diferença entre as duas contagens.
    let meio = pesos(4.5);
    let novas = meio.iter().filter(|w| (**w - 0.5).abs() < 1e-6).count();
    let velhas = meio.iter().filter(|w| (**w - 1.0).abs() < 1e-6).count();
    assert_eq!(
        (novas, velhas),
        (16, 15),
        "a g=4,5 as 15 folhas velhas ficam cheias e as 16 novas abrem a meio: {meio:?}"
    );
}

/// ⛔⛔ **DUAS FOLHAS NÃO SE EMPILHAM NO MESMO PONTO** — a outra metade do report de
/// 2026-08-30 (*"elas aparecem em cada segmento"*), e esta era do MOLDE, não da lei.
///
/// A gramática de fábrica era `A(s) -> F(s)![+A(s*0.7)J][-A(s*0.7)J]`: o `J` vinha **depois**
/// da sub-árvore inteira, e a tartaruga, ao sair dela, está de volta ao fim do `F` — onde as
/// marcas de todas as gerações que a envolvem também caem. Medido: **62 marcas em 30 sítios**
/// (`2,07×`), folhas idênticas uma em cima da outra.
///
/// ⇒ o `J` passa a vir **logo a seguir ao segmento** (`F(s)[J]!…`): uma marca por segmento,
/// `1,00×`. *Uma contagem de marcas não vê um empilhamento; o que o vê é contar os SÍTIOS.*
#[test]
fn no_two_leaves_land_on_the_same_spot() {
    for p in PRESETS {
        let s = probe_build(
            p.axiom,
            p.rules,
            p.generations,
            &[(param::ANGLE, p.angle), (param::STEP, p.step)],
        );
        let sym = scal(&s, "sym");
        let pos = match s.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => panic!("sem P"),
        };
        let mut sitios: Vec<(i64, i64)> = (0..sym.len())
            .filter(|&i| crate::LEAF_SYMBOLS.contains(&(sym[i] as i32 as u8)))
            .map(|i| ((pos[i][0] * 1e4) as i64, (pos[i][1] * 1e4) as i64))
            .collect();
        let marcas = sitios.len();
        sitios.sort_unstable();
        sitios.dedup();
        assert_eq!(
            marcas,
            sitios.len(),
            "{}: {marcas} marcas em {} sitios — folhas empilhadas",
            p.label,
            sitios.len()
        );
    }
}
