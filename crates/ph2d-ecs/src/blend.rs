//! Per-sprite blend mode — Sprite Inspector v2 §10 (spec
//! [`03_inspector_secoes.md`](../../../docs/Sprite_projeto/03_inspector_secoes.md)
//! §3.10, [`09_sampling_e_material.md`](../../../docs/Sprite_projeto/09_sampling_e_material.md)).
//!
//! Optional component: **absence = [`BlendMode::Mix`]** (the renderer's
//! existing premultiplied-alpha "over"), so legacy sprites are
//! bit-identical. The resolved mode is packed into `RenderInstance.flip_uv`
//! bits 5-7 at extract (zero ABI — see `ph2d-render` `Sprite::BLEND_SHIFT`)
//! and read back by the renderer to key draw runs onto the matching
//! blend pipeline. Blend is **pipeline state**, not shader logic, so the
//! fragment shader ignores these bits.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

/// How a sprite composites against what's already in the target.
///
/// Tag values (0..5) are the on-the-wire encoding packed into
/// `flip_uv` bits 5-7; keep them stable and in lockstep with the
/// renderer's blend-pipeline array order.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMode {
    /// Standard straight-alpha "over" — the everyday mode and the
    /// component default (matches the renderer's pre-§10 blend, so an
    /// absent component renders identically). Tag 0.
    #[default]
    Mix,
    /// Additive / glow: `dst + src·a`. Lightens; great for fire, light,
    /// energy. Tag 1.
    Add,
    /// Subtractive: `dst − src·a` (reverse-subtract). Darkens. Tag 2.
    Subtract,
    /// Multiply: `dst · src`. Darkens; shadows, tints. Tag 3.
    Multiply,
    /// Screen: `1 − (1−dst)(1−src)`. Lightens softer than Add. Tag 4.
    Screen,
    /// Premultiplied-alpha "over" — for sources whose RGB is already
    /// multiplied by alpha (no double-darkening at edges). Tag 5.
    PremultAlpha,
}

impl BlendMode {
    /// Number of distinct blend modes (size of the renderer's pipeline
    /// array and the headless-gate matrix).
    pub const COUNT: usize = 6;

    /// All modes in canonical (tag) order.
    pub const ALL: [BlendMode; Self::COUNT] = [
        BlendMode::Mix,
        BlendMode::Add,
        BlendMode::Subtract,
        BlendMode::Multiply,
        BlendMode::Screen,
        BlendMode::PremultAlpha,
    ];

    /// Stable 0..5 tag — the `flip_uv` bits-5-7 encoding.
    pub const fn tag(self) -> u8 {
        match self {
            BlendMode::Mix => 0,
            BlendMode::Add => 1,
            BlendMode::Subtract => 2,
            BlendMode::Multiply => 3,
            BlendMode::Screen => 4,
            BlendMode::PremultAlpha => 5,
        }
    }

    /// Inverse of [`tag`](Self::tag); out-of-range tags fall back to the
    /// default [`Mix`](Self::Mix) (defensive — a corrupt bit field never
    /// panics, it renders normally).
    pub const fn from_tag(tag: u8) -> Self {
        match tag {
            1 => BlendMode::Add,
            2 => BlendMode::Subtract,
            3 => BlendMode::Multiply,
            4 => BlendMode::Screen,
            5 => BlendMode::PremultAlpha,
            _ => BlendMode::Mix,
        }
    }

    /// Short UI label (English; see HR-15 — i18n migrates in W7).
    pub const fn label(self) -> &'static str {
        match self {
            BlendMode::Mix => "Mix",
            BlendMode::Add => "Add",
            BlendMode::Subtract => "Subtract",
            BlendMode::Multiply => "Multiply",
            BlendMode::Screen => "Screen",
            BlendMode::PremultAlpha => "Premult",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_mix_tag_zero() {
        assert_eq!(BlendMode::default(), BlendMode::Mix);
        assert_eq!(BlendMode::Mix.tag(), 0);
    }

    #[test]
    fn tag_round_trips_for_every_mode() {
        for m in BlendMode::ALL {
            assert_eq!(BlendMode::from_tag(m.tag()), m);
        }
    }

    #[test]
    fn from_tag_out_of_range_is_mix() {
        assert_eq!(BlendMode::from_tag(6), BlendMode::Mix);
        assert_eq!(BlendMode::from_tag(255), BlendMode::Mix);
    }

    #[test]
    fn all_is_tag_ordered() {
        for (i, m) in BlendMode::ALL.iter().enumerate() {
            assert_eq!(m.tag() as usize, i);
        }
    }
}
