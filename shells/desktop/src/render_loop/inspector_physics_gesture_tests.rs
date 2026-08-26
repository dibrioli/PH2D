//! **Os gestos COMPOSTOS da §11** — uma sequência de cliques produz uma coisa que
//! funciona?
//!
//! Separado do irmão `inspector_physics_tests` (que pergunta, por EDIT, se o clique
//! escreve o componente certo) quando a pergunta do Enio — *"como eu criaria uma zona
//! de água usando apenas a UI?"* — mostrou que as duas coisas são diferentes: **todo
//! edit pode ter gate e o gesto ainda não levar a lugar nenhum**. Uma row que só
//! aparece depois de outra, um default que atrapalha, um passo que exige um número que
//! o artista não tem como saber — nada disso um teste por-edit enxerga.
//!
//! Por isso o oráculo aqui nunca é "os componentes existem". É a CENA: o sprite está
//! deitado no chão um segundo depois, o corpo estático parou de cair, a caixa que caiu
//! na piscina está mais alta que a idêntica que caiu ao lado.

use super::inspector_physics_tests::{apply, sprite_scene};
use ph2d_core::Vec2;
use ph2d_ecs::{SimWorld, Transform};
use ph2d_editor::PhysicsFieldEdit;
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

use super::inspector_physics::build_physics_info;

/// O snapshot do §11 para `e`, com os fatos que só a shell tem em seus valores
/// NEUTROS (sem joins, sem rig, sem peças, sem gesto armado).
///
/// ⚠️ Existe porque cada chamada deletrava os mesmos sete defaults, e a lista
/// cresce: quando a W-PartFace acrescentou `part_count`, o `fmt` explodiu as
/// dezesseis chamadas em multi-linha e o arquivo passou o cap de 600 LOC
/// (555 → 653) **sem uma linha de teste nova**. Uma porta só, um lugar para o
/// oitavo argumento.
/// ⭐ **Anexa um componente pela PORTA de produção** — a mesma que o `+` do Inspector usa
/// (ADR-0166 / F3), *com o seed*.
///
/// ⚠️ Existe porque a face vazia da §11 morreu: os gestos que começavam por *"clique em Add
/// Physics Body"* passam a começar por *"escolha Rigid Body na paleta"*, e encená-los com um
/// `world.insert()` à mão mediria a encenação em vez da porta.
pub(super) fn attach(sim: &mut ph2d_ecs::SimWorld, e: ph2d_ecs::Entity, name: &str) {
    let reg = crate::init::build_component_registry();
    crate::component_attach::attach_by_name(sim, &reg, e.to_bits(), name)
        .unwrap_or_else(|m| panic!("a porta de anexar recusou {name}: {m}"));
}

pub(super) fn snapshot(
    sim: &ph2d_ecs::SimWorld,
    e: ph2d_ecs::Entity,
) -> ph2d_editor::InspectorPhysicsInfo {
    build_physics_info(sim.world(), e.to_bits(), 0, 0, 0, false, 0, (0.0, 5.0), 0)
        .expect("§11 aparece para qualquer entidade com Transform")
}

/// **The whole feature, end to end.** Add on a plain sprite, then run the
/// clock: it has to fall and land on the floor.
#[test]
fn adding_a_body_from_the_inspector_makes_the_sprite_fall() {
    let (mut sim, e) = sprite_scene();

    // A floor to land on.
    sim.world_mut().spawn((
        Transform::from_translation(Vec2::new(0.0, 0.0)),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 50.0,
                half_y: 0.1,
            },
            ..Collider::default()
        },
    ));

    let before = sim.world().get::<Transform>(e).unwrap().translation.y;
    apply(&mut sim, e, PhysicsFieldEdit::Add);

    let mut bridge = PhysicsBridge::new();
    for tick in 1..=240u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let after = sim.world().get::<Transform>(e).unwrap().translation.y;
    assert!(
        after < before - 1.0,
        "the sprite never fell (y {before} -> {after}) — Add Physics Body reached the ECS but \
         the entity is not being simulated"
    );
    // Half-height 0.5 (the sprite is 1 m tall) resting on a floor whose top
    // is at y = 0.1.
    assert!(
        (after - 0.6).abs() < 0.15,
        "the sprite settled at y={after}, expected ~0.6 (floor top 0.1 + half height 0.5)"
    );
}

/// A body kind flip is a real change in the simulation, not just a tag: a
/// Static body must stop falling.
#[test]
fn making_a_body_static_stops_it_falling() {
    let (mut sim, e) = sprite_scene();
    apply(&mut sim, e, PhysicsFieldEdit::Add);
    apply(&mut sim, e, PhysicsFieldEdit::Kind(1)); // Static

    let before = sim.world().get::<Transform>(e).unwrap().translation.y;
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=120u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let after = sim.world().get::<Transform>(e).unwrap().translation.y;
    assert_eq!(
        after, before,
        "a Static body moved — the kind edit reached the component but not the solver"
    );
}

/// **A sequência do sinal de saída LEVA a algum lugar** (W-SignalLeave) — a
/// quarta condição de UI do módulo, aquela que as outras três não implicam.
///
/// ⚠️ **O que este gate cobre e o de seam não:** o seam prova que digitar na row
/// emite a ação; este prova que os BYTES que o commit escreve viram um componente
/// que o publicador lê. O trecho entre os dois é uma codificação postcard passada
/// pelo REGISTRO, e é exatamente ali que mora a armadilha que esta wave recusou:
/// `SignalOnHit`/`SignalOnLeave` são newtypes de `String` e hoje codificam igual,
/// então serializar a string e chamá-la de componente passaria HOJE e escreveria
/// lixo bem-formado no dia em que um dos dois ganhasse um campo.
///
/// O oráculo é a CENA: a porta abre quando o andarilho entra e fecha quando ele
/// sai — os dois nomes, na ordem, depois de a autoria ter passado pelo mesmo
/// caminho que o clique do artista usa.
#[test]
fn authoring_both_signal_names_makes_the_door_open_and_close() {
    use ph2d_ecs::scene::{
        ComponentRegistry, EditorCommand, EditorCommandQueue, apply_editor_commands,
    };
    use ph2d_physics_ecs::{GravityScale, InitialVelocity, SignalOnHit, SignalOnLeave};

    let mut registry = ComponentRegistry::new();
    ph2d_physics_ecs::register_physics_components(&mut registry);

    let mut sim = SimWorld::new();
    let door = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                is_sensor: true,
                shape: ColliderShape::Cuboid {
                    half_x: 1.0,
                    half_y: 1.0,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        GravityScale(0.0),
        InitialVelocity {
            linvel: [4.0, 0.0],
            angvel: 0.0,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.1,
                half_y: 0.1,
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(-3.0, 0.0)),
    ));

    // A autoria, pelo MESMO caminho do commit: cada tipo codificado pelo
    // `Serialize` DELE, e o `type_id` vindo do registro.
    let queue = EditorCommandQueue::new();
    for (data, type_name) in [
        (
            postcard::to_allocvec(&SignalOnHit("open".to_string())).expect("encode"),
            "ph2d::physics::SignalOnHit",
        ),
        (
            postcard::to_allocvec(&SignalOnLeave("close".to_string())).expect("encode"),
            "ph2d::physics::SignalOnLeave",
        ),
    ] {
        let entry = registry
            .get_by_name(type_name)
            .unwrap_or_else(|| panic!("{type_name} nao esta' registrado"));
        queue
            .push(EditorCommand::SetComponent {
                entity: door.to_bits(),
                type_id: entry.type_id,
                data,
            })
            .expect("queue");
    }
    apply_editor_commands(sim.world_mut(), &queue, &registry).expect("apply");

    let mut bridge = PhysicsBridge::new();
    let mut names = Vec::new();
    for t in 0..=180 {
        bridge.dispatch(&mut sim, true, t);
        for s in bridge.signal_events(&sim) {
            names.push(s.name);
        }
    }
    assert_eq!(
        names,
        vec!["open".to_string(), "close".to_string()],
        "autorar os dois nomes pelo caminho do commit tem de abrir E fechar a porta"
    );
}

/// **`Make Independent Body` PRESERVA a forma que o artista autorou** (W-PartAdopt).
///
/// ⚠️ **Nasceu VERMELHO sobre um defeito silencioso do produto.** O braço `Add`
/// deriva um `Collider` da caixa do SPRITE e o escreve por cima, e ele é a porta
/// das TRÊS faces da §11 — a vazia (onde semear é o certo: não há forma nenhuma)
/// e a de PEÇA (onde a forma já existe e é autorada). Medido numa peça `0,17 ×
/// 0,91` com offset `[0,13, −0,07]`, densidade `3,5`, camada `2` e restituição
/// `0,42`, clicar o botão devolvia `0,10 × 0,50` com **tudo zerado**.
///
/// É o MESMO defeito que a W-PartFace mediu e curou para o `Add Shape`
/// (*"a porta que CRIA a peça, clicada de novo, reescreve o collider com os
/// defaults e apaga a forma autorada em silêncio"*) — a cura foi aplicada a uma
/// das duas portas, e a outra ficou. ⚠️ **E a nota do tracker chamava o aberto
/// desta face de *"o rótulo não avisa que a peça vai saltar"***: o rótulo era o
/// menor dos problemas, e uma nota que nomeia o sintoma cosmético de um defeito
/// de dados é pior que nota nenhuma, porque ela TRANQUILIZA.
///
/// A lei é a mesma que o `Add Shape` já honra, dita uma vez: **semear é para quem
/// não tem forma**. Numa peça o botão só anexa o `RigidBody` — que é literalmente
/// o que *"tornar-se um corpo independente"* significa.
///
/// Mutação (semear incondicionalmente) ⇒ este gate sangra e o irmão da face
/// vazia fica VERDE, que é por que os dois existem.
#[test]
fn make_independent_body_keeps_the_shape_the_artist_authored() {
    use ph2d_ecs::{ChildOf, Name};
    use ph2d_render::Sprite;

    let mut sim = SimWorld::new();
    let torso = sim
        .world_mut()
        .spawn((
            Name::new("Torso"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.4,
                    half_y: 0.8,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 1.0)),
        ))
        .id();
    let part = sim
        .world_mut()
        .spawn((
            Name::new("Wide Bit"),
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.17,
                    half_y: 0.91,
                },
                offset: [0.13, -0.07],
                density: 3.5,
                layer: 2,
                restitution: 0.42,
                ..Collider::default()
            },
            // ⚠️ O sprite é de OUTRO tamanho, de propósito: é dele que o braço
            // derivava a caixa, então sem esta linha o gate não distingue
            // *preservou* de *re-derivou e deu no mesmo*.
            Sprite::atlas(
                ph2d_render::WHITE_TILE_KEY,
                [0.2, 1.0],
                [1.0, 1.0, 1.0, 1.0],
            ),
            Transform::from_translation(Vec2::new(0.5, 0.0)),
            ChildOf(torso),
        ))
        .id();

    let before = *sim
        .world()
        .get::<Collider>(part)
        .expect("a peça tem collider");
    apply(&mut sim, part, PhysicsFieldEdit::Add);
    let after = *sim
        .world()
        .get::<Collider>(part)
        .expect("collider após o Add");

    assert!(
        sim.world().get::<RigidBody>(part).is_some(),
        "o botao nao tornou a peca um corpo -- e' a unica coisa que ele deve fazer"
    );
    assert_eq!(
        after, before,
        "o botao REESCREVEU o collider autorado: {before:?} -> {after:?}. \
         Semear e' para quem NAO tem forma; numa peca ele so' anexa o corpo."
    );

    // ⚠️ **E a peça NÃO SALTA** — a outra metade da nota do tracker, medida em
    // vez de acreditada. Ela dizia que *"o rótulo não avisa que a peça vai
    // SALTAR para a pose de mundo dela"*, com um parêntese que já se contradizia
    // (*"ela já estava lá"*). Está: o `Transform` de uma peça é LOCAL e continua
    // local depois do Add — o que muda é QUEM a integra, e o `readback` volta
    // pela álgebra invertível do W5. Um aviso para um salto que não acontece
    // seria a UI mentindo com convicção.
    let mut bridge = PhysicsBridge::new();
    let world_of = |sim: &SimWorld| {
        let t = sim.world().get::<Transform>(part).expect("a peça tem pose");
        let p = sim
            .world()
            .get::<Transform>(torso)
            .expect("o dono tem pose");
        [
            p.translation.x + t.translation.x,
            p.translation.y + t.translation.y,
        ]
    };
    let before_pose = world_of(&sim);
    bridge.dispatch(&mut sim, true, 0);
    let after_pose = world_of(&sim);
    let d = (after_pose[0] - before_pose[0]).hypot(after_pose[1] - before_pose[1]);
    assert!(
        d < 1e-3,
        "a peca SALTOU {d:.4} m ao virar corpo ({before_pose:?} -> {after_pose:?})"
    );
}

/// **E na face VAZIA o botão SEMEIA** — o controle do gate acima.
///
/// Sem ele a cura vira *"nunca semeie"*, e a porta que torna um sprite pelado
/// física — o único gesto que faz a física existir na cena — nasceria sem forma.
#[test]
fn adding_a_body_to_a_plain_sprite_still_seeds_a_collider_from_it() {
    let (mut sim, e) = sprite_scene();
    assert!(
        sim.world().get::<Collider>(e).is_none(),
        "a fixture tem de comecar SEM collider, senao este gate mede o outro caso"
    );
    apply(&mut sim, e, PhysicsFieldEdit::Add);
    let col = sim
        .world()
        .get::<Collider>(e)
        .expect("um sprite pelado tem de GANHAR um collider");
    assert!(
        matches!(col.shape, ColliderShape::Cuboid { half_x, half_y } if half_x > 0.0 && half_y > 0.0),
        "o collider semeado nao tem tamanho: {:?}",
        col.shape
    );
}

/// **O PESO DE UM PLAYER CINEMÁTICO É AUTORÁVEL PELA §11, e ele CHEGA ao chão.**
///
/// A 4ª condição da política do módulo — *a sequência leva a algum lugar* — para o
/// par de controles que a `W-KinWeight` destravou. Um teste por-edit diria que o
/// `MassMode` e o `Mass` escrevem os componentes certos e ficaria verde com a row
/// **nunca oferecida**, que era o mundo até agora: o `paint_mass_source` gateava em
/// `kind == Dynamic` com a razão *"a Static/Kinematic body has infinite mass"* — uma
/// premissa que envelheceu, porque a 3ª lei (K6) transmite o peso de um player Snap
/// ao chão pela massa DELE.
///
/// ⚠️ **O oráculo é a CENA, nunca o componente:** uma jangada sem peso próprio
/// (`GravityScale(0)`, então todo milímetro é do personagem) tem de afundar MAIS
/// quando o artista autora uma massa maior. Medido pela sonda irmã
/// (`ph2d-physics-ecs/tests/measure_kinematic_case.rs`), um player Snap pressiona
/// com 100,0% de `m·g`, exactamente como o dinâmico.
#[test]
fn authoring_the_mass_of_a_kinematic_player_reaches_the_ground() {
    /// Quanto a jangada acelera com o player em cima, com e sem massa autorada.
    fn raft_accel(authored: Option<f32>) -> f32 {
        let mut sim = SimWorld::new();
        let raft = sim
            .world_mut()
            .spawn((
                Transform::from_translation(Vec2::new(0.0, 0.0)),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 3.0,
                        half_y: 0.25,
                    },
                    ..Collider::default()
                },
                ph2d_physics_ecs::GravityScale(0.0),
            ))
            .id();
        let who = sim
            .world_mut()
            .spawn((
                Transform::from_translation(Vec2::new(0.0, 0.75)),
                RigidBody {
                    kind: BodyKind::Kinematic,
                },
                Collider {
                    shape: ColliderShape::Capsule {
                        half_height: 0.3,
                        radius: 0.2,
                    },
                    ..Collider::default()
                },
                ph2d_physics_ecs::LockRotation,
                ph2d_physics_ecs::PlatformPlayer::default(),
                ph2d_physics_ecs::PlayerMode::Kinematic,
            ))
            .id();

        // ⚠️ **Pela §11, não pelo ECS** — é o gesto que está sob teste. E o
        // snapshot tem de OFERECER o par antes: sem `mass_is_read` o painel nem
        // pinta a row, e o edit abaixo seria um clique que o artista não pode dar.
        assert!(
            snapshot(&sim, who).mass_is_read,
            "a §11 tem de oferecer a massa a um player CINEMATICO — e' o chao que a le'"
        );
        if let Some(kg) = authored {
            apply(&mut sim, who, PhysicsFieldEdit::MassMode(true));
            apply(&mut sim, who, PhysicsFieldEdit::Mass(kg));
        }

        let mut bridge = PhysicsBridge::new();
        let y = |s: &SimWorld| s.world().get::<Transform>(raft).unwrap().translation.y;
        for tick in 1..=60u64 {
            bridge.dispatch(&mut sim, true, tick);
        }
        // ⚠️ A ACELERAÇÃO por segunda diferença, e não o deslocamento: depois do
        // assentamento a jangada já tem velocidade (nada a segura), e um oráculo
        // de posição mediria `v₀·t` junto.
        let y0 = y(&sim);
        for tick in 61..=120u64 {
            bridge.dispatch(&mut sim, true, tick);
        }
        let y1 = y(&sim);
        for tick in 121..=180u64 {
            bridge.dispatch(&mut sim, true, tick);
        }
        y1.mul_add(-2.0, y(&sim)) + y0
    }

    let auto = raft_accel(None);
    // ⚠️ **1 kg, e o TETO desta fixture é a massa da jangada (3 kg).** Ela não
    // tem peso próprio de propósito — todo milímetro é do personagem —, então
    // uma massa autorada acima dela faz a jangada fugir para baixo mais rápido
    // que a gravidade, e aí o personagem separa-se dela (correctamente) em vez
    // de a afundar mais. Medido: 5 kg dão `16,35 m/s²` e o número COLAPSA.
    let heavy = raft_accel(Some(1.0));
    assert!(
        auto < -0.1,
        "a jangada tem de sentir o player cinematico ja' na massa automatica \
         (acel {auto:+.4}); zero aqui e' a 3a lei a nao atravessar o modo"
    );
    assert!(
        heavy < auto * 2.0,
        "autorar 1 kg tem de afundar a jangada MUITO mais que a massa automatica \
         (~0,37 kg): auto {auto:+.4} contra autorada {heavy:+.4}"
    );
}

/// **E a massa deixa de ser oferecida ao PURO SANGUE** (W-KinPure).
///
/// ⚠️ Esta é a nota da W-KinWeight RECONFERIDA porque o número dela mudou: ela
/// abriu a row para *"um player cinemático"*, e naquele dia isso era exactamente
/// *"alguém lê a massa"* — a 3ª lei atravessava o modo. O terceiro modo a cala,
/// e sob ele nada volta a ler o número: manter a row seria devolver ao toggle
/// Auto/Manual o estado de controle morto que aquela wave existiu para curar.
///
/// ⚠️ A metade do Kinematic é o CONTROLE, e sem ela este gate não distingue
/// *"o modo fechou a row"* de *"a row nunca abriu"*.
#[test]
fn the_mass_row_follows_who_reads_it_across_the_third_mode() {
    for (mode, offered) in [
        (ph2d_physics_ecs::PlayerMode::Kinematic, true),
        (ph2d_physics_ecs::PlayerMode::Pure, false),
    ] {
        let mut sim = SimWorld::new();
        let who = sim
            .world_mut()
            .spawn((
                Transform::from_translation(Vec2::new(0.0, 0.75)),
                RigidBody {
                    kind: BodyKind::Kinematic,
                },
                Collider {
                    shape: ColliderShape::Capsule {
                        half_height: 0.3,
                        radius: 0.2,
                    },
                    ..Collider::default()
                },
                ph2d_physics_ecs::LockRotation,
                ph2d_physics_ecs::PlatformPlayer::default(),
                mode,
            ))
            .id();
        assert_eq!(
            snapshot(&sim, who).mass_is_read,
            offered,
            "{mode:?}: a row de massa segue quem LE' o numero, nunca o kind"
        );
    }
}
