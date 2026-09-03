//! ⭐ **O Inspector está ABERTO ao abrir o app** — Enio, 2026-09-03: *«o inspector deve estar
//! aberto ao abrir o app»*.
//!
//! # ⚠️ O que este gate descobriu
//!
//! Que **já estava**. O `default_panel_visibility()` diz `("inspector", true)` desde sempre, e o
//! botão do rail nasce `Pressed` com um comentário a dizê-lo (*"Both panels start visible"*).
//!
//! ⇒ o valor deste ficheiro **não é mudar o comportamento, é PINÁ-LO**: a visibilidade de arranque
//! vive num mapa que qualquer wave pode editar sem reparar, e o sintoma — *o painel simplesmente
//! não aparece* — não parte teste nenhum. ⛔ *Uma decisão de produto que só existe como um `true`
//! numa tabela é uma decisão que a próxima pessoa apaga por acidente.*
//!
//! ⚠️ E ele afirma as **duas** metades, porque elas podem divergir: o painel visível **e** o botão
//! do rail que o diz. Um rail a mostrar «desligado» sobre um painel aberto é o mesmo defeito, do
//! outro lado.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::widget::ButtonState;

/// **Um `HeroScreen` acabado de nascer tem o Inspector visível.**
///
/// **Mutação que deve sangrar:** pôr `("inspector", false)` no `default_panel_visibility` — o app
/// abriria sem o painel onde vivem todas as propriedades, e nada mais o acusaria.
#[test]
fn a_fresh_app_opens_with_the_inspector_visible() {
    let hero = HeroScreen::new(NodeId(1));
    assert!(
        hero.is_panel_visible("inspector"),
        "o app abriu SEM o Inspector: e' o painel onde vivem todas as propriedades, e o dono \
         pediu explicitamente que abrisse com ele"
    );
}

/// **E o botão do rail concorda com ele.**
///
/// ⚠️ Duas fontes para o mesmo facto — o mapa de visibilidade e o estado do botão — e nada as
/// obriga a concordar no arranque: são dois `insert` em ficheiros diferentes. *Um rail a dizer
/// «desligado» sobre um painel aberto ensina ao artista que o botão está partido.*
#[test]
fn the_rail_toggle_agrees_that_it_is_open() {
    let hero = HeroScreen::new(NodeId(1));
    let pressed = matches!(
        hero.store.get(ids::RAIL_SHOW_INSPECTOR),
        Some(InteractiveState::Button {
            state: ButtonState::Pressed
        })
    );
    assert_eq!(
        pressed,
        hero.is_panel_visible("inspector"),
        "o botao do rail e a visibilidade do painel discordam no arranque"
    );
}

/// **A Hierarchy também** — ela partilha a mesma tabela e o mesmo comentário, e uma wave que mexa
/// numa mexe nas duas.
#[test]
fn the_hierarchy_opens_with_it() {
    let hero = HeroScreen::new(NodeId(1));
    assert!(
        hero.is_panel_visible("hierarchy"),
        "o app abriu sem a Hierarchy: as duas colunas laterais nascem juntas"
    );
}
