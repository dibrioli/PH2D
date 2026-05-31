//! Clip & mask components — Sprite Inspector v2 W3 (spec
//! [`06_mask_clip.md`](../../../docs/Sprite_projeto/06_mask_clip.md)).
//!
//! Two independent concepts (spec §6.1):
//! - [`ClipChildren`] on a *parent* clips its descendants to the
//!   parent silhouette (Godot).
//! - [`MaskInteraction`] on a *responder* makes the sprite obey an
//!   external `Mask2D` sibling (Unity SpriteMask). The `Mask2D` entity
//!   itself is a future module (spec §6.6) — W3 ships the responder
//!   side as a data stub.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

/// How [`ClipChildren`] clips a node's descendants (spec §6.2).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClipMode {
    /// No clip; render normally.
    #[default]
    Disabled,
    /// Children clipped to the node silhouette, but the node itself
    /// does **not** draw (arbitrary-shape progress bar / stencil mould).
    ClipOnly,
    /// Node draws *and* children are clipped to it (circular avatar,
    /// rounded-border UI).
    ClipAndDraw,
}

/// Clip this node's descendants to its silhouette (spec §6.2). Uses a
/// back-buffer pass; the node alpha becomes a binary mask via
/// [`ClipChildren::alpha_cutoff`].
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClipChildren {
    pub mode: ClipMode,
    /// Alpha threshold counted as "inside" the mask. Range `[0, 1]`.
    pub alpha_cutoff: f32,
}

impl ClipChildren {
    /// Default cutoff (intuitive midpoint, spec §6.4).
    pub const DEFAULT_CUTOFF: f32 = 0.5;

    /// Construct with the canonical default cutoff.
    pub const fn new(mode: ClipMode) -> Self {
        Self {
            mode,
            alpha_cutoff: Self::DEFAULT_CUTOFF,
        }
    }

    /// Clamp `alpha_cutoff` into `[0, 1]` (spec §6.8).
    pub fn clamped(self) -> Self {
        Self {
            mode: self.mode,
            alpha_cutoff: self.alpha_cutoff.clamp(0.0, 1.0),
        }
    }
}

impl Default for ClipChildren {
    fn default() -> Self {
        Self::new(ClipMode::Disabled)
    }
}

/// How a sprite responds to an external mask (spec §6.4).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaskMode {
    /// Ignore masks (default).
    #[default]
    None,
    /// Render where the mask is "inside" (alpha > cutoff).
    VisibleInside,
    /// Render where the mask is "outside" (alpha ≤ cutoff). Inverse.
    VisibleOutside,
}

/// This sprite responds to a sibling `Mask2D` (spec §6.4). The mask
/// entity itself is a future module; W3 carries the responder data.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaskInteraction {
    pub mode: MaskMode,
    /// Alpha threshold that counts as "inside" the mask. Range `[0, 1]`.
    pub alpha_cutoff: f32,
}

impl MaskInteraction {
    pub const DEFAULT_CUTOFF: f32 = 0.5;

    pub const fn new(mode: MaskMode) -> Self {
        Self {
            mode,
            alpha_cutoff: Self::DEFAULT_CUTOFF,
        }
    }

    pub fn clamped(self) -> Self {
        Self {
            mode: self.mode,
            alpha_cutoff: self.alpha_cutoff.clamp(0.0, 1.0),
        }
    }
}

impl Default for MaskInteraction {
    fn default() -> Self {
        Self::new(MaskMode::None)
    }
}

/// A sprite that serves as a mask SOURCE for sibling [`MaskInteraction`]
/// responders (Unity SpriteMask, spec §6.4/§6.6). The entity's `Sprite`
/// texture alpha — thresholded by [`Mask2D::alpha_cutoff`] — is the mask
/// silhouette. The mask itself does NOT draw its own color (it's a
/// mould, like Unity's default SpriteMask). Scope is GLOBAL in W3 (every
/// responder reacts to the union of every `Mask2D`); per-sorting-layer
/// scoping (`MaskCustomRange`, spec §6.5) is a future refinement. The
/// full Mask2D module (source = SpriteRef vs renderer, sort point, its
/// own Inspector — spec §6.6) is future; this is the minimal source
/// component that makes [`MaskInteraction`] demonstrable.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mask2D {
    /// Alpha threshold that counts as "inside" the mask. Range `[0, 1]`.
    pub alpha_cutoff: f32,
}

impl Mask2D {
    /// Default cutoff (intuitive midpoint, mirrors [`MaskInteraction`]).
    pub const DEFAULT_CUTOFF: f32 = 0.5;

    /// Construct with the canonical default cutoff.
    pub const fn new() -> Self {
        Self {
            alpha_cutoff: Self::DEFAULT_CUTOFF,
        }
    }

    /// Clamp `alpha_cutoff` into `[0, 1]`.
    pub fn clamped(self) -> Self {
        Self {
            alpha_cutoff: self.alpha_cutoff.clamp(0.0, 1.0),
        }
    }
}

impl Default for Mask2D {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_defaults_disabled_with_mid_cutoff() {
        let c = ClipChildren::default();
        assert_eq!(c.mode, ClipMode::Disabled);
        assert_eq!(c.alpha_cutoff, ClipChildren::DEFAULT_CUTOFF);
    }

    #[test]
    fn cutoffs_clamp_to_unit_range() {
        assert_eq!(
            ClipChildren {
                mode: ClipMode::ClipOnly,
                alpha_cutoff: 5.0,
            }
            .clamped()
            .alpha_cutoff,
            1.0
        );
        assert_eq!(
            MaskInteraction {
                mode: MaskMode::VisibleInside,
                alpha_cutoff: -1.0,
            }
            .clamped()
            .alpha_cutoff,
            0.0
        );
    }

    #[test]
    fn modes_serde_round_trip() {
        for m in [
            ClipMode::Disabled,
            ClipMode::ClipOnly,
            ClipMode::ClipAndDraw,
        ] {
            let b = postcard::to_allocvec(&m).unwrap();
            assert_eq!(postcard::from_bytes::<ClipMode>(&b).unwrap(), m);
        }
        for m in [
            MaskMode::None,
            MaskMode::VisibleInside,
            MaskMode::VisibleOutside,
        ] {
            let b = postcard::to_allocvec(&m).unwrap();
            assert_eq!(postcard::from_bytes::<MaskMode>(&b).unwrap(), m);
        }
    }

    #[test]
    fn mask2d_defaults_and_clamps() {
        assert_eq!(Mask2D::default().alpha_cutoff, Mask2D::DEFAULT_CUTOFF);
        assert_eq!(Mask2D { alpha_cutoff: 9.0 }.clamped().alpha_cutoff, 1.0);
        assert_eq!(Mask2D { alpha_cutoff: -2.0 }.clamped().alpha_cutoff, 0.0);
        let b = postcard::to_allocvec(&Mask2D::new()).unwrap();
        assert_eq!(postcard::from_bytes::<Mask2D>(&b).unwrap(), Mask2D::new());
    }
}
