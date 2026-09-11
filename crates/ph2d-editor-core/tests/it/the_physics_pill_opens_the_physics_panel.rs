//! **O pill PHYS abre o painel de física** — e é a MESMA visibilidade da tecla `W`.
//!
//! ⚠️ Um pill pintado que ninguém despacha é o apodrecimento que este repo já
//! pegou várias vezes (o Redo da barra, os 36 checkboxes da matriz de camadas, os
//! dez chips da lista de TOOL do Painter). Aqui há **duas** maneiras de falhar em
//! silêncio, e cada uma tem uma metade deste gate:
//!
//! 1. o cluster é pintado e o `dispatch_all` não o consome — botão morto;
//! 2. ele consome, mas escreve num bool PRÓPRIO — e aí o pill diz *fechado*
//!    sobre um painel que a tecla `W` abriu.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::{HeroScreen, chrome, fixture, ids};

fn hero() -> HeroScreen {
    HeroScreen::new(NodeId(1))
}

#[test]
fn the_pill_is_painted_next_to_the_image_tools_one() {
    let clusters = fixture::topbar_clusters();
    let img = clusters
        .iter()
        .position(|(id, _)| *id == ids::TOPBAR_IMAGE_TOOLS)
        .expect("o pill IMG existe");
    let phys = clusters
        .iter()
        .position(|(id, _)| *id == ids::TOPBAR_PHYSICS)
        .expect("o pill PHYS existe");
    assert_eq!(
        phys,
        img + 1,
        "o pedido foi 'ao lado de IMG' — se ele escorregar, o pill vai parar noutro grupo"
    );
}

#[test]
fn clicking_the_pill_toggles_the_same_visibility_the_w_key_writes() {
    let mut hero = hero();
    assert!(!hero.is_panel_visible("physics"), "nasce fechado");

    assert!(
        chrome::dispatch_all(&mut hero, WidgetEvent::Click(ids::TOPBAR_PHYSICS)),
        "o clique tem de ser CONSUMIDO por alguém — senão o pill é botão morto"
    );
    assert!(hero.is_panel_visible("physics"), "o clique tem de ABRIR");

    assert!(chrome::dispatch_all(
        &mut hero,
        WidgetEvent::Click(ids::TOPBAR_PHYSICS)
    ));
    assert!(
        !hero.is_panel_visible("physics"),
        "e o segundo tem de FECHAR"
    );
}

#[test]
fn the_pill_reads_the_visibility_it_does_not_keep_its_own() {
    // A tecla `W` do shell escreve `panel_visibility["physics"]` direto. Se o
    // pill guardasse um bool próprio, ele ficaria dizendo o contrário do que a
    // tela mostra — o modo de falha exato de "duas portas para um fato".
    let mut hero = hero();
    hero.panel_visibility.insert("physics", true);
    assert!(chrome::dispatch_all(
        &mut hero,
        WidgetEvent::Click(ids::TOPBAR_PHYSICS)
    ));
    assert!(
        !hero.is_panel_visible("physics"),
        "o pill inverteu o bool de outra pessoa, então ele o LÊ — como deve"
    );
}
