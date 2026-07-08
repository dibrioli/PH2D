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

pub mod sprite;

pub use sprite::{SpriteAnimation, SpriteProp, apply_sprite_animations};
