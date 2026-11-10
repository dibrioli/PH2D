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
