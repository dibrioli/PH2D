//! ⭐ **As acções do Input Map para os scripts** (`ph2d.action_*`, `docs/InputMap/01_plano.md` §5):
//! o idioma do `key_*`, lido do MESMO mapa e `ActionState` que o Play resolve
//! ([`Devices::feed_actions`]). Um nome que o mapa não tem é erro que lista as acções dele.

use crate::io::{Devices, Edge};

fn unknown(func: &str, name: &str, d: &Devices) -> mlua::Error {
    let names = d.action_names();
    let list = if names.is_empty() {
        "the Input Map has no actions".to_owned()
    } else {
        let quoted: Vec<String> = names.iter().map(|n| format!("\"{n}\"")).collect();
        format!("the Input Map has {}", quoted.join(", "))
    };
    mlua::Error::RuntimeError(format!("ph2d.{func}: unknown action '{name}' — {list}"))
}

pub(crate) fn wire(lua: &mlua::Lua, devices: &Devices) -> mlua::Result<()> {
    let ph2d: mlua::Table = lua.globals().get("ph2d")?;
    for (func, edge) in [
        ("action_down", Edge::Down),
        ("action_pressed", Edge::Pressed),
        ("action_released", Edge::Released),
    ] {
        let d = devices.clone();
        let f = lua.create_function(move |_, name: String| {
            d.action(&name, edge)
                .ok_or_else(|| unknown(func, &name, &d))
        })?;
        ph2d.set(func, f)?;
    }
    let d = devices.clone();
    let strength = lua.create_function(move |_, name: String| {
        d.action_strength(&name)
            .map(f64::from)
            .ok_or_else(|| unknown("action_strength", &name, &d))
    })?;
    ph2d.set("action_strength", strength)
}
