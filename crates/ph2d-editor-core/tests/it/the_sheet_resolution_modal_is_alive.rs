//! **O modal de resolução da folha, do clique ao pedido.**
//!
//! Enio, 2026-08-19: *"Ao criar uma sheet um modal com a resolução deve aparecer antes da
//! criação"*. Este ficheiro fixa as três coisas que fazem esse modal existir de facto, e cada uma
//! já falhou neste repositório noutro modal:
//!
//! 1. **Toda resolução da tabela tem estado no store** — sem ele o botão é pintado, hit-registered
//!    e **morto sob o rato** (o `4096` do New Image, Enio 2026-07-26: *"não aceita ser
//!    selecionado"*). O irmão deste gate é
//!    [`every_new_image_choice_is_alive_under_the_mouse`], e a lição herda-se aqui **iterando a
//!    tabela**, nunca listando ids.
//! 2. **Escolher deixa a escolha onde o Create a lê.**
//! 3. **O Create arma o pedido E fecha o modal**, e — a metade que engana — *cancelar não arma
//!    pedido nenhum*. Um modal que armasse o pedido ao abrir criaria a folha ao ser cancelado.
//!
//! [`every_new_image_choice_is_alive_under_the_mouse`]: ../every_new_image_choice_is_alive_under_the_mouse.rs

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{ContextMenuKind, WidgetEvent};
use ph2d_editor_core::screens::hero::HeroScreen;

#[test]
fn every_sheet_resolution_has_interactive_state() {
    let store = HeroScreen::new(NodeId(1)).store;
    // Controle positivo: sem isto o laço passa por não iterar nada.
    assert!(
        ids::CTX_MENU_SHEET_SIZES.len() >= 7,
        "controle: a tabela de resolucoes encolheu ({})",
        ids::CTX_MENU_SHEET_SIZES.len()
    );
    for (px, id) in ids::CTX_MENU_SHEET_SIZES {
        assert!(
            store.get(id).is_some(),
            "a resolucao {px} esta na tabela (logo o modal a PINTA e o handler a aceita) mas nao \
             tem estado no store — nasce nao-focavel, nunca vira `active` no Down e nunca emite \
             `Click`: pintada, hit-registered e MORTA sob o rato"
        );
    }
    assert!(
        store.get(ids::CTX_MENU_SHEET_SIZE_CREATE).is_some(),
        "o Create do modal tem de estar no store"
    );
}

/// ⚠️ **A maior resolução, de propósito** — é a que um teto escrito à mão noutro sítio cortaria
/// primeiro, e a que o modal precisa de oferecer para uma folha de atlas real.
#[test]
fn choosing_a_resolution_lands_in_the_store() {
    let mut hero = HeroScreen::new(NodeId(1));
    let (px, id) = *ids::CTX_MENU_SHEET_SIZES
        .last()
        .expect("a tabela de resolucoes nao pode estar vazia");
    assert!(
        hero.store.sheet_size() != px,
        "controle: a maior resolucao nao pode ja ser o default, senao o assert abaixo passa sem o clique"
    );
    assert!(hero.apply_event(WidgetEvent::Click(id)));
    assert_eq!(hero.store.sheet_size(), px);
}

/// O modal abre semeado com a SUGESTÃO, não com a última escolha — é isso que faz aceitar o que o
/// app propõe ser o caminho certo.
#[test]
fn the_modal_opens_seeded_with_the_suggestion() {
    let mut hero = HeroScreen::new(NodeId(1));
    hero.store.set_sheet_size(4096);
    hero.store.open_sheet_size_dialog(256);
    assert_eq!(hero.store.sheet_size(), 256, "a abertura semeia a sugestao");
    assert!(matches!(
        hero.store.context_menu().map(|r| r.kind),
        Some(ContextMenuKind::SheetSizeDialog)
    ));
}

#[test]
fn create_arms_the_request_and_closes_the_modal() {
    let mut hero = HeroScreen::new(NodeId(1));
    hero.store.open_sheet_size_dialog(512);
    assert!(hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_SHEET_SIZE_CREATE)));
    assert_eq!(hero.store.take_sheet_size_request(), Some(512));
    assert_eq!(
        hero.store.take_sheet_size_request(),
        None,
        "drenado uma vez so'"
    );
    assert!(
        hero.store.context_menu().is_none(),
        "o Create fecha o modal"
    );
}

/// ⚠️ **A metade que engana.** Abrir o modal e fechá-lo sem Create não pode deixar pedido nenhum —
/// senão a folha nasceria de um modal cancelado, e o artista veria o app fazer o que ele acabou de
/// desistir de pedir. *Cancelar tem de ser a AUSÊNCIA de um pedido.*
#[test]
fn cancelling_the_modal_arms_nothing() {
    let mut hero = HeroScreen::new(NodeId(1));
    hero.store.open_sheet_size_dialog(512);
    // Escolher uma resolução também não arma: só o Create arma.
    assert!(hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_SHEET_SIZES[0].1)));
    hero.store.close_context_menu();
    assert_eq!(hero.store.take_sheet_size_request(), None);
}

/// As duas caixas de diálogo lembram-se **em separado**: escolher a resolução de uma folha não
/// pode mudar o tamanho que o Cmd+N oferece a seguir. É o motivo de os ids serem próprios.
#[test]
fn the_sheet_modal_and_the_new_image_modal_do_not_share_memory() {
    let mut hero = HeroScreen::new(NodeId(1));
    let before = hero.store.new_image_size();
    let (px, id) = *ids::CTX_MENU_SHEET_SIZES.last().expect("tabela nao vazia");
    assert!(hero.apply_event(WidgetEvent::Click(id)));
    assert_eq!(hero.store.sheet_size(), px);
    assert_eq!(
        hero.store.new_image_size(),
        before,
        "escolher a resolucao da folha nao pode mexer no tamanho do New Image"
    );
}
