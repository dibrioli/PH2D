//! Os gates de [`super::axes_for`] — as fileiras saem de DADO, e o nome não decide nada.

use super::{VariantAxis, VariantMember, axes_for};
use std::collections::BTreeMap;

fn m(master: u64, name: &str, pairs: &[(&str, &str)]) -> VariantMember {
    VariantMember {
        master,
        name: name.to_string(),
        values: pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect::<BTreeMap<_, _>>(),
    }
}

fn labels(a: &VariantAxis) -> Vec<&str> {
    a.options.iter().map(|o| o.label.as_str()).collect()
}

/// ⭐⭐⭐ **Uma propriedade declarada vira UMA fileira com os valores distintos** — e o valor da
/// versão vigente é o que está aceso.
///
/// (Mutação: `row_for` devolver `None` sempre ⇒ RED.)
#[test]
fn a_declared_property_becomes_one_row_of_its_distinct_values() {
    let fam = [
        m(1, "Casa", &[("Size", "Small")]),
        m(2, "Casa Variant", &[("Size", "Big")]),
    ];
    let (rows, beyond) = axes_for(&fam, 1);
    assert_eq!(beyond, 0);
    assert_eq!(rows.len(), 1, "uma propriedade, uma fileira");
    assert_eq!(rows[0].name, "Size");
    assert_eq!(labels(&rows[0]), ["Small", "Big"]);
    assert!(rows[0].options[0].current, "a vigente e' a do «me»");
    assert!(!rows[0].options[1].current);
    assert_eq!(
        rows[0].options[1].master, 2,
        "o chip leva a IDENTIDADE do alvo"
    );
}

/// ⛔⛔ **O NOME não decide nada** — a mesma família com nomes trocados dá a MESMA fileira.
///
/// É a ordem do Enio de 2026-09-01 escrita como gate. ⚠️ **A fixtura tem de carregar o fenómeno**:
/// os nomes trazem chaves, que é o que a lei velha lia. *Com nomes limpos isto provaria nada.*
#[test]
fn renaming_anything_never_changes_a_row() {
    let a = [
        m(1, "Casa {Size=Zzz}", &[("Size", "Small")]),
        m(2, "Bob {Size=Nada, State=Nada}", &[("Size", "Big")]),
    ];
    let b = [
        m(1, "outro nome totalmente diferente", &[("Size", "Small")]),
        m(2, "e outro", &[("Size", "Big")]),
    ];
    assert_eq!(axes_for(&a, 1), axes_for(&b, 1));
}

/// ⛔⛔ **Uma fileira com UM valor não é oferecida** — um chip único é um controlo que não escolhe
/// nada, e como a fileira é derivada ela desaparece sozinha quando os valores concordam.
#[test]
fn a_row_with_a_single_value_is_never_offered() {
    let fam = [
        m(1, "Casa", &[("Size", "Small"), ("Tag", "City")]),
        m(2, "Outra", &[("Size", "Big"), ("Tag", "City")]),
    ];
    let (rows, _) = axes_for(&fam, 1);
    assert_eq!(
        rows.len(),
        1,
        "o `Tag` concorda nas duas — nao ha' o que escolher"
    );
    assert_eq!(rows[0].name, "Size");
}

/// ⭐⭐⭐ **Numa GRELHA o alvo é quem concorda em TUDO menos nesta chave.**
///
/// Sem esta lei, `Color=Red` sozinho é ambíguo — há um por cada `Size` — e o chip levaria o artista
/// para outra linha da grelha sem que nada o dissesse.
///
/// (Mutação: trocar `matches_except` por «tem este valor» ⇒ RED no `master` esperado.)
#[test]
fn in_a_grid_the_target_agrees_on_every_other_key() {
    let fam = [
        m(1, "a", &[("Size", "Small"), ("Color", "Normal")]),
        m(2, "b", &[("Size", "Small"), ("Color", "Red")]),
        m(3, "c", &[("Size", "Big"), ("Color", "Normal")]),
        m(4, "d", &[("Size", "Big"), ("Color", "Red")]),
    ];
    // Estou na `Big / Normal`: carregar em `Red` tem de levar à `Big / Red`, nunca à `Small / Red`.
    let (rows, _) = axes_for(&fam, 3);
    let color = rows
        .iter()
        .find(|r| r.name == "Color")
        .expect("a fileira Color");
    let red = color
        .options
        .iter()
        .find(|o| o.label == "Red")
        .expect("o chip Red");
    assert_eq!(red.master, 4);
    assert!(!red.missing);
}

/// ⭐⭐⭐ **A combinação que não existe vem MARCADA, nunca omitida.**
///
/// Omiti-la esconderia metade da grelha; dá-la a um alvo qualquer faria o app acender um valor e
/// mostrar outro. O painel pinta-a esmaecida com `+`, e o clique cria-a.
#[test]
fn a_combination_that_does_not_exist_comes_back_marked() {
    let fam = [
        m(1, "a", &[("Size", "Small"), ("Color", "Normal")]),
        m(2, "b", &[("Size", "Small"), ("Color", "Red")]),
        m(3, "c", &[("Size", "Big"), ("Color", "Normal")]),
    ];
    let (rows, _) = axes_for(&fam, 3);
    let color = rows
        .iter()
        .find(|r| r.name == "Color")
        .expect("a fileira Color");
    let red = color
        .options
        .iter()
        .find(|o| o.label == "Red")
        .expect("o chip Red");
    assert!(red.missing, "«Big / Red» nao existe");
    assert_eq!(red.master, 0, "e nao aponta para ninguem");
}

/// ⭐ **Sem declaração nenhuma o modo é PLANO** — um chip por versão, rotulado pelo nome dela.
///
/// ⚠️ Mostrar o nome não é a lei velha: aqui ele é RÓTULO de uma versão que não declara nada, e
/// ninguém o lê para decidir. *A doença era o nome ser mecanismo.*
#[test]
fn with_no_declarations_the_mode_is_flat() {
    let fam = [m(1, "Casa", &[]), m(2, "Casa Variant", &[])];
    let (rows, _) = axes_for(&fam, 2);
    assert_eq!(rows.len(), 1);
    assert!(
        rows[0].name.is_empty(),
        "a fileira plana nao tem nome (HR-15)"
    );
    assert_eq!(labels(&rows[0]), ["Casa", "Casa Variant"]);
    assert!(rows[0].options[1].current);
}

/// ⛔ **Uma versão sozinha não oferece nada** — e é o caso comum de uma receita simples.
#[test]
fn a_lone_recipe_offers_nothing() {
    let fam = [m(1, "Casa", &[("Size", "Small")])];
    assert_eq!(axes_for(&fam, 1), (Vec::new(), 0));
}

/// ⛔ **A versão vigente tem de ser da família** — senão não há combinação de onde perguntar, e
/// responder com a da base acenderia um chip que descreve outro objecto.
#[test]
fn a_current_version_outside_the_family_offers_nothing() {
    let fam = [
        m(1, "a", &[("Size", "Small")]),
        m(2, "b", &[("Size", "Big")]),
    ];
    assert_eq!(axes_for(&fam, 999), (Vec::new(), 0));
}

/// ⛔ **O que passa do teto da tabela de ids é ESCRITO, nunca truncado em silêncio.**
#[test]
fn the_rows_beyond_the_id_table_are_counted() {
    let keys = ["A", "B", "C", "D", "E", "F"];
    let one: Vec<(&str, &str)> = keys.iter().map(|k| (*k, "x")).collect();
    let two: Vec<(&str, &str)> = keys.iter().map(|k| (*k, "y")).collect();
    let fam = [m(1, "a", &one), m(2, "b", &two)];
    let (rows, beyond) = axes_for(&fam, 1);
    assert_eq!(rows.len(), crate::ids::MAX_INSTANCE_AXES);
    assert_eq!(beyond, keys.len() - crate::ids::MAX_INSTANCE_AXES);
}

/// ⚠️ **A quem falta a chave, falta** — não é «igual nas outras». Sem isto, uma receita que ainda
/// não declara `Color` seria alvo de todo chip de `Color`.
#[test]
fn a_member_missing_the_key_is_not_a_match() {
    let fam = [
        m(1, "a", &[("Size", "Small"), ("Color", "Normal")]),
        m(2, "b", &[("Size", "Small")]),
        m(3, "c", &[("Size", "Big"), ("Color", "Red")]),
    ];
    let (rows, _) = axes_for(&fam, 1);
    let color = rows
        .iter()
        .find(|r| r.name == "Color")
        .expect("a fileira Color");
    let red = color
        .options
        .iter()
        .find(|o| o.label == "Red")
        .expect("o chip Red");
    assert!(
        red.missing,
        "a `c` e' de outro Size, e a `b` nao declara Color"
    );
}
