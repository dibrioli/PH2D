//! Os cartões da ficha f1: Fit to picture, Density guardada e o Undo do Remove. Irmão de [`super`] pelo tecto de LOC.

use super::*;

/// A info do Inspector com `partes` peças penduradas (o resto neutro).
fn info_de(sim: &SimWorld, e: Entity, partes: u8) -> ph2d_editor_core::InspectorPhysicsInfo {
    let tree = ph2d_tags::TagTree::default();
    let bake = (0.0, 1.0);
    build_physics_info(
        sim.world(),
        e.to_bits(),
        0,
        0,
        partes,
        false,
        0,
        bake,
        0,
        &tree,
    )
    .expect("i")
}

/// A captura do undo (`ProjectState` leva o `WorldSnapshot`).
fn snap(sim: &mut SimWorld) -> ph2d_ecs::scene::WorldSnapshot {
    let mut prop = ph2d_ecs::TransformPropagationState::new(sim.world_mut());
    let (mut work, reg) = (ph2d_ecs::WorklistBuf::default(), registry());
    let mut out = ph2d_ecs::scene::WorldSnapshot::default();
    ph2d_ecs::scene::world_to_snapshot(sim.world_mut(), &mut prop, &mut work, &reg, &mut out)
        .expect("captura");
    out
}

/// Fit to picture devolve a caixa do desenho, centrada, e o estado volta a «Follows the picture»;
/// o resto do colisor fica. A bola e a mão dizem «Custom size». (Ficha f1.)
#[test]
fn fit_to_picture_volta_a_acompanhar_a_imagem() {
    let (mut sim, e) = sprite_scene();
    apply(&mut sim, e, PhysicsFieldEdit::Add);
    let segue = |sim: &SimWorld| info_de(sim, e, 0).shape_follows;
    assert_eq!(segue(&sim), Some(true), "a caixa nasce com o desenho");
    apply(&mut sim, e, PhysicsFieldEdit::Density(3.0));
    apply(&mut sim, e, PhysicsFieldEdit::HalfX(9.0));
    apply(&mut sim, e, PhysicsFieldEdit::OffsetX(0.5));
    assert_eq!(segue(&sim), Some(false), "mudada à mão");
    apply(&mut sim, e, PhysicsFieldEdit::FitToPicture);
    assert_eq!(segue(&sim), Some(true));
    let c = *sim
        .world()
        .get::<Collider>(shape_of(&sim, e))
        .expect("forma");
    assert_eq!(c.offset, [0.0, 0.0]);
    assert_eq!(c.density, 3.0, "o Fit só mexe na medida e no sítio");
    apply(&mut sim, e, PhysicsFieldEdit::Shape(0));
    assert_eq!(segue(&sim), Some(false), "a bola não acompanha");
}

/// Fit to picture mantém o tipo: a bola fica bola, a cápsula fica cápsula, e nenhuma diz «Follows». (Ficha f1.)
#[test]
fn fit_to_picture_mantem_o_tipo_da_forma() {
    let (mut sim, e) = sprite_scene(); // 2 × 1 m
    apply(&mut sim, e, PhysicsFieldEdit::Add);
    let forma = |sim: &SimWorld| {
        sim.world()
            .get::<Collider>(shape_of(sim, e))
            .expect("forma")
            .shape
    };
    let segue = |sim: &SimWorld| info_de(sim, e, 0).shape_follows;
    apply(&mut sim, e, PhysicsFieldEdit::OffsetX(0.5));
    apply(&mut sim, e, PhysicsFieldEdit::Shape(0));
    apply(&mut sim, e, PhysicsFieldEdit::FitToPicture);
    assert_eq!(
        forma(&sim),
        ColliderShape::Ball { radius: 0.5 },
        "metade do menor lado"
    );
    assert_eq!(
        sim.world()
            .get::<Collider>(shape_of(&sim, e))
            .expect("forma")
            .offset,
        [0.0, 0.0]
    );
    assert_eq!(segue(&sim), Some(false), "a bola nunca diz que acompanha");
    apply(&mut sim, e, PhysicsFieldEdit::Shape(2));
    apply(&mut sim, e, PhysicsFieldEdit::FitToPicture);
    assert_eq!(
        forma(&sim),
        ColliderShape::Capsule {
            half_height: 0.0,
            radius: 1.0
        },
        "radius = meia largura; a altura não dá para mais"
    );
    assert_eq!(segue(&sim), Some(false));
    if let Some(mut s) = sim.world_mut().get_mut::<ph2d_render::Sprite>(e) {
        s.size = [1.0, 3.0];
    }
    apply(&mut sim, e, PhysicsFieldEdit::FitToPicture);
    assert_eq!(
        forma(&sim),
        ColliderShape::Capsule {
            half_height: 1.0,
            radius: 0.5
        }
    );
    apply(&mut sim, e, PhysicsFieldEdit::Shape(1));
    apply(&mut sim, e, PhysicsFieldEdit::FitToPicture);
    assert_eq!(
        forma(&sim),
        ColliderShape::Cuboid {
            half_x: 0.5,
            half_y: 1.5
        }
    );
    assert_eq!(segue(&sim), Some(true), "só a caixa acompanha");
}

/// Density some do cartão em Static e Kinematic, e o valor volta com Dynamic. (Ficha f1.)
#[test]
fn density_fica_guardada_ao_trocar_de_tipo() {
    let (mut sim, e) = sprite_scene();
    apply(&mut sim, e, PhysicsFieldEdit::Add);
    apply(&mut sim, e, PhysicsFieldEdit::Density(7.5));
    for kind in [1, 2, 0] {
        apply(&mut sim, e, PhysicsFieldEdit::Kind(kind));
        assert_eq!(
            sim.world()
                .get::<Collider>(shape_of(&sim, e))
                .expect("forma")
                .density,
            7.5
        );
    }
}

/// ⭐ **O «Undo» do recibo é de verdade** (ficha f1): o Remove muda a captura do undo (`ProjectState`
/// leva o `WorldSnapshot`), e repor a captura de antes devolve corpo e forma, com os valores.
#[test]
fn undoing_remove_brings_body_and_shape_back() {
    let (mut sim, e) = sprite_scene();
    apply(&mut sim, e, PhysicsFieldEdit::Add);
    apply(&mut sim, e, PhysicsFieldEdit::Kind(1));
    apply(&mut sim, e, PhysicsFieldEdit::Density(3.5));
    let antes = snap(&mut sim);
    let forma = *sim
        .world()
        .get::<Collider>(shape_of(&sim, e))
        .expect("forma");
    apply(&mut sim, e, PhysicsFieldEdit::Remove);
    assert_ne!(antes, snap(&mut sim), "o Remove não daria passo de undo");
    // Como o `undo::restore`: despawna e repõe; os bits mudam, o objecto é o que tem Transform.
    let _ = sim.world_mut().despawn(e);
    ph2d_ecs::scene::snapshot_to_world(sim.world_mut(), &antes, &registry()).expect("restore");
    // Opção B: a forma volta na filha principal do corpo, nunca no próprio corpo.
    let (corpo, kind) = sim
        .world_mut()
        .query::<(Entity, &RigidBody)>()
        .iter(sim.world())
        .map(|(e, b)| (e, b.kind))
        .next()
        .expect("o corpo volta");
    let de_volta = ph2d_physics_ecs::main_shape(sim.world(), corpo)
        .and_then(|f| sim.world().get::<Collider>(f).copied())
        .expect("a forma volta");
    assert_eq!(kind, BodyKind::Static);
    assert_eq!(de_volta, forma);
}

/// ⭐ **Corpo sem forma** (dono 08/11): a info diz `body_without_shape`, o «Add Shape» dá a caixa da
/// imagem (numa filha, opção B) sem tocar no tipo, e é UM passo de undo (a captura muda e repô-la
/// tira só a forma).
#[test]
fn corpo_sem_forma_ganha_a_caixa_da_imagem_num_passo() {
    let (mut sim, e) = sprite_scene();
    let estatico = RigidBody {
        kind: BodyKind::Static,
    };
    sim.world_mut().entity_mut(e).insert(estatico);
    let info = |sim: &SimWorld, partes| info_de(sim, e, partes);
    assert!(info(&sim, 0).body_without_shape && !info(&sim, 0).has_body);
    assert!(
        !info(&sim, 1).body_without_shape,
        "uma peça pendurada já é forma"
    );
    let antes = snap(&mut sim);
    apply(&mut sim, e, PhysicsFieldEdit::AddShape);
    // Opção B: a forma nasce FILHA «Collision Shape», nunca no corpo.
    assert!(sim.world().get::<Collider>(e).is_none(), "a forma no corpo");
    let f = ph2d_physics_ecs::main_shape(sim.world(), e).expect("a forma nasce");
    assert_eq!(
        sim.world()
            .get::<ph2d_ecs::ChildOf>(f)
            .map(ph2d_ecs::ChildOf::parent),
        Some(e)
    );
    let c = *sim.world().get::<Collider>(f).expect("a forma nasce");
    assert_eq!(
        c.shape,
        ColliderShape::Cuboid {
            half_x: 1.0,
            half_y: 0.5
        }
    );
    assert_eq!(sim.world().get::<RigidBody>(e), Some(&estatico));
    // A filha conta como peça pela mesma contagem do produto (o cartão da filha é a B4).
    let w = sim.world_mut();
    let mut q = w.query_filtered::<Entity, (
        bevy_ecs::query::With<Collider>,
        bevy_ecs::query::Without<RigidBody>,
    )>();
    let candidatas: Vec<Entity> = q.iter(w).collect();
    let partes = ph2d_physics_ecs::count_parts(sim.world(), e, candidatas);
    assert_eq!(partes, 1);
    assert!(!info(&sim, 1).body_without_shape, "o aviso fica");
    assert_ne!(antes, snap(&mut sim), "o Add Shape não daria passo de undo");
    let _ = sim.world_mut().despawn(e);
    ph2d_ecs::scene::snapshot_to_world(sim.world_mut(), &antes, &registry()).expect("restore");
    let mut q = sim.world_mut().query::<(&RigidBody, Option<&Collider>)>();
    let (b, c) = q.iter(sim.world()).next().expect("o corpo volta");
    assert_eq!((b.kind, c), (BodyKind::Static, None));
}

/// Dono 09/11: a filha sem «Remove Shape» apaga-se na Hierarchy (`hierarchy_delete::drain` = `despawn`):
/// UM passo de undo, o corpo fica sem forma, e repor a captura devolve a filha com a mesma forma.
#[test]
fn apagar_a_forma_filha_na_hierarchy_e_um_passo_de_undo() {
    let (mut sim, e) = sprite_scene();
    apply(&mut sim, e, PhysicsFieldEdit::Add);
    let (antes, f) = (snap(&mut sim), shape_of(&sim, e));
    let forma = sim.world().get::<Collider>(f).copied();
    assert!(f != e && forma.is_some(), "opção B: a forma é filha");
    let _ = sim.world_mut().despawn(f);
    let (depois, info) = (snap(&mut sim), info_de(&sim, e, 0));
    assert!(antes != depois, "apagar a filha não daria passo de undo");
    assert!(info.body_without_shape, "o corpo fica, sem forma");
    let _ = sim.world_mut().despawn(e);
    ph2d_ecs::scene::snapshot_to_world(sim.world_mut(), &antes, &registry()).expect("restore");
    let mut q = sim.world_mut().query::<(Entity, &RigidBody)>();
    let (corpo, _) = q.iter(sim.world()).next().expect("o corpo");
    let de_volta = sim.world().get::<Collider>(shape_of(&sim, corpo)).copied();
    assert_eq!(de_volta, forma, "a filha volta com a forma");
}
