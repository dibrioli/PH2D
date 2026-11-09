//! **A forma FILHA simula igual à forma no corpo** (GEF02 B1, opção B 08/11).
//!
//! A mesma cena construída duas vezes: (A) cada `Collider` e o seu lado-forma no próprio corpo, (B)
//! movidos para uma filha «Collision Shape» com `Transform` identidade. Os corpos nascem primeiro nas
//! duas, então têm os mesmos bits de entidade e o hash compara-se direto.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    AreaEffector, BodyKind, Collider, ColliderShape, CombineRule, LockRotation, MaterialCombine,
    OneWayPlatform, PhysicsBridge, PlatformPlayer, PlayerInput, RigidBody, SignalOnHit,
};

fn caixa(half_x: f32, half_y: f32) -> Collider {
    Collider {
        shape: ColliderShape::Cuboid { half_x, half_y },
        density: 1.0,
        ..Collider::default()
    }
}

fn rb(kind: BodyKind) -> RigidBody {
    RigidBody { kind }
}

pub(super) struct Cena {
    pub(super) sim: SimWorld,
    pub(super) corpos: Vec<Entity>,
    pub(super) formas: Vec<Entity>,
    player: Entity,
}

/// Põe a forma (e o lado-forma) no corpo ou numa filha, conforme `filha`.
fn forma(
    sim: &mut SimWorld,
    corpo: Entity,
    filha: bool,
    f: impl FnOnce(&mut bevy_ecs::world::EntityWorldMut),
) -> Entity {
    let alvo = if filha {
        sim.world_mut()
            .spawn((
                Name::new("Collision Shape"),
                Transform::IDENTITY,
                ChildOf(corpo),
            ))
            .id()
    } else {
        corpo
    };
    f(&mut sim.world_mut().entity_mut(alvo));
    alvo
}

pub(super) fn cena(filha: bool) -> Cena {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let em = |x: f32, y: f32| Transform::from_translation(Vec2::new(x, y));
    let chao = w
        .spawn((Name::new("Floor"), rb(BodyKind::Static), em(0.0, -0.5)))
        .id();
    let borda = w
        .spawn((Name::new("Ledge"), rb(BodyKind::Static), em(3.0, 1.5)))
        .id();
    let bloco = w
        .spawn((Name::new("Box"), rb(BodyKind::Dynamic), em(3.0, 4.0)))
        .id();
    let zona = w
        .spawn((Name::new("Zone"), rb(BodyKind::Static), em(-3.0, 1.0)))
        .id();
    let bola = w
        .spawn((Name::new("Ball"), rb(BodyKind::Dynamic), em(-3.0, 3.5)))
        .id();
    let player = w
        .spawn((
            Name::new("Player"),
            rb(BodyKind::Dynamic),
            LockRotation,
            PlatformPlayer {
                float_height: 1.1,
                crouch_height: 0.6,
                crouch_speed: 2.0,
                ..PlatformPlayer::default()
            },
            em(0.0, 1.1),
        ))
        .id();
    let material = MaterialCombine {
        restitution: CombineRule::Max,
        friction: CombineRule::Min,
    };
    let formas = vec![
        forma(&mut sim, chao, filha, |e| {
            e.insert((caixa(30.0, 0.5), material));
        }),
        forma(&mut sim, borda, filha, |e| {
            e.insert((caixa(2.0, 0.1), OneWayPlatform));
        }),
        forma(&mut sim, bloco, filha, |e| {
            e.insert((
                Collider {
                    restitution: 0.6,
                    ..caixa(0.3, 0.3)
                },
                material,
            ));
        }),
        forma(&mut sim, zona, filha, |e| {
            e.insert((
                Collider {
                    is_sensor: true,
                    ..caixa(1.0, 1.0)
                },
                AreaEffector { force: [0.0, 6.0] },
                SignalOnHit("hit".into()),
            ));
        }),
        forma(&mut sim, bola, filha, |e| {
            e.insert(Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                restitution: 0.4,
                ..Collider::default()
            });
        }),
        forma(&mut sim, player, filha, |e| {
            e.insert(Collider {
                shape: ColliderShape::Capsule {
                    half_height: 0.3,
                    radius: 0.2,
                },
                ..Collider::default()
            });
        }),
    ];
    Cena {
        sim,
        corpos: vec![chao, borda, bloco, zona, bola, player],
        formas,
        player,
    }
}

fn hash_bits(h: &mut u64, v: u64) {
    *h = (*h ^ v).wrapping_mul(0x0100_0000_01b3);
}

/// 300 tiques: o player anda à direita, agachado a meio. Hash das poses (bits), contactos (pares,
/// ponto, normal) e gatilhos (sensor levado ao seu CORPO). Devolve também os sensores vistos.
pub(super) fn corre(c: &mut Cena) -> (u64, Vec<Entity>) {
    let mut bridge = PhysicsBridge::new();
    let mut h = 0xcbf2_9ce4_8422_2325_u64;
    let mut sensores = Vec::new();
    for t in 1..=300_u64 {
        let input = PlayerInput {
            drive: 1.0,
            down: (100..200).contains(&t),
            ..PlayerInput::default()
        };
        bridge.set_player_input(c.player, input);
        bridge.dispatch(&mut c.sim, true, t);
        for &e in &c.corpos {
            let tr = c.sim.world().get::<Transform>(e).copied().unwrap();
            for v in [tr.translation.x, tr.translation.y, tr.rotation] {
                hash_bits(&mut h, u64::from(v.to_bits()));
            }
        }
        for k in bridge.contacts() {
            hash_bits(&mut h, k.a.to_bits() ^ k.b.to_bits().rotate_left(17));
            for v in [k.point[0], k.point[1], k.normal[0], k.normal[1]] {
                hash_bits(&mut h, u64::from(v.to_bits()));
            }
        }
        for ev in bridge.trigger_events() {
            sensores.push(ev.sensor);
            let corpo = c
                .sim
                .world()
                .get::<ChildOf>(ev.sensor)
                .map_or(ev.sensor, ChildOf::parent);
            hash_bits(&mut h, corpo.to_bits() ^ ev.other.to_bits().rotate_left(29));
        }
    }
    (h, sensores)
}

#[test]
fn a_forma_filha_simula_igual_a_forma_no_corpo() {
    let mut a = cena(false);
    let mut b = cena(true);
    assert_eq!(a.corpos, b.corpos, "os corpos nascem primeiro: mesmos bits");
    let (ha, sa) = corre(&mut a);
    let (hb, sb) = corre(&mut b);
    assert!(
        !sa.is_empty(),
        "a bola tem de entrar na zona (senão o gatilho não é medido)"
    );
    assert_eq!(
        ha, hb,
        "a forma filha tem de simular byte a byte como a forma no corpo"
    );
    // ⭐ O gatilho reporta a FORMA: o corpo em (A), a filha em (B).
    assert!(sa.iter().all(|&s| s == a.corpos[3]));
    assert!(
        sb.iter().all(|&s| s == b.formas[3]),
        "o sensor filho reporta a FILHA"
    );
    // O player agachou (o Crouch encolheu a cápsula filha como encolhe a do corpo).
    let y = |c: &Cena| {
        c.sim
            .world()
            .get::<Transform>(c.player)
            .unwrap()
            .translation
            .y
    };
    assert_eq!(y(&a).to_bits(), y(&b).to_bits());
}

/// Uma bola cai sobre uma rampa estática. (A) o corpo na pose de mundo da rampa, forma no corpo; (B)
/// o corpo na origem e a forma FILHA deslocada `0,5` e rodada `0,3`. Mesma trajectória ⇒ a pose da
/// filha chegou ao collider; e uma terceira, com a filha sem pose, cai noutro sítio.
#[test]
fn mover_a_forma_filha_move_o_collider() {
    fn corre_rampa(corpo: Transform, filha: Option<Transform>) -> (f32, f32) {
        let mut sim = SimWorld::new();
        let w = sim.world_mut();
        let rampa = w
            .spawn((Name::new("Ramp"), rb(BodyKind::Static), corpo))
            .id();
        let bola = w
            .spawn((
                Name::new("Ball"),
                rb(BodyKind::Dynamic),
                Collider {
                    shape: ColliderShape::Ball { radius: 0.2 },
                    ..Collider::default()
                },
                Transform::from_translation(Vec2::new(0.9, 3.0)),
            ))
            .id();
        match filha {
            None => {
                w.entity_mut(rampa).insert(caixa(1.0, 0.1));
            }
            Some(t) => {
                w.spawn((caixa(1.0, 0.1), t, ChildOf(rampa)));
            }
        }
        let mut bridge = PhysicsBridge::new();
        for t in 1..=120 {
            bridge.dispatch(&mut sim, true, t);
        }
        let p = sim.world().get::<Transform>(bola).unwrap().translation;
        (p.x, p.y)
    }
    let mut pose = Transform::from_translation(Vec2::new(0.5, 0.0));
    pose.rotation = 0.3;
    let a = corre_rampa(pose, None);
    let b = corre_rampa(Transform::IDENTITY, Some(pose));
    let parado = corre_rampa(Transform::IDENTITY, Some(Transform::IDENTITY));
    assert!(
        (a.0 - b.0).abs() < 1e-4 && (a.1 - b.1).abs() < 1e-4,
        "{a:?} vs {b:?}"
    );
    assert!(
        (a.0 - parado.0).abs() > 0.05,
        "a pose da filha tem de mudar onde a bola rola"
    );
}

/// ⭐ **O sinal de um sensor FILHO grita pelo CORPO** (opção B, B3a): o «só se fui eu» de uma moeda
/// mora no corpo, e a forma filha leva o `SignalOnHit`. O gatilho continua a nomear a forma; o
/// `SignalEvent.source` é o corpo nas duas construções (a migrada da B2 incluída).
#[test]
fn o_sinal_do_sensor_filho_tem_o_corpo_por_fonte() {
    for filha in [false, true] {
        let mut c = cena(filha);
        let mut bridge = PhysicsBridge::new();
        let mut fontes = Vec::new();
        for t in 1..=300_u64 {
            let input = PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            };
            bridge.set_player_input(c.player, input);
            bridge.dispatch(&mut c.sim, true, t);
            for s in bridge.signal_events(&c.sim, &ph2d_tags::TagTree::new()) {
                if s.name == "hit" {
                    fontes.push(s.source);
                }
            }
        }
        assert!(!fontes.is_empty(), "filha={filha}: a zona não gritou");
        assert!(
            fontes.iter().all(|&s| s == c.corpos[3]),
            "filha={filha}: a fonte não é o corpo da zona: {fontes:?} (forma {:?})",
            c.formas[3]
        );
    }
}

/// Dono 09/11: cada forma nova nasce com nome único (a lei do Duplicate, `ph2d-unique-name`), em qualquer corpo.
#[test]
fn as_formas_novas_nascem_com_nomes_unicos() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let a = w.spawn(rb(BodyKind::Static)).id();
    let b = w.spawn(rb(BodyKind::Dynamic)).id();
    let bola = ColliderShape::Ball { radius: 0.5 };
    let formas = [a, b, a].map(|c| ph2d_physics_ecs::nasce_com_forma(w, c, bola));
    let nomes: Vec<String> = formas
        .iter()
        .map(|&f| w.get::<Name>(f).map(|n| n.0.clone()).unwrap_or_default())
        .collect();
    assert_eq!(
        nomes,
        [
            "Collision Shape",
            "Collision Shape (1)",
            "Collision Shape (2)"
        ]
    );
}

/// Smoke 09/11 (forma neta): uma porta que pendura numa entidade NASCIDA RAIZ (`RootOrder`) deixa-a filha só com
/// ordem de irmã — `RootOrder` só vale numa raiz — e com o seu lugar escrito como dado (`SiblingOrder`) no MESMO
/// gesto; o *Make Independent Body* de uma forma solta na raiz põe o corpo no lugar dela entre as raízes.
#[test]
fn quem_as_portas_penduram_fica_filha_sem_ordem_de_raiz() {
    use ph2d_ecs::{RootOrder, SiblingOrder};
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let corpo = w.spawn((rb(BodyKind::Static), RootOrder(0))).id();
    let s1 = ph2d_physics_ecs::nasce_com_forma(w, corpo, ColliderShape::Ball { radius: 0.5 });
    let raiz =
        |w: &mut bevy_ecs::world::World, o| w.spawn((Transform::IDENTITY, RootOrder(o))).id();
    let (r1, r2) = (raiz(w, 1), raiz(w, 2));
    let neta = ph2d_physics_ecs::pendura_forma(w, r1, Some(s1));
    let irma = ph2d_physics_ecs::pendura_forma(w, r2, Some(corpo));
    let ordem = |w: &bevy_ecs::world::World, e| {
        (
            w.get::<RootOrder>(e).copied(),
            w.get::<SiblingOrder>(e).copied(),
        )
    };
    assert_eq!(
        ordem(w, neta),
        (None, Some(SiblingOrder(0))),
        "a neta pendurada na forma"
    );
    assert_eq!(
        ordem(w, s1),
        (None, Some(SiblingOrder(0))),
        "a forma principal fica a primeira"
    );
    assert_eq!(
        ordem(w, irma),
        (None, Some(SiblingOrder(1))),
        "a forma nova vai para o fim das irmãs"
    );

    let solta = w.spawn((caixa(0.5, 0.5), RootOrder(3))).id();
    let fim = w.spawn((Transform::IDENTITY, RootOrder(4))).id();
    let novo = ph2d_physics_ecs::torna_corpo(w, solta, BodyKind::Dynamic, [0.5, 0.5], false);
    assert_eq!(w.get::<ChildOf>(solta).map(ChildOf::parent), Some(novo));
    assert_eq!(
        ordem(w, solta),
        (None, Some(SiblingOrder(0))),
        "a forma solta dentro do corpo novo"
    );
    assert_eq!(
        ordem(w, novo),
        (Some(RootOrder(3)), None),
        "o corpo novo no lugar da forma entre as raízes"
    );
    assert_eq!(ordem(w, fim).0, Some(RootOrder(4)));
}
