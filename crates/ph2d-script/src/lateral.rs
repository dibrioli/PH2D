//! Per-entity lateral state storage — the canonical home for FSM /
//! BT / Dialogue / Tween-progress state that lives **outside** the
//! ECS but still needs to roundtrip through save / replay / hot
//! reload.
//!
//! # Why a separate store
//!
//! Per ADR-0019 §"Storage lateral" and HR-16, state that varies per
//! script-attached entity (e.g. an enemy's FSM cursor, a dialogue
//! branch index) does not belong in the ECS as bespoke components —
//! every new field would mean a new archetype, and the editor has no
//! schema to inspect. Defold/Roblox both expose a lateral
//! `state_table(entity)` for the same reason. Here it's
//! [`StateTable`].
//!
//! # HR-16 constraints (enforced at the FFI boundary)
//!
//! - **POD-like values only.** [`PodValue`] enumerates the accepted
//!   types: `Nil`, `Bool`, `Number(f64)`, `String`, `Entity(u64)`,
//!   `Table(BTreeMap<String, PodValue>)`. Lua `function` / `thread`
//!   / `userdata` / `lightuserdata` are rejected with a
//!   [`mlua::Error::RuntimeError`] before they reach the store —
//!   they wouldn't serialize (HR-14) nor survive a hot reload.
//! - **Sorted iteration.** Backing storage is `BTreeMap`, never
//!   `HashMap` (ADR-0022). `keys()` and `iter()` return
//!   alphabetically-sorted order so two replays on different
//!   platforms produce identical byte streams (HR-5).
//! - **Bounded nesting depth.** Nested `Table` values cannot exceed
//!   [`StateTable::MAX_DEPTH`] = 16. Defends against accidental
//!   self-referential cycles that would loop the recursive setter.
//!
//! # Concurrency model
//!
//! `Arc<Mutex<...>>` matches the existing `WriteQueue` / `ReadSnapshot`
//! pattern in [`crate::io`]. Luau is single-threaded by design
//! (§12.2 of the SKILL); the mutex is uncontended and amortizes to a
//! single atomic — keeping the pattern consistent across the crate
//! is more valuable than micro-optimizing to `Rc<RefCell<>>` here.
//!
//! # Hot reload (M14.2 contract)
//!
//! [`ScriptHost::load_script`] preserves the `StateTable` across a
//! reset: lateral keys survive bytecode reload. The script's `init()`
//! function re-runs and can read its previous state via
//! `ph2d.state_get(self, "foo")`. If the user wants a clean slate,
//! they call `ph2d.state_clear(self)` explicitly.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use mlua::{IntoLua, Lua, Value as LuaValue};
use serde::{Deserialize, Serialize};

/// POD-like value type accepted by [`StateTable`].
///
/// Every variant is `serde`-portable; the enum participates directly
/// in the postcard pipeline that `PrefabDoc` / `SceneDoc` use.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PodValue {
    Nil,
    Bool(bool),
    /// `f64` is the canonical Lua number; integer values fit losslessly
    /// up to 2^53.
    Number(f64),
    String(String),
    /// Opaque entity handle — `bevy_ecs::Entity::to_bits()`. Kept
    /// distinct from `Number` so save migrations can distinguish a
    /// reference from arithmetic data.
    Entity(u64),
    /// Nested map (max depth [`StateTable::MAX_DEPTH`]). Iteration is
    /// alphabetical by `BTreeMap` invariant.
    Table(BTreeMap<String, PodValue>),
}

impl PodValue {
    /// Maximum nesting depth permitted in `Table` values. Matched to
    /// HR-16 ("max depth 16"). The check fires when the converter
    /// recurses on a deeply-nested Lua table.
    pub const MAX_DEPTH: usize = 16;

    /// Convert a `mlua::Value` into a [`PodValue`], rejecting any
    /// non-POD type with a Luau runtime error.
    ///
    /// `depth` is the current nesting level (call with 0 from the
    /// top-level binding). Exceeding [`PodValue::MAX_DEPTH`] is a
    /// runtime error — defends against pathological inputs that
    /// would blow the stack in a recursive serializer.
    pub fn from_lua(value: LuaValue, depth: usize) -> mlua::Result<Self> {
        if depth > Self::MAX_DEPTH {
            return Err(mlua::Error::RuntimeError(format!(
                "HR-16: state table nesting depth exceeded ({} > {})",
                depth,
                Self::MAX_DEPTH
            )));
        }
        match value {
            LuaValue::Nil => Ok(PodValue::Nil),
            LuaValue::Boolean(b) => Ok(PodValue::Bool(b)),
            LuaValue::Integer(n) => Ok(PodValue::Number(n as f64)),
            LuaValue::Number(n) => Ok(PodValue::Number(n)),
            LuaValue::String(s) => Ok(PodValue::String(s.to_str()?.to_owned())),
            LuaValue::Table(t) => {
                let mut map = BTreeMap::new();
                for pair in t.pairs::<LuaValue, LuaValue>() {
                    let (k, v) = pair?;
                    let key = match k {
                        LuaValue::String(s) => s.to_str()?.to_owned(),
                        LuaValue::Integer(n) => n.to_string(),
                        LuaValue::Number(n) => n.to_string(),
                        _ => {
                            return Err(mlua::Error::RuntimeError(
                                "HR-16: state table keys must be string or number".into(),
                            ));
                        }
                    };
                    map.insert(key, PodValue::from_lua(v, depth + 1)?);
                }
                Ok(PodValue::Table(map))
            }
            LuaValue::Function(_)
            | LuaValue::Thread(_)
            | LuaValue::UserData(_)
            | LuaValue::LightUserData(_) => Err(mlua::Error::RuntimeError(
                "HR-16: state table rejects function / thread / userdata values \
                 (they cannot be serialized for save/replay/hot-reload)"
                    .into(),
            )),
            _ => Err(mlua::Error::RuntimeError(format!(
                "HR-16: unsupported lua value kind: {value:?}"
            ))),
        }
    }

    /// Recover a `mlua::Value` from a [`PodValue`] for the
    /// `ph2d.state_get` path. The reverse of [`PodValue::from_lua`].
    pub fn to_lua(self, lua: &Lua) -> mlua::Result<LuaValue> {
        match self {
            PodValue::Nil => Ok(LuaValue::Nil),
            PodValue::Bool(b) => Ok(LuaValue::Boolean(b)),
            PodValue::Number(n) => Ok(LuaValue::Number(n)),
            PodValue::String(s) => s.into_lua(lua),
            PodValue::Entity(e) => {
                // Entity ids round-trip as Lua numbers (f64). Luau's
                // `Integer` is i32 — too narrow for `Entity::to_bits()`
                // which is u64. f64 holds the full 53-bit safe-integer
                // range, which is more than enough for any realistic
                // bevy_ecs entity (id 32-bit + generation 32-bit).
                // The Luau side treats this as an opaque handle (HR-8);
                // any arithmetic on it is the script author's bug.
                Ok(LuaValue::Number(e as f64))
            }
            PodValue::Table(map) => {
                let t = lua.create_table()?;
                for (k, v) in map {
                    t.set(k, v.to_lua(lua)?)?;
                }
                Ok(LuaValue::Table(t))
            }
        }
    }
}

/// Per-entity lateral state. Key is a `lateral_key` `u64` (derived
/// from the entity + bytecode hash at attach time — see
/// [`crate::component::LuauScript`]); inner map keys are arbitrary
/// strings sorted alphabetically.
#[derive(Clone, Default)]
pub struct StateTable {
    inner: Arc<Mutex<BTreeMap<u64, BTreeMap<String, PodValue>>>>,
}

impl StateTable {
    pub const MAX_DEPTH: usize = PodValue::MAX_DEPTH;

    pub fn new() -> Self {
        Self::default()
    }

    /// Insert (or overwrite) a single field on `lateral_key`'s table.
    pub fn set(&self, lateral_key: u64, field: &str, value: PodValue) {
        let mut g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        g.entry(lateral_key)
            .or_default()
            .insert(field.to_owned(), value);
    }

    /// Read a single field. Returns `None` if either the entity or
    /// the field is absent.
    pub fn get(&self, lateral_key: u64, field: &str) -> Option<PodValue> {
        let g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        g.get(&lateral_key).and_then(|m| m.get(field).cloned())
    }

    /// Alphabetically-sorted keys for `lateral_key`. Returns an empty
    /// vec if the entity has no entries yet. Sorted by `BTreeMap`
    /// invariant — no extra sort step needed (HR-5 + HR-16).
    pub fn keys(&self, lateral_key: u64) -> Vec<String> {
        let g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        g.get(&lateral_key)
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Drop every entry for `lateral_key`. No-op if absent.
    pub fn clear(&self, lateral_key: u64) {
        let mut g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        g.remove(&lateral_key);
    }

    /// Drop every entry for every entity. Used in tests and by
    /// host-level "new scene" flows. **Never** called from a hot
    /// reload — preserving state across reload is the point of this
    /// store.
    pub fn clear_all(&self) {
        let mut g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        g.clear();
    }

    /// Number of distinct entities with at least one field set.
    pub fn len(&self) -> usize {
        self.inner.lock().unwrap_or_else(|p| p.into_inner()).len()
    }

    /// True if no entity has any lateral state stored.
    pub fn is_empty(&self) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .is_empty()
    }

    /// Snapshot the entire store into a deterministic flat list of
    /// `(lateral_key, field, value)` triples — sorted by
    /// `(lateral_key, field)`. Used by save/replay (HR-14) and the
    /// editor's component inspector (M14.3).
    pub fn snapshot(&self) -> Vec<(u64, String, PodValue)> {
        let g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        let mut out = Vec::with_capacity(g.values().map(|m| m.len()).sum());
        for (&key, fields) in g.iter() {
            for (field, value) in fields.iter() {
                out.push((key, field.clone(), value.clone()));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_round_trip() {
        let s = StateTable::new();
        s.set(42, "hp", PodValue::Number(100.0));
        assert_eq!(s.get(42, "hp"), Some(PodValue::Number(100.0)));
        assert_eq!(s.get(42, "missing"), None);
        assert_eq!(s.get(7, "hp"), None);
    }

    #[test]
    fn keys_are_alphabetical() {
        let s = StateTable::new();
        s.set(1, "zeta", PodValue::Nil);
        s.set(1, "alpha", PodValue::Nil);
        s.set(1, "mu", PodValue::Nil);
        assert_eq!(s.keys(1), vec!["alpha", "mu", "zeta"]);
    }

    #[test]
    fn clear_removes_entity() {
        let s = StateTable::new();
        s.set(7, "x", PodValue::Bool(true));
        assert_eq!(s.len(), 1);
        s.clear(7);
        assert_eq!(s.len(), 0);
        assert_eq!(s.get(7, "x"), None);
    }

    #[test]
    fn snapshot_order_is_deterministic() {
        let s = StateTable::new();
        s.set(2, "b", PodValue::Number(2.0));
        s.set(1, "z", PodValue::Number(1.0));
        s.set(1, "a", PodValue::Number(0.5));
        let snap = s.snapshot();
        assert_eq!(snap.len(), 3);
        // Sorted by (key, field): (1,a) (1,z) (2,b)
        assert_eq!(snap[0].0, 1);
        assert_eq!(snap[0].1, "a");
        assert_eq!(snap[1].0, 1);
        assert_eq!(snap[1].1, "z");
        assert_eq!(snap[2].0, 2);
        assert_eq!(snap[2].1, "b");
    }
}
