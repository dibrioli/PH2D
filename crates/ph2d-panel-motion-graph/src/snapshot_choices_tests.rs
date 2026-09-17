//! Os gates da [`super::CardChoices`] — irmãos por RESPONSABILIDADE (HR-18) do módulo que
//! publica as escolhas de cada cartão.
//!
//! ⛔ Eles saíram do `snapshot.rs` por **tecto de LOC** (`608` contra `600`) quando a 5.ª
//! fatia do HR-15 os trouxe — a cura de um tecto é o corte, nunca uma entrada nova de dívida.

use super::CardChoices;

/// ⭐⭐⭐ **A LISTA DE OPÇÕES DO CARTÃO ABRE COM PALAVRAS, NÃO COM IDENTIFICADORES.**
///
/// ⚠️ Esta é a **terceira** superfície do mesmo array e não passa por nenhuma das outras
/// duas: o painel resolve no pintor dele, o estado do cartão resolve no `paint_card_params`,
/// e a LISTA resolve aqui. ⛔ Um `tr` em falta neste método deixa os outros dois certos e põe
/// `node.motion.wave.param.edges.0` na lista que o artista abre — *três consumidores do mesmo
/// array são três sítios onde o `tr` pode faltar.*
///
/// ⭐ E a tradução mora aqui porque é de GRAÇA: este método corre **ao abrir a lista**, não
/// por quadro — o que a `Live` faz do outro lado já era uma cópia.
#[test]
fn a_static_choice_list_opens_with_words() {
    // Uma chave REAL do catálogo: uma fixtura inventada mediria uma tabela que ninguém ship.
    const K: &[&str] = &[
        "node.motion.wave.param.edges.0",
        "node.motion.wave.param.edges.1",
    ];
    // ⛔ Controlo da fixtura: se a chave deixasse de resolver, os dois lados seriam iguais
    // **por serem os dois identificadores**, e o gate passava a medir nada.
    for k in K {
        assert_ne!(
            ph2d_i18n::tr(k),
            *k,
            "a chave {k:?} não resolve — a fixtura deixou de conter o fenómeno"
        );
    }
    let saiu = CardChoices::Static(K).labels();
    assert_eq!(
        saiu,
        K.iter()
            .map(|k| ph2d_i18n::tr(k).to_string())
            .collect::<Vec<_>>(),
        "a lista abriu com os identificadores em vez das palavras"
    );
    assert!(
        saiu.iter().all(|s| !s.starts_with("node.")),
        "a lista abriu com chaves: {saiu:?}"
    );
}

/// ⚠️ **E a metade VIVA não é tocada** — ela já são palavras (formas que o artista desenhou,
/// colunas que a corrente cozinhou), e passá-las pelo `tr` faria `leak_key` (`Box::leak`)
/// sobre cada uma.
#[test]
fn a_live_choice_list_is_handed_through_untouched() {
    let vivas = vec!["Bezier 3".to_string(), "node.nao.e.chave".to_string()];
    assert_eq!(CardChoices::Live(vivas.clone()).labels(), vivas);
}
