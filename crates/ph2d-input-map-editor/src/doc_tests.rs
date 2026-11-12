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
    // Sem acções: a frase. Com acções e nenhuma escolhida: a primeira, a que a List marca (uma fonte nas três vistas).
    assert_eq!(crate::inspect::pane(&d), InputMapPane::Pick);
    d.snap.map = InputMap::with_player_defaults();
    let first = d.snap.map.actions()[0].name.clone();
    assert!(matches!(crate::inspect::pane(&d), InputMapPane::Action(a) if a.name == first));
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

/// ⭐ 7b: a publicação vem DEPOIS do snapshot do quadro. O mapa novo fica sem acções ⇒ no MESMO quadro o Inspector já
/// diz a frase. Controlo: publicar ANTES do snapshot (a ordem de antes) ainda mostra a acção velha.
#[test]
fn the_inspector_shows_the_new_snapshot_in_the_same_frame() {
    use ph2d_input::InputMap;
    use ph2d_input::inspected::{InputMapPane, current_input_map};
    let mut d = InputMapDoc::new(View::List);
    d.snap.map = InputMap::with_player_defaults();
    let jump = d.snap.map.id(ph2d_input::PLAYER_JUMP).expect("jump");
    d.kb.open = Some(jump);
    let mut sem_jump = d.snap.clone();
    sem_jump.map = InputMap::new();
    // Controlo: a ordem de antes publica o quadro velho.
    crate::inspect::publish(&d, true);
    assert!(matches!(current_input_map(), Some(InputMapPane::Action(_))));
    crate::inspect::set_snapshot(&mut d, sem_jump, true);
    assert_eq!(current_input_map(), Some(InputMapPane::Pick));
    ph2d_input::inspected::set_current_input_map(None);
}

/// ⭐ Auditoria 7b: o `ActionId` é um índice. Outro projecto com os MESMOS ids e outros nomes ⇒ a escolha cai (o
/// Inspector mostra a primeira do projecto novo, nunca a do mesmo número). Controlo: o mesmo mapa com uma ligação a menos (undo) mantém-na aberta.
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
    assert_eq!(d.kb.open, None);
    assert!(matches!(crate::inspect::pane(&d), InputMapPane::Action(a) if a.name == "other_0"));
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

/// ⭐ Foto o4 (cena 9): os rótulos Dead zone / Press at da linha ABERTA leem-se (≥ 4,5:1, texto pequeno) em todo
/// tema, no repouso e no pairado, e ficam abaixo do valor (`Text1`). Controlo: a linha não aberta mantém o
/// `CardLabel`; as cinco faces da linha diferem, e a aberta pairada é a pairada de toda linha (não a da escuta).
#[test]
fn the_open_row_labels_read_in_every_theme_and_its_face_is_its_own() {
    use crate::view_list::rows::{row_face, zone_label_ink};
    use ph2d_tokens::{ColorToken, Theme};
    assert_eq!(zone_label_ink(false), ColorToken::CardLabel);
    let mut falhas = Vec::new();
    for hover in [false, true] {
        let (fill, _) = row_face(false, false, true, hover);
        for t in Theme::ALL {
            let bg = fill.resolve(t);
            let l = zone_label_ink(true).resolve(t).contrast_ratio(&bg);
            let v = ColorToken::Text1.resolve(t).contrast_ratio(&bg);
            if l < 4.5 || l >= v {
                falhas.push(format!("{t:?} {fill:?}: rótulo {l:.2} · valor {v:.2}"));
            }
        }
    }
    assert!(falhas.is_empty(), "{falhas:?}");
    assert_eq!(
        row_face(false, false, true, true),
        row_face(false, false, false, true)
    );
    assert_ne!(
        row_face(false, false, true, true),
        row_face(false, true, false, false)
    );
    let faces = [
        row_face(true, false, false, false),
        row_face(false, true, false, false),
        row_face(false, false, true, false),
        row_face(false, false, false, true),
        row_face(false, false, false, false),
    ];
    for (i, a) in faces.iter().enumerate() {
        for b in &faces[i + 1..] {
            assert_ne!(a, b);
        }
    }
}

/// ⭐ O «Bind…» da aba sobre a escuta do cartão do Trigger NA MESMA acção cancela (o `Down` já a desarmou) e não
/// re-arma como «acrescentar»; o de outra acção arma; sem cancelamento, arma.
#[test]
fn bind_on_the_triggers_listen_cancels_instead_of_rearming() {
    let mut map = ph2d_input::InputMap::default();
    let (x, y) = (map.create("fire"), map.create("jump"));
    let mut store = ph2d_editor_core::interaction::WidgetStore::default();
    let mut d = InputMapDoc::new(View::List);
    for (alvo, quer) in [(x, None), (y, Some(y))] {
        store.listen_for_binding_replacing(x);
        store.desarma_escuta_no_clique();
        d.end_frame(&map, &mut store, Some(Some(alvo)), true);
        assert_eq!(store.input_map_listening(), quer);
    }
    store.stop_listening();
    d.end_frame(&map, &mut store, Some(Some(x)), true);
    assert_eq!(
        store.input_map_listening(),
        Some(x),
        "sem cartão a cancelar, arma"
    );
}
