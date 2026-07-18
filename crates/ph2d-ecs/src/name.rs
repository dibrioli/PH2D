//! `Name` component — human-readable label for a `SimWorld` entity.
//!
//! Authored by hand in the editor, by `ph2d.attach_name(entity, "...")`
//! in Luau (M14.2+), or auto-derived from a Prefab's filename when
//! spawned via `spawn_prefab` (M14.3). Consumed by:
//!
//! - the editor Hierarchy panel (display label) — M15+ wiring,
//! - `ph2d.find_by_name` from Luau (M14.2),
//! - `audit.log` records (HR-11) — name is the friendly identifier
//!   that survives id churn across saves.
//!
//! HR-14 mitigation: [`Name::VERSION`] is the stable schema marker
//! until `#[derive(Saveable)]` macro lands.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// Human-readable label for a sim entity. Wraps `String` so it can be
/// cheaply cloned on the rare paths that need to (`HierarchySnapshot`
/// build, audit log emit) while still being `serde`-portable.
///
/// Empty names are tolerated — querying by `""` returns nothing.
/// Names are **not** required to be unique; `ph2d.find_by_name`
/// returns the first match in entity-id order. Author tools that
/// care about uniqueness enforce it in the editor layer.
#[derive(Component, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Name(pub String);

impl Name {
    /// Schema version. Bumped (alongside a migration function) when
    /// the on-disk layout of `Name` changes.
    pub const VERSION: u32 = 1;

    /// Construct from anything stringy. Accepts `&str`, `String`,
    /// `Cow<str>`, etc. — `Into<String>` is the right bound here.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Borrow the underlying name without allocating.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// FNV-1a of a name → a **stable id for the entity that carries it**.
///
/// # Why this exists, and why it is one function
///
/// The ECS has no stable id. `Entity::to_bits()` is an *allocation* id: the
/// undo restore **respawns** every entity with fresh bits, and a project load
/// spawns them anew. So nothing that has to outlive a Ctrl+Z or a save can
/// point at an entity by its bits.
///
/// Worse than "it dangles": entity bits stored *inside a component's bytes*
/// poison the undo itself. `canonicalize` sorts entity rows by their component
/// bytes and remaps only the structural `parent` field — so a component holding
/// bits makes two logically identical states compare different, which is
/// exactly the spurious-undo-step bug `canonicalize` exists to kill.
///
/// The editor's answer is the **name**: unique (`shells/desktop/name_unique.rs`)
/// and stable across sessions. The timeline binding has used this hash since
/// W4.T6 to survive delete+undo and project load; a physics joint needs the same
/// thing to name the two bodies it connects (W3). Two copies of this function
/// would be two answers to *"what is this object's durable id?"*, so it lives
/// here — beside [`Name`], the concept it is derived from.
///
/// `0` is reserved as "no id" by callers (the timeline's `WireId::NULL`), so a
/// hash that lands on zero is nudged to one.
///
/// ⚠️ **Renaming an object changes its id**, and therefore detaches whatever
/// pointed at it. That is a property of choosing the name as the identity, and
/// it is shared with every timeline binding in the repo — not something this
/// function papers over. The day that is not acceptable, the fix is a real
/// `StableId` component assigned at spawn, migrated for *both* consumers at
/// once; a second identity scheme for one of them would be the divergence this
/// doc-comment is here to prevent.
pub fn stable_name_id(name: &str) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut h = OFFSET;
    for b in name.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(PRIME);
    }
    if h == 0 { 1 } else { h }
}

impl From<&str> for Name {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl From<String> for Name {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl SimComponent for Name {}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The hash is a FILE FORMAT, and these literals are the oracle.**
    ///
    /// Every project already on disk carries timeline bindings stamped with
    /// these exact numbers. "Improve" the hash and every saved animation
    /// quietly detaches from its object on the next load — nothing crashes,
    /// nothing warns, the rows just stop being there.
    ///
    /// So the expected values are **not** computed by calling the function
    /// (that oracle is always green — the trap this line has hit five times).
    /// They come from an independent FNV-1a run outside this codebase.
    #[test]
    fn the_stable_id_is_pinned_to_values_computed_outside_this_codebase() {
        assert_eq!(stable_name_id(""), 14_695_981_039_346_656_037);
        assert_eq!(stable_name_id("Sprite"), 17_555_429_528_309_779_676);
        assert_eq!(stable_name_id("Sprite 2"), 14_331_061_606_276_528_690);
        assert_eq!(stable_name_id("Player"), 3_692_324_345_213_718_176);
        // Non-ASCII: the hash walks BYTES, so this pins the UTF-8 reading too.
        assert_eq!(stable_name_id("Chão"), 16_810_678_303_362_338_921);
    }

    /// Zero is reserved by callers to mean "no id", so it must never be
    /// produced. This gate cannot reach a real collision (finding a preimage
    /// of 0 is the point of a hash), so it pins the *branch* instead — the
    /// nudge is the only thing standing between a legitimate name and an id
    /// its consumers read as null.
    #[test]
    fn the_stable_id_is_never_zero() {
        for n in ["", "a", "Sprite", "Chão", "  "] {
            assert_ne!(stable_name_id(n), 0, "name {n:?} hashed to the null id");
        }
    }

    #[test]
    fn round_trip_str() {
        let n = Name::new("Player");
        assert_eq!(n.as_str(), "Player");
    }

    #[test]
    fn from_str_and_string_equivalent() {
        let a: Name = "Enemy".into();
        let b: Name = String::from("Enemy").into();
        assert_eq!(a, b);
    }

    #[test]
    fn serde_roundtrip_via_postcard() {
        let n = Name::new("Boss");
        let bytes = postcard_util::to_vec(&n);
        let back: Name = postcard_util::from_bytes(&bytes);
        assert_eq!(n, back);
    }

    // Inline postcard fns to avoid adding postcard as a workspace dep
    // just for a doctest. (Production save/load uses postcard via
    // ph2d-asset.)
    mod postcard_util {
        use super::*;
        pub fn to_vec(n: &Name) -> Vec<u8> {
            // Manual encode: u32-LE len + bytes. Mirrors postcard's
            // varint-encoded length for short strings (< 128).
            let s = n.as_str().as_bytes();
            let mut out = Vec::with_capacity(s.len() + 4);
            out.extend_from_slice(&(s.len() as u32).to_le_bytes());
            out.extend_from_slice(s);
            out
        }
        pub fn from_bytes(bytes: &[u8]) -> Name {
            let (len_b, rest) = bytes.split_at(4);
            let len = u32::from_le_bytes(len_b.try_into().unwrap()) as usize;
            let s = std::str::from_utf8(&rest[..len]).unwrap().to_owned();
            Name(s)
        }
    }
}
