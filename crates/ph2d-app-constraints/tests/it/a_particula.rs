//! O Copy Particle no cartão e no quadro (`docs/BlenderConstraints/07_W6_pontas.md` §5): o menu
//! cria-o, o cartão mostra o «Particle ID» e escreve-o, e o quadro pede ao Motion o objecto alvo.

use ph2d_app_constraints::inspector::{apply_all, info};
use ph2d_constraints::{Kind, PontoDeSimulacao};
use ph2d_constraints_ecs::Constraints;
use ph2d_ecs::canal_constraint::PONTO;
use ph2d_ecs::motion_generator::{Particulas, ParticulasPedidas};
use ph2d_ecs::{SimWorld, StableId, Transform, world_transform};
use ph2d_editor_core::constraint_edits::{ConstraintFieldEdit as E, TIPO_PARTICULA, TIPOS};
use ph2d_preview_drive::PreviewDrive;
use ph2d_vec_scene::VecScene;

#[test]
fn the_card_offers_copy_particle_and_writes_its_id() {
    assert_eq!(
        TIPOS[usize::from(TIPO_PARTICULA)],
        "panel.inspector.constraint.copy_particle"
    );
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((StableId(2), Transform::IDENTITY, Constraints::default()))
        .id();
    let b = e.to_bits();
    assert!(apply_all(&mut sim, &[(b, E::Adicionar(TIPO_PARTICULA))]));
    let item = sim.world().get::<Constraints>(e).unwrap().items[0];
    assert_eq!(
        item.kind,
        Kind::PontoDeSimulacao(PontoDeSimulacao { id: 0 })
    );
    let linha = &info(&sim, b, 1).unwrap().rows[0];
    assert_eq!(linha.tipo, TIPO_PARTICULA);
    assert_eq!(linha.numeros.len(), 1);
    assert_eq!(linha.numeros[0].campo, PONTO);
    assert_eq!(
        linha.numeros[0].rotulo,
        "panel.inspector.constraint.particle_id"
    );
    for (escrito, lido) in [(42.4, 42), (-3.0, 0), (16_777_215.0, 16_777_215)] {
        assert!(apply_all(
            &mut sim,
            &[(b, E::Campo(item.id, PONTO, escrito))]
        ));
        let k = sim.world().get::<Constraints>(e).unwrap().items[0].kind;
        assert_eq!(k, Kind::PontoDeSimulacao(PontoDeSimulacao { id: lido }));
        assert_eq!(
            info(&sim, b, 1).unwrap().rows[0].numeros[0].valor,
            lido as f32,
            "o cartão lê o id exacto até 2²⁴ − 1"
        );
    }
}

#[test]
fn the_frame_asks_motion_for_the_target_and_follows_its_particle() {
    let mut sim = SimWorld::new();
    let mut cena = VecScene::new();
    let mut drive = PreviewDrive::default();
    let mut objeto = Transform::IDENTITY;
    objeto.translation.x = 10.0;
    sim.world_mut()
        .spawn((StableId(5), objeto, ph2d_ecs::MotionGenerator));
    let mut c = Constraints::default();
    c.push(
        Kind::PontoDeSimulacao(PontoDeSimulacao { id: 3 }),
        Some(StableId(5)),
    );
    let mut autorado = Transform::IDENTITY;
    autorado.translation.y = 7.0;
    let dono = sim.world_mut().spawn((StableId(1), autorado, c)).id();
    ph2d_app_constraints::quadro(&mut sim, &mut cena, &mut drive, None);
    assert_eq!(
        sim.world().resource::<ParticulasPedidas>().0,
        [StableId(5)].into(),
        "o pedido nomeia o objecto Motion alvo"
    );
    assert_eq!(
        world_transform(sim.world(), dono).unwrap(),
        autorado,
        "sem partículas ainda: a pose autorada"
    );
    sim.world_mut()
        .insert_resource(Particulas([(StableId(5), [(3, [1.0, 2.0])].into())].into()));
    ph2d_app_constraints::quadro(&mut sim, &mut cena, &mut drive, None);
    let t = world_transform(sim.world(), dono).unwrap();
    assert_eq!([t.translation.x, t.translation.y], [11.0, 2.0]);
}
