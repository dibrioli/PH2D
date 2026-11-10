use super::*;
use ph2d_ecs::SimWorld;

#[test]
fn sem_look_a_forma_resolvida_e_a_mesma_bit_a_bit() {
    let mut sim = SimWorld::new();
    let col = Collider {
        offset: [0.3, -0.7],
        ..Collider::default()
    };
    let e = sim.world_mut().spawn(col).id();
    assert_eq!(forma_resolvida(sim.world(), e, &col), col);
    sim.world_mut().entity_mut(e).insert(ShapeLook::default());
    assert_eq!(forma_resolvida(sim.world(), e, &col), col);
}

#[test]
fn o_offset_para_centro_desfaz_a_porta() {
    let mut sim = SimWorld::new();
    let col = Collider {
        shape: ColliderShape::Capsule {
            half_height: 0.4,
            radius: 0.2,
        },
        offset: [0.1, 0.2],
        ..Collider::default()
    };
    let e = sim
        .world_mut()
        .spawn((
            col,
            ShapeLook {
                pivot: [0.5, -1.0],
                ..ShapeLook::default()
            },
        ))
        .id();
    let r = forma_resolvida(sim.world(), e, &col);
    assert_eq!(r.offset, [0.1 - 0.5 * 0.2, 0.2 + 0.6]);
    let volta = offset_para_centro(sim.world(), e, col.shape, r.offset);
    assert!((volta[0] - col.offset[0]).abs() < 1e-6 && (volta[1] - col.offset[1]).abs() < 1e-6);
}

#[test]
fn o_sanitize_poe_cor_e_opacidade_na_faixa() {
    let mut l = ShapeLook {
        pivot: [f32::NAN, 3.0],
        tint: [2.0, -1.0, 0.5, f32::INFINITY],
        opacity: 7.0,
        cor_propria: true,
    };
    l.sanitize();
    assert_eq!(l.pivot, [0.0, 3.0]);
    assert_eq!(l.tint, [1.0, 0.0, 0.5, 1.0]);
    assert_eq!(l.opacity, 1.0);
    l.opacity = 0.0;
    l.sanitize();
    assert_eq!(l.opacity, OPACIDADE_MINIMA, "a forma nunca some de todo");
}
