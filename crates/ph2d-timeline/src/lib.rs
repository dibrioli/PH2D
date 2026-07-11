#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! `ph2d-timeline` — the general-timeline **runtime**: bind animation data
//! ([`ph2d_anim`] `Clip`s) to scene objects and apply it at the engine
//! [`Playhead`](ph2d_core::Playhead).
//!
//! `ph2d-anim` holds the timeline *data* (target-agnostic keyframes + curves)
//! and `ph2d-core` holds the *cursor* (the Playhead). This crate is the bridge
//! that makes them animate real objects: it owns the per-system **binding**
//! (which clip drives which entity) and **apply** (sample at the playhead →
//! write the property).
//!
//! # Target resolution
//!
//! The opaque [`ph2d_anim::AnimTarget`] is resolved to a concrete property by a
//! **per-system convention that lives with the consumer**, not in `ph2d-anim`.
//! The first such consumer is [`sprite`] (a [`SpriteAnimation`] drives an
//! entity's `Transform`); vector / painter / node-param resolvers plug in later
//! as sibling modules following the same shape.

pub mod apply;
pub mod autokey;
pub mod binding;
pub mod clipboard;
pub mod doc;
pub mod graph;
pub mod history;
pub mod intent;
pub mod persist;
pub mod prop;
pub mod snapshot;
pub mod speed;
pub mod sprite;
pub mod state;

pub use apply::{apply_from_doc, apply_from_doc_except};
pub use autokey::{PoseSample, autokey_props};
pub use binding::{TargetBinding, WireId};
pub use clipboard::{ClipboardKey, TimelineClipboard};
pub use doc::{DEFAULT_FPS, DOC_VERSION, Marker, NamedClip, TimelineDoc};
pub use graph::{
    drawn_extent, handle_coords, handle_point, sample_keys, segment_handle_points,
    speed_handle_tip, value_extent, weighted_with_handle, weighted_with_speed_handle,
};
pub use history::{HISTORY_CAP, TimelineHistory};
pub use intent::{TimelineIntent, apply_intent, snap_time};
pub use persist::{refresh_and_heal_bindings, resolve_entities, stamp_wire_ids};
pub use prop::PropKind;
pub use snapshot::{KeyView, TimelineViewSnapshot, TrackView};
pub use speed::{sample_speed, segment_endpoint_speed, speed_extent};
// The anim vocab the public snapshot/doc API names, re-exported so consumers of
// `KeyView`/`TrackView`/`SelectedKey` don't need a direct `ph2d-anim` dep.
pub use ph2d_anim::{AnimTarget, AnimValue, Easing, EasingFamily, EasingMode, Interp, KeyId};
pub use sprite::{SpriteAnimation, SpriteProp, apply_sprite_animations};
pub use state::{SelectedKey, Selection, TimelineFlags, TimelineState};
