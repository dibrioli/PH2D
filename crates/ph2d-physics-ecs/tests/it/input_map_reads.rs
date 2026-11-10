//! As tabelas `PLATFORM_PLAYER_READS` / `TOP_DOWN_PLAYER_READS` do `ph2d-input` são o que o
//! `resolve_player_input` lê de verdade: cada nome delas muda o `PlayerInput`, e um nome fora delas
//! não muda nada (plano InputMap §2).

use ph2d_input::{
    ActionState, Binding, Event, InputMap, InputState, Key, PLATFORM_PLAYER_READS,
    TOP_DOWN_PLAYER_READS,
};
use ph2d_physics_ecs::PlayerInput;
use ph2d_physics_ecs::player_input::resolve_player_input;

const KEY: Key = Key(0x4B);

/// O `PlayerInput` com só `name` ligado a [`KEY`], segurada ou não.
fn resolved(name: &str, held: bool) -> PlayerInput {
    let mut map = InputMap::new();
    let id = map.create(name);
    map.get_mut(id)
        .expect("acabou de nascer")
        .bindings
        .push(Binding::Key(KEY));
    let mut dev = InputState::new();
    dev.begin_frame();
    if held {
        dev.apply_event(Event::KeyDown(KEY));
    }
    resolve_player_input(&mut ActionState::new(), Some(&map), &dev)
}

#[test]
fn every_name_the_tables_list_moves_the_player_input() {
    for name in PLATFORM_PLAYER_READS
        .into_iter()
        .chain(TOP_DOWN_PLAYER_READS)
    {
        assert_eq!(
            resolved(name, false),
            PlayerInput::default(),
            "{name} solta"
        );
        assert_ne!(
            resolved(name, true),
            PlayerInput::default(),
            "{name} está na tabela e ninguém o lê"
        );
    }
}

#[test]
fn a_name_outside_the_tables_changes_nothing() {
    for name in ["fire", "reload", "Jump", "move left", "up", ""] {
        assert_eq!(
            resolved(name, true),
            PlayerInput::default(),
            "{name:?} não está nas tabelas e alguém o lê"
        );
    }
}
