//! O ficheiro do jogador da aba sai da MESMA lei da tela do Play (`player_file_of`).

use super::{InputMapDoc, View};

/// `game_id` 0 não tem ficheiro (partilharia o `0000000000000000.txt` entre projectos); sem base, também não.
#[test]
fn game_id_zero_means_no_player_file() {
    let base = std::path::PathBuf::from("/nowhere/.ph2d");
    let mut d = InputMapDoc::new(View::List);
    d.tryit.base = Some(base.clone());
    assert_eq!(d.player_file(), None, "game_id 0");
    d.snap.game_id = 0x2A;
    let want = ph2d_player_controls::player_file(&base, 0x2A);
    assert_eq!(d.player_file(), Some(want));
    d.tryit.base = None;
    assert_eq!(d.player_file(), None, "sem base");
}

/// ⭐ Lei 2 (a guarda do clique): o `add` da B recusa a tecla do SISTEMA MESMO sem o controlo desligado (o clique pelo
/// `press` passa antes pelo controlo; por `press_legacy` — o caminho de antes do registo — só o `add` o impede). A
/// tecla do jogo, no mesmo caminho, liga: o controlo de que o teste não é vermelho por qualquer mudança.
#[test]
fn the_click_path_refuses_a_system_key_without_the_disabled_control() {
    use crate::edit::InputMapEdit;
    use crate::view_keyboard::state::KbHit;
    use ph2d_editor_core::zones::Rect;
    use ph2d_input::{Binding, InputMap, Key};

    let mut d = InputMapDoc::new(View::Keyboard);
    d.snap.map = InputMap::with_player_defaults();
    let id = d.snap.map.id("jump").expect("de fábrica");
    d.kb.open = Some(id);
    d.layout.body = Rect::new(0.0, 0.0, 100.0, 100.0);
    let win = Key(0xF718);
    let game = Key::named_in_script("q").expect("a letra Q")[0];
    assert!(game.system_owned().is_none() && win.system_owned().is_some());
    for (k, at) in [(win, 5.0), (game, 25.0)] {
        d.kb.targets.push(KbHit::Key(k));
        d.hits
            .register(KbHit::Key(k).id(), Rect::new(at, 5.0, 10.0, 10.0));
    }
    d.press_legacy(10.0, 10.0);
    assert!(
        d.pending.is_empty(),
        "a do sistema não pede nada: {:?}",
        d.pending
    );
    d.press_legacy(30.0, 10.0);
    assert_eq!(
        d.pending,
        vec![InputMapEdit::AddBinding(id, Binding::Key(game))],
        "a do jogo, no mesmo caminho, liga"
    );
}

fn key(n: &str) -> ph2d_input::Key {
    ph2d_input::Key::named_in_script(n).expect("tecla")[0]
}

/// O Try it com `held` em baixo e a bandeira Alt do editor: a tecla F é do banco (`true`) ou passa ao editor?
fn try_takes_f(held: &str, alt: bool) -> bool {
    let mut d = InputMapDoc::new(View::TryIt);
    d.alt = alt;
    let _ = d.raw_key(key(held), true, false);
    d.raw_key(key("f"), true, true)
}

/// ⭐ AltGr (onda 4, `chord_keys`): no `br(abnt2)` o Alt direito é o AltGr e não acende o Alt do editor ⇒ AltGr+F é do
/// banco. Controlo: o Alt esquerdo acende-o ⇒ Alt+F é acorde do editor e passa (recusado pelo banco).
#[test]
fn altgr_f_is_tried_and_left_alt_f_is_the_editors() {
    assert!(try_takes_f("right_alt", false), "AltGr+F");
    assert!(!try_takes_f("left_alt", true), "Alt esquerdo+F");
    // Sem a bandeira, o mesmo Alt direito volta a ser Alt (o `us` simples): a lei lê a bandeira, não o lado.
    assert!(!try_takes_f("right_alt", true), "Alt direito que é Alt");
}

/// ⭐ 7b: o Inspector lê a acção aberta do MESMO snapshot. Keyboard ⇒ a aberta (ou a primeira); List sem escolha ⇒
/// a frase; escolhida ⇒ ela, com as ligações e o grupo.
#[test]
fn the_inspector_pane_follows_the_open_action() {
    use ph2d_input::InputMap;
    use ph2d_input::inspected::InputMapPane;
    let mut d = InputMapDoc::new(View::List);
    d.snap.map = InputMap::with_player_defaults();
    assert_eq!(crate::inspect::pane(&d), InputMapPane::Pick);
    let jump = d.snap.map.id(ph2d_input::PLAYER_JUMP).expect("jump");
    d.kb.open = Some(jump);
    let InputMapPane::Action(a) = crate::inspect::pane(&d) else {
        panic!("a acção");
    };
    assert_eq!(a.name, ph2d_input::PLAYER_JUMP);
    assert!(!a.keys.is_empty());
    d.set_view(View::Keyboard);
    d.kb.open = None;
    assert!(matches!(crate::inspect::pane(&d), InputMapPane::Action(_)));
    // Publicar com a aba atrás apaga o que o Inspector mostra.
    crate::inspect::publish(&d, false);
    assert_eq!(ph2d_input::inspected::current_input_map(), None);
    crate::inspect::publish(&d, true);
    assert!(ph2d_input::inspected::current_input_map().is_some());
    ph2d_input::inspected::set_current_input_map(None);
}

/// ⭐ 7b: a publicação vem DEPOIS do snapshot do quadro. A acção aberta é apagada no mapa novo ⇒ no MESMO quadro o
/// Inspector já diz a frase. Controlo: publicar ANTES do snapshot (a ordem de antes) ainda mostra a acção velha.
#[test]
fn the_inspector_shows_the_new_snapshot_in_the_same_frame() {
    use ph2d_input::InputMap;
    use ph2d_input::inspected::{InputMapPane, current_input_map};
    let mut d = InputMapDoc::new(View::List);
    d.snap.map = InputMap::with_player_defaults();
    let jump = d.snap.map.id(ph2d_input::PLAYER_JUMP).expect("jump");
    d.kb.open = Some(jump);
    let mut sem_jump = d.snap.clone();
    let _ = sem_jump.map.remove(jump);
    // Controlo: a ordem de antes publica o quadro velho.
    crate::inspect::publish(&d, true);
    assert!(matches!(current_input_map(), Some(InputMapPane::Action(_))));
    crate::inspect::set_snapshot(&mut d, sem_jump, true);
    assert_eq!(current_input_map(), Some(InputMapPane::Pick));
    ph2d_input::inspected::set_current_input_map(None);
}

/// ⭐ Auditoria 7b: o `ActionId` é um índice. Outro projecto com os MESMOS ids e outros nomes ⇒ a acção aberta cai
/// (o Inspector diz a frase). Controlo: o mesmo mapa com uma ligação a menos (undo) mantém-na aberta.
#[test]
fn a_swapped_map_with_the_same_ids_closes_the_open_action() {
    use ph2d_input::InputMap;
    use ph2d_input::inspected::InputMapPane;
    let a = InputMap::with_player_defaults();
    let jump = a.id(ph2d_input::PLAYER_JUMP).expect("jump");
    let fresh = |d: &mut InputMapDoc| {
        d.snap.map = a.clone();
        d.applied = Some(a.clone());
        d.kb.open = Some(jump);
    };
    let mut d = InputMapDoc::new(View::List);
    fresh(&mut d);
    let mut b = InputMap::new();
    for i in 0..a.len() {
        let _ = b.create(format!("other_{i}"));
    }
    assert!(b.get(jump).is_some(), "o mesmo id existe no outro projecto");
    assert!(d.begin_frame(&b));
    d.snap.map = b;
    assert_eq!(crate::inspect::pane(&d), InputMapPane::Pick);
    // Controlo: a mesma acção, com uma ligação a menos.
    fresh(&mut d);
    let mut a2 = a.clone();
    a2.get_mut(jump).expect("jump").bindings.pop();
    assert!(d.begin_frame(&a2));
    d.snap.map = a2;
    assert_eq!(d.kb.open, Some(jump));
    assert!(matches!(crate::inspect::pane(&d), InputMapPane::Action(_)));
}

/// A vista Keyboard com a acção aberta sem ligações: a tecla F, com `held` em baixo e a bandeira Alt, liga-se?
fn keyboard_binds_f(held: &str, alt: bool) -> bool {
    use ph2d_input::InputMap;
    let mut d = InputMapDoc::new(View::Keyboard);
    let mut map = InputMap::new();
    let id = map.create("fire");
    d.snap.map = map;
    d.kb.open = Some(id);
    d.alt = alt;
    let _ = d.raw_key(key(held), true, false);
    d.raw_key(key("f"), true, true) && !d.pending.is_empty()
}

/// ⭐ AltGr na vista KEYBOARD (a linha de `view_keyboard/input.rs` com `chord_keys`): AltGr+F liga; Alt esquerdo+F é
/// acorde do editor (controlo).
#[test]
fn keyboard_view_binds_altgr_f_and_not_left_alt_f() {
    assert!(keyboard_binds_f("right_alt", false), "AltGr+F");
    assert!(!keyboard_binds_f("left_alt", true), "Alt esquerdo+F");
}
