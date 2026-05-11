//! Integration tests for the `LuauScript` Component + its bindings.
//!
//! Verifies:
//! - `LuauScript::derive_lateral_key` is deterministic across entities.
//! - Luau bindings populate `SpawnQueue` correctly.
//! - `ph2d.state_get`/`set`/`keys` round-trip POD values.

use ph2d_asset::AssetId;
use ph2d_ecs::{Entity, SimComponent, SimWorld};
use ph2d_script::component::LuauScript;
use ph2d_script::io::SpawnCommand;
use ph2d_script::{PodValue, ScriptHost};

#[test]
fn lateral_key_is_deterministic_per_entity_bytecode_pair() {
    let e = Entity::from_raw_u32(13).unwrap();
    let a = AssetId::from_digest([0xAB; 32]);
    let k1 = LuauScript::derive_lateral_key(e, a);
    let k2 = LuauScript::derive_lateral_key(e, a);
    assert_eq!(k1, k2);
}

#[test]
fn lateral_key_differs_per_entity() {
    let a = AssetId::from_digest([0xAB; 32]);
    let k1 = LuauScript::derive_lateral_key(Entity::from_raw_u32(1).unwrap(), a);
    let k2 = LuauScript::derive_lateral_key(Entity::from_raw_u32(2).unwrap(), a);
    assert_ne!(k1, k2);
}

#[test]
fn luau_script_attaches_to_sim_entity() {
    let mut sim = SimWorld::new();
    let bytecode = AssetId::from_bytes(b"-- some script source");
    let entity = sim.world_mut().spawn_empty().id();
    let script = LuauScript::new(entity, bytecode);
    sim.world_mut().entity_mut(entity).insert(script);

    let got = sim.world_mut().get::<LuauScript>(entity).copied();
    assert_eq!(got, Some(script));
}

#[test]
fn luau_spawn_binding_queues_command() {
    let host = ScriptHost::new().unwrap();
    host.runtime().eval("ph2d.spawn()").unwrap();
    let cmds = host.drain_spawns();
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0], SpawnCommand::SpawnEmpty);
}

#[test]
fn luau_spawn_named_carries_name() {
    let host = ScriptHost::new().unwrap();
    host.runtime()
        .eval("ph2d.spawn_named('Player')")
        .unwrap();
    let cmds = host.drain_spawns();
    assert_eq!(cmds.len(), 1);
    assert_eq!(
        cmds[0],
        SpawnCommand::SpawnNamed {
            name: "Player".into()
        }
    );
}

#[test]
fn luau_despawn_carries_entity_bits() {
    let host = ScriptHost::new().unwrap();
    // Lua numbers are f64; we pass entity bits as a number.
    host.runtime().eval("ph2d.despawn(12345)").unwrap();
    let cmds = host.drain_spawns();
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0], SpawnCommand::Despawn { entity: 12345 });
}

#[test]
fn luau_attach_script_parses_hex_bytecode() {
    let host = ScriptHost::new().unwrap();
    // 64 chars of hex = 32 bytes — must match the AssetId::to_hex form.
    let hex = "deadbeef".repeat(8); // = 64 chars
    let script = format!("ph2d.attach_script(7, '{}')", hex);
    host.runtime().eval(script.as_str()).unwrap();
    let cmds = host.drain_spawns();
    assert_eq!(cmds.len(), 1);
    let expected_digest = {
        let mut d = [0u8; 32];
        for (i, chunk) in hex.as_bytes().chunks_exact(2).enumerate() {
            let s = std::str::from_utf8(chunk).unwrap();
            d[i] = u8::from_str_radix(s, 16).unwrap();
        }
        d
    };
    assert_eq!(
        cmds[0],
        SpawnCommand::AttachScript {
            entity: 7,
            bytecode: expected_digest
        }
    );
}

#[test]
fn luau_attach_script_rejects_non_64_char_hex() {
    let host = ScriptHost::new().unwrap();
    let err = host
        .runtime()
        .eval("ph2d.attach_script(1, 'short')")
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("64-char hex"), "got: {msg}");
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
fn luau_find_by_name_resolves_provided_snapshot() {
    let host = ScriptHost::new().unwrap();
    host.provide_name("Player", 0x1234_5678_u64);
    let got: f64 = host
        .runtime()
        .lua()
        .load("return ph2d.find_by_name('Player')")
        .eval()
        .unwrap();
    assert_eq!(got as u64, 0x1234_5678_u64);
}

#[test]
fn luau_find_by_name_returns_nil_for_missing_entity() {
    let host = ScriptHost::new().unwrap();
    let got: mlua::Value = host
        .runtime()
        .lua()
        .load("return ph2d.find_by_name('Nobody')")
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
