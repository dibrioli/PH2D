//! Integration tests for the `LuauScript` Component + its bindings.
//!
//! Verifies:
//! - `LuauScript` is a plain CONFIG component that sits on a sim entity.
//! - The world functions say why outside a tick.
//! - `ph2d.state_get`/`set`/`keys` round-trip POD values.
//!
//! ⚠️ Os dois gates do `lateral_key` saíram com o campo (TOP-20 #16): eles fixavam a forma
//! `entity.to_bits() ^ hash`, que punha bits de alocação dentro dos bytes de um componente — ver o
//! cabeçalho de `ph2d_script::component`.

use ph2d_ecs::{SimComponent, SimWorld};
use ph2d_script::component::LuauScript;
use ph2d_script::{PodValue, ScriptHost, ScriptValue};

#[test]
fn luau_script_attaches_to_sim_entity() {
    let mut sim = SimWorld::new();
    let entity = sim.world_mut().spawn_empty().id();
    let mut script = LuauScript::library(ph2d_script::ScriptId(1));
    script.own.insert("speed".into(), ScriptValue::Number(3.0));
    sim.world_mut().entity_mut(entity).insert(script.clone());

    let got = sim.world_mut().get::<LuauScript>(entity).cloned();
    assert_eq!(got, Some(script));
}

/// ⭐ As funções que mexem no MUNDO passam pela pista, que só existe durante um tique da cena
/// (`lane.rs`): fora dele dizem porquê, em vez de pôr um pedido numa fila que ninguém drenava (o
/// controlo morto que a página de ajuda tinha de confessar). O que elas fazem com a cena a correr
/// mede-se em `ph2d-script/src/scene_api_tests.rs`.
#[test]
fn the_world_functions_outside_a_tick_say_why() {
    let host = ScriptHost::new().unwrap();
    for call in [
        "ph2d.get(1, 'x')",
        "ph2d.find_by_name('Player')",
        "ph2d.spawn()",
        "ph2d.despawn(12345)",
        "ph2d.attach_script(7, 'bob.luau')",
    ] {
        let err = host.runtime().eval(call).unwrap_err().to_string();
        assert!(err.contains("only while the scene runs"), "{call}: {err}");
    }
}

#[test]
fn luau_state_set_get_round_trips_number() {
    let host = ScriptHost::new().unwrap();
    host.runtime()
        .eval("ph2d.state_set(42, 'hp', 87.5)")
        .unwrap();
    let got: f64 = host
        .runtime()
        .lua()
        .load("return ph2d.state_get(42, 'hp')")
        .eval()
        .unwrap();
    assert!((got - 87.5).abs() < 1e-9);
}

#[test]
fn luau_state_set_get_round_trips_string() {
    let host = ScriptHost::new().unwrap();
    host.runtime()
        .eval("ph2d.state_set(42, 'name', 'Boss')")
        .unwrap();
    let got: String = host
        .runtime()
        .lua()
        .load("return ph2d.state_get(42, 'name')")
        .eval()
        .unwrap();
    assert_eq!(got, "Boss");
}

#[test]
fn luau_state_keys_are_alphabetical() {
    let host = ScriptHost::new().unwrap();
    host.runtime()
        .eval(
            r#"
            ph2d.state_set(1, 'zeta', 1)
            ph2d.state_set(1, 'alpha', 2)
            ph2d.state_set(1, 'mu', 3)
        "#,
        )
        .unwrap();
    let keys: Vec<String> = host
        .runtime()
        .lua()
        .load("return ph2d.state_keys(1)")
        .eval()
        .unwrap();
    assert_eq!(keys, vec!["alpha", "mu", "zeta"]);
}

#[test]
fn luau_state_clear_drops_entity_state() {
    let host = ScriptHost::new().unwrap();
    host.runtime()
        .eval("ph2d.state_set(7, 'x', 1) ph2d.state_clear(7)")
        .unwrap();
    let got: mlua::Value = host
        .runtime()
        .lua()
        .load("return ph2d.state_get(7, 'x')")
        .eval()
        .unwrap();
    assert!(matches!(got, mlua::Value::Nil));
}

#[test]
fn host_seeds_state_visible_to_lua() {
    let host = ScriptHost::new().unwrap();
    host.provide_state(99, "hp", PodValue::Number(50.0));
    let got: f64 = host
        .runtime()
        .lua()
        .load("return ph2d.state_get(99, 'hp')")
        .eval()
        .unwrap();
    assert!((got - 50.0).abs() < 1e-9);
}

/// `Velocity` is the kind of component a downstream crate (shells/desktop)
/// would add — this test just exercises that the `SimComponent` blanket
/// trait works on a local type that derives `Component` from bevy_ecs.
#[test]
fn luau_script_is_a_sim_component() {
    fn assert_sim<T: SimComponent>() {}
    assert_sim::<LuauScript>();
}
