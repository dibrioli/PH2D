//! **Os gates da composição e da propagação de `Transform`** — irmão de
//! [`super::transform`] por CAP de LOC (`workspace_src_files_under_loc_cap`).
//!
//! ⚠️ **Corte mecânico, conteúdo verbatim.** O módulo de testes saiu inteiro do ficheiro que
//! o continha quando a injeção do quadro de âncora (`crate::anchor_mount`) o levou de 768 a
//! 777 linhas. A regra da casa é cortar para o IRMÃO, nunca subir a entrada da allowlist — os
//! números dela só descem.

use super::*;


#[test]
fn identity_is_neutral() {
    let t = Transform::IDENTITY;
    let got = Transform::compose(t, t);
    assert_eq!(got, t);
}

/// ADR-0025-amendment-1 §2.5 frozen caps: v2 schema is 5 fields,
/// 28 bytes (translation 8 + rotation 4 + scale 8 + skew_x 4 +
/// skew_y 4, 4-align), VERSION = 2. A size drift here means a field
/// was added/reordered without an amendment-2 + cooker bump.
#[test]
fn transform_v2_caps_frozen() {
    assert_eq!(
        std::mem::size_of::<Transform>(),
        28,
        "Transform must be 28 bytes (5 fields); changing the layout requires \
         ADR-0025-amendment-2 + a Transform::VERSION bump + cooker migration"
    );
    assert_eq!(Transform::VERSION, 2);
}

#[test]
fn skew_clamp_bounds_tan_finite() {
    // Over-range skew is clamped so tan() never reaches ±∞.
    let t = Transform::IDENTITY.with_skew(10.0, -10.0);
    assert_eq!(t.skew_x, Transform::SKEW_LIMIT);
    assert_eq!(t.skew_y, -Transform::SKEW_LIMIT);
    assert!(libm::tanf(t.skew_x).is_finite());
    assert!(libm::tanf(t.skew_y).is_finite());
    // In-range values pass through untouched.
    let u = Transform::IDENTITY.with_skew(0.2, -0.3);
    assert_eq!(u.skew_x, 0.2);
    assert_eq!(u.skew_y, -0.3);
}

#[test]
fn zero_skew_compose_matches_v1_math_bit_identical() {
    // The whole golden-hash-survives-the-bump guarantee: skew=0
    // compose must produce bit-identical translation to the pre-skew
    // formula. Exercise a non-trivial rotation+scale parent.
    let parent = Transform {
        translation: Vec2::new(3.0, -2.0),
        rotation: 0.9,
        scale: Vec2::new(2.0, 0.5),
        ..Transform::IDENTITY
    };
    let child = Transform::from_translation(Vec2::new(1.5, -0.75));
    let got = Transform::compose(parent, child);
    // Recompute with the explicit v1 formula (no skew terms).
    let (sin, cos) = libm::sincosf(parent.rotation);
    let sx = child.translation.x * parent.scale.x;
    let sy = child.translation.y * parent.scale.y;
    let rx = sx * cos - sy * sin;
    let ry = sx * sin + sy * cos;
    let v1_x = parent.translation.x + rx;
    let v1_y = parent.translation.y + ry;
    assert_eq!(got.translation.x.to_bits(), v1_x.to_bits());
    assert_eq!(got.translation.y.to_bits(), v1_y.to_bits());
}

#[test]
fn translation_composes_additively_with_identity_rotation() {
    let parent = Transform::from_translation(Vec2::new(10.0, 5.0));
    let child = Transform::from_translation(Vec2::new(2.0, 3.0));
    let got = Transform::compose(parent, child);
    assert_eq!(got.translation, Vec2::new(12.0, 8.0));
    assert_eq!(got.rotation, 0.0);
    assert_eq!(got.scale, Vec2::new(1.0, 1.0));
}

#[test]
fn scale_multiplies_through_child_translation() {
    let parent = Transform {
        translation: Vec2::new(0.0, 0.0),
        rotation: 0.0,
        scale: Vec2::new(2.0, 3.0),
        ..Transform::IDENTITY
    };
    let child = Transform::from_translation(Vec2::new(1.0, 1.0));
    let got = Transform::compose(parent, child);
    // child.translation (1,1) is scaled by parent.scale (2,3).
    assert_eq!(got.translation, Vec2::new(2.0, 3.0));
    assert_eq!(got.scale, Vec2::new(2.0, 3.0));
}

#[test]
fn rotation_rotates_child_translation() {
    let parent = Transform {
        translation: Vec2::new(0.0, 0.0),
        rotation: std::f32::consts::FRAC_PI_2,
        scale: Vec2::new(1.0, 1.0),
        ..Transform::IDENTITY
    };
    let child = Transform::from_translation(Vec2::new(1.0, 0.0));
    let got = Transform::compose(parent, child);
    // 90° CCW: (1,0) → (0,1)
    assert!((got.translation.x).abs() < 1e-6);
    assert!((got.translation.y - 1.0).abs() < 1e-6);
    assert_eq!(got.rotation, std::f32::consts::FRAC_PI_2);
}

#[test]
fn global_transform_translation_matches_local() {
    let t = Transform {
        translation: Vec2::new(7.0, -3.0),
        rotation: 1.5,
        scale: Vec2::new(2.0, 2.0),
        ..Transform::IDENTITY
    };
    let gt = GlobalTransform::from_transform(t);
    assert_eq!(gt.translation(), t.translation);
}

#[test]
fn root_draw_order_follows_root_order_not_entity_id() {
    // Hierarchy order must drive canvas stacking (Godot-style): the
    // DFS visit order (→ draw_order) follows `RootOrder`, not spawn
    // order. Spawn so entity-id order (a<b<c) disagrees with RootOrder
    // (c=0 < a=1 < b=2); the visit order must be c, a, b.
    let mut world = World::new();
    let a = world.spawn((Transform::IDENTITY, crate::RootOrder(1))).id();
    let b = world.spawn((Transform::IDENTITY, crate::RootOrder(2))).id();
    let c = world.spawn((Transform::IDENTITY, crate::RootOrder(0))).id();
    let mut state = TransformPropagationState::new(&mut world);
    let mut present = World::new();
    let mut buf = WorklistBuf::with_capacity(8);
    let mut order = Vec::new();
    propagate_transforms(
        &world,
        &mut state,
        &mut present,
        &mut buf,
        |_s, _p, e, _gt| {
            order.push(e);
        },
    );
    // First-visited = smallest draw_order = furthest back, matching
    // the Hierarchy list (top of list = behind).
    assert_eq!(order, vec![c, a, b]);
}

#[test]
fn worklist_buf_clear_preserves_capacity() {
    let mut buf = WorklistBuf::with_capacity(64);
    for i in 0..32 {
        buf.stack
            .push((Entity::from_raw_u32(i).unwrap(), Transform::IDENTITY));
    }
    let cap_before = buf.stack_capacity();
    buf.clear();
    assert_eq!(buf.stack_len(), 0);
    assert_eq!(buf.stack_capacity(), cap_before);
}
