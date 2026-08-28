//! Os gates do [`super::walk`] — a convenção do lado e a lei do passe mútuo.
//!
//! ⚠️ **Separado por tecto de LOC**, e o corte é o de sempre nesta casa: o teste sai, o
//! algoritmo fica.

use super::{WalkStats, mutual_links, on_edge_side};
use crate::exact::Xf;

/// ⭐⭐⭐ **SÓ SE LIGA O QUE CADA LADO NOMEIA.**
///
/// ⛔⛔⛔ É o gate do desempate da §23.30, e ele existe porque as **três** tentativas
/// anteriores falharam todas do mesmo modo: ligavam a candidata assim que ela aparecia,
/// e algumas eram as **erradas** (a `sculpt_hooked` ia de `χ = 0` para `−1`).
///
/// ⭐ A lei é sobre a TABELA de candidatas, não sobre a malha — por isso este gate não
/// precisa de fixtura, e por isso ele mata a mutação que o resto da wave não mataria.
#[test]
fn only_pairs_that_name_each_other_are_linked() {
    let x = Xf::IDENTITY;
    // `0 ↔ 1` mútuo · `2 → 3` unilateral (o `3` não nomeia ninguém) · `4 → 5`
    // unilateral (o `5` nomeia o `6`) · `5 ↔ 6` mútuo · `7` sem candidata.
    let cand = [
        vec![(1u32, x)],
        vec![(0u32, x)],
        vec![(3u32, x)],
        Vec::new(),
        // ⭐ O `4` oferece DUAS candidatas: a `5` (que nomeia o `6`) e a `7` (que não
        // nomeia ninguém). *Nenhuma é recíproca ⇒ o `4` não liga*, e é isso que prova que
        // a lista não afrouxa a lei: **oferecer mais não é ligar mais.**
        vec![(5u32, x), (7u32, x)],
        vec![(6u32, x)],
        vec![(5u32, x)],
        Vec::new(),
    ];
    let mut st = WalkStats::default();
    let out = mutual_links(&cand, &mut st);
    let pairs: Vec<(u32, u32)> = out.iter().map(|&(a, b, _)| (a, b)).collect();
    assert_eq!(pairs, vec![(0, 1), (5, 6)], "so' os mutuos ligam");
    assert_eq!(
        st.rescue_mutual, 2,
        "cada par mutuo conta UMA vez, nao duas"
    );
    // ⚠️ O `4` aponta ao `5` e o `5` aponta ao `6` — *ser nomeado não chega, tem de ser
    // recíproco*; e o `5 ↔ 6` continua mútuo apesar de alguém apontar para lá de fora.
    assert_eq!(st.rescue_not_mutual, 2, "o 2 e o 4 nao tem correspondencia");
}

/// ⭐⭐⭐ **A CONVENÇÃO DO LADO — `k` é a aresta do canto `k` para o `k+1`.**
///
/// ⛔⛔ **É ela que indexa [`Topo::twin`] e [`Topo::xf`]**, e um índice trocado manda o
/// resgate perguntar à face errada. ⚠️ *Uma convenção de lado escrita duas vezes é duas
/// convenções, e nenhum tipo as separa: as três são `usize`.*
#[test]
fn the_side_index_is_the_corner_it_starts_from() {
    let tri = [[0, 0], [6, 0], [0, 6]];
    // O ponto médio de cada lado tem de dar o índice desse lado.
    assert_eq!(on_edge_side(tri, [3, 0]), Some(0), "lado 0 = canto 0 -> 1");
    assert_eq!(on_edge_side(tri, [3, 3]), Some(1), "lado 1 = canto 1 -> 2");
    assert_eq!(on_edge_side(tri, [0, 3]), Some(2), "lado 2 = canto 2 -> 0");
}

/// ⭐⭐ **O interior não é aresta nenhuma** — e sem esta metade a lei aceitaria tudo.
#[test]
fn a_point_inside_is_on_no_side() {
    let tri = [[0, 0], [6, 0], [0, 6]];
    assert_eq!(on_edge_side(tri, [1, 1]), None);
    assert_eq!(on_edge_side(tri, [2, 2]), None);
}

/// ⭐⭐⭐ **UM CANTO pertence a DOIS lados, e a resposta é o de índice MENOR.**
///
/// ⚠️ Não é uma preferência: é o que torna a escolha **determinista**. *Um empate
/// resolvido de outra maneira em cada chamada faria o resgate perguntar a uma face
/// diferente a cada corrida* — e o hash da grade é contrato (HR-5).
#[test]
fn a_corner_belongs_to_the_lower_side() {
    let tri = [[0, 0], [6, 0], [0, 6]];
    assert_eq!(on_edge_side(tri, [0, 0]), Some(0), "canto 0: lados 0 e 2");
    assert_eq!(on_edge_side(tri, [6, 0]), Some(0), "canto 1: lados 0 e 1");
    assert_eq!(on_edge_side(tri, [0, 6]), Some(1), "canto 2: lados 1 e 2");
}

/// ⭐⭐ **Um ponto sobre o PROLONGAMENTO de um lado também é colinear** — e a função
/// diz que sim, de propósito.
///
/// ⚠️ **Ela é um predicado de COLINEARIDADE, não de pertença ao segmento**, e quem a
/// chama já sabe que o ponto está *dentro* do triângulo ([`contains`] correu antes).
/// *Dizer isto aqui é mais barato que alguém a reutilizar noutro sítio e descobrir.*
#[test]
fn the_predicate_is_collinearity_not_membership() {
    let tri = [[0, 0], [6, 0], [0, 6]];
    assert_eq!(on_edge_side(tri, [99, 0]), Some(0));
}
