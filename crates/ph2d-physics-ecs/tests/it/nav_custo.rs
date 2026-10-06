//! ⭐⭐ **A W7 de ponta a ponta** (plano 30): a lava que o inimigo evita SOZINHO (a decisão do dono,
//! §11.1), a lama que custa, a zona proibida e o teleporte — pela porta do produto
//! (`PhysicsBridge::dispatch`), cada um com o CONTROLO ao lado.
//!
//! ⚠️ A régua de «pisou a zona» é a DISTÂNCIA do centro do corpo ao rectângulo da zona contra o raio
//! dele (a lição da W5: a área recuada tem a quina REDONDA, e um teste de «dentro do rectângulo
//! alargado» acusava a quina).

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_nav::Status;
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, Damage, Health, NavAgent, NavCostArea, NavLink, NavRegion,
    NavTarget, PhysicsBridge, Resistance, RigidBody, TopDownPlayer,
};
use ph2d_topdown::{TopDownLaw, direction::DirectionMode};

const R: f32 = 0.3;

fn regiao(sim: &mut SimWorld) {
    sim.world_mut().spawn((
        Name::new("Região"),
        NavRegion {
            half_extents: [8.0, 6.0],
            obstacle_layers: u8::MAX,
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
}

/// Um corpo estático rectangular, sensor ou não.
fn caixa(sim: &mut SimWorld, nome: &str, c: (f32, f32), meio: (f32, f32), sensor: bool) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new(nome),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: meio.0,
                    half_y: meio.1,
                },
                is_sensor: sensor,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(c.0, c.1)),
        ))
        .id()
}

fn agente(sim: &mut SimWorld, em: (f32, f32), alvo: &str) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new("Inimigo"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: R },
                ..Collider::default()
            },
            TopDownPlayer::from_law(TopDownLaw {
                default_controls: false,
                direction: DirectionMode::Free,
                ..TopDownLaw::default()
            }),
            NavAgent {
                target: NavTarget::Named(stable_name_id(alvo)),
                arrive_distance: 0.1,
                ..NavAgent::default()
            },
            Transform::from_translation(Vec2::new(em.0, em.1)),
        ))
        .id()
}

fn marco(sim: &mut SimWorld, nome: &str, em: (f32, f32)) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new(nome),
            Transform::from_translation(Vec2::new(em.0, em.1)),
        ))
        .id()
}

fn pos(sim: &SimWorld, e: Entity) -> (f32, f32) {
    let t = sim.world().get::<Transform>(e).expect("o corpo");
    (t.translation.x, t.translation.y)
}

/// A distância do ponto ao rectângulo de centro `c` e meias-medidas `h` (`0` dentro).
fn ao_rect(p: (f32, f32), c: (f32, f32), h: (f32, f32)) -> f32 {
    let dx = ((p.0 - c.0).abs() - h.0).max(0.0);
    let dy = ((p.1 - c.1).abs() - h.1).max(0.0);
    (dx * dx + dy * dy).sqrt()
}

/// Corre até `ate` e devolve as posições do agente.
fn corre(sim: &mut SimWorld, b: &mut PhysicsBridge, quem: Entity, ate: u64) -> Vec<(f32, f32)> {
    (1..=ate)
        .map(|t| {
            b.dispatch(sim, true, t);
            pos(sim, quem)
        })
        .collect()
}

const LAVA_C: (f32, f32) = (0.0, 0.0);
const LAVA_H: (f32, f32) = (1.0, 3.0);

/// A cena da LAVA: uma faixa de lava (sensor, fere `fogo` por segundo) entre o inimigo e o alvo.
fn cena_lava(vida: Option<Health>, avoid_harm: bool) -> (SimWorld, PhysicsBridge, Entity) {
    let mut sim = SimWorld::new();
    regiao(&mut sim);
    let lava = caixa(&mut sim, "Lava", LAVA_C, LAVA_H, true);
    sim.world_mut().entity_mut(lava).insert(Damage {
        amount: 5.0,
        per_second: true,
        kind: "fogo".to_owned(),
        ..Damage::default()
    });
    marco(&mut sim, "Alvo", (5.0, 0.0));
    let quem = agente(&mut sim, (-5.0, 0.0), "Alvo");
    if let Some(v) = vida {
        sim.world_mut().entity_mut(quem).insert(v);
    }
    if let Some(mut a) = sim.world_mut().get_mut::<NavAgent>(quem) {
        a.avoid_harm = avoid_harm;
    }
    (sim, PhysicsBridge::new(), quem)
}

fn pisou(caminho: &[(f32, f32)], c: (f32, f32), h: (f32, f32)) -> bool {
    caminho.iter().any(|&p| ao_rect(p, c, h) < R * 0.5)
}

#[test]
fn o_inimigo_evita_a_lava_sozinho_e_os_quatro_controlos_atravessam() {
    let (mut sim, mut b, quem) = cena_lava(Some(Health::default()), true);
    let caminho = corre(&mut sim, &mut b, quem, 600);
    assert_eq!(b.nav_agent(quem).map(|r| r.status), Some(Status::Arrived));
    let mais_perto = caminho
        .iter()
        .map(|&p| ao_rect(p, LAVA_C, LAVA_H))
        .fold(f32::INFINITY, f32::min);
    assert!(
        mais_perto >= R - 0.02,
        "o corpo chegou a {mais_perto} m da lava (raio {R})"
    );
    // CONTROLO 1: a caixa desligada — atravessa.
    let (mut sim, mut b, quem) = cena_lava(Some(Health::default()), false);
    let caminho = corre(&mut sim, &mut b, quem, 600);
    assert!(
        pisou(&caminho, LAVA_C, LAVA_H),
        "com a caixa desligada devia atravessar"
    );
    // CONTROLO 2: imune ao fogo (a resistência `0` — a dobra da casa: `Fogo` = `fogo`).
    let imune = Health {
        resistances: vec![Resistance {
            kind: "Fogo".to_owned(),
            rate: 0.0,
            absorbs: false,
        }],
        ..Health::default()
    };
    let (mut sim, mut b, quem) = cena_lava(Some(imune), true);
    let caminho = corre(&mut sim, &mut b, quem, 600);
    assert!(
        pisou(&caminho, LAVA_C, LAVA_H),
        "o imune ao fogo devia atravessar"
    );
    // CONTROLO 3: sem vida nada o magoa.
    let (mut sim, mut b, quem) = cena_lava(None, true);
    let caminho = corre(&mut sim, &mut b, quem, 600);
    assert!(pisou(&caminho, LAVA_C, LAVA_H), "sem vida devia atravessar");
    // CONTROLO 4: o fogo CURA-o (a resistência que absorve) — não é uma zona que magoa.
    let cura = Health {
        resistances: vec![Resistance {
            kind: "fogo".to_owned(),
            rate: 1.0,
            absorbs: true,
        }],
        ..Health::default()
    };
    let (mut sim, mut b, quem) = cena_lava(Some(cura), true);
    let caminho = corre(&mut sim, &mut b, quem, 600);
    assert!(
        pisou(&caminho, LAVA_C, LAVA_H),
        "quem se cura com o fogo devia atravessar"
    );
}

#[test]
fn onde_duas_areas_se_sobrepoem_manda_a_mais_cara() {
    // Uma lama barata grande (1,05) com uma CARA (20) por dentro: a sobreposição é cara, e o agente
    // contorna o miolo. Com a barata a mandar, o miolo seria barato e ele ia a direito.
    let mut sim = SimWorld::new();
    regiao(&mut sim);
    let grande = caixa(&mut sim, "Lama larga", (0.0, 0.0), (3.0, 4.0), true);
    sim.world_mut().entity_mut(grande).insert(NavCostArea {
        cost: 1.05,
        forbidden: false,
    });
    let miolo = caixa(&mut sim, "Lama funda", (0.0, 0.0), (1.0, 1.5), true);
    sim.world_mut().entity_mut(miolo).insert(NavCostArea {
        cost: 20.0,
        forbidden: false,
    });
    marco(&mut sim, "Alvo", (5.0, 0.0));
    let quem = agente(&mut sim, (-5.0, 0.0), "Alvo");
    let mut b = PhysicsBridge::new();
    let caminho = corre(&mut sim, &mut b, quem, 600);
    assert_eq!(b.nav_agent(quem).map(|r| r.status), Some(Status::Arrived));
    assert!(
        !pisou(&caminho, (0.0, 0.0), (1.0, 1.5)),
        "atravessou o miolo caro — a barata mandou na sobreposição"
    );
}

/// A cena da LAMA (uma `NavCostArea` sensor no meio, a este custo).
fn cena_lama(area: NavCostArea) -> (SimWorld, PhysicsBridge, Entity) {
    let mut sim = SimWorld::new();
    regiao(&mut sim);
    let lama = caixa(&mut sim, "Lama", (0.0, 0.0), (1.5, 2.0), true);
    sim.world_mut().entity_mut(lama).insert(area);
    marco(&mut sim, "Alvo", (5.0, 0.0));
    let quem = agente(&mut sim, (-5.0, 0.0), "Alvo");
    (sim, PhysicsBridge::new(), quem)
}

#[test]
fn a_lama_cara_contorna_a_barata_atravessa_e_a_proibida_e_um_furo() {
    let (h, c) = ((1.5, 2.0), (0.0, 0.0));
    for (area, deve_pisar, nome) in [
        (
            NavCostArea {
                cost: 10.0,
                forbidden: false,
            },
            false,
            "a lama a 10",
        ),
        (
            NavCostArea {
                cost: 1.05,
                forbidden: false,
            },
            true,
            "a lama a 1,05 (CONTROLO)",
        ),
        (
            NavCostArea {
                cost: 1.05,
                forbidden: true,
            },
            false,
            "a proibida",
        ),
    ] {
        let (mut sim, mut b, quem) = cena_lama(area);
        let caminho = corre(&mut sim, &mut b, quem, 600);
        assert_eq!(
            b.nav_agent(quem).map(|r| r.status),
            Some(Status::Arrived),
            "{nome}"
        );
        assert_eq!(pisou(&caminho, c, h), deve_pisar, "{nome}");
    }
}

/// ⭐ (W18) **Mexer no `Cost` de uma lama com o agente a ANDAR refaz-lhe o caminho** — era um controlo
/// morto: só um custo que passava a `1` (a lama sai da malha) mudava alguma coisa, e o dono mudava `10`
/// para `2` no Inspector sem efeito nenhum (medido na cena `PH2D_NAV_SMOKE=5`: `0` procuras novas). A
/// lama a `10` (contorna); no tique `20`, já a contornar, passa a `1,05`: UMA procura nova, e atravessa.
/// O CONTROLO: a mesma corrida sem a mudança — nenhuma procura nova, e não pisa.
#[test]
fn mexer_no_custo_refaz_o_caminho_de_quem_anda() {
    let (h, c) = ((1.5, 2.0), (0.0, 0.0));
    let corre_com = |muda: bool| {
        let (mut sim, mut b, quem) = cena_lama(NavCostArea {
            cost: 10.0,
            forbidden: false,
        });
        let lama = sim
            .world_mut()
            .query::<(Entity, &NavCostArea)>()
            .iter(sim.world())
            .map(|(e, _)| e)
            .next()
            .expect("a lama");
        let mut caminho = corre(&mut sim, &mut b, quem, 20);
        let antes = b.nav_agent(quem).map_or(0, |r| r.searches);
        if muda && let Some(mut a) = sim.world_mut().get_mut::<NavCostArea>(lama) {
            a.cost = 1.05;
        }
        caminho.extend((21..=600).map(|t| {
            b.dispatch(&mut sim, true, t);
            pos(&sim, quem)
        }));
        let novas = b.nav_agent(quem).map_or(0, |r| r.searches) - antes;
        let chegou = b.nav_agent(quem).map(|r| r.status) == Some(Status::Arrived);
        (
            pisou(&caminho[20..], c, h),
            pisou(&caminho[..20], c, h),
            novas,
            chegou,
        )
    };
    let (depois, antes, novas, chegou) = corre_com(true);
    assert!(
        !antes,
        "a fixtura: a lama a 10 contorna-se antes da mudança"
    );
    assert!(
        depois && chegou,
        "o custo mudou e ele não atravessou ({novas} procuras novas)"
    );
    assert_eq!(novas, 1, "uma procura pela mudança, e nenhuma a mais");
    let (pisou_ctl, _, novas_ctl, _) = corre_com(false);
    assert!(
        !pisou_ctl && novas_ctl == 0,
        "o CONTROLO: {novas_ctl} procuras sem mudança"
    );
}

/// Duas salas separadas por uma parede maciça, um portal em cada uma.
fn cena_portal(com_atalho: bool) -> (SimWorld, PhysicsBridge, Entity, Entity) {
    let mut sim = SimWorld::new();
    regiao(&mut sim);
    caixa(&mut sim, "Parede", (0.0, 0.0), (0.5, 7.0), false);
    let a = marco(&mut sim, "Portal A", (-5.0, 4.0));
    marco(&mut sim, "Portal B", (5.0, 4.0));
    if com_atalho {
        sim.world_mut().entity_mut(a).insert(NavLink {
            to: stable_name_id("Portal B"),
            teleport: true,
            on_crossed: " atravessou ".to_owned(),
            ..NavLink::default()
        });
    }
    marco(&mut sim, "Alvo", (5.0, -3.0));
    let quem = agente(&mut sim, (-5.0, -3.0), "Alvo");
    (sim, PhysicsBridge::new(), quem, a)
}

#[test]
fn o_teleporte_leva_o_corpo_e_diz_que_atravessou_uma_vez() {
    let (mut sim, mut b, quem, portal) = cena_portal(true);
    let tree = ph2d_tags::TagTree::default();
    let mut ouvidos = Vec::new();
    let mut antes = pos(&sim, quem);
    let mut salto = 0.0f32;
    for t in 1..=600 {
        b.dispatch(&mut sim, true, t);
        let agora = pos(&sim, quem);
        salto = salto.max(((agora.0 - antes.0).powi(2) + (agora.1 - antes.1).powi(2)).sqrt());
        antes = agora;
        ouvidos.extend(
            b.signal_events(&sim, &tree)
                .into_iter()
                .filter(|s| s.name == "atravessou")
                .map(|s| (s.source, s.other)),
        );
    }
    assert_eq!(b.nav_agent(quem).map(|r| r.status), Some(Status::Arrived));
    assert!(
        salto > 5.0,
        "o corpo devia SALTAR de portal a portal (maior passo {salto} m)"
    );
    assert_eq!(
        ouvidos,
        vec![(portal, quem)],
        "o sinal: do portal, com o agente"
    );
    // CONTROLO: sem o atalho, a outra sala é inalcançável.
    let (mut sim, mut b, quem, _) = cena_portal(false);
    corre(&mut sim, &mut b, quem, 600);
    assert_eq!(
        b.nav_agent(quem).map(|r| r.status),
        Some(Status::MovingPartial)
    );
}

#[test]
fn um_scrub_depois_do_salto_devolve_a_mesma_corrida() {
    let (mut sim, mut b, quem, _) = cena_portal(true);
    let primeira = corre(&mut sim, &mut b, quem, 400);
    // O tique do salto (o maior passo).
    let salto = (1..primeira.len())
        .max_by(|&i, &j| {
            let d = |k: usize| {
                (primeira[k].0 - primeira[k - 1].0).abs()
                    + (primeira[k].1 - primeira[k - 1].1).abs()
            };
            d(i).total_cmp(&d(j))
        })
        .expect("passos");
    let meio = (salto + 20) as u64;
    b.dispatch(&mut sim, false, meio);
    assert_eq!(pos(&sim, quem), primeira[(meio - 1) as usize], "o scrub");
    let resto: Vec<(f32, f32)> = ((meio + 1)..=400)
        .map(|t| {
            b.dispatch(&mut sim, true, t);
            pos(&sim, quem)
        })
        .collect();
    assert_eq!(
        resto,
        primeira[meio as usize..].to_vec(),
        "o resto da corrida"
    );
}
