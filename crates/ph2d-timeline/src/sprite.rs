//! Sprite binding — a `Clip` bound to a sprite entity, applied to its
//! `Transform` at the engine Playhead. The first consumer of the general
//! timeline.
//!
//! The opaque [`AnimTarget`] is resolved to a `Transform` property by the
//! [`SpriteProp`] convention that lives **here** (with the consumer), keeping
//! `ph2d-anim` target-agnostic. A track bound to `SpriteProp::X.target()`
//! animates that property; unbound properties keep their authored value.

use bevy_ecs::component::Component;
use ph2d_anim::{AnimTarget, AnimValue, Clip};
use ph2d_ecs::{SimComponent, Transform, World};

/// Which `Transform` property an opaque [`AnimTarget`] drives, in this sprite
/// resolver's convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum SpriteProp {
    /// Local translation X (meters).
    TranslationX = 0,
    /// Local translation Y (meters).
    TranslationY = 1,
    /// Rotation (radians, CCW from +X).
    Rotation = 2,
    /// Local scale X.
    ScaleX = 3,
    /// Local scale Y.
    ScaleY = 4,
}

impl SpriteProp {
    /// Every property, for authoring / iteration.
    pub const ALL: [SpriteProp; 5] = [
        SpriteProp::TranslationX,
        SpriteProp::TranslationY,
        SpriteProp::Rotation,
        SpriteProp::ScaleX,
        SpriteProp::ScaleY,
    ];

    /// The opaque [`AnimTarget`] a track uses to drive this property.
    #[must_use]
    pub const fn target(self) -> AnimTarget {
        AnimTarget::new(self as u64)
    }
}

/// A `Clip` bound to a sprite entity. [`apply_sprite_animations`] samples it at
/// the Playhead and writes the entity's [`Transform`].
#[derive(Component, Clone, Debug)]
pub struct SpriteAnimation {
    /// The animation; its tracks are keyed by [`SpriteProp::target`].
    pub clip: Clip,
    /// If `true`, the playhead time wraps to the clip duration (loop); else the
    /// clip holds at its ends past its duration.
    pub looping: bool,
}

impl SimComponent for SpriteAnimation {}

impl SpriteAnimation {
    /// Bind a looping clip.
    #[must_use]
    pub fn new(clip: Clip) -> Self {
        Self {
            clip,
            looping: true,
        }
    }

    /// Bind a one-shot clip (holds past its duration).
    #[must_use]
    pub fn once(clip: Clip) -> Self {
        Self {
            clip,
            looping: false,
        }
    }

    /// The clip-local time for an absolute playhead time (wrapped if looping).
    #[must_use]
    pub fn local_time(&self, playhead_time: f64) -> f64 {
        let dur = self.clip.duration().to_seconds();
        if self.looping && dur > 0.0 {
            playhead_time.rem_euclid(dur)
        } else {
            playhead_time
        }
    }
}

/// Sample every [`SpriteAnimation`] at `playhead_time` (seconds) and write the
/// bound properties into each entity's [`Transform`]. Unbound properties are
/// left untouched. Call once per frame, before transform propagation.
pub fn apply_sprite_animations(world: &mut World, playhead_time: f64) {
    let mut q = world.query::<(&mut Transform, &SpriteAnimation)>();
    for (mut xf, anim) in q.iter_mut(world) {
        let t = anim.local_time(playhead_time);
        if let Some(v) = float_at(&anim.clip, SpriteProp::TranslationX, t) {
            xf.translation.x = v;
        }
        if let Some(v) = float_at(&anim.clip, SpriteProp::TranslationY, t) {
            xf.translation.y = v;
        }
        if let Some(v) = float_at(&anim.clip, SpriteProp::Rotation, t) {
            xf.rotation = v;
        }
        if let Some(v) = float_at(&anim.clip, SpriteProp::ScaleX, t) {
            xf.scale.x = v;
        }
        if let Some(v) = float_at(&anim.clip, SpriteProp::ScaleY, t) {
            xf.scale.y = v;
        }
    }
}

/// Resolve one sprite property's value at time `t`, if a `Float` track is bound.
fn float_at(clip: &Clip, prop: SpriteProp, t: f64) -> Option<f32> {
    match clip.sample(prop.target(), t) {
        Some(AnimValue::Float(v)) => Some(v),
        _ => None,
    }
}
