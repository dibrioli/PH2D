//! **O gesto da SUPERFÍCIE, ponta a ponta** (`W-Surface`) — irmão de
//! [`super::inspector_physics_gesture_tests`], cortado pelo teto de LOC do
//! shell e pelo mesmo assunto que separa os escritores: *de que este chão é
//! feito para quem anda*.

use ph2d_core::Vec2;
use ph2d_ecs::{SimWorld, Transform};
use ph2d_editor::PhysicsFieldEdit;
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

/// **A SEQUÊNCIA da superfície leva a algum lugar** (`W-Surface`) — a quarta
/// condição de UI do módulo, a que as outras três não implicam.
///
/// ⚠️ **O que este gate cobre e o de seam não:** o seam prova que digitar na row
/// emite a ação; este prova que os BYTES que o commit escreve viram um
/// componente **que a LEI DA CAMINHADA lê**. O trecho entre os dois é uma
/// codificação postcard passada pelo REGISTRO, e a superfície é o primeiro
/// componente desta seção cujo consumidor não é o solver: ela atravessa a
/// ponte, entra na `GroundSample` e só então muda alguma coisa.
///
/// ⚠️ **E a autoria vai pelo `apply_physics_edit`, a porta do CLIQUE — não por
/// um postcard montado à mão.** É o que põe o *guard* do braço sob teste, e ele
/// é a razão de a wave existir: uma superfície é propriedade da FACE que o pé
/// encontra, e uma face pode ser a PEÇA de um corpo composto, que carrega
/// `Collider` e **não** carrega `RigidBody`. Herdar o guard dos irmãos desta
/// seção deixaria a terceira pista inexprimível — e é por isso que ela está
/// aqui.
///
/// O oráculo é a CENA: os três personagens são idênticos e dois deles pisam em
/// gelo — quem tem tração arranca, quem não tem fica onde estava.
#[test]
fn authoring_a_surface_makes_one_of_three_identical_players_slip() {
    use ph2d_ecs::ChildOf;
    use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue, apply_editor_commands};
    use ph2d_physics_ecs::{LockRotation, PlatformPlayer, PlayerInput, WalkSurface};

    let mut registry = ComponentRegistry::new();
    ph2d_physics_ecs::register_physics_components(&mut registry);

    let mut sim = SimWorld::new();
    // Dois pisos idênticos, longe um do outro, e um jogador em cada.
    let mut lane = |x: f32| {
        let floor = sim
            .world_mut()
            .spawn((
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 30.0,
                        half_y: 0.5,
                    },
                    ..Collider::default()
                },
                Transform::from_translation(Vec2::new(x, -0.5)),
            ))
            .id();
        let player = sim
            .world_mut()
            .spawn((
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Capsule {
                        half_height: 0.3,
                        radius: 0.2,
                    },
                    ..Collider::default()
                },
                LockRotation,
                PlatformPlayer {
                    float_height: 0.9,
                    // ⚠️ Baixos de propósito, a mesma nota da suíte da ponte: com
                    // os 60 m/s² do default o cruzeiro chega em poucos tiques
                    // mesmo com um quarto da tração, e a diferença some no ruído.
                    speed: 8.0,
                    acceleration: 8.0,
                    ..PlatformPlayer::default()
                },
                Transform::from_translation(Vec2::new(x, 0.9)),
            ))
            .id();
        (floor, player)
    };
    let (_wood_floor, on_wood) = lane(0.0);
    let (ice_floor, on_ice) = lane(200.0);
    // A TERCEIRA pista: o chão é uma PEÇA — um filho com `Collider` e **sem**
    // `RigidBody` (W-Compound). É ela que põe o guard do braço sob teste.
    //
    // ⚠️ **As duas metades são DISJUNTAS em x, e sem isso o gate mede a coisa
    // errada:** com a peça POR CIMA do collider do pai o pé encontra o do pai —
    // que não tem superfície — e a pista media 13,05 m de arranque sobre um
    // produto correto. O pai cobre `340..400`, a peça `400..460`, e o
    // personagem nasce sobre a peça.
    let (deck, on_part) = lane(370.0);
    let ice_part = sim
        .world_mut()
        .spawn((
            ChildOf(deck),
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 30.0,
                    half_y: 0.5,
                },
                ..Collider::default()
            },
            // ⚠️ LOCAL, não mundo (a lição do W5): o pai está em `(370, -0.5)`,
            // então esta translação é a DIFERENÇA até `(430, -0.5)`.
            Transform::from_translation(Vec2::new(60.0, 0.0)),
        ))
        .id();
    // E o personagem desta pista nasce sobre a PEÇA, não sobre o pai.
    sim.world_mut()
        .get_mut::<Transform>(on_part)
        .expect("transform")
        .translation
        .x = 430.0;

    // A autoria do GELO, pela porta do CLIQUE — a mesma função que a §11 chama.
    let queue = EditorCommandQueue::new();
    for floor in [ice_floor, ice_part] {
        super::inspector_physics_apply::apply_physics_edit(
            &sim,
            floor.to_bits(),
            PhysicsFieldEdit::WalkGrip(0.0),
            &queue,
            &registry,
        );
    }
    apply_editor_commands(sim.world_mut(), &queue, &registry).expect("apply");
    // ⚠️ A premissa que o resto do gate assume: a autoria de facto ATERRISSOU.
    // Sem isto, um braço que recusasse em silêncio deixaria as duas metades de
    // baixo verdes por vácuo — os jogadores parados por não terem arrancado.
    for floor in [ice_floor, ice_part] {
        assert!(
            sim.world().get::<WalkSurface>(floor).is_some(),
            "a autoria pelo caminho do clique nao escreveu a superficie"
        );
    }

    let mut bridge = PhysicsBridge::new();
    let start = |sim: &SimWorld, e| {
        sim.world()
            .get::<ph2d_ecs::Transform>(e)
            .expect("transform")
            .translation
            .x
    };
    // ⚠️ A perna assenta com o eixo SOLTO — a janela do spawn é aérea, e um
    // `grip = 0` colhe velocidade ali e a guarda para sempre.
    for t in 1..=30 {
        for p in [on_wood, on_ice, on_part] {
            bridge.set_player_input(p, PlayerInput::default());
        }
        bridge.dispatch(&mut sim, true, t);
    }
    let (wood0, ice0, part0) = (
        start(&sim, on_wood),
        start(&sim, on_ice),
        start(&sim, on_part),
    );
    for t in 31..=150 {
        for p in [on_wood, on_ice, on_part] {
            bridge.set_player_input(
                p,
                PlayerInput {
                    drive: 1.0,
                    ..PlayerInput::default()
                },
            );
        }
        bridge.dispatch(&mut sim, true, t);
    }
    let wood = start(&sim, on_wood) - wood0;
    let ice = start(&sim, on_ice) - ice0;
    let part = start(&sim, on_part) - part0;
    assert!(wood > 4.0, "quem tem tracao arranca: {wood:.3} m em 2 s");
    assert!(
        ice.abs() < 0.05,
        "e o gelo autorado pelo caminho do clique nao deixa arrancar: {ice:.4} m"
    );
    assert!(
        part.abs() < 0.05,
        "e o gelo autorado numa PECA (Collider sem RigidBody) tambem nao: {part:.4} m"
    );
}
