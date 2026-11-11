use super::Sprite;
use ph2d_ecs::{ChildOf, SimWorld};

/// O «Opacity» de um pai apaga a subárvore: root 0,5 → child 0,5 → grandchild 1,0 = 0,25 (plano
/// Timeline 17, onda 2). *Mutação: o laço sem o `*=` do antepassado ⇒ o filho fica a 0,5.*
#[test]
fn opacity_cascades_to_descendants() {
    let mut sim = SimWorld::new();
    let com = |o: f32| {
        let mut s = Sprite::atlas(0, [1.0, 1.0], [1.0; 4]);
        s.opacity = o;
        s
    };
    let root = sim.world_mut().spawn(com(0.5)).id();
    let child = sim.world_mut().spawn((com(0.5), ChildOf(root))).id();
    let neto = sim.world_mut().spawn((com(1.0), ChildOf(child))).id();
    let w = sim.world();
    let op = |e| w.get::<Sprite>(e).unwrap().cascaded_opacity(w, e);
    assert_eq!([op(root), op(child), op(neto)], [0.5, 0.25, 0.25]);
}
