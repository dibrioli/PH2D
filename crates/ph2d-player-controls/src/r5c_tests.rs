//! R5c: a lista rola, a tecla repetida avisa-se na linha, e a edição do autor no Play entra já (plano InputMap §4).

use ph2d_editor_core::zones::Rect;
use ph2d_input::{Binding, InputMap, Key, PlayerKeys};

use crate::{ControlsScreen, Hit, Layout, Outcome, PlaySession, rebind, rows, rows_marked};

const F1: Key = Key(0xF704);
const UP: Key = Key(0xF700);
const DOWN: Key = Key(0xF701);
const LEFT: Key = Key(0xF702);
const J: Key = Key(0x4A);
const K: Key = Key(0x4B);

fn game() -> InputMap {
    let mut m = InputMap::with_player_defaults();
    m.controls_mut().players_can_change_keys = true;
    m
}

fn twenty() -> InputMap {
    let mut m = InputMap::new();
    m.controls_mut().players_can_change_keys = true;
    for i in 0..20u32 {
        let id = m.create(format!("action_{i}"));
        m.get_mut(id)
            .expect("nasceu")
            .bindings
            .push(Binding::Key(Key(0x41 + i)));
    }
    m
}

fn keys_of(map: &InputMap, name: &str) -> Vec<Binding> {
    map.actions()
        .iter()
        .find(|a| a.name == name)
        .expect("acção")
        .bindings
        .clone()
}

/// Lays out like the paint does and writes the scroll back, as `PlaySession::paint`.
fn lay(area: Rect, s: &mut ControlsScreen, map: &InputMap) -> Layout {
    let l = Layout::scrolled(area, &rows(map), s, &mut ph2d_text::TextSystem::new());
    s.scroll_to(l.first);
    l
}

#[test]
fn twenty_actions_in_a_small_rect_scroll_and_keep_the_selected_row_visible() {
    let g = twenty();
    let area = Rect::new(0.0, 0.0, 640.0, 300.0);
    let (mut s, mut k) = (ControlsScreen::default(), PlayerKeys::new());
    assert_eq!(s.handle_key(F1, true, &g, &mut k), Outcome::Opened);
    let l = lay(area, &mut s, &g);
    let visible = l.rows.len();
    assert!(visible > 1 && visible < 20, "{visible} linhas à vista");
    assert!(l.panel.y >= area.y && l.panel.y + l.panel.h <= area.y + area.h + 0.5);
    let (track, thumb) = l.bar.expect("rola: há indicador");
    assert!(
        track.w < 5.0 && thumb.h < track.h,
        "fino, e o polegar é parte"
    );
    assert!(
        l.reset.y + l.reset.h <= l.hint.y,
        "o Reset fica à vista em baixo"
    );
    for i in 1..20 {
        s.handle_key(DOWN, true, &g, &mut k);
        let l = lay(area, &mut s, &g);
        assert_eq!(s.selected(), i);
        assert!(
            (l.first..l.first + l.rows.len()).contains(&i),
            "a linha {i} à vista (de {} a {})",
            l.first,
            l.first + l.rows.len()
        );
    }
    let l = lay(area, &mut s, &g);
    assert_eq!(l.first, 20 - visible, "no fundo");
    let band = l.rows[0].slots[0];
    assert_eq!(
        l.hit(band.x + 1.0, band.y + 1.0),
        Some(Hit::Slot {
            row: l.first,
            slot: 0
        }),
        "o clique conta a linha rolada"
    );
    // Subir só rola quando passa da borda de cima.
    let top = l.first;
    s.handle_key(UP, true, &g, &mut k);
    assert_eq!(lay(area, &mut s, &g).first, top);
    for _ in 0..19 {
        s.handle_key(UP, true, &g, &mut k);
        let l = lay(area, &mut s, &g);
        assert!((l.first..l.first + l.rows.len()).contains(&s.selected()));
    }
    assert_eq!((s.selected(), s.top()), (0, 0));
    let big = Layout::of(
        Rect::new(0.0, 0.0, 1280.0, 1000.0),
        &rows(&g),
        &mut ph2d_text::TextSystem::new(),
    );
    assert!(big.bar.is_none() && big.rows.len() == 20, "cabe: não rola");
}

#[test]
fn a_repeated_key_the_player_made_says_so_in_the_row() {
    let g = game();
    assert!(
        rows_marked(&g, &g).iter().all(|r| r.also.is_none()),
        "a partilha que vem do jogo (↑ em jump e move_up) não avisa"
    );
    let mut k = PlayerKeys::new();
    assert_eq!(rebind(&g, &mut k, "jump", 1, LEFT), Outcome::Changed);
    let eff = k.apply_over(&g);
    let r = rows_marked(&eff, &g);
    let jump = r.iter().find(|r| r.action == "jump").expect("jump");
    assert_eq!(jump.also, Some((1, "Move left".to_owned())));
    assert_eq!(
        jump.note().as_deref(),
        Some("Left Arrow is also Move left.")
    );
    let left = r.iter().find(|r| r.action == "move_left").expect("left");
    assert_eq!(left.note().as_deref(), Some("Left Arrow is also Jump."));
    assert!(
        keys_of(&eff, "jump").contains(&Binding::Key(LEFT))
            && keys_of(&eff, "move_left").contains(&Binding::Key(LEFT)),
        "avisa e guarda as duas: nada se tira em silêncio"
    );
    let l = Layout::of(
        Rect::new(0.0, 0.0, 1280.0, 720.0),
        &r,
        &mut ph2d_text::TextSystem::new(),
    );
    let jr = r.iter().position(|r| r.action == "jump").expect("jump");
    let note = l.rows[jr].note;
    assert!(note.w > 0.0 && note.x >= l.rows[jr].slots[1].x + l.rows[jr].slots[1].w);
    assert!(
        note.x + note.w <= l.panel.x + l.panel.w,
        "o aviso cabe no painel"
    );
    assert_eq!(rebind(&g, &mut k, "jump", 1, J), Outcome::Changed);
    let eff = k.apply_over(&g);
    assert!(
        rows_marked(&eff, &g).iter().all(|r| r.also.is_none()),
        "J é só do jump"
    );
}

#[test]
fn the_authors_edit_during_play_reaches_the_effective_map_and_keeps_the_players_key() {
    let g = game();
    let mut k = PlayerKeys::new();
    rebind(&g, &mut k, "jump", 0, J);
    let mut s = PlaySession::default();
    s.start_with(&g, k, None);
    let mut project = g.clone();
    let dash = project.id("dash").expect("dash");
    project
        .get_mut(dash)
        .expect("dash")
        .bindings
        .push(Binding::Key(K));
    assert!(!keys_of(s.map(&project), "dash").contains(&Binding::Key(K)));
    assert!(s.follow(&project), "o mapa do projecto mudou");
    assert!(!s.follow(&project), "comparar outra vez não refaz");
    assert!(keys_of(s.map(&project), "dash").contains(&Binding::Key(K)));
    assert_eq!(keys_of(s.map(&project), "jump")[0], Binding::Key(J));
    // O mesmo pela porta da tecla (que corre a comparação).
    let mut again = project.clone();
    again.get_mut(dash).expect("dash").bindings.clear();
    assert_eq!(s.key(Key(0x50), true, &again), Outcome::Ignored);
    assert!(keys_of(s.map(&again), "dash").is_empty());
    assert_eq!(keys_of(s.map(&again), "jump")[0], Binding::Key(J));
    s.stop();
    assert!(!s.follow(&project), "fora do Play, nada");
}
