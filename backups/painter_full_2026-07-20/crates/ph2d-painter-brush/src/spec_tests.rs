//! Tests for [`crate::spec::BrushSpec`] — split out of `spec.rs` for the workspace file-LOC cap
//! (the same move `cook.rs` made: the engine stays under the cap, the tests live in a sibling).

use crate::spec::{BrushSpec, MAX_BRUSH_RADIUS_PX};
use crate::{BrushBlend, DepthSource, DrawTo, Falloff};

#[test]
fn defaults_are_sane() {
    let b = BrushSpec::default();
    assert_eq!(b.blend, BrushBlend::Mix);
    assert_eq!(b.falloff, Falloff::Smooth);
    assert!(b.dab_spacing_px() >= 1.0);
    // Per-dab randomize is off by default (so a default stroke is byte-identical to baseline).
    assert!(!b.color_jitter_enabled);
    assert!(!b.has_colour_jitter_amount());
    assert!(!b.has_per_dab_rotation());
    assert_eq!(b.jitter_scale, 0.0);
    assert_eq!(b.jitter_rotate, 0.0);
}

#[test]
fn jitter_rotate_makes_an_active_shape_rotate_per_dab() {
    use crate::texture::TextureKind;
    // Jitter Rotate spins the whole stamp, so an active Shape with jitter > 0 needs a per-dab basis
    // (routes off the constant-orientation caches) even without Shape Rake/Random (Enio 2026-06-28).
    let mut b = BrushSpec {
        jitter_rotate: 0.5,
        ..Default::default()
    };
    b.shape.kind = TextureKind::Image;
    assert!(
        b.shape_has_per_dab_rotation(true),
        "an active Shape + Jitter Rotate rotates per dab"
    );
    // No Shape ⇒ nothing to rotate (the bare falloff is isotropic).
    assert!(!b.shape_has_per_dab_rotation(false));
    // Jitter off ⇒ no per-dab rotation (byte-identical baseline).
    b.jitter_rotate = 0.0;
    assert!(!b.shape_has_per_dab_rotation(true));
}

#[test]
fn compose_shape_image_replaces_procedural_masks() {
    use crate::texture::TextureKind;
    let mut b = BrushSpec::default();
    // Image kind: the silhouette IS the image sample — the falloff is ignored (crisp tip).
    b.shape.kind = TextureKind::Image;
    assert_eq!(b.compose_shape_silhouette(0.7, 0.3), 0.7);
    // A procedural kind: the falloff MASKS the pattern (falloff × pattern).
    b.shape.kind = TextureKind::Checker;
    assert!((b.compose_shape_silhouette(0.7, 0.3) - 0.21).abs() < 1e-6);
    // Falloff 0 (dab edge) zeroes a procedural silhouette, but never an Image one.
    assert_eq!(b.compose_shape_silhouette(1.0, 0.0), 0.0);
    b.shape.kind = TextureKind::Image;
    assert_eq!(b.compose_shape_silhouette(1.0, 0.0), 1.0);
}

#[test]
fn impasto_master_switch_gates_every_knob() {
    // Off by default ⇒ no relief, no colour suppression, and the *stored* knobs read as zero.
    let b = BrushSpec::default();
    assert!(!b.impasto);
    assert!(!b.deposits_height());
    assert!(b.deposits_color());
    assert_eq!(b.effective_impasto_depth(), 0.0);
    assert_eq!(b.effective_impasto_smoothing(), 0.0);

    // The knobs carry live *when-enabled* values, so the master switch — not neutral zeros — is
    // what makes the default inert. Prove it: with the switch OFF, wild knobs stay inert.
    let wild = BrushSpec {
        impasto_depth: 1.0,
        impasto_smoothing: 1.0,
        impasto_source: DepthSource::Grain,
        impasto_draw_to: DrawTo::Depth, // would suppress colour if it were read
        ..Default::default()
    };
    assert!(
        !wild.deposits_height(),
        "switch off ⇒ no height, at any depth"
    );
    assert!(
        wild.deposits_color(),
        "switch off ⇒ `Draw To` is not read; a brush left on Depth still paints"
    );
    assert_eq!(wild.effective_impasto_depth(), 0.0);

    // Switch ON ⇒ the same knobs come alive.
    let on = BrushSpec {
        impasto: true,
        ..wild
    };
    assert!(on.deposits_height());
    assert!(!on.deposits_color(), "Draw To = Depth suppresses pigment");
    assert_eq!(on.effective_impasto_depth(), 1.0);
}

#[test]
fn impasto_zero_depth_is_a_no_op_and_negative_carves() {
    // Zero depth deposits nothing — it must NOT write a flat zero over existing relief.
    let flat = BrushSpec {
        impasto: true,
        impasto_depth: 0.0,
        ..Default::default()
    };
    assert!(!flat.deposits_height());
    assert!(
        flat.deposits_color(),
        "it still paints; it just has no body"
    );

    // Negative depth carves (Painter's "Negative Depth") — it is live, not clamped away.
    let carve = BrushSpec {
        impasto: true,
        impasto_depth: -0.7,
        ..Default::default()
    };
    assert!(carve.deposits_height());
    assert_eq!(carve.effective_impasto_depth(), -0.7);
    // Out-of-range depth clamps to the signed unit range, both ways.
    let over = BrushSpec {
        impasto: true,
        impasto_depth: -9.0,
        ..Default::default()
    };
    assert_eq!(over.effective_impasto_depth(), -1.0);
}

#[test]
fn radius_clamped() {
    let b = BrushSpec {
        radius_px: 999_999.0,
        ..Default::default()
    };
    assert_eq!(b.clamped_radius(), MAX_BRUSH_RADIUS_PX);
    let b = BrushSpec {
        radius_px: 0.0,
        ..Default::default()
    };
    assert_eq!(b.clamped_radius(), 0.5);
}

#[test]
fn hardness_full_is_hard_disk() {
    let b = BrushSpec {
        hardness: 1.0,
        ..Default::default()
    };
    assert_eq!(b.falloff_weight(0.0), 1.0);
    assert_eq!(b.falloff_weight(0.99), 1.0);
    assert_eq!(b.falloff_weight(1.0), 0.0);
}

#[test]
fn hardness_plateau_then_falls() {
    let b = BrushSpec {
        hardness: 0.5,
        falloff: Falloff::Linear,
        ..Default::default()
    };
    assert_eq!(b.falloff_weight(0.5), 1.0); // inside plateau
    // At t=0.75, remapped = (0.75-0.5)/0.5 = 0.5 → linear weight 0.5.
    assert!((b.falloff_weight(0.75) - 0.5).abs() < 1e-6);
}
