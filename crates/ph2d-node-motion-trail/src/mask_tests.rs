//! **A MÁSCARA do `motion.trail`** — o `falloff` a decidir quem deixa rastro, quanto
//! dele, e o que um fantasma herda de quem o fez.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 para `crates/`, e ele conta
//! arquivos de teste também). O corte é por ASSUNTO: os dois helpers que só a máscara
//! usa vieram com ela, e os quatro genéricos ficaram no irmão — importados por nome, que
//! é o que torna a dependência visível em vez de implícita num `use super::*`.

use super::*;
use crate::tests::{alphas, dot, trail_ages, xs};

/// Um ponto em `x` com a máscara `f` na coluna `falloff`.
fn dot_masked(x: f32, f: f32) -> Stream {
    dot(x, 1.0).with("falloff", Column::Scalar(vec![f]))
}

/// Roda `ticks` ticks de um ponto mascarado, devolvendo quantas linhas sobraram.
fn run_masked(ticks: usize, length: f32, f: f32) -> Stream {
    let mut state = Stream::new(0);
    for t in 0..ticks {
        state = step(
            &dot_masked(t as f32, f),
            &state,
            length,
            Decay::new(1.0, 1.0),
            1.0,
        );
    }
    state
}

/// **O CAMPO DECIDE QUEM DEIXA RASTRO** — o P0 da fam. 7: este era o único behaviour da
/// família que não lia `falloff`.
///
/// ⚠️ A cadeia óbvia não substituía: `field.* → motion.cull → trail` remove a LINHA do
/// stream inteiro (some o elemento, não só o rastro dele). Mascarado a zero, o elemento
/// continua VIVO e desenhado — só não deixa eco, que é o que a lei da família diz
/// (`falloff = 0` ⇒ como se o nó não estivesse ali).
#[test]
fn the_field_decides_who_leaves_a_trail() {
    let full = run_masked(6, 4.0, 1.0);
    let none = run_masked(6, 4.0, 0.0);
    assert_eq!(full.count(), 4, "sem máscara o rastro tem as 4 linhas");
    assert_eq!(
        none.count(),
        1,
        "mascarado a zero sobra só a cabeça VIVA — o elemento não some, o eco sim"
    );
    assert_eq!(
        trail_ages(&none),
        vec![0.0],
        "e a linha que sobra é a cabeça, não um fantasma preso"
    );
}

/// **UM MASCARAMENTO PARCIAL ENCURTA A CAUDA, NÃO A APAGA** — o que interpola entre os dois
/// extremos é a CONTAGEM de ecos, e ela é discreta.
#[test]
fn a_partial_mask_shortens_the_tail() {
    let counts: Vec<usize> = [0.0, 0.25, 0.5, 0.75, 1.0]
        .iter()
        .map(|&f| run_masked(8, 5.0, f).count())
        .collect();
    assert_eq!(counts, vec![1, 2, 3, 4, 5], "a cauda cresce com a máscara");
}

/// **SEM A COLUNA, É O RASTRO QUE JÁ SHIPAVA** — o `falloff_at` devolve `1.0` na ausência,
/// então todo grafo autorado antes desta wave rende exatamente o mesmo.
///
/// ⚠️ O oráculo é o STREAM inteiro, não a contagem: uma contagem igual sobre posições
/// diferentes passaria.
#[test]
fn an_absent_mask_is_the_trail_that_shipped() {
    let mut bare = Stream::new(0);
    let mut masked = Stream::new(0);
    for t in 0..6 {
        bare = step(&dot(t as f32, 1.0), &bare, 4.0, Decay::new(1.0, 1.0), 1.0);
        masked = step(
            &dot_masked(t as f32, 1.0),
            &masked,
            4.0,
            Decay::new(1.0, 1.0),
            1.0,
        );
    }
    assert_eq!(xs(&bare), xs(&masked), "as posições");
    assert_eq!(alphas(&bare), alphas(&masked), "as alfas");
    assert_eq!(trail_ages(&bare), trail_ages(&masked), "as idades");
}

/// **O FANTASMA HERDA A MÁSCARA DE QUEM O FEZ** — e é isso que mantém o rastro estável
/// enquanto o elemento atravessa um campo espacial: o eco lembra o peso que o gerou, em vez
/// de ser re-julgado por onde ele ficou.
///
/// ⚠️ O gate constrói o caso que só a herança distingue: o ESTADO carrega ecos nascidos com
/// máscara cheia e a cabeça VIVA chega com máscara zero. Se a janela fosse decidida pela
/// cabeça, os ecos velhos evaporariam todos de uma vez.
#[test]
fn the_ghost_inherits_the_mask_of_the_element_that_made_it() {
    let mut state = Stream::new(0);
    for t in 0..5 {
        state = step(
            &dot_masked(t as f32, 1.0),
            &state,
            4.0,
            Decay::new(1.0, 1.0),
            1.0,
        );
    }
    assert_eq!(state.count(), 4, "a cauda cheia foi construída");
    // O elemento entra numa região de peso zero: a CABEÇA nova não deixa eco, mas os que já
    // existem continuam a envelhecer pela janela com que nasceram.
    let after = step(
        &dot_masked(5.0, 0.0),
        &state,
        4.0,
        Decay::new(1.0, 1.0),
        1.0,
    );
    assert!(
        after.count() > 1,
        "os ecos já nascidos sobrevivem — vieram com máscara cheia (linhas {})",
        after.count()
    );
}
