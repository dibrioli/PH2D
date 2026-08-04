//! **O pill TOK abre o painel de Tokens** — e é a MESMA visibilidade da tecla `T`.
//!
//! Irmão exacto do `the_physics_pill_opens_the_physics_panel`, e pelas mesmas duas maneiras de
//! falhar em silêncio:
//!
//! 1. o cluster é pintado e o `dispatch_all` não o consome — botão morto;
//! 2. ele consome, mas escreve num bool PRÓPRIO — e aí o pill diz *fechado* sobre um painel que a
//!    tecla `T` abriu.
//!
//! ⚠️ **E há uma terceira, que só aparece com dois pills irmãos:** um `if id != …` copiado sem
//! trocar o id faz os DOIS handlers responderem ao mesmo clique, e o pill TOK passaria a abrir o
//! painel de física. O último gate deste arquivo é essa metade.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::{HeroScreen, chrome, fixture, ids};

fn hero() -> HeroScreen {
    HeroScreen::new(NodeId(1))
}

/// **O pill existe e fica ao lado do PHYS** — os dois painéis de MUNDO juntos.
#[test]
fn the_pill_is_painted_next_to_the_physics_one() {
    let clusters = fixture::topbar_clusters();
    let phys = clusters
        .iter()
        .position(|(id, _)| *id == ids::TOPBAR_PHYSICS)
        .expect("o pill PHYS existe");
    let tok = clusters
        .iter()
        .position(|(id, _)| *id == ids::TOPBAR_TOKENS)
        .expect("o pill TOK existe");
    assert_eq!(
        tok,
        phys + 1,
        "os dois painéis de MUNDO ficam juntos — se o TOK escorregar, ele vai parar noutro grupo"
    );
}

#[test]
fn clicking_the_pill_toggles_the_same_visibility_the_t_key_writes() {
    let mut hero = hero();
    assert!(!hero.is_panel_visible("tokens"), "nasce fechado");

    assert!(
        chrome::dispatch_all(&mut hero, WidgetEvent::Click(ids::TOPBAR_TOKENS)),
        "o clique tem de ser CONSUMIDO por alguém — senão o pill é botão morto"
    );
    assert!(hero.is_panel_visible("tokens"), "o clique tem de ABRIR");

    assert!(chrome::dispatch_all(
        &mut hero,
        WidgetEvent::Click(ids::TOPBAR_TOKENS)
    ));
    assert!(
        !hero.is_panel_visible("tokens"),
        "e o segundo tem de FECHAR"
    );
}

#[test]
fn the_pill_reads_the_visibility_it_does_not_keep_its_own() {
    // A tecla `T` do shell escreve `panel_visibility["tokens"]` direto. Se o pill guardasse um
    // bool próprio, ele ficaria a dizer o contrário do que a tela mostra.
    let mut hero = hero();
    hero.panel_visibility.insert("tokens", true);
    assert!(chrome::dispatch_all(
        &mut hero,
        WidgetEvent::Click(ids::TOPBAR_TOKENS)
    ));
    assert!(
        !hero.is_panel_visible("tokens"),
        "o pill inverteu o bool de outra pessoa, então ele o LÊ — como deve"
    );
}

/// **Cada pill abre O SEU painel** — a metade que só existe porque agora há dois irmãos.
///
/// ⚠️ O `tokens_toggle` nasceu de um `cp` do `physics_toggle`; um `if id != ids::TOPBAR_PHYSICS`
/// que sobrevivesse à cópia faria os dois handlers responderem ao mesmo clique, e o pill novo
/// abriria o painel velho. Os três gates acima ficariam VERDES (o `dispatch_all` consome, e o
/// bool de tokens continuaria a ser escrito pelo handler certo... até alguém trocar a ordem).
#[test]
fn each_pill_opens_only_its_own_panel() {
    let mut tok = hero();
    assert!(chrome::dispatch_all(
        &mut tok,
        WidgetEvent::Click(ids::TOPBAR_TOKENS)
    ));
    assert!(tok.is_panel_visible("tokens"));
    assert!(
        !tok.is_panel_visible("physics"),
        "o pill TOK abriu o painel de FÍSICA junto"
    );

    let mut other = hero();
    assert!(chrome::dispatch_all(
        &mut other,
        WidgetEvent::Click(ids::TOPBAR_PHYSICS)
    ));
    assert!(other.is_panel_visible("physics"));
    assert!(
        !other.is_panel_visible("tokens"),
        "o pill PHYS abriu o painel de TOKENS junto"
    );
}
