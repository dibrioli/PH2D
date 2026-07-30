//! **A cena de um grupo da FASE E autora pelo CATÁLOGO e abre o card** (§7.1.5 do plano 12).
//!
//! ⚠️ A regra que esta suíte existe para pinar já custou uma jornada: *"uma cena que arma
//! estado por baixo da mesa pula exactamente a costura que ela deveria provar"*. A do G1
//! tem de (a) construir a fórmula pela porta que o card projeta (`RecipeStack::to_formula`),
//! (b) pedir o card, e (c) ser uma cena de TRÊS — uma por receita.

use std::path::Path;

fn src() -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/expr_group_smoke.rs");
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{} não lido: {e}", p.display()))
}

/// **A cena autora pelo catálogo, nunca por texto escrito à mão.**
///
/// **Mutação que deve sangrar:** trocar o `to_formula()` por um `format!` de fórmula.
#[test]
fn the_group_scene_authors_through_the_catalog() {
    let s = src();
    assert!(
        s.contains("stack.to_formula()"),
        "a fórmula da cena vem da MESMA porta que o card projeta — uma fórmula escrita à \
         mão prova o avaliador e não diz nada sobre o catálogo que o artista clica"
    );
    assert!(
        s.contains("Row::new(id)"),
        "e as linhas vêm do catálogo, com os knobs dele"
    );
}

/// **A cena ABRE o card** — a costura que ela existe para exercitar.
#[test]
fn the_group_scene_opens_the_card() {
    assert!(
        src().contains("request_expr_card("),
        "sem isto o artista cai numa cena que se move e não tem onde julgar os knobs"
    );
}

/// **São TRÊS objetos, um por receita, e o roteiro DIZ o que provar.**
///
/// ⚠️ O terceiro é um MODIFICADOR, e é por isso que ele vem sob uma onda grande: um `Limit`
/// sozinho não anima nada, e uma cena que o pusesse solto não diria nada dele.
#[test]
fn the_group_one_scene_names_its_three_and_says_what_to_prove() {
    let s = src();
    for name in ["Shaker", "Swayer", "Limiter"] {
        assert!(s.contains(name), "a cena do G1 tem de trazer o {name}");
    }
    assert!(
        s.contains("[expr-group-smoke]"),
        "a cena IMPRIME o que montou — se a linha não sai, o resto do smoke não significa nada"
    );
    for id in ["\"shake\"", "\"sway\"", "\"limit\""] {
        assert!(s.contains(id), "a receita {id} tem de ser autorada na cena");
    }
    // O `Limit` só é julgável sob uma fonte: a cena o empilha, não o solta.
    let limiter = s.split("limit_src").nth(1).expect("o Limiter é autorado");
    assert!(
        limiter.contains("\"sway\"") && limiter.contains("\"limit\""),
        "o modificador tem de vir EMPILHADO sob uma fonte — sozinho ele não anima nada"
    );
}

/// **Var PRÓPRIA, não um número a mais no interruptor da outra cena.**
#[test]
fn the_group_scene_has_its_own_switch() {
    // ⚠️ Só as linhas que LEEM o ambiente: o doc-comment cita a outra variável de
    // propósito (para dizer por que ela NÃO é usada), e um gate que varre a prosa junto
    // falha sobre uma explicação correta — foi o que ele fez na primeira rodada.
    let text = src();
    let reads: Vec<&str> = text
        .lines()
        .filter(|l| l.contains("std::env::var"))
        .map(str::trim)
        .collect();
    assert_eq!(reads.len(), 1, "a cena lê UMA variável: {reads:?}");
    assert!(
        reads[0].contains("PH2D_EXPR_GROUP_SMOKE"),
        "e é a do grupo — dividir o interruptor com a cena dos instrumentos seria duas \
         perguntas num botão: {reads:?}"
    );
}
