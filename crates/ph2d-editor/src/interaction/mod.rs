//! Editor input pipeline + retained widget state (ADR-0024).
//!
//! Three pieces work together:
//!
//! - [`WidgetStore`] holds the per-widget interactive state
//!   (hovered, pressed, focused, drag value, text content) keyed by
//!   the same `NodeId` AccessKit uses. Pre-populated when a screen
//!   is constructed; never grows during the hot path.
//! - [`HitIndex`] maps screen coords back to the `NodeId` of the
//!   widget under the cursor. Rebuilt by the paint pass each frame
//!   (paint registers each rect as it emits geometry).
//! - [`dispatch_pointer`] / [`dispatch_key`] read the index, mutate
//!   the store, and emit [`WidgetEvent`]s into a per-frame `bumpalo`
//!   arena that the caller drains and resets.
//!
//! ## HR-3 (zero alloc on the hot path)
//!
//! The four mitigations from the ADR are wired here:
//!
//! 1. The store uses `HashMap::with_capacity(N)` and exposes only
//!    `register` (called at construction) — never `insert`. Mutations
//!    are `get_mut`, which never reallocates.
//! 2. [`HitIndex`] backs the rect list with [`smallvec::SmallVec`]
//!    inline up to 128 entries.
//! 3. Emitted events live in a caller-supplied `bumpalo::Bump` and
//!    are returned as `&'frame [WidgetEvent]`.
//! 4. [`WidgetEvent`] has no heap payload — value-bearing variants
//!    carry only the `NodeId`; the caller re-reads from the store.
//!
//! The bench `tests/interaction_no_alloc.rs` enforces this with
//! `dhat-rs`.

pub mod dispatch;
pub mod hit;
pub mod state;

pub use dispatch::{
    KEY_ARROW_DOWN, KEY_ARROW_LEFT, KEY_ARROW_RIGHT, KEY_ARROW_UP, KEY_BACKSPACE, KEY_ENTER,
    KEY_ESCAPE, KEY_SPACE, KEY_TAB, dispatch_key, dispatch_pointer, dispatch_text_input,
};
pub use hit::HitIndex;
pub use state::{BlenderHitKind, InteractiveState, WidgetEvent, WidgetStore, format_number};
