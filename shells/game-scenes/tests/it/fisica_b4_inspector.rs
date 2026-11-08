//! GEF02 B4 (opção B, ADR-0199): o Inspector do corpo, o da filha «Collision Shape» e o corpo antigo.

use ph2d_core::Vec2;
use ph2d_ecs::scene::{EditorCommandQueue, apply_editor_commands};
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_editor_core::PhysicsFieldEdit;
use ph2d_physics_ecs::{Collider, RigidBody};
use ph2d_render::Sprite;

fn apply(sim: &mut SimWorld, e: Entity, edit: PhysicsFieldEdit) {
    let reg = crate::comp::registo();
    let q = EditorCommandQueue::default();
    ph2d_app_physics::physics_apply::apply_physics_edit(sim, e.to_bits(), edit, &q, &reg);
    apply_editor_commands(sim.world_mut(), &q, &reg).expect("commit");
}

fn shape_of(sim: &SimWorld, e: Entity) -> Entity {
    ph2d_physics_ecs::main_shape(sim.world(), e).unwrap_or(e)
}

fn info(sim: &SimWorld, e: Entity) -> ph2d_editor_core::InspectorPhysicsInfo {
    ph2d_app_physics::canais_corpo::retrato(sim, e.to_bits()).expect("retrato")
}

fn torso() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(0.0, 3.0)),
            Sprite::atlas(0, [2.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
            ph2d_ecs::Name::new("Torso"),
        ))
        .id();
    (sim, e)
}

/// (a) O corpo com a forma na filha só tem o cartão Body; (b) a filha tem a forma e diz de quem é.
#[test]
fn b4_o_corpo_so_mostra_o_body_e_a_filha_diz_de_quem_e() {
    let (mut sim, e) = torso();
    apply(&mut sim, e, PhysicsFieldEdit::Add);
    let corpo = info(&sim, e);
    assert!(
        corpo.has_body && corpo.shape_on_child,
        "Body só: sem Shape nem camadas"
    );
    assert!(corpo.shape_of.is_empty());
    let filha = shape_of(&sim, e);
    assert_ne!(filha, e);
    let f = info(&sim, filha);
    assert!(!f.has_body && f.has_collider && !f.shape_on_child);
    assert_eq!(f.shape_of, "Torso", "«Shape of: Torso»");
    assert_eq!(f.kind_tag, corpo.kind_tag, "a forma segue o tipo do corpo");
    assert_eq!(
        f.shape_follows,
        Some(true),
        "a imagem do corpo mede a forma"
    );
}

/// (c) O corpo antigo com `Collider` próprio: «Move shape to a child» leva a forma e o lado-forma.
#[test]
fn b4_move_shape_to_a_child_leva_a_forma_do_corpo_antigo() {
    use ph2d_physics_ecs::{BodyKind, WalkSurface};
    let (mut sim, e) = torso();
    let col = Collider {
        layer: 2,
        density: 3.5,
        ..Collider::default()
    };
    let chao = WalkSurface {
        grip: 0.5,
        belt: 1.0,
    };
    sim.world_mut().entity_mut(e).insert((
        RigidBody {
            kind: BodyKind::Static,
        },
        col,
        chao,
    ));
    let velho = info(&sim, e);
    assert!(velho.has_body && velho.has_collider && !velho.shape_on_child);
    apply(&mut sim, e, PhysicsFieldEdit::MoveShapeToChild);
    let w = sim.world();
    assert!(w.get::<Collider>(e).is_none(), "o Collider sai do corpo");
    assert!(w.get::<WalkSurface>(e).is_none(), "o lado-forma vai junto");
    let filha = shape_of(&sim, e);
    assert_ne!(filha, e);
    assert_eq!(w.get::<Collider>(filha), Some(&col));
    assert_eq!(w.get::<WalkSurface>(filha), Some(&chao));
    assert_eq!(
        w.get::<ph2d_ecs::Name>(filha)
            .map(|n| n.as_str().to_owned()),
        Some(ph2d_physics_ecs::NOME_DA_FORMA.to_owned())
    );
    assert!(info(&sim, e).shape_on_child);
    apply(&mut sim, e, PhysicsFieldEdit::MoveShapeToChild);
    assert_eq!(
        ph2d_ecs::ordered_children(sim.world(), e).len(),
        1,
        "segunda vez: nada muda"
    );
}
