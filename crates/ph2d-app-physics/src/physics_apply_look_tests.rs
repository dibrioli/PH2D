use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::ShapeLook;

/// ⭐ A alça mostra e puxa pelo CENTRO real; com o pivô na borda esquerda, o `offset` cru que volta põe o centro
/// exatamente onde a alça o deixou, e o pivô continua na borda da forma nova.
#[test]
fn a_alca_numa_forma_com_pivo_deixa_o_centro_onde_puxou() {
    let mut sim = SimWorld::new();
    let cur = Collider {
        shape: ColliderShape::Cuboid {
            half_x: 0.5,
            half_y: 0.5,
        },
        ..Collider::default()
    };
    let e = sim
        .world_mut()
        .spawn((
            cur,
            ShapeLook {
                pivot: [-1.0, 0.0],
                ..ShapeLook::default()
            },
        ))
        .id();
    let novo = forma_arrastada(sim.world(), e, &cur, [1.5, 0.5], [1.5, 0.0]);
    let r = ph2d_physics_ecs::forma_resolvida(sim.world(), e, &novo);
    assert_eq!(r.offset, [1.5, 0.0], "o centro que a alça deixou");
    assert_eq!(
        r.offset[0] - 1.5,
        0.0,
        "a borda esquerda, o pivô, na origem"
    );
}
