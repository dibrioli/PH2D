use super::*;
use ph2d_ecs::ChildOf;

fn x(sim: &SimWorld, e: Entity) -> f32 {
    sim.world()
        .get::<Transform>(e)
        .expect("transform")
        .translation
        .x
}

/// A ponta fica na ponta e o meio no meio; ⛔ CONTROLO: sem eixo (`l0 = 0`) ou parado, nada anda.
#[test]
fn o_comprimento_empurra_os_filhos_ao_longo_do_eixo() {
    let mut sim = SimWorld::new();
    let pai = sim.world_mut().spawn(Transform::IDENTITY).id();
    let mut t = Transform::IDENTITY;
    t.translation.x = 10.0;
    let ponta = sim.world_mut().spawn((t, ChildOf(pai))).id();
    t.translation.x = 5.0;
    t.translation.y = 2.0;
    let meio = sim.world_mut().spawn((t, ChildOf(pai))).id();
    assert!(empurra_filhos(&mut sim, pai, 0.0, 5.0).is_empty());
    assert!(empurra_filhos(&mut sim, pai, 10.0, 10.0).is_empty());
    assert_eq!(empurra_filhos(&mut sim, pai, 10.0, 16.0).len(), 2);
    assert!((x(&sim, ponta) - 16.0).abs() < 1e-5 && (x(&sim, meio) - 8.0).abs() < 1e-5);
    let y = sim.world().get::<Transform>(meio).expect("t").translation.y;
    assert!((y - 2.0).abs() < 1e-6, "o y não é do eixo");
}
