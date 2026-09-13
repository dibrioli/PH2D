use super::cascade_tint_with_ancestors;
use ph2d_ecs::{ChildOf, SimWorld};
use ph2d_render::Sprite;

/// Root(tint A, self_tint B) → child(white) → grandchild(tint C).
/// Render tint = self_tint × tint × Π(ancestor.tint); an ancestor
/// contributes its `tint` (modulate, cascades) but NOT its `self_tint`
/// (local) — the exact distinction Enio's smoke flagged as missing.
#[test]
fn cascade_folds_ancestor_modulate_not_self_modulate() {
    let mut sim = SimWorld::new();
    // Root: tint = half-red, self_tint = half-green (local only).
    let mut root_s = Sprite::atlas(0, [1.0, 1.0], [0.5, 1.0, 1.0, 1.0]);
    root_s.self_tint = [1.0, 0.5, 1.0, 1.0];
    let root = sim.world_mut().spawn(root_s).id();
    let child = sim
        .world_mut()
        .spawn((
            Sprite::atlas(0, [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
            ChildOf(root),
        ))
        .id();
    let grandchild = sim
        .world_mut()
        .spawn((
            Sprite::atlas(0, [1.0, 1.0], [1.0, 1.0, 0.5, 1.0]),
            ChildOf(child),
        ))
        .id();

    let w = sim.world();
    let cascade = |e| {
        let s = w.get::<Sprite>(e).unwrap();
        cascade_tint_with_ancestors(w, e, s)
    };
    // Root: self_tint × tint, no ancestors.
    assert_eq!(cascade(root), [0.5, 0.5, 1.0, 1.0]);
    // Child: root's TINT (half-red) cascades; root's SELF_TINT
    // (half-green) does NOT — so green stays 1.0. THE key assertion.
    assert_eq!(cascade(child), [0.5, 1.0, 1.0, 1.0]);
    // Grandchild: own tint (half-blue) × child (white) × root tint.
    assert_eq!(cascade(grandchild), [0.5, 1.0, 0.5, 1.0]);
}

#[test]
fn root_sprite_cascade_equals_per_sprite_collapse() {
    // No parent → identical to the old `collapsed_tint` (zero
    // regression for every root sprite).
    let mut sim = SimWorld::new();
    let mut s = Sprite::atlas(0, [1.0, 1.0], [0.4, 0.5, 0.6, 0.7]);
    s.self_tint = [0.5, 0.5, 1.0, 0.8];
    let e = sim.world_mut().spawn(s).id();
    let w = sim.world();
    assert_eq!(
        cascade_tint_with_ancestors(w, e, w.get::<Sprite>(e).unwrap()),
        w.get::<Sprite>(e).unwrap().collapsed_tint()
    );
}
