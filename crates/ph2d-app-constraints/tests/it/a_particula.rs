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
        [(StableId(5), [3].into())].into(),
        "o pedido nomeia o objecto Motion alvo e o id: o Motion só procura esse"
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

/// ⭐ O quadro do PRODUTO com o extract (`07_W6_pontas.md` §3): a pilha corre com as partículas do
/// cozimento anterior, o extract espelha a Bola (e o filho) no `present`, o Motion coze, e o passe
/// depois do cozimento repõe-nas — a sprite e a caixa do gizmo desenham-se na partícula DESTE quadro.
#[test]
fn a_sprite_pinned_to_a_particle_draws_at_this_frames_particle() {
    use ph2d_ecs::{ChildOf, GlobalTransform, PresentWorld, SimRef};
    use ph2d_editor_core::GizmoView;
    use ph2d_editor_core::screens::hero::GizmoStateGroup;
    use ph2d_render::RenderInstance;
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
    let bola = sim
        .world_mut()
        .spawn((StableId(1), Transform::IDENTITY, c))
        .id();
    let mut perto = Transform::IDENTITY;
    perto.translation.y = 1.0;
    let filho = sim.world_mut().spawn((perto, ChildOf(bola))).id();
    let tique = |k: u8| Particulas([(StableId(5), [(3, [f64::from(k), 2.0])].into())].into());
    // O extract: um espelho com a instância, da pose de mundo (`RenderInstance::pose_de`).
    let extrai = |sim: &SimWorld, present: &mut PresentWorld| {
        present.clear();
        for e in [bola, filho] {
            let gt = GlobalTransform::from_transform(world_transform(sim.world(), e).unwrap());
            let (world_pos, basis) = RenderInstance::pose_de(&gt);
            let ri = RenderInstance {
                world_pos,
                size: [1.0, 1.0],
                atlas_uv: [0.0, 0.0, 1.0, 1.0],
                tint: [1.0; 4],
                basis,
                premultiplied: 0.0,
                anchor: [0.0, 0.0],
                per_corner_tint: [[1.0; 4]; 4],
                opacity: 1.0,
                flip_uv: 0,
                texture_id: 0,
                z_order: 0,
                sampling: RenderInstance::SAMPLING_DEFAULT,
                uv_xform: RenderInstance::IDENTITY_UV_XFORM,
                clip_group: RenderInstance::CLIP_GROUP_NONE,
                clip_meta: 0,
                sub_order: 0,
            };
            present.world_mut().spawn((SimRef(e), gt, ri));
        }
    };
    let desenhada = |present: &mut PresentWorld, e| {
        let mut q = present.world_mut().query::<(&SimRef, &RenderInstance)>();
        q.iter(present.world())
            .find(|(s, _)| s.0 == e)
            .map(|(_, r)| r.world_pos)
            .unwrap()
    };
    let mut present = PresentWorld::new();
    let mut gizmo = GizmoStateGroup {
        selection: Some(bola.to_bits()),
        ..Default::default()
    };
    for k in 1..6u8 {
        sim.world_mut().insert_resource(tique(k - 1));
        ph2d_app_constraints::quadro(&mut sim, &mut cena, &mut drive, None);
        extrai(&sim, &mut present);
        let antes = desenhada(&mut present, bola);
        gizmo.view = Some(GizmoView {
            bbox_min_world: [antes[0] - 0.5, antes[1] - 0.5],
            bbox_max_world: [antes[0] + 0.5, antes[1] + 0.5],
            pivot_world: antes,
            ..vista_de_mundo()
        });
        sim.world_mut().insert_resource(tique(k));
        let n = ph2d_app_constraints::depois_do_motion(
            &mut sim,
            &mut present,
            &mut drive,
            Some(&mut gizmo),
        );
        drive.settle();
        let agora = [10.0 + f32::from(k), 2.0];
        assert_eq!(
            n, 1,
            "tique {k}: a Bola moveu-se no passe depois do cozimento"
        );
        assert_eq!(
            desenhada(&mut present, bola),
            agora,
            "tique {k}: a sprite na partícula de AGORA"
        );
        assert_eq!(
            desenhada(&mut present, filho),
            [agora[0], agora[1] + 1.0],
            "tique {k}: o filho segue-a"
        );
        assert_eq!(
            gizmo.view.unwrap().pivot_world,
            agora,
            "tique {k}: e a caixa do gizmo"
        );
    }
    // Nada movido (o mesmo tique outra vez): nada se repõe.
    assert_eq!(
        ph2d_app_constraints::depois_do_motion(
            &mut sim,
            &mut present,
            &mut drive,
            Some(&mut gizmo)
        ),
        0
    );
}

/// Uma vista de gizmo de mundo, sem câmera.
fn vista_de_mundo() -> ph2d_editor_core::GizmoView {
    ph2d_editor_core::GizmoView {
        bbox_min_world: [0.0; 2],
        bbox_max_world: [0.0; 2],
        rotation: 0.0,
        camera_center: [0.0; 2],
        camera_height_world: 10.0,
        window_w: 100.0,
        window_h: 100.0,
        canvas: ph2d_editor_core::zones::Rect::new(0.0, 0.0, 100.0, 100.0),
        cursor_screen: None,
        pivot_world: [0.0; 2],
        pivot_tool_active: false,
        model: ph2d_editor_core::gizmo::MODEL_IDENTITY,
    }
}
