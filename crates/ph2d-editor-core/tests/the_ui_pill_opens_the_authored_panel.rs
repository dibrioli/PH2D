//! **O pill UI abre o painel que o artista desenhou** — e é a MESMA visibilidade que o chip
//! *Show as Panel* escreve (plano UI/UX W8b.3).
//!
//! Irmão exacto do `the_tokens_pill_opens_the_tokens_panel`, e pelas mesmas maneiras de falhar em
//! silêncio:
//!
//! 1. o cluster é pintado e o `dispatch_all` não o consome — botão morto;
//! 2. ele consome, mas escreve num bool PRÓPRIO — e aí o pill diz *fechado* sobre um painel que o
//!    chip da seção Frame abriu;
//! 3. ⚠️ e a terceira, que agora tem **TRÊS** irmãos: um `if id != …` copiado sem trocar o id faz
//!    dois handlers responderem ao mesmo clique, e o pill UI passaria a abrir o painel de física.
//!    O repo já pagou esta exacta lição quando o TOK nasceu de um `cp` do PHYS.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::{HeroScreen, chrome, fixture, ids};

fn fresh() -> HeroScreen {
    HeroScreen::new(NodeId(1))
}

/// **O pill existe e fica ao lado do TOK** — os abridores sem chip no rail, juntos.
#[test]
fn the_pill_is_painted_next_to_the_tokens_one() {
    let clusters = fixture::topbar_clusters();
    let tok = clusters
        .iter()
        .position(|(id, _)| *id == ids::TOPBAR_TOKENS)
        .expect("o pill TOK existe");
    let ui = clusters
        .iter()
        .position(|(id, _)| *id == ids::TOPBAR_AUTHORED)
        .expect("o pill UI existe");
    assert_eq!(
        ui,
        tok + 1,
        "os abridores sem chip no rail ficam juntos — se o UI escorregar, ele vai parar noutro grupo"
    );
}

/// **Clicar o pill ABRE e FECHA a mesma visibilidade que o chip escreve.**
#[test]
fn clicking_the_pill_toggles_the_panel() {
    let mut hero = fresh();
    assert!(!hero.is_panel_visible("authored"), "nasce fechado");

    assert!(
        chrome::dispatch_all(&mut hero, WidgetEvent::Click(ids::TOPBAR_AUTHORED)),
        "o clique tem de ser CONSUMIDO por alguém — senão o pill é botão morto"
    );
    assert!(hero.is_panel_visible("authored"), "o clique tem de ABRIR");

    chrome::dispatch_all(&mut hero, WidgetEvent::Click(ids::TOPBAR_AUTHORED));
    assert!(
        !hero.is_panel_visible("authored"),
        "e o segundo tem de FECHAR"
    );
}

/// **O pill LÊ a visibilidade, nunca uma cópia própria.**
///
/// ⚠️ A metade que separa *"o botão funciona"* de *"o botão diz a verdade"*: abrir por FORA (o
/// chip da seção Frame, ou o X do painel) e depois clicar o pill tem de FECHAR. Com um bool
/// próprio o pill acharia que está fechado e o clique abriria um painel já aberto — nada
/// aconteceria, e o artista clicaria duas vezes para fechar.
#[test]
fn the_pill_reads_the_visibility_instead_of_a_copy_of_its_own() {
    let mut hero = fresh();
    // Alguém abriu por fora — exactamente o que o chip `Show as Panel` faz.
    hero.panel_visibility.insert("authored", true);

    chrome::dispatch_all(&mut hero, WidgetEvent::Click(ids::TOPBAR_AUTHORED));
    assert!(
        !hero.is_panel_visible("authored"),
        "o pill tem de FECHAR o que outro abridor abriu — ele esta' a ler uma copia propria"
    );
}

/// **Cada pill abre SÓ o painel dele.**
///
/// ⚠️ O gate do `cp`: com três handlers irmãos quase idênticos, um `if id != …` que sobreviva à
/// cópia faz o clique num abrir o painel do outro — e os dois primeiros gates deste arquivo ficam
/// VERDES, porque o painel certo também abre.
#[test]
fn each_pill_opens_only_its_own_panel() {
    let mut hero = fresh();
    chrome::dispatch_all(&mut hero, WidgetEvent::Click(ids::TOPBAR_AUTHORED));
    assert!(hero.is_panel_visible("authored"));
    assert!(
        !hero.is_panel_visible("physics"),
        "o pill UI abriu o painel de FISICA — o `if id !=` ficou com o id do irmao"
    );
    assert!(
        !hero.is_panel_visible("tokens"),
        "o pill UI abriu o painel de TOKENS — o `if id !=` ficou com o id do irmao"
    );

    // E a recíproca: os irmãos não abrem o autorado.
    for (pill, what) in [(ids::TOPBAR_TOKENS, "TOK"), (ids::TOPBAR_PHYSICS, "PHYS")] {
        let mut h = fresh();
        chrome::dispatch_all(&mut h, WidgetEvent::Click(pill));
        assert!(
            !h.is_panel_visible("authored"),
            "o pill {what} abriu o painel AUTORADO"
        );
    }
}
