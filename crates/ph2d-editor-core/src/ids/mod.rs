//! Stable [`NodeId`] constants for the hero screen's interactive
//! widgets + helper mappings between fixture entity names and ids.
//!
//! Pre-populated in [`crate::interaction::WidgetStore`] at
//! construction time so the dispatcher always finds an entry on
//! hit-test.
//!
//! ## NodeId derivation (Wave 2 PR 11.3 — convention-by-discovery)
//!
//! Chrome ids are derived from stable string slugs via
//! [`ph2d_tool_registry::hash_node_id`] (FNV-1a 64-bit, `const fn`).
//! Adding a new chrome widget no longer requires hunting for a free
//! integer in some hand-allocated range — pick a unique slug
//! (`"topbar.save"`-style by convention) and the hash is deterministic
//! cross-platform. Collisions are caught by the
//! `tests/architecture/node_id_collisions.rs` regression test, which
//! enumerates every chrome const and asserts pairwise uniqueness.
//!
//! Pre-PR-11.3 the file allocated ids by hand in numeric buckets
//! (100..199 TopBar, 200..299 Rail, 300..399 Inspector, 400..499
//! Hierarchy, 600..699 BlenderColorPicker, 800..899 Notes, 900..999
//! Context menus, 950..999 Widget Gallery). Six collisions had already
//! slipped in (e.g. 380, 381, 382 + 853, 854, 855) — exactly the class
//! of bug the M14.4d audit comment below warned about, and exactly the
//! class of bug hash-based ids eliminate.
//!
//! ## Hierarchy fixture rows kept numeric
//!
//! [`HIER_PLAYER`]..[`HIER_MAIN_CAMERA`] (12 fixture entity row ids in
//! the 400..411 range) are deliberately NOT hashed. They participate
//! in the [`EYE_TOGGLE_BIT`] / [`EXPAND_TOGGLE_BIT`] companion-id math
//! at the bottom of this file (`row.0 | bit`), which assumes the high
//! bits 61+62 are free. FNV-1a output is uniformly distributed over 64
//! bits, so hashing rows would silently break the companion-detection
//! invariant on ~25% of slugs. Real (non-fixture) rows allocated by
//! the host bridge sit at `BASE_NODE_ID = 100_000` upward and also
//! have those bits clear. See `hero_bridge.rs` for the runtime path.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

mod chrome;
mod gallery;
mod inspector;
mod menus;

pub use chrome::*;
pub use gallery::*;
pub use inspector::*;
pub use menus::*;
