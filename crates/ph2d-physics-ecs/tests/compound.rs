//! **O corpo composto, atravessando a PONTE** (W-Compound).
//!
//! O wrapper já prova a física (`ph2d-physics/tests/compound.rs`). O que estes
//! gates provam é o que só existe deste lado: que **um filho com `Collider` e sem
//! `RigidBody` vira uma forma do corpo ancestral**, que um GRUPO no meio é
//! transparente, e que um rewind re-pendura as peças.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

fn box_of(half_x: f32, half_y: f32) -> Collider {
    Collider {
        shape: ColliderShape::Cuboid { half_x, half_y },
        ..Collider::default()
    }
}

/// Um "L": braço horizontal (o corpo) e perna vertical (a peça) pendurada na
/// ponta direita dele. `via_group` põe um nó SEM desenho e SEM corpo no meio — a
/// pasta que a transparência tem de atravessar.
fn ell(via_group: bool) -> SimWorld {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        box_of(20.0, 0.5),
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    let arm = sim
        .world_mut()
        .spawn((
            Name::new("Arm"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            box_of(1.0, 0.2),
            Transform::from_translation(Vec2::new(0.0, 5.0)),
        ))
        .id();
    let parent = if via_group {
        sim.world_mut()
            .spawn((
                Name::new("Shapes"),
                Transform::from_translation(Vec2::new(0.0, 0.0)),
                ChildOf(arm),
            ))
            .id()
    } else {
        arm
    };
    sim.world_mut().spawn((
        Name::new("Leg"),
        box_of(0.2, 1.0),
        Transform::from_translation(Vec2::new(0.8, -1.0)),
        ChildOf(parent),
    ));
    sim
}

fn run(sim: &mut SimWorld, ticks: u64) -> PhysicsBridge {
    let mut bridge = PhysicsBridge::new();
    for t in 0..=ticks {
        bridge.dispatch(sim, true, t);
    }
    bridge
}

fn named(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entidade viva")
}

fn world_y(sim: &SimWorld, e: Entity) -> f32 {
    ph2d_ecs::world_transform(sim.world(), e)
        .expect("transform")
        .translation
        .y
}

/// **A peça EXISTE para o solver** — e a medição de que ela não existia é o
/// motivo desta wave: a perna atravessava o chão (`y = −0,30`, topo do chão em
/// `0,5`), sem erro e sem warning.
#[test]
fn a_child_collider_becomes_a_shape_of_the_ancestor_body() {
    let mut sim = ell(false);
    run(&mut sim, 180);
    let leg = named(&mut sim, "Leg");
    assert!(
        world_y(&sim, leg) > 0.4,
        "a perna afundou até {:.3} — o chão está em 0,5, então ela não tem collider",
        world_y(&sim, leg)
    );
}

/// **Um GRUPO no meio é transparente** — o mesmo walk do `rig_edges`, e pela
/// mesma razão: pôr as formas de uma peça dentro de uma pasta não pode
/// desligá-las do corpo.
#[test]
fn a_group_between_the_part_and_the_body_is_transparent() {
    let mut sim = ell(true);
    run(&mut sim, 180);
    let leg = named(&mut sim, "Leg");
    assert!(
        world_y(&sim, leg) > 0.4,
        "com um grupo no meio a perna afundou até {:.3}",
        world_y(&sim, leg)
    );
}

/// **A peça é uma FORMA, não um segundo corpo** — a diferença que a sonda mediu:
/// dois corpos ligados se espalham (o offset autorado `[0,8, −1,0]` virou
/// `[2,08, +0,80]`), uma peça viaja rígida com o dono.
#[test]
fn a_part_travels_rigidly_with_its_body() {
    let mut sim = ell(false);
    run(&mut sim, 180);
    let arm = named(&mut sim, "Arm");
    let leg = named(&mut sim, "Leg");
    let a = ph2d_ecs::world_transform(sim.world(), arm)
        .expect("t")
        .translation;
    let l = ph2d_ecs::world_transform(sim.world(), leg)
        .expect("t")
        .translation;
    // A perna é FILHA, então o `Transform` dela é local e o readback nunca o
    // escreve — a pose de mundo dela é a composição, e o offset relativo tem de
    // ser exatamente o autorado, girado com o corpo.
    let d = ((l.x - a.x).powi(2) + (l.y - a.y).powi(2)).sqrt();
    let authored = (0.8f32).hypot(1.0);
    assert!(
        (d - authored).abs() < 1e-4,
        "a peça está a {d:.4} do corpo, e foi autorada a {authored:.4} — ela não é \
         rígida com ele"
    );
}

/// **Um rewind re-pendura as peças.** O `rebuild_from_rest` troca o mundo
/// inteiro; uma peça esquecida ali deixa o corpo composto replayar com metade das
/// formas — e o silêncio disso é o do Weston (alvo `0` replaya zero passos, então
/// o primeiro Reset parece certo).
#[test]
fn a_rewind_re_hangs_the_parts() {
    let mut sim = ell(false);
    let mut bridge = PhysicsBridge::new();
    for t in 0..=180u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let leg = named(&mut sim, "Leg");
    let live = world_y(&sim, leg);

    // Scrub ao zero e re-simula pelo mesmo caminho.
    bridge.dispatch(&mut sim, true, 0);
    for t in 0..=180u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let replayed = world_y(&sim, leg);
    assert!(
        (replayed - live).abs() < 0.05,
        "o replay divergiu: {replayed:.3} contra {live:.3} ao vivo — a peça não \
         voltou ao mundo reconstruído"
    );
    assert!(
        replayed > 0.4,
        "depois do rewind a perna afundou até {replayed:.3}"
    );
}

/// **Um collider SEM corpo acima não vira nada**, e isso é a recusa honesta: ele
/// não é simulado, e inventar um dono seria escolher por conta própria de quem
/// aquela forma é.
#[test]
fn a_collider_with_no_body_above_it_is_not_a_part() {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        box_of(20.0, 0.5),
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    // Solta: nenhum ancestral, nenhum corpo.
    let lone = sim
        .world_mut()
        .spawn((
            Name::new("Lone"),
            box_of(0.3, 0.3),
            Transform::from_translation(Vec2::new(0.0, 5.0)),
        ))
        .id();
    run(&mut sim, 120);
    assert!(
        (world_y(&sim, lone) - 5.0).abs() < 1e-6,
        "a forma solta se moveu para {:.3} — ela não devia ser simulada",
        world_y(&sim, lone)
    );
}

/// **As regras de COMBINE de uma peça chegam ao solver** (W-PartFace).
///
/// `reconcile_parts` passava `MaterialCombine::default()` enquanto o
/// `OneWayPlatform` logo abaixo já vinha da entidade — a única assimetria da
/// lista, e um descuido da W-Compound. Ela decidia uma pergunta de UI por
/// omissão: com ela, os dois chips de combine numa peça seriam controles que o
/// solver ignora.
///
/// O oráculo é a `Max` do W-Material: uma superball de `Max` quica em QUALQUER
/// piso, mesmo num chão morto — e o CONTROLE (`Average`, a média com o zero do
/// chão) é o que torna a diferença atribuível.
#[test]
fn a_parts_combine_rules_reach_the_solver() {
    use ph2d_physics_ecs::{CombineRule, MaterialCombine};

    // Um corpo cuja ÚNICA forma que toca o chão é a PEÇA: o braço fica alto e a
    // perna desce até o piso, então o contato medido é o dela.
    fn drop_with(rule: CombineRule) -> f32 {
        let mut sim = SimWorld::new();
        sim.world_mut().spawn((
            Name::new("Floor"),
            RigidBody {
                kind: BodyKind::Static,
            },
            // Chão MORTO: o quique tem de vir da regra, não do piso.
            Collider {
                restitution: 0.0,
                ..box_of(20.0, 0.5)
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ));
        let arm = sim
            .world_mut()
            .spawn((
                Name::new("Arm"),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                box_of(0.4, 0.2),
                Transform::from_translation(Vec2::new(0.0, 4.0)),
            ))
            .id();
        sim.world_mut().spawn((
            Name::new("Foot"),
            Collider {
                restitution: 1.0,
                ..box_of(0.4, 0.4)
            },
            MaterialCombine {
                restitution: rule,
                friction: CombineRule::Average,
            },
            Transform::from_translation(Vec2::new(0.0, -0.8)),
            ChildOf(arm),
        ));
        let mut bridge = PhysicsBridge::new();
        // O pico depois do primeiro impacto: com `Max` o corpo sobe de volta.
        let mut peak = f32::MIN;
        for t in 0..=140u64 {
            bridge.dispatch(&mut sim, true, t);
            if t > 60 {
                peak = peak.max(world_y(&sim, arm));
            }
        }
        peak
    }

    let max = drop_with(CombineRule::Max);
    let avg = drop_with(CombineRule::Average);
    assert!(
        max > avg + 0.3,
        "a regra da peça não chegou ao solver: Max subiu a {max:.3} e Average a \
         {avg:.3} — a peça está usando o default"
    );
}

/// **A pose de uma PEÇA editada com o relógio ANDANDO chega ao solver**
/// (W-PartFace, report do Enio: *"houve penetração da Shape no Static"*).
///
/// O `reconcile_parts` só re-descrevia uma peça **em repouso** — um gate copiado
/// do corpo, cuja pose o SOLVER escreve. Numa peça a pose é AUTORADA o tempo
/// todo, e o preço do gate era exatamente o reportado: mover a peça movia o
/// DESENHO e deixava o collider parado, então a forma atravessava o estático em
/// silêncio.
///
/// O oráculo é a consequência FÍSICA, não a bookkeeping: a peça é empurrada para
/// dentro de um teto estático, e o dono tem de ser empurrado para baixo. Medido
/// antes do fix: `0.2990` com e sem o movimento — bit-idêntico, isto é, nada
/// aconteceu.
#[test]
fn a_part_moved_while_the_clock_runs_takes_its_collider_along() {
    fn owner_y(move_it: bool) -> f32 {
        let mut sim = SimWorld::new();
        sim.world_mut().spawn((
            Name::new("Floor"),
            RigidBody {
                kind: BodyKind::Static,
            },
            box_of(20.0, 0.5),
            Transform::from_translation(Vec2::new(0.0, -0.5)),
        ));
        sim.world_mut().spawn((
            Name::new("Ceiling"),
            RigidBody {
                kind: BodyKind::Static,
            },
            box_of(20.0, 0.5),
            Transform::from_translation(Vec2::new(0.0, 3.0)),
        ));
        let owner = sim
            .world_mut()
            .spawn((
                Name::new("Owner"),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                box_of(0.3, 0.3),
                Transform::from_translation(Vec2::new(0.0, 0.3)),
            ))
            .id();
        let part = sim
            .world_mut()
            .spawn((
                Name::new("Part"),
                box_of(0.3, 0.3),
                Transform::from_translation(Vec2::new(1.0, 0.0)),
                ChildOf(owner),
            ))
            .id();
        let mut bridge = PhysicsBridge::new();
        for t in 0..=60u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        if move_it {
            // O artista arrasta a PEÇA para cima, até dentro do teto.
            if let Some(mut t) = sim.world_mut().get_mut::<Transform>(part) {
                t.translation.y = 2.6;
            }
        }
        for t in 61..=200u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        world_y(&sim, owner)
    }
    let still = owner_y(false);
    let moved = owner_y(true);
    assert!(
        moved < still - 0.05,
        "mover a peça não mexeu no solver: dono em {moved:.4} contra {still:.4} \
         parado — o collider ficou onde estava e o desenho atravessou o teto"
    );
}

/// **Nada escreve de volta na pose AUTORADA de uma peça.**
///
/// O `readback` da física escreve o `Transform` dos CORPOS; uma peça não é um, e
/// a pose dela é autoria pura. Sem isto, a simulação reescreveria o número que o
/// artista digitou — a doença que o W5 curou nos corpos filhos, um nível abaixo.
///
/// ⚠️ **Este gate NÃO guarda a DERIVAÇÃO, e o nome antigo dizia que sim.** Ele
/// nasceu chamado *"a pose local não deriva durante o play"*, e a mutação que
/// devolve o round-trip pelo solver **passou por ele** — porque o que deriva é o
/// `local` calculado DENTRO da ponte, que nenhum `Transform` mostra. Quem guarda
/// aquilo é o gate de CHURN abaixo. Renomeado para afirmar o que de fato mede.
#[test]
fn nothing_writes_back_to_a_parts_authored_pose() {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        box_of(20.0, 0.5),
        Transform::from_translation(Vec2::new(0.0, -0.5)),
    ));
    // Uma RAMPA: um corpo caindo reto não exercita a rotação, que é onde um
    // round-trip deriva.
    sim.world_mut().spawn((
        Name::new("Ramp"),
        RigidBody {
            kind: BodyKind::Static,
        },
        box_of(3.0, 0.2),
        Transform {
            translation: Vec2::new(0.0, 1.0),
            rotation: 0.35,
            ..Transform::IDENTITY
        },
    ));
    let owner = sim
        .world_mut()
        .spawn((
            Name::new("Owner"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            box_of(0.3, 0.3),
            Transform::from_translation(Vec2::new(-1.0, 3.0)),
        ))
        .id();
    let authored = Transform {
        translation: Vec2::new(0.55, 0.1),
        rotation: 0.4,
        ..Transform::IDENTITY
    };
    let part = sim
        .world_mut()
        .spawn((
            Name::new("Part"),
            box_of(0.2, 0.2),
            authored,
            ChildOf(owner),
        ))
        .id();
    let mut bridge = PhysicsBridge::new();
    for t in 0..=300u64 {
        bridge.dispatch(&mut sim, true, t);
        let live = sim.world().get::<Transform>(part).copied().expect("t");
        assert!(
            live.translation.x == authored.translation.x
                && live.translation.y == authored.translation.y
                && live.rotation == authored.rotation,
            "no tick {t} a pose autorada da peça andou para \
             [{:.9}, {:.9}, {:.9}] — o readback está escrevendo nela",
            live.translation.x,
            live.translation.y,
            live.rotation
        );
    }
    // E o corpo de fato SIMULOU (senão a estabilidade acima é vácuo).
    assert!(
        (world_y(&sim, owner) - 3.0).abs() > 0.5,
        "o dono não se moveu: a cena não exercita o que o gate afirma"
    );
}

/// **A MÃO pega uma peça pelo DONO** (W-PartFace, report do Enio: *"a interação
/// com o mouse na simulação não funcionou com a shape filha"*).
///
/// `grab_with` fazia `self.bodies.get(&entity)`, e uma peça não está lá — o gesto
/// era recusado em silêncio e o press caía adiante para o gizmo.
#[test]
fn the_hand_grabs_a_part_by_its_owner() {
    let mut sim = ell(false);
    let mut bridge = run(&mut sim, 60);
    let leg = named(&mut sim, "Leg");
    let arm = named(&mut sim, "Arm");
    assert!(
        bridge.grab(leg, [0.8, 4.0]),
        "clicar na PEÇA não pegou nada — a mão procurou um corpo que ela não tem"
    );
    bridge.release_grab();
    // O CONTROLE: pegar o corpo direto sempre funcionou, então o gate acima só
    // vale se este passar pelo mesmo caminho.
    assert!(
        bridge.grab(arm, [0.0, 5.0]),
        "o corpo deixou de ser pegável"
    );
    bridge.release_grab();
    // E a recusa honesta continua: um collider sem corpo acima não é peça de
    // ninguém, e inventar um dono seria pior que recusar.
    let mut lone = SimWorld::new();
    let e = lone
        .world_mut()
        .spawn((
            Name::new("Lone"),
            box_of(0.3, 0.3),
            Transform::from_translation(Vec2::new(0.0, 5.0)),
        ))
        .id();
    let mut b2 = run(&mut lone, 10);
    assert!(
        !b2.grab(e, [0.0, 5.0]),
        "uma forma solta foi pega por alguém"
    );
}

/// **Uma PEÇA quieta é pendurada UMA vez** (W-PartFace).
///
/// O gate de churn, e ele existe porque nenhum outro oráculo o enxerga: uma peça
/// despendurada e re-pendurada a cada frame **assenta igual** (o tremor medido
/// foi `0,0`), então nem pose nem aparência denunciam. O que muda é o HANDLE.
///
/// Duas causas independentes, medidas numa descida de rampa de 301 ticks:
///
/// - **133** re-pendurações porque `p.rest == desc` comparava o `BodyDesc`
///   INTEIRO, e os campos `x`/`y`/`rotation` dele são a pose de MUNDO da peça —
///   que muda a cada frame em que o dono se move, e que `attach_part` **nem lê**;
/// - **107** porque o `local` era derivado por round-trip pela pose que o SOLVER
///   escreveu (estável a 1,2e-7, e a comparação é exata).
///
/// Cada cura sozinha deixa a outra: 133 → 107 com só a primeira. Com as duas: 1.
///
/// ⚠️ Isto só ficou VISÍVEL quando o gate de repouso saiu — ele escondia o churn
/// ao proibir toda re-descrição durante o play, que era o defeito reportado.
#[test]
fn a_settled_part_is_attached_once_not_once_per_frame() {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        box_of(20.0, 0.5),
        Transform::from_translation(Vec2::new(0.0, -0.5)),
    ));
    // Uma RAMPA: um corpo caindo reto não gira, e a rotação é metade do churn.
    sim.world_mut().spawn((
        Name::new("Ramp"),
        RigidBody {
            kind: BodyKind::Static,
        },
        box_of(3.0, 0.2),
        Transform {
            translation: Vec2::new(0.0, 1.0),
            rotation: 0.35,
            ..Transform::IDENTITY
        },
    ));
    let owner = sim
        .world_mut()
        .spawn((
            Name::new("Owner"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            box_of(0.3, 0.3),
            Transform::from_translation(Vec2::new(-1.0, 3.0)),
        ))
        .id();
    let part = sim
        .world_mut()
        .spawn((
            Name::new("Part"),
            box_of(0.2, 0.2),
            Transform {
                translation: Vec2::new(0.55, 0.1),
                rotation: 0.4,
                ..Transform::IDENTITY
            },
            ChildOf(owner),
        ))
        .id();
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, true, 0);
    let first = bridge.part_handle(part).expect("a peça foi pendurada");
    for t in 1..=300u64 {
        bridge.dispatch(&mut sim, true, t);
        let now = bridge.part_handle(part).expect("a peça sumiu do solver");
        assert!(
            now == first,
            "no tick {t} a peça foi RE-PENDURADA (handle {first:?} -> {now:?}) — \
             ninguém a editou, então a comparação de re-descrição está \
             respondendo a algo que o consumidor não lê"
        );
    }
    // E o dono de fato desceu a rampa (senão a estabilidade acima é vácuo).
    assert!(
        (world_y(&sim, owner) - 3.0).abs() > 0.5,
        "o dono não se moveu: a cena não exercita o que o gate afirma"
    );
}
