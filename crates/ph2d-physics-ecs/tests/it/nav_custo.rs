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

/// ⭐ (W18) **Uma área MAIS BARATA que o chão anda-se por DENTRO** — o corpo inteiro nela (a lei: o custo de
/// uma posição é o MAIOR debaixo do corpo; `ph2d_navmesh::Area::dentro`). Uma estrada em ∩ a `Cost 0.3`
/// por cima do alvo: o agente sobe por ela e desce, com o corpo DENTRO de uma das três tiras. Medido antes
/// da lei: a área recuava para FORA e o caminho seguia a berma por fora (`0,25 m` dela, a cena `=6`). O
/// CONTROLO: a mesma estrada a `Cost 1` — vai a direito, por baixo dela.
#[test]
fn uma_area_mais_barata_anda_se_por_dentro() {
    const TIRAS: [((f32, f32), (f32, f32)); 3] = [
        ((-5.0, 2.3), (0.4, 1.7)),
        ((0.0, 3.6), (5.4, 0.4)),
        ((5.0, 2.3), (0.4, 1.7)),
    ];
    // Quanto o centro está DENTRO da tira (negativo = fora).
    let fundo = |p: (f32, f32), (c, h): ((f32, f32), (f32, f32))| {
        (h.0 - (p.0 - c.0).abs()).min(h.1 - (p.1 - c.1).abs())
    };
    let corre_com = |custo: f32| {
        let mut sim = SimWorld::new();
        regiao(&mut sim);
        for (i, &(c, h)) in TIRAS.iter().enumerate() {
            let e = caixa(&mut sim, &format!("Estrada {i}"), c, h, true);
            sim.world_mut().entity_mut(e).insert(NavCostArea {
                cost: custo,
                forbidden: false,
            });
        }
        marco(&mut sim, "Alvo", (5.0, 0.0));
        let quem = agente(&mut sim, (-5.0, 0.0), "Alvo");
        let mut b = PhysicsBridge::new();
        let caminho = corre(&mut sim, &mut b, quem, 900);
        let chegou = b.nav_agent(quem).map(|r| r.status) == Some(Status::Arrived);
        let por_cima = caminho
            .iter()
            .filter(|&&p| TIRAS.iter().any(|&t| fundo(p, t) >= R - 0.02))
            .count();
        let alto = caminho.iter().map(|p| p.1).fold(f32::MIN, f32::max);
        (por_cima, alto, chegou)
    };
    let (por_cima, alto, chegou) = corre_com(0.3);
    let (por_cima_ctl, alto_ctl, _) = corre_com(1.0);
    eprintln!(
        "Cost 0.3: {por_cima} tiques com o corpo inteiro na estrada, subiu a {alto:.2} · CONTROLO: {por_cima_ctl}, {alto_ctl:.2}"
    );
    assert!(chegou, "não chegou ao alvo");
    assert!(
        alto > 3.0,
        "a fixtura: o caminho barato é pela estrada de cima ({alto})"
    );
    assert!(
        por_cima >= 60,
        "o corpo não andou POR CIMA da estrada ({por_cima} tiques)"
    );
    assert!(
        por_cima_ctl == 0 && alto_ctl < 0.5,
        "o CONTROLO subiu à estrada ({por_cima_ctl} tiques, {alto_ctl})"
    );
}

/// ⭐ (W18) **Um custo que atravessa o `1` com o agente a andar troca o recuo da área** — de DENTRO (barata:
/// o corpo inteiro nela) para FORA (cara: o corpo paga-a quando lhe toca), e o mosaico refaz-se (a
/// assinatura dele leva `Area::dentro`). A lama a `0.3` atravessa-se a direito; no tique `20`, ainda antes
/// dela, passa a `10`: ele contorna-a sem lhe TOCAR com o corpo. Sem a malha refeita, o custo `10` ficava na
/// área encolhida e o corpo passava pela margem dela.
#[test]
fn um_custo_que_atravessa_o_1_troca_o_recuo_da_area() {
    let (c, h) = ((0.0, 0.0), (1.5, 2.0));
    let corre_com = |muda: bool| {
        let (mut sim, mut b, quem) = cena_lama(NavCostArea {
            cost: 0.3,
            forbidden: false,
        });
        let lama = sim
            .world_mut()
            .query::<(Entity, &NavCostArea)>()
            .iter(sim.world())
            .map(|(e, _)| e)
            .next()
            .expect("a lama");
        let antes = corre(&mut sim, &mut b, quem, 20);
        if muda && let Some(mut a) = sim.world_mut().get_mut::<NavCostArea>(lama) {
            a.cost = 10.0;
        }
        let depois: Vec<(f32, f32)> = (21..=600)
            .map(|t| {
                b.dispatch(&mut sim, true, t);
                pos(&sim, quem)
            })
            .collect();
        let chegou = b.nav_agent(quem).map(|r| r.status) == Some(Status::Arrived);
        let mais_perto = depois
            .iter()
            .map(|&p| ao_rect(p, c, h))
            .fold(f32::MAX, f32::min);
        (pisou(&antes, c, h), mais_perto, chegou)
    };
    let (antes, mais_perto, chegou) = corre_com(true);
    assert!(!antes, "a fixtura: no tique 20 ainda não chegou à lama");
    assert!(chegou, "não chegou ao alvo");
    assert!(
        mais_perto >= R - 0.02,
        "o corpo tocou a lama a 10 (o centro a {mais_perto} dela; o raio é {R})"
    );
    let (_, mais_perto_ctl, _) = corre_com(false);
    assert_eq!(mais_perto_ctl, 0.0, "o CONTROLO (a 0.3) não a atravessou");
}

/// ⭐ (W19, plano 30 §28.2) **Uma área barata LONGE não desliga o «alvo à vista»** — um perseguidor em
/// campo aberto atrás de um alvo que anda, uma estrada a `Cost 0.3` a `5 m` do caminho. Medido antes (a
/// tabela com um custo `< 1` desligava a recta no mundo inteiro): o trabalho de procura `11 → 112`. Com a
/// cota (`ph2d_nav::cota`): o mesmo que sem a estrada (a recta, sem procura). O CONTROLO: a mesma estrada
/// paralela ao caminho a `0,9 m` dele — ir até ela e voltar compensa, a recta não é a resposta, a procura
/// corre e ele anda por ela.
#[test]
fn uma_area_barata_longe_nao_desliga_o_alvo_a_vista() {
    let corre_com = |estrada: Option<((f32, f32), (f32, f32))>| {
        let mut sim = SimWorld::new();
        regiao(&mut sim);
        if let Some((c, h)) = estrada {
            let e = caixa(&mut sim, "Estrada", c, h, true);
            sim.world_mut().entity_mut(e).insert(NavCostArea {
                cost: 0.3,
                forbidden: false,
            });
        }
        let alvo = marco(&mut sim, "Alvo", (-2.0, -4.0));
        let quem = agente(&mut sim, (-7.0, -4.0), "Alvo");
        let mut b = PhysicsBridge::new();
        let (mut trabalho, mut na_estrada) = (0u64, 0usize);
        for t in 1..=420u64 {
            if t % 30 == 0
                && let Some(mut tr) = sim.world_mut().get_mut::<Transform>(alvo)
            {
                tr.translation.x += 0.5;
            }
            b.dispatch(&mut sim, true, t);
            trabalho += b.nav_search_work();
            let p = pos(&sim, quem);
            na_estrada += usize::from(estrada.is_some_and(|(c, h)| ao_rect(p, c, h) == 0.0));
        }
        (
            trabalho,
            na_estrada,
            b.nav_agent(quem).map_or(0, |r| r.searches),
        )
    };
    let (sem, _, procuras) = corre_com(None);
    let (longe, _, procuras_longe) = corre_com(Some(((1.0, 1.6), (4.0, 0.6))));
    let (ctl, na_estrada_ctl, _) = corre_com(Some(((0.0, -2.8), (6.0, 0.6))));
    eprintln!(
        "trabalho de procura: sem a estrada {sem} ({procuras} procuras) · longe {longe} ({procuras_longe}) · CONTROLO paralela {ctl}, {na_estrada_ctl} tiques nela"
    );
    assert!(
        procuras >= 8,
        "a fixtura: o alvo anda e ele replaneia ({procuras})"
    );
    assert!(
        longe as f64 <= 1.1 * sem as f64,
        "a estrada longe pesou na procura: {sem} → {longe}"
    );
    // Medido: `0 · 0 · 194`, `78` tiques com o centro na estrada paralela.
    assert!(
        ctl > 2 * sem.max(1) && na_estrada_ctl >= 40,
        "o CONTROLO: a estrada no caminho pede a procura e anda-se ({ctl}, {na_estrada_ctl} tiques)"
    );
}

/// ⭐ (W19, plano 30 §28.3) **A ponte diz que área BARATA é mais estreita que o corpo** (`NavCostAreaNow`)
/// — pela mesma erosão que faz a malha. Uma estrada de `0,4 m` para um corpo de raio `0,3`: some da malha
/// dele, e o mundo leva o raio. Os CONTROLOS: a mesma estrada LARGA (`1,2 m`) e a mesma estreita mas CARA
/// (recua para fora: vale) não levam nada; e sem agente nenhum, nada.
#[test]
fn a_ponte_publica_a_area_barata_mais_estreita_que_o_corpo() {
    let corre_com = |meia_largura: f32, custo: f32, com_agente: bool| {
        let mut sim = SimWorld::new();
        regiao(&mut sim);
        let e = caixa(&mut sim, "Estrada", (0.0, 2.0), (4.0, meia_largura), true);
        sim.world_mut().entity_mut(e).insert(NavCostArea {
            cost: custo,
            forbidden: false,
        });
        marco(&mut sim, "Alvo", (5.0, -3.0));
        if com_agente {
            agente(&mut sim, (-5.0, -3.0), "Alvo");
        }
        let mut b = PhysicsBridge::new();
        corre(&mut sim, &mut b, e, 3);
        sim.world()
            .get::<ph2d_physics_ecs::NavCostAreaNow>(e)
            .map(|n| n.too_narrow_for)
    };
    // O raio da malha, na grelha da chave dela (`1/256 m`, para CIMA): `0,3 → 77/256`.
    let raio_da_malha = (R * 256.0).ceil() / 256.0;
    assert_eq!(
        corre_com(0.2, 0.3, true),
        Some(raio_da_malha),
        "a estreita barata leva o raio do corpo"
    );
    assert_eq!(corre_com(0.6, 0.3, true), None, "o CONTROLO: larga, cabe");
    assert_eq!(
        corre_com(0.2, 4.0, true),
        None,
        "o CONTROLO: estreita mas cara, vale"
    );
    assert_eq!(corre_com(0.2, 0.3, false), None, "o CONTROLO: ninguém anda");
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
