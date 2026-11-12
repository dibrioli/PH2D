use super::*;
use crate::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use ph2d_a11y::NodeId;

fn ctrl() -> Modifiers {
    Modifiers {
        ctrl: true,
        ..Modifiers::default()
    }
}

/// Escolhe `id` pelo menu File aberto, como o clique na linha.
fn pick_in_file(hero: &mut HeroScreen, id: NodeId) {
    hero.store.open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 0.0,
        kind: ContextMenuKind::MenuBarFile,
    });
    hero.apply_event(WidgetEvent::Click(id));
    assert!(
        hero.store.context_menu().is_none(),
        "a linha não fechou o menu"
    );
}

#[test]
fn o_texto_do_atalho_le_se_como_acorde_e_uma_pasta_nao() {
    let c = chord_of_text("Ctrl+Shift+S").expect("acorde");
    assert!(c.ctrl && c.shift && !c.alt && c.key == "S");
    assert_eq!(chord_of_text("G").map(|c| c.key), Some("G".into()));
    assert_eq!(chord_of_text("F5").map(|c| c.key), Some("F5".into()));
    assert!(chord_of_text("~/Games/level").is_none());
    assert!(chord_of_text("Ctrl+").is_none());
    assert_eq!(
        Some(chord_of_key(ph2d_input::Key(0x53), ctrl())),
        chord_of_text("Ctrl+S")
    );
    assert_ne!(
        Some(chord_of_key(ph2d_input::Key(0x53), Modifiers::default())),
        chord_of_text("Ctrl+S")
    );
}

/// ⭐ **Na 3.ª escolha pelo menu o chip ensina a tecla; a tecla apaga-o e ele não volta.**
/// Mutação: `TEACH_AT` = 4 ⇒ sem chip na 3.ª, sangra; o `heard_key` sem `dismiss` ⇒ sangra.
#[test]
fn na_terceira_vez_pelo_menu_o_chip_ensina_e_a_tecla_o_apaga() {
    let mut hero = HeroScreen::new(NodeId(1));
    pick_in_file(&mut hero, ids::CTX_MENU_SAVE);
    pick_in_file(&mut hero, ids::CTX_MENU_SAVE);
    assert!(hero.store.teach_chip().is_none(), "ensinou cedo demais");
    pick_in_file(&mut hero, ids::CTX_MENU_SAVE);
    let chip = hero
        .store
        .teach_chip()
        .expect("a 3.ª vez não mostrou o chip")
        .clone();
    assert_eq!((chip.key.as_str(), chip.name.as_str()), ("Ctrl+S", "Save"));
    heard_key(&mut hero, ph2d_input::Key(0x41), ctrl());
    assert!(
        hero.store.teach_chip().is_some(),
        "outra tecla apagou o chip"
    );
    heard_key(&mut hero, ph2d_input::Key(0x53), ctrl());
    assert!(
        hero.store.teach_chip().is_none(),
        "a tecla do atalho não apagou o chip"
    );
    assert!(hero.menu_memory.knows(ids::CTX_MENU_SAVE));
    pick_in_file(&mut hero, ids::CTX_MENU_SAVE);
    assert!(
        hero.store.teach_chip().is_none(),
        "o chip de quem já sabe voltou"
    );
}

/// O «×» é «já sei» e há um chip de cada vez; a escolha pela BUSCA não conta como menu.
#[test]
fn o_x_fecha_para_sempre_e_nunca_ha_dois_chips() {
    let mut hero = HeroScreen::new(NodeId(1));
    for _ in 0..3 {
        pick_in_file(&mut hero, ids::CTX_MENU_SAVE);
    }
    for _ in 0..3 {
        pick_in_file(&mut hero, ids::CTX_MENU_OPEN_PROJECT);
    }
    let chip = hero.store.teach_chip().expect("chip").clone();
    assert_eq!(
        chip.id,
        ids::CTX_MENU_OPEN_PROJECT,
        "o chip novo não tomou o lugar"
    );
    assert!(hero.apply_event(WidgetEvent::Click(ids::MENUBAR_TEACH_CLOSE)));
    assert!(hero.store.teach_chip().is_none());
    assert!(hero.menu_memory.knows(ids::CTX_MENU_OPEN_PROJECT));
    let antes = hero.menu_memory.menu_picks(ids::CTX_MENU_SAVE);
    hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_SAVE));
    assert_eq!(
        hero.menu_memory.menu_picks(ids::CTX_MENU_SAVE),
        antes,
        "um clique fora do menu contou como escolha pelo menu"
    );
}

/// ⭐ **O CLIQUE DE RATO conta como o Enter** (smoke do dono 12/11: o chip não apareceu): o Down numa
/// linha do menu não o fecha, e o Up (o `Click`) chega com o menu aberto.
/// Mutação: voltar o `down_is_of_the_menu` a só as linhas-cascata ⇒ o Down fecha, não conta, sangra.
#[test]
fn o_clique_de_rato_numa_linha_conta_como_escolha_pelo_menu() {
    use crate::interaction::{HitIndex, dispatch_pointer};
    use crate::zones::Rect;
    use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
    let mut hero = HeroScreen::new(NodeId(1));
    // Um segundo entre cliques: o mesmo instante seria um duplo clique.
    let ev = |kind, t: u128| PointerEvent {
        x: 10.0,
        y: 10.0,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: t * 1_000_000_000,
    };
    for n in 1..=3 {
        hero.store.open_context_menu(ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: ContextMenuKind::MenuBarEdit,
        });
        let mut hits = HitIndex::default();
        hits.register(ids::TOOL_UNDO, Rect::new(0.0, 0.0, 100.0, 20.0));
        let arena = bumpalo::Bump::new();
        let down = dispatch_pointer(
            &mut hero.store,
            &hits,
            ev(PointerKind::Down, u128::from(n)),
            &arena,
        )
        .to_vec();
        assert!(hero.store.context_menu().is_some(), "o Down fechou o menu");
        let up = dispatch_pointer(
            &mut hero.store,
            &hits,
            ev(PointerKind::Up, u128::from(n)),
            &arena,
        )
        .to_vec();
        for e in down.into_iter().chain(up) {
            hero.apply_event(e);
        }
        assert_eq!(hero.menu_memory.menu_picks(ids::TOOL_UNDO), n);
    }
    assert_eq!(
        hero.store.teach_chip().map(|c| c.key.as_str()),
        Some("Ctrl+Z")
    );
}
