//! ⭐⭐⭐ **A LISTA DOS MODIFICADORES É COMPLETA** — e até 2026-08-30 nada o provava.
//!
//! # ⛔ O buraco que isto fecha
//!
//! `UnaryKind::ALL` é um array de tamanho **fixo**. Isso só é erro de compilação para quem mexe no
//! número: acrescentar uma variante ao `enum` e esquecer a lista **compila limpo**, e o modificador
//! nasce **inalcançável** — sem chip no painel, sem slot de intent, sem clique.
//!
//! ⚠️ **E não é hipótese.** A corrente que o compilador de facto fecha é
//! `Unary::kind()` → `UnaryKind` → `born()` → `key()`; **nada** nela toca na `ALL`. Os consumidores
//! que herdam a cegueira são o `mods_for` do painel, o `ToggleMod` do intent, o censo do
//! `field3d_notice_tests` e o `the_specialisation_gives_up_under_every_modifier_that_remaps_coordinates`
//! — todos derivados, todos verdes.
//!
//! *É a mesma família do defeito que a paleta de formas pagou, um andar acima.*

use ph2d_field::UnaryKind;

/// A metade que torna a `ALL` provável: o [`UnaryKind::index`] é um `match` **exaustivo**, logo uma
/// variante nova não compila até alguém lhe dar posição — e aqui os dois lados encontram-se.
#[test]
fn every_modifier_kind_is_in_the_list() {
    for (i, k) in UnaryKind::ALL.iter().enumerate() {
        assert_eq!(
            k.index(),
            i,
            "{k:?} está na posição {i} da `ALL` e diz ser a {} — as duas respostas divergiram",
            k.index()
        );
    }
    // ⛔ **O outro lado**: sem isto, uma variante nova com `index()` escrito e ausente da `ALL`
    // passaria (o laço acima só varre o que a lista tem).
    let maior = UnaryKind::ALL
        .iter()
        .map(|k| k.index())
        .max()
        .expect("a lista não pode estar vazia");
    assert_eq!(
        maior + 1,
        UnaryKind::ALL.len(),
        "a maior posição é {maior} e a lista tem {} entradas — falta um modificador na `ALL`, e ele \
         nasceu INALCANÇÁVEL: sem chip, sem slot e sem clique",
        UnaryKind::ALL.len()
    );
}

/// ⭐ **Cada natureza sabe nomear-se, e cada NÚMERO dela também** — uma chave sem tradução pinta a
/// própria chave na tela.
///
/// ⚠️ O gate irmão do shell (`every_dimension_name_has_a_translation`) **não** cobre isto: ele varre
/// os nós das cenas de smoke, e nenhuma delas tem modificador ⇒ nenhuma linha de modificador chega
/// ao retrato. *Um censo que não instancia a coisa nova não a defende.*
#[test]
fn every_modifier_kind_names_itself_and_its_numbers() {
    for k in UnaryKind::ALL {
        let key = k.key();
        assert!(
            key.starts_with("panel.model3d.mod."),
            "{k:?}: a chave `{key}` saiu da família do painel"
        );
        assert_ne!(
            ph2d_i18n::tr(key),
            key,
            "{k:?}: sem tradução, o botão diz `{key}` na tela"
        );
        for d in ph2d_field::Unary::born(k, 0.5, [0.5; 3]).dims() {
            assert_ne!(
                ph2d_i18n::tr(d.key),
                d.key,
                "{k:?}: a linha de número diz `{}` na tela",
                d.key
            );
        }
    }
}
