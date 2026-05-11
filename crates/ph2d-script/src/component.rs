//! `LuauScript` Component — the canonical way to attach gameplay
//! script behavior to a SimWorld entity (ADR-0025 §"Gameplay script
//! is a Component, not a class").
//!
//! Unity's `MonoBehaviour` model is a class hierarchy that PH2D
//! deliberately doesn't have. Instead, a `LuauScript` is a
//! `SimComponent` carrying two opaque references:
//!
//! - **`bytecode: AssetId`** — content-addressed identity (HR-6) of
//!   the compiled Luau bytecode. Multiple entities sharing the same
//!   `bytecode` value share one compiled program (no per-entity
//!   bytecode copy).
//! - **`lateral_key: u64`** — the key in
//!   [`crate::lateral::StateTable`] where this entity's per-instance
//!   state lives (`hp`, FSM cursor, dialogue branch, …). Computed
//!   deterministically from the entity id + bytecode hash at
//!   attach time, so a hot reload — which preserves entity ids and
//!   re-uses the same bytecode hash — can find the same row.
//!
//! HR-14 mitigation: [`LuauScript::VERSION`] is the stable schema
//! marker until the `Saveable` derive lands.

use bevy_ecs::component::Component;
use ph2d_asset::AssetId;
use ph2d_ecs::{Entity, SimComponent};
use serde::{Deserialize, Serialize};

/// A Luau script attachment on a sim entity. The `Component` itself
/// is small and `Copy` so it lives cheaply in archetype storage; all
/// the actual state lives behind the two opaque ids.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LuauScript {
    /// Content-addressed handle for the compiled Luau bytecode. The
    /// host's [`crate::ScriptHost`] resolves this against an
    /// [`ph2d_asset::AssetDb`] before invoking entity-scoped script
    /// hooks.
    pub bytecode: AssetId,
    /// Key into [`crate::lateral::StateTable`]. Derived via
    /// [`LuauScript::derive_lateral_key`] so we don't need a global
    /// counter (deterministic across hot reloads and replay).
    pub lateral_key: u64,
}

impl LuauScript {
    /// Schema version. Bumped (alongside a migration function) when
    /// the on-disk layout of `LuauScript` changes.
    pub const VERSION: u32 = 1;

    /// Derive a deterministic `lateral_key` from the sim entity and
    /// bytecode hash. The construction:
    ///
    /// ```text
    /// lateral_key = entity.to_bits() XOR (bytecode_hash[0..8] as u64-LE)
    /// ```
    ///
    /// Survives:
    /// - Entity id reuse (different `to_bits()` for the same numeric
    ///   id at a different generation).
    /// - Bytecode hot reload with identical source (hash is
    ///   stable → key unchanged → state survives).
    /// - Bytecode swap (different source → different hash → fresh
    ///   key → fresh state without manual clear).
    pub fn derive_lateral_key(entity: Entity, bytecode: AssetId) -> u64 {
        let h = bytecode.as_bytes();
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&h[..8]);
        let bytecode_lo = u64::from_le_bytes(buf);
        entity.to_bits() ^ bytecode_lo
    }

    /// Convenience constructor that derives the lateral key for the
    /// caller. Most call sites should prefer this over building the
    /// struct field-by-field.
    pub fn new(entity: Entity, bytecode: AssetId) -> Self {
        Self {
            bytecode,
            lateral_key: Self::derive_lateral_key(entity, bytecode),
        }
    }
}

impl SimComponent for LuauScript {}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw_entity(bits: u32) -> Entity {
        Entity::from_raw_u32(bits).unwrap()
    }

    fn raw_asset(byte: u8) -> AssetId {
        AssetId::from_digest([byte; 32])
    }

    #[test]
    fn derive_is_pure() {
        let e = raw_entity(7);
        let a = raw_asset(0x10);
        let k1 = LuauScript::derive_lateral_key(e, a);
        let k2 = LuauScript::derive_lateral_key(e, a);
        assert_eq!(k1, k2);
    }

    #[test]
    fn different_entities_get_different_keys() {
        let a = raw_asset(0x10);
        let k1 = LuauScript::derive_lateral_key(raw_entity(1), a);
        let k2 = LuauScript::derive_lateral_key(raw_entity(2), a);
        assert_ne!(k1, k2);
    }

    #[test]
    fn different_bytecode_gets_different_keys() {
        let e = raw_entity(42);
        let k1 = LuauScript::derive_lateral_key(e, raw_asset(0x10));
        let k2 = LuauScript::derive_lateral_key(e, raw_asset(0x20));
        assert_ne!(k1, k2);
    }

    #[test]
    fn new_matches_derive() {
        let e = raw_entity(99);
        let a = raw_asset(0xAA);
        let s = LuauScript::new(e, a);
        assert_eq!(s.lateral_key, LuauScript::derive_lateral_key(e, a));
        assert_eq!(s.bytecode, a);
    }
}
