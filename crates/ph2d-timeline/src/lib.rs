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
mod apply_path;
mod apply_views;
pub mod autokey;
pub mod binding;
pub mod clipboard;
mod clock;
pub mod doc;
pub mod doc_path;
mod expr_pass;
pub mod graph;
pub mod history;
pub mod intent;
pub mod intent_apply;
mod intent_apply_fade;
mod intent_apply_path;
pub mod nest;
pub mod nest_map;
pub mod onion;
pub mod path;
pub mod path_convert;
pub mod persist;
pub mod pose;
pub mod prop;
mod refusal;
pub mod signal;
pub mod snapshot;
pub mod speed;
pub mod sprite;
pub mod stack;
pub mod stack_edit;
mod stack_eval;
mod stack_frames;
mod stack_hold;
pub mod state;
mod strip_edge_edit;

pub use apply::{
    apply_from_doc, apply_from_doc_except, clip_playhead, key_home, key_time, remapped_time,
};
pub use apply_views::{apply_active_clip, apply_container};
pub use autokey::{
    AutokeyPlan, PoseSample, autokey_props, autokey_props_solo, key_value_in_active_clip,
};
pub use binding::{TargetBinding, WireId};
pub use clipboard::{ClipboardKey, TimelineClipboard};
pub use doc::{
    DEFAULT_DURATION_SECONDS, DEFAULT_FPS, DOC_VERSION, MAX_CLIPS, Marker, NamedClip, TimelineDoc,
};
pub use doc_path::AutoOrient;
pub use graph::{
    drawn_extent, handle_coords, handle_point, sample_keys, segment_handle_points,
    speed_handle_tip, value_extent, weighted_with_handle, weighted_with_speed_handle,
};
pub use history::{HISTORY_CAP, TimelineHistory};
pub use intent::TimelineIntent;
pub use intent_apply::{apply_intent, snap_time, sync_container_loop, sync_transport_loop};
pub use nest::{
    EMPTY_CONTAINER_SECONDS, MAX_CONTAINERS, NamedContainer, StackHost, container_bar_seconds,
};
pub use nest_map::{ContainerMap, EnterStep, HostClock, entry_clock, entry_map, entry_reach};
pub use onion::{OnionMode, OnionSettings};
pub use path::{MotionPath, PathAnchor, PathSample, TangentKind};
pub use path_convert::{ConversionReport, PositionKeyMode};
pub use persist::{refresh_and_heal_bindings, resolve_entities, stamp_wire_ids};
pub use pose::{animated_entities, entity_key_times, pose_at};
pub use prop::{Algebra, PropKind};
pub use refusal::{KeyRefusal, NestRefusal};
pub use signal::{TimelineSignal, signals_crossed};
pub use snapshot::{ContainerView, KeyView, LaneView, StripView, TimelineViewSnapshot, TrackView};
pub use speed::{sample_speed, segment_endpoint_speed, speed_extent};
pub use stack::{ClipLane, ClipStrip, LaneMode, StripId, StripLoop, StripSource, mark_index};
pub use stack_edit::MAX_LANES;
// The anim vocab the public snapshot/doc API names, re-exported so consumers of
// `KeyView`/`TrackView`/`SelectedKey` don't need a direct `ph2d-anim` dep.
pub use ph2d_anim::{
    AnimTarget, AnimValue, Easing, EasingFamily, EasingMode, Extrap, ExtrapSide, Interp, KeyId,
};
pub use sprite::{SpriteAnimation, SpriteProp, apply_sprite_animations};
pub use state::{SelectedKey, Selection, TimelineFlags, TimelineState};
