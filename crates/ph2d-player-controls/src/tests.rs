use ph2d_editor_core::zones::Rect;
use ph2d_input::{Binding, InputMap, Key, PlayerKeys};

use crate::{ControlsScreen, Hit, Layout, Outcome, PlaySession, player_file, readable_name, rows};

const F1: Key = Key(0xF704);
const UP: Key = Key(0xF700);
const DOWN: Key = Key(0xF701);
const ENTER: Key = Key(0x0D);
const ESC: Key = Key(0x1B);
const J: Key = Key(0x4A);
const Z: Key = Key(0x5A);

fn game() -> InputMap {
    let mut m = InputMap::with_player_defaults();
    m.controls_mut().players_can_change_keys = true;
    m
}

fn row_of(map: &InputMap, name: &str) -> usize {
    rows(map)
        .iter()
        .position(|r| r.action == name)
        .expect("na tela")
}

fn keys_of(map: &InputMap, name: &str) -> Vec<Binding> {
    map.actions()
        .iter()
        .find(|a| a.name == name)
        .expect("acção")
        .bindings
        .clone()
}

/// Abre e desce até `jump`.
fn on_jump(s: &mut ControlsScreen, g: &InputMap, k: &mut PlayerKeys) {
    assert_eq!(s.handle_key(F1, true, g, k), Outcome::Opened);
    for _ in 0..row_of(g, "jump") {
        s.handle_key(DOWN, true, g, k);
    }
    assert_eq!(s.selected(), row_of(g, "jump"));
}

#[test]
fn enter_then_j_replaces_up_on_jump_and_keeps_the_rest() {
    let g = game();
    let (mut s, mut k) = (ControlsScreen::default(), PlayerKeys::new());
    on_jump(&mut s, &g, &mut k);
    assert_eq!(s.handle_key(ENTER, true, &g, &mut k), Outcome::Consumed);
    assert!(s.listening());
    assert_eq!(s.handle_key(J, true, &g, &mut k), Outcome::Changed);
    assert!(!s.listening());
    let mut want = keys_of(&g, "jump");
    assert_eq!(want[0], Binding::Key(UP), "a fábrica começa pela seta");
    want[0] = Binding::Key(J);
    assert_eq!(k.get("jump"), Some(want.as_slice()));
    let eff = k.apply_over(&g);
    assert!(keys_of(&eff, "jump").contains(&Binding::Key(J)));
    assert!(!keys_of(&eff, "jump").contains(&Binding::Key(UP)));
    assert_eq!(
        keys_of(&g, "jump")[0],
        Binding::Key(UP),
        "o mapa do jogo não muda"
    );
}

#[test]
fn esc_cancels_listening_then_closes() {
    let g = game();
    let (mut s, mut k) = (ControlsScreen::default(), PlayerKeys::new());
    on_jump(&mut s, &g, &mut k);
    s.handle_key(ENTER, true, &g, &mut k);
    let soltar = s.handle_key(Key(0xF703), false, &g, &mut k);
    assert_eq!(
        soltar,
        Outcome::Ignored,
        "o soltar passa ao jogo: nada fica preso"
    );
    assert!(s.listening(), "e não gasta a escuta");
    assert_eq!(s.handle_key(ESC, true, &g, &mut k), Outcome::Consumed);
    assert!(!s.listening() && s.is_open());
    assert!(k.is_empty(), "cancelar não troca nada");
    assert_eq!(s.handle_key(ESC, true, &g, &mut k), Outcome::Closed);
    assert!(!s.is_open());
}

#[test]
fn reset_to_defaults_clears_the_players_keys() {
    let g = game();
    let (mut s, mut k) = (ControlsScreen::default(), PlayerKeys::new());
    on_jump(&mut s, &g, &mut k);
    s.handle_key(ENTER, true, &g, &mut k);
    s.handle_key(J, true, &g, &mut k);
    assert!(!k.is_empty());
    for _ in 0..rows(&g).len() + 2 {
        s.handle_key(DOWN, true, &g, &mut k);
    }
    assert_eq!(s.selected(), rows(&g).len(), "a última linha é o Reset");
    assert_eq!(s.handle_key(ENTER, true, &g, &mut k), Outcome::Changed);
    assert!(k.is_empty());
}

#[test]
fn binding_back_the_games_key_forgets_the_change() {
    let g = game();
    let (mut s, mut k) = (ControlsScreen::default(), PlayerKeys::new());
    on_jump(&mut s, &g, &mut k);
    s.handle_key(ENTER, true, &g, &mut k);
    s.handle_key(J, true, &g, &mut k);
    s.handle_key(ENTER, true, &g, &mut k);
    assert_eq!(s.handle_key(UP, true, &g, &mut k), Outcome::Changed);
    assert!(k.is_empty(), "o ficheiro guarda só o que o jogador mudou");
    // A tecla que a linha já tem noutra caixa não aparece duas vezes.
    s.handle_key(ENTER, true, &g, &mut k);
    s.handle_key(Z, true, &g, &mut k);
    let kb: Vec<_> = keys_of(&k.apply_over(&g), "jump")
        .into_iter()
        .filter(|b| matches!(b, Binding::Key(_)))
        .collect();
    assert_eq!(kb, [Binding::Key(Z)]);
}

#[test]
fn hidden_actions_are_not_listed() {
    let mut g = game();
    let dash = g.id("dash").expect("fábrica");
    g.controls_mut().set_shown_in_game(dash, false);
    let r = rows(&g);
    assert!(r.iter().all(|r| r.action != "dash"));
    assert_eq!(r.len(), g.actions().len() - 1);
    assert_eq!(r[0].label, "Move left");
}

#[test]
fn without_players_can_change_keys_the_open_key_does_nothing() {
    let mut g = game();
    g.controls_mut().players_can_change_keys = false;
    let (mut s, mut k) = (ControlsScreen::default(), PlayerKeys::new());
    assert_eq!(s.handle_key(F1, true, &g, &mut k), Outcome::Ignored);
    assert!(!s.is_open());
}

#[test]
fn readable_names() {
    assert_eq!(readable_name("move_left"), "Move left");
    assert_eq!(readable_name("jump"), "Jump");
    assert_eq!(readable_name(""), "");
}

#[test]
fn a_click_on_a_key_box_listens_on_that_box() {
    let g = game();
    let r = rows(&g);
    let lay = Layout::of(
        Rect::new(0.0, 0.0, 1280.0, 720.0),
        &r,
        &mut ph2d_text::TextSystem::new(),
    );
    let row = row_of(&g, "jump");
    let b = lay.rows[row].slots[1];
    let hit = lay.hit(b.x + b.w * 0.5, b.y + b.h * 0.5);
    assert_eq!(hit, Some(Hit::Slot { row, slot: 1 }));
    assert_eq!(lay.hit(-1.0, -1.0), None);
    let (mut s, mut k) = (ControlsScreen::default(), PlayerKeys::new());
    s.handle_key(F1, true, &g, &mut k);
    assert_eq!(s.click(hit, &mut k), Outcome::Consumed);
    assert!(s.listening());
    assert_eq!((s.selected(), s.slot()), (row, 1));
    assert_eq!(s.handle_key(J, true, &g, &mut k), Outcome::Changed);
    let kb = keys_of(&k.apply_over(&g), "jump");
    assert!(kb.contains(&Binding::Key(UP)) && kb.contains(&Binding::Key(J)));
    assert!(!kb.contains(&Binding::Key(Z)), "a segunda caixa era o Z");
}

#[test]
fn the_player_file_round_trips_in_a_temp_dir() {
    let base = std::env::temp_dir().join(format!("ph2d-player-controls-{}", std::process::id()));
    let path = player_file(&base, 0xABCD);
    assert!(path.ends_with("players/000000000000abcd.txt"));
    let g = game();
    let mut s = PlaySession::default();
    s.start(&g, Some(path.clone()));
    assert!(s.keys().is_empty(), "sem ficheiro, nada trocado");
    s.key(F1, true, &g);
    for _ in 0..row_of(&g, "jump") {
        s.key(DOWN, true, &g);
    }
    s.key(ENTER, true, &g);
    assert_eq!(s.key(J, true, &g), Outcome::Changed);
    assert!(path.exists(), "grava a cada troca");
    let mut again = PlaySession::default();
    again.start(&g, Some(path.clone()));
    assert_eq!(again.keys(), s.keys());
    assert!(keys_of(again.map(&g), "jump").contains(&Binding::Key(J)));
    let mut closed = g.clone();
    closed.controls_mut().players_can_change_keys = false;
    again.start(&closed, Some(path.clone()));
    assert!(
        again.keys().is_empty(),
        "o autor fechou as trocas: valem as do jogo"
    );
    assert!(path.exists(), "o ficheiro do jogador fica");
    for _ in 0..rows(&g).len() {
        s.key(DOWN, true, &g);
    }
    assert_eq!(s.key(ENTER, true, &g), Outcome::Changed);
    assert!(!path.exists(), "Reset apaga o ficheiro");
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn the_session_map_is_the_project_outside_play() {
    let g = game();
    let mut s = PlaySession::default();
    assert!(std::ptr::eq(s.map(&g), &g));
    assert_eq!(
        s.key(F1, true, &g),
        Outcome::Ignored,
        "fora do Play a tecla não abre"
    );
    s.start_with(&g, PlayerKeys::new(), None);
    assert!(s.running());
    s.stop();
    assert!(!s.running() && std::ptr::eq(s.map(&g), &g));
}
