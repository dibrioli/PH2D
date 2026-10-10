use super::tests::doc_with_segment;
use super::*;
use ph2d_ecs::Visibility;

/// ⭐ F8: **o *Enquadrar tudo* conta os objectos Flip DESENHADOS** — a caixa de mundo de cada um, e o
/// escondido (pelo olho dele ou de um pai) fica de fora.
#[test]
fn the_drawn_flip_boxes_leave_the_hidden_out() {
    let (doc, mut sim, _, _, e) = doc_with_segment([10.0, 20.0], [30.0, 40.0]);
    let b = world_aabb(&sim, &doc, e).unwrap();
    assert_eq!((b.min, b.max), ([10.0, 20.0], [30.0, 40.0]));
    assert_eq!(drawn_aabbs(&sim, &doc).len(), 1);
    let pai = sim.world_mut().spawn(Visibility::hidden()).id();
    sim.world_mut().entity_mut(e).insert(ph2d_ecs::ChildOf(pai));
    assert!(
        drawn_aabbs(&sim, &doc).is_empty(),
        "o filho de um pai escondido contou"
    );
}
