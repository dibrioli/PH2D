//! Os cartões da ficha f1: Fit to picture, Density guardada e o Undo do Remove. Irmão de [`super`] pelo tecto de LOC.

use super::*;

/// Fit to picture devolve a caixa do desenho, centrada, e o estado volta a «Follows the picture»;
/// o resto do colisor fica. A bola e a mão dizem «Custom size». (Ficha f1.)
#[test]
fn fit_to_picture_volta_a_acompanhar_a_imagem() {
    let (mut sim, e) = sprite_scene();
    apply(&mut sim, e, PhysicsFieldEdit::Add);
    let segue = |sim: &SimWorld| {
        let tree = ph2d_tags::TagTree::default();
        build_physics_info(
            sim.world(),
            e.to_bits(),
            0,
            0,
            0,
            false,
            0,
            (0.0, 1.0),
            0,
            &tree,
        )
        .expect("info")
        .shape_follows
    };
    assert_eq!(segue(&sim), Some(true), "a caixa nasce com o desenho");
    apply(&mut sim, e, PhysicsFieldEdit::Density(3.0));
    apply(&mut sim, e, PhysicsFieldEdit::HalfX(9.0));
    apply(&mut sim, e, PhysicsFieldEdit::OffsetX(0.5));
    assert_eq!(segue(&sim), Some(false), "mudada à mão");
    apply(&mut sim, e, PhysicsFieldEdit::FitToPicture);
    assert_eq!(segue(&sim), Some(true));
    let c = *sim.world().get::<Collider>(e).expect("forma");
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
    let forma = |sim: &SimWorld| sim.world().get::<Collider>(e).expect("forma").shape;
    let segue = |sim: &SimWorld| {
        let tree = ph2d_tags::TagTree::default();
        build_physics_info(
            sim.world(),
            e.to_bits(),
            0,
            0,
            0,
            false,
            0,
            (0.0, 1.0),
            0,
            &tree,
        )
        .expect("info")
        .shape_follows
    };
    apply(&mut sim, e, PhysicsFieldEdit::OffsetX(0.5));
    apply(&mut sim, e, PhysicsFieldEdit::Shape(0));
    apply(&mut sim, e, PhysicsFieldEdit::FitToPicture);
    assert_eq!(
        forma(&sim),
        ColliderShape::Ball { radius: 0.5 },
        "metade do menor lado"
    );
    assert_eq!(
        sim.world().get::<Collider>(e).expect("forma").offset,
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
        assert_eq!(sim.world().get::<Collider>(e).expect("forma").density, 7.5);
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
    let reg = registry();
    let mut prop = ph2d_ecs::TransformPropagationState::new(sim.world_mut());
    let mut work = ph2d_ecs::WorklistBuf::default();
    let mut snap = |sim: &mut SimWorld| {
        let mut out = ph2d_ecs::scene::WorldSnapshot::default();
        ph2d_ecs::scene::world_to_snapshot(sim.world_mut(), &mut prop, &mut work, &reg, &mut out)
            .expect("captura");
        out
    };
    let antes = snap(&mut sim);
    let forma = *sim.world().get::<Collider>(e).expect("forma");
    apply(&mut sim, e, PhysicsFieldEdit::Remove);
    assert_ne!(antes, snap(&mut sim), "o Remove não daria passo de undo");
    // Como o `undo::restore`: despawna e repõe; os bits mudam, o objecto é o que tem Transform.
    let _ = sim.world_mut().despawn(e);
    ph2d_ecs::scene::snapshot_to_world(sim.world_mut(), &antes, &reg).expect("restore");
    let de_volta = sim
        .world_mut()
        .query::<(Entity, &RigidBody, &Collider)>()
        .iter(sim.world())
        .map(|(e, b, c)| (e, b.kind, *c))
        .next()
        .expect("o corpo e a forma voltam");
    assert_eq!(de_volta.1, BodyKind::Static);
    assert_eq!(de_volta.2, forma);
}
