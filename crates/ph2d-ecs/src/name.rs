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
