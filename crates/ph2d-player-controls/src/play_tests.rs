//! A sessão do Play com os leitores reais: a resolução do jogador, o relógio e o Luau (plano InputMap §4, W7).

use ph2d_ecs::{SimWorld, StableId, Transform};
use ph2d_input::{ActionState, Event, Input, InputMap, InputState, Key, PlayerKeys};
use ph2d_script::{LuauScript, ProjectScripts, ScriptHost};

use crate::{Outcome, PlaySession, rows};

const F1: Key = Key(0xF704);
const DOWN: Key = Key(0xF701);
const ENTER: Key = Key(0x0D);
const ESC: Key = Key(0x1B);
const UP: Key = Key(0xF700);
const J: Key = Key(0x4A);

fn game() -> InputMap {
    let mut m = InputMap::with_player_defaults();
    m.controls_mut().players_can_change_keys = true;
    m
}

/// O Play começou e o jogador trocou o `jump` para J pela tela.
fn jump_on_j(g: &InputMap) -> PlaySession {
    let mut s = PlaySession::default();
    s.start_with(g, PlayerKeys::new(), None);
    assert_eq!(s.key(F1, true, g), Outcome::Opened);
    let row = rows(g)
        .iter()
        .position(|r| r.action == "jump")
        .expect("jump");
    for _ in 0..row {
        s.key(DOWN, true, g);
    }
    s.key(ENTER, true, g);
    assert_eq!(s.key(J, true, g), Outcome::Changed);
    s
}

fn pressed_jump(map: &InputMap, key: Key) -> bool {
    let (mut acts, mut dev) = (ActionState::new(), InputState::new());
    dev.apply_event(Event::KeyDown(key));
    let _ = ph2d_physics_ecs::player_input::resolve_player_input(&mut acts, Some(map), &dev);
    Input::new(map, &acts).pressed("jump")
}

#[test]
fn the_player_resolution_sees_the_players_j_and_the_project_map_stays() {
    let g = game();
    let before = g.clone();
    let s = jump_on_j(&g);
    assert!(pressed_jump(s.map(&g), J), "o J do jogador salta");
    assert!(
        !pressed_jump(s.map(&g), UP),
        "a seta que ele trocou já não salta"
    );
    assert!(
        pressed_jump(&g, UP),
        "controlo: no mapa do jogo a seta salta"
    );
    assert_eq!(g, before, "o mapa do projecto (o do undo) não muda");
}

#[test]
fn exiting_play_forgets_the_effective_map() {
    let g = game();
    let mut s = jump_on_j(&g);
    assert!(!std::ptr::eq(s.map(&g), &g));
    s.stop();
    assert!(std::ptr::eq(s.map(&g), &g));
    assert!(!s.screen().is_open());
}

#[test]
fn the_clock_pauses_while_open_and_resumes_on_close() {
    let g = game();
    let mut s = PlaySession::default();
    s.start_with(&g, PlayerKeys::new(), None);
    let out = s.key(F1, true, &g);
    assert_eq!(s.clock_after(out, true), Some(false), "aberta, o jogo pára");
    let out = s.key(ESC, true, &g);
    assert_eq!(out, Outcome::Closed);
    assert_eq!(
        s.clock_after(out, false),
        Some(true),
        "fechada, volta a correr"
    );
    let out = s.key(F1, true, &g);
    assert_eq!(s.clock_after(out, false), Some(false));
    let out = s.key(F1, true, &g);
    assert_eq!(
        s.clock_after(out, false),
        None,
        "parado antes de abrir: fechar não o põe a correr"
    );
}

const SALTO: &str = r#"
function update(self)
  if ph2d.action_pressed("jump") then ph2d.set(self.id, "x", 7) end
end
"#;

/// ⭐ W7: o script lê a ACÇÃO pelo mapa que o Play lê — com a troca do jogador, o J salta.
#[test]
fn a_luau_script_sees_the_players_j() {
    let g = game();
    let s = jump_on_j(&g);
    let map = s.map(&g);
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let id = w
        .get_resource_or_init::<ProjectScripts>()
        .insert("salto.luau", SALTO.to_owned());
    let e = w
        .spawn((Transform::IDENTITY, StableId(1), LuauScript::library(id)))
        .id();
    let mut host = ScriptHost::new().expect("vm");
    let (mut acts, mut dev) = (ActionState::new(), InputState::new());
    dev.apply_event(Event::KeyDown(J));
    acts.tick(map, &dev);
    host.devices().feed_actions(Some((map, &acts)));
    host.scene_sync(sim.world_mut());
    let _ = host.scene_tick(&mut sim, 1.0 / 60.0);
    let x = sim.world().get::<Transform>(e).expect("pose").translation.x;
    assert_eq!(x, 7.0, "o action_pressed(\"jump\") viu o J do jogador");
}
