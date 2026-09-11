//! ⭐ **O pill MODEL abre o módulo de modelagem 3D** — a porta que um artista encontra.
//!
//! # Por que este arquivo existe
//!
//! Três smokes reprovados no mesmo dia (Enio, 2026-08-19): *"o painel não abre"*, *"os objetos não
//! aparecem na hierarchy"*, *"não temos um Pill no topo"*. Os três eram a mesma classe: a peça
//! existia, os gates dela passavam, e **nenhuma porta chegava até ela**.
//!
//! O que se prende aqui é a corrente inteira do pill:
//!   registrado no top bar → o clique alterna a visibilidade → a chave é a que o painel declara.
//!
//! ⚠️ **A chave é o elo frágil**, e por isso tem gate próprio: a crate do chrome não pode importar a
//! do painel (a dependência aponta ao contrário), então as duas escrevem o mesmo literal. Uma
//! divergência ali alterna um painel que ninguém pinta — em silêncio, porque uma chave desconhecida
//! só lê como `false`.

use ph2d_editor::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor::screens::hero::{HeroScreen, chrome, fixture, ids};

/// O pill **existe no top bar e está registado** — sem o registro ele desenha e nasce morto sob o
/// mouse (a nota que `topbar/mod.rs` já carrega para os vizinhos).
#[test]
fn the_pill_is_registered_in_the_top_bar() {
    assert!(
        fixture::topbar_clusters()
            .iter()
            .any(|(id, _)| *id == ids::TOPBAR_MODEL3D),
        "o pill não está entre os clusters que a topbar PINTA"
    );
    let hero = HeroScreen::new(ph2d_editor::NodeId(1));
    // ⚠️ A do meio é a que já matou um pill neste repo: pintado no fixture mas sem registro no
    // `populate`, ele não tem `InteractiveState`, o `Up` nunca emite `Click`, e o botão nasce morto
    // sob o mouse — com todo o resto verde.
    assert!(
        matches!(
            hero.store.get(ids::TOPBAR_MODEL3D),
            Some(InteractiveState::Button { .. })
        ),
        "o pill não foi registrado no `populate`: ele desenha e está morto sob o mouse"
    );
}

/// ⭐ **Clicar abre; clicar de novo fecha** — e a chave é a MESMA que o painel declara.
#[test]
fn the_model_pill_toggles_the_panel_the_shell_knows() {
    assert_eq!(
        ph2d_editor::screens::hero::chrome::MODEL3D_PANEL_KEY,
        ph2d_panel_model3d::PANEL_ID,
        "a chave do pill e a do painel divergiram — o pill passa a alternar um painel que ninguém \
         pinta, e nada avisa porque uma chave desconhecida lê como `false`"
    );

    let mut hero = HeroScreen::new(ph2d_editor::NodeId(1));
    let key = ph2d_panel_model3d::PANEL_ID;
    assert!(
        !hero.is_panel_visible(key),
        "o painel começa fechado: ele ocuparia o encaixe da direita em toda sessão"
    );

    assert!(
        chrome::dispatch_all(&mut hero, WidgetEvent::Click(ids::TOPBAR_MODEL3D)),
        "o clique no pill tem de ser consumido pelo chrome"
    );
    assert!(hero.is_panel_visible(key), "o primeiro clique ABRE");

    assert!(chrome::dispatch_all(
        &mut hero,
        WidgetEvent::Click(ids::TOPBAR_MODEL3D)
    ));
    assert!(!hero.is_panel_visible(key), "o segundo clique FECHA");
}
