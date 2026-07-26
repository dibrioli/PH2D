//! **Todo tamanho do modal New Image tem de estar VIVO sob o mouse.**
//!
//! ⚠️ Este gate existe porque o defeito aconteceu: acrescentar `4096` à tabela
//! [`ids::CTX_MENU_NEW_IMAGE_SIZES`] deu um botão **pintado, hit-registered e morto** — Enio,
//! 2026-07-26: *"não aceita ser selecionado"*. A tabela era a fonte única do **paint**, do **dispatch**
//! e do **handler de clique**, e o QUARTO consumidor — o `pre_populate`, que dá ao id o
//! `InteractiveState` sem o qual ele nunca vira `active` no Down nem emite `Click` — **enumerava os ids
//! à mão**.
//!
//! É a forma canônica do apodrecimento neste repo: *uma condição que ENUMERA seus leitores apodrece*.
//! O `pre_populate` agora ITERA as tabelas; este gate garante que a próxima pessoa que trocar a
//! iteração por uma lista à mão descubra na hora, e não num smoke.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::screens::hero::HeroScreen;

/// O store que o app de fato monta — não um construído à mão para o teste.
fn populated_store() -> WidgetStore {
    HeroScreen::new(NodeId(1)).store
}

#[test]
fn every_size_and_background_choice_has_interactive_state() {
    let store = populated_store();

    // Controle positivo: as tabelas têm de ter conteúdo, senão o laço abaixo passa por não iterar nada.
    assert!(
        ids::CTX_MENU_NEW_IMAGE_SIZES.len() >= 8,
        "controle: a tabela de tamanhos ficou menor do que o modal ja ofereceu ({})",
        ids::CTX_MENU_NEW_IMAGE_SIZES.len()
    );
    assert!(!ids::CTX_MENU_NEW_IMAGE_BGS.is_empty(), "controle: bgs");

    for (px, id) in ids::CTX_MENU_NEW_IMAGE_SIZES {
        assert!(
            store.get(id).is_some(),
            "o tamanho {px} esta na tabela (logo o modal o PINTA e o handler de clique o aceita) mas \
             nao tem estado no store — ele nasce focavel-nao, nunca vira `active` no Down e nunca \
             emite `Click`: pintado, hit-registered e MORTO sob o mouse"
        );
    }
    for (bg, id) in ids::CTX_MENU_NEW_IMAGE_BGS {
        assert!(
            store.get(id).is_some(),
            "o background {bg} esta na tabela e nao tem estado no store (ver a mensagem acima)"
        );
    }
    // E o Create, que não tem tabela e por isso segue listado à mão.
    assert!(
        store.get(ids::CTX_MENU_NEW_IMAGE_CREATE).is_some(),
        "o Create do modal tem de estar no store"
    );
}

/// **E o 4096 chega ao STORE quando clicado** — a outra metade: estar registrado é necessário, levar a
/// algum lugar é o que o artista pediu.
#[test]
fn choosing_the_largest_size_lands_in_the_store() {
    let mut hero = HeroScreen::new(NodeId(1));
    let (px, id) = *ids::CTX_MENU_NEW_IMAGE_SIZES
        .last()
        .expect("a tabela de tamanhos nao pode estar vazia");
    assert!(
        hero.store.new_image_size() != px,
        "controle: o maior tamanho nao pode ja ser o default, senao o assert abaixo passa sem o clique"
    );
    let handled = hero.apply_event(ph2d_editor_core::interaction::WidgetEvent::Click(id));
    assert!(handled, "o clique no tamanho {px} tem de ser consumido");
    assert_eq!(
        hero.store.new_image_size(),
        px,
        "escolher {px} tem de deixar {px} no store — e o que o Create le"
    );
}
