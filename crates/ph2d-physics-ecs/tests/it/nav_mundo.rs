//! ⭐⭐⭐ **O MUNDO QUE MUDA** (plano 30, W6) — a costura da malha por mosaicos com a ponte.
//!
//! O que só a ponte pode partir: quem é obstáculo NESTE tique (o cinemático parado sim, o que anda
//! não, o personagem nunca), e quem esquece o caminho quando a malha muda (só os agentes dela).
//! As leis da malha (por mosaicos = inteira, incremental = a frio) têm gates na `ph2d-navmesh`.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_nav::Status;
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, NavAgent, NavRegion, NavTarget, PhysicsBridge, RigidBody,
    TopDownPlayer,
};
use ph2d_topdown::{TopDownLaw, direction::DirectionMode};

fn corpo(
    sim: &mut SimWorld,
    nome: &str,
    kind: BodyKind,
    em: (f32, f32),
    meio: (f32, f32),
) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new(nome),
            RigidBody { kind },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: meio.0,
                    half_y: meio.1,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(em.0, em.1)),
        ))
        .id()
}

fn regiao(sim: &mut SimWorld, centro: (f32, f32)) {
    sim.world_mut().spawn((
        Name::new("Região"),
        NavRegion {
            half_extents: [8.0, 6.0],
            obstacle_layers: u8::MAX,
        },
        Transform::from_translation(Vec2::new(centro.0, centro.1)),
    ));
}

fn mover(default_controls: bool) -> TopDownPlayer {
    TopDownPlayer::from_law(TopDownLaw {
        default_controls,
        direction: DirectionMode::Free,
        ..TopDownLaw::default()
    })
}

fn agente(sim: &mut SimWorld, nome: &str, em: (f32, f32), alvo: NavTarget) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new(nome),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                ..Collider::default()
            },
            mover(false),
            NavAgent {
                target: alvo,
                ..NavAgent::default()
            },
            Transform::from_translation(Vec2::new(em.0, em.1)),
        ))
        .id()
}

fn poe(sim: &mut SimWorld, e: Entity, em: (f32, f32)) {
    sim.world_mut()
        .get_mut::<Transform>(e)
        .expect("o corpo")
        .translation = Vec2::new(em.0, em.1);
}

/// A área andável de todas as malhas.
fn area(b: &PhysicsBridge) -> f64 {
    b.nav_meshes().map(|(_, _, m)| m.area()).sum()
}

/// O caminho do agente CRUZA a recta `x = 0` dentro do vão (`|y| < 1`)?
fn pela_porta(b: &PhysicsBridge, e: Entity) -> bool {
    b.nav_agent(e).is_some_and(|rt| {
        rt.path.windows(2).any(|w| {
            let (a, c) = (w[0], w[1]);
            if (a[0] < 0.0) == (c[0] < 0.0) {
                return false;
            }
            let y = a[1] + (c[1] - a[1]) * (0.0 - a[0]) / (c[0] - a[0]);
            y.abs() < 1.0
        })
    })
}

/// Duas salas e uma parede em `x = 0` com um vão de 2 m no meio; a porta (cinemática) começa
/// escondida dentro da parede de cima.
fn duas_salas() -> (SimWorld, PhysicsBridge, Entity, Entity) {
    let mut sim = SimWorld::new();
    regiao(&mut sim, (0.0, 0.0));
    corpo(
        &mut sim,
        "Parede de cima",
        BodyKind::Static,
        (0.0, 3.5),
        (0.3, 2.5),
    );
    corpo(
        &mut sim,
        "Parede de baixo",
        BodyKind::Static,
        (0.0, -3.5),
        (0.3, 2.5),
    );
    let porta = corpo(
        &mut sim,
        "Porta",
        BodyKind::Kinematic,
        (0.0, 4.0),
        (0.3, 1.2),
    );
    let quem = agente(
        &mut sim,
        "Guarda",
        (-5.0, 0.0),
        NavTarget::Point([5.0, 0.0]),
    );
    (sim, PhysicsBridge::new(), porta, quem)
}

#[test]
fn a_porta_que_desliza_e_para_fecha_o_caminho_e_abrir_devolve_o() {
    let (mut sim, mut b, porta, quem) = duas_salas();
    for t in 1..=10 {
        b.dispatch(&mut sim, true, t);
    }
    let aberta = area(&b);
    assert!(
        pela_porta(&b, quem),
        "com a porta aberta o caminho é pelo vão"
    );
    assert_eq!(b.nav_agent(quem).map(|r| r.status), Some(Status::Moving));

    poe(&mut sim, porta, (0.0, 0.0));
    for t in 11..=14 {
        b.dispatch(&mut sim, true, t);
    }
    assert!(
        area(&b) < aberta - 1.0,
        "a porta parada no vão não recortou a malha ({} contra {aberta})",
        area(&b)
    );
    assert!(
        !pela_porta(&b, quem),
        "o caminho ainda atravessa a porta fechada"
    );
    assert_eq!(
        b.nav_agent(quem).map(|r| r.status),
        Some(Status::MovingPartial),
        "com a porta fechada o outro lado é outra ilha"
    );

    poe(&mut sim, porta, (0.0, 4.0));
    for t in 15..=18 {
        b.dispatch(&mut sim, true, t);
    }
    assert_eq!(area(&b), aberta, "reaberta, a malha é a de antes ao bit");
    assert!(pela_porta(&b, quem), "reaberta, o caminho volta ao vão");
}

/// ⚠️ **Parado é a velocidade LINEAR e a ANGULAR a zero**: uma porta que RODA no sítio (um
/// torniquete) anda, e não recorta. O CONTROLO é a mesma porta quando pára de rodar.
#[test]
fn uma_porta_a_rodar_no_sitio_nao_recorta() {
    let (mut sim, mut b, porta, _) = duas_salas();
    poe(&mut sim, porta, (0.0, 0.0));
    for t in 1..=6 {
        b.dispatch(&mut sim, true, t);
    }
    let parada = area(&b);
    // Sem a porta (escondida na parede) a área é maior: ela recorta o vão.
    poe(&mut sim, porta, (0.0, 4.0));
    for t in 7..=12 {
        b.dispatch(&mut sim, true, t);
    }
    let aberta = area(&b);
    assert!(parada < aberta - 1.0, "o CONTROLO: parada no vão, recorta");
    poe(&mut sim, porta, (0.0, 0.0));
    for t in 13..=18 {
        b.dispatch(&mut sim, true, t);
    }
    // Agora roda no sítio, um pouco a cada tique.
    for t in 19..=40u64 {
        sim.world_mut()
            .get_mut::<Transform>(porta)
            .expect("a porta")
            .rotation = 0.05 * (t - 18) as f32;
        b.dispatch(&mut sim, true, t);
        if t > 20 {
            assert_eq!(
                area(&b),
                aberta,
                "tique {t}: a porta a RODAR recortou a malha"
            );
        }
    }
}

#[test]
fn uma_porta_a_andar_nao_recorta() {
    let (mut sim, mut b, porta, _) = duas_salas();
    for t in 1..=5 {
        b.dispatch(&mut sim, true, t);
    }
    let aberta = area(&b);
    // A porta desce até ao meio do vão a andar SEM parar: nunca é parede (é o desvio que a evita).
    for t in 6..=35u64 {
        let y = 3.0 - 0.1 * (t - 5) as f32;
        poe(&mut sim, porta, (0.0, y));
        b.dispatch(&mut sim, true, t);
        assert_eq!(
            area(&b),
            aberta,
            "tique {t}: a porta a andar recortou a malha"
        );
    }
    // O CONTROLO: parada a meio do vão, recorta.
    for t in 36..=39 {
        b.dispatch(&mut sim, true, t);
    }
    assert!(area(&b) < aberta - 1.0, "o controlo (parada) não recortou");
}

#[test]
fn um_personagem_parado_nao_vira_parede() {
    let mut sim = SimWorld::new();
    regiao(&mut sim, (0.0, 0.0));
    let heroi = corpo(
        &mut sim,
        "Herói",
        BodyKind::Kinematic,
        (3.0, 0.0),
        (0.4, 0.4),
    );
    sim.world_mut().entity_mut(heroi).insert(mover(true));
    let caixa = corpo(
        &mut sim,
        "Caixa",
        BodyKind::Kinematic,
        (-3.0, 3.0),
        (0.4, 0.4),
    );
    let quem = agente(
        &mut sim,
        "Guarda",
        (-5.0, 0.0),
        NavTarget::Named(stable_name_id("Herói")),
    );
    let mut b = PhysicsBridge::new();
    for t in 1..=4 {
        b.dispatch(&mut sim, true, t);
    }
    let com_caixa = area(&b);
    // O CONTROLO: tirar a caixa (cinemática SEM mover, parada) devolve área — ela recortava.
    sim.world_mut().despawn(caixa);
    for t in 5..=8 {
        b.dispatch(&mut sim, true, t);
    }
    let sem_caixa = area(&b);
    assert!(sem_caixa > com_caixa + 0.5, "a caixa parada não era parede");
    // O herói parado tem o MESMO corpo que a caixa: se recortasse, a região inteira menos ele não
    // seria a área toda (16 × 12 recuada pelo raio do guarda, arredondado PARA CIMA a 1/256 m).
    let r = (0.3f64 * 256.0).ceil() / 256.0;
    let toda = (16.0 - 2.0 * r) * (12.0 - 2.0 * r);
    assert!(
        (sem_caixa - toda).abs() < 1e-3,
        "o herói parado recortou a malha ({sem_caixa} contra {toda})"
    );
    assert!(pela_ilha_do_heroi(&b, quem));
}

fn pela_ilha_do_heroi(b: &PhysicsBridge, quem: Entity) -> bool {
    b.nav_agent(quem)
        .is_some_and(|rt| !rt.partial && rt.status != Status::NoPath)
}

#[test]
fn so_os_agentes_da_malha_que_mudou_refazem_o_caminho() {
    let mut sim = SimWorld::new();
    regiao(&mut sim, (-10.0, 0.0));
    regiao(&mut sim, (10.0, 0.0));
    let esq = agente(
        &mut sim,
        "Esquerda",
        (-15.0, 0.0),
        NavTarget::Point([-5.0, 0.0]),
    );
    let dir = agente(
        &mut sim,
        "Direita",
        (5.0, 0.0),
        NavTarget::Point([15.0, 0.0]),
    );
    let mut b = PhysicsBridge::new();
    for t in 1..=5 {
        b.dispatch(&mut sim, true, t);
    }
    let procuras = |b: &PhysicsBridge, e| b.nav_agent(e).map_or(0, |r| r.searches);
    let (e0, d0) = (procuras(&b, esq), procuras(&b, dir));
    corpo(&mut sim, "Pedra", BodyKind::Static, (-8.0, 4.0), (0.5, 0.5));
    for t in 6..=7 {
        b.dispatch(&mut sim, true, t);
    }
    assert_eq!(
        procuras(&b, esq),
        e0 + 1,
        "a malha da esquerda mudou e o agente dela não refez o caminho"
    );
    assert_eq!(
        procuras(&b, dir),
        d0,
        "a malha da direita não mudou e o agente dela refez o caminho"
    );
}

// ── (W9) A FILA DO REPLANEIO ────────────────────────────────────────────────────────────────────

/// Um componente só de teste: metade dos guardas leva-o, e a CONSULTA ao mundo passa a percorrê-los
/// por tabelas — outra ordem que a das entidades (a fila tem de seguir a das entidades).
#[derive(bevy_ecs::component::Component)]
struct Marca;

/// Uma parede em `x = 0` com dois vãos: o de cima (`4 < y < 6`, o caminho LONGO) sempre aberto, e o
/// do meio (`|y| < 1`, o CURTO) com uma porta. Oito guardas à esquerda com alvos à direita.
fn atalho(porta_fechada: bool) -> (SimWorld, Entity, Vec<Entity>) {
    let mut sim = SimWorld::new();
    regiao(&mut sim, (0.0, 0.0));
    corpo(&mut sim, "Baixo", BodyKind::Static, (0.0, -3.5), (0.3, 2.5));
    corpo(&mut sim, "Cima", BodyKind::Static, (0.0, 2.5), (0.3, 1.5));
    let porta = corpo(
        &mut sim,
        "Porta",
        BodyKind::Kinematic,
        if porta_fechada {
            (0.0, 0.0)
        } else {
            (30.0, 30.0)
        },
        (0.3, 1.2),
    );
    let guardas = (0..8)
        .map(|i| {
            let (x, y) = (-7.0 + 1.0 * (i % 4) as f32, -5.0 + 1.2 * (i / 4) as f32);
            let e = agente(
                &mut sim,
                &format!("Guarda {i}"),
                (x, y),
                NavTarget::Point([6.0, -4.0 + 0.8 * i as f32]),
            );
            if i % 2 == 0 {
                sim.world_mut().entity_mut(e).insert(Marca);
            }
            e
        })
        .collect();
    (sim, porta, guardas)
}

/// As procuras de cada um.
fn procuras_de(b: &PhysicsBridge, quem: &[Entity]) -> Vec<u64> {
    quem.iter()
        .map(|&e| b.nav_agent(e).map_or(0, |r| r.searches))
        .collect()
}

/// A porta como uma CURVA da cena (o que a timeline faz): em `de` até ao tique `4`, em `para` a partir
/// do `5` — um replay põe-na onde ela estava em cada tique.
struct PortaQueMuda {
    porta: Entity,
    de: (f32, f32),
    para: (f32, f32),
    /// O tique em que ela muda.
    quando: u64,
}

impl ph2d_physics_ecs::SceneAtTick for PortaQueMuda {
    fn put(&mut self, sim: &mut SimWorld, tick: u64) -> bool {
        poe(
            sim,
            self.porta,
            if tick < self.quando {
                self.de
            } else {
                self.para
            },
        );
        true
    }
}

/// Corre até `fim`, a porta muda de sítio no tique `5`; devolve as procuras de cada um em cada tique.
fn com_a_porta_a_mudar(
    sim: &mut SimWorld,
    b: &mut PhysicsBridge,
    cena: &mut PortaQueMuda,
    quem: &[Entity],
    fim: u64,
) -> Vec<Vec<u64>> {
    (1..=fim)
        .map(|t| {
            b.dispatch_with_scene(sim, true, t, cena);
            procuras_de(b, quem)
        })
        .collect()
}

/// O tique (índice + 1) em que cada um procurou pela 1.ª vez depois do tique `5`.
fn servido_em(por_tique: &[Vec<u64>], i: usize) -> Option<usize> {
    let base = por_tique[4][i];
    (5..por_tique.len())
        .find(|&k| por_tique[k][i] > base)
        .map(|k| k + 1)
}

/// ⭐ Uma porta que ABRE o atalho não põe os oito a procurar no mesmo tique: com um orçamento que só
/// paga um, a fila serve um por tique, pela ordem das entidades (ninguém tem o caminho partido — o
/// longo continua a andar-se). CONTROLO: sem fila (`u64::MAX`), os oito no mesmo tique.
#[test]
fn a_porta_que_abre_um_atalho_serve_os_agentes_um_por_tique() {
    let servidos = |orc: u64| {
        let (mut sim, porta, quem) = atalho(true);
        let mut b = PhysicsBridge::new();
        b.set_nav_replan_budget(orc);
        let mut cena = PortaQueMuda {
            porta,
            de: (0.0, 0.0),
            para: (30.0, 30.0),
            quando: 5,
        };
        let pt = com_a_porta_a_mudar(&mut sim, &mut b, &mut cena, &quem, 30);
        // E todos acabam pelo atalho.
        assert!(quem.iter().all(|&e| pela_porta(&b, e)), "orçamento {orc}");
        // O tique de cada um, pela ordem das ENTIDADES (a da ponte — a do `Ord` delas).
        let mut por_ordem: Vec<(Entity, usize)> = quem
            .iter()
            .enumerate()
            .map(|(i, &e)| (e, servido_em(&pt, i).expect("cada um acaba por procurar")))
            .collect();
        por_ordem.sort();
        por_ordem.into_iter().map(|(_, t)| t).collect::<Vec<_>>()
    };
    let fila = servidos(1);
    let primeiro = fila[0];
    assert_eq!(
        fila,
        (0..8).map(|i| primeiro + i).collect::<Vec<_>>(),
        "um por tique, pela ordem das entidades"
    );
    let todos = servidos(u64::MAX);
    assert!(
        todos.iter().all(|&t| t == todos[0]),
        "o CONTROLO: sem fila os oito procuram no mesmo tique ({todos:?})"
    );
}

/// ⭐ Quem tem o caminho PARTIDO passa à frente na fila: a porta FECHA o atalho por onde os oito iam;
/// um VIGIA cujo caminho não passa pela porta — e que é o PRIMEIRO na ordem das entidades — é servido
/// DEPOIS deles todos.
#[test]
fn o_caminho_partido_passa_a_frente_na_fila() {
    let (mut sim, porta, guardas) = atalho(false);
    let vigia = agente(
        &mut sim,
        "Vigia",
        (-7.0, 5.0),
        NavTarget::Point([-2.0, 5.0]),
    );
    // A fixtura tem de conter o fenómeno: sem a prioridade, a ordem das entidades servia-o primeiro.
    assert!(
        guardas.iter().all(|&g| vigia < g),
        "o vigia tem de vir antes dos guardas na ordem das entidades"
    );
    let mut b = PhysicsBridge::new();
    b.set_nav_replan_budget(1);
    let mut quem = vec![vigia];
    quem.extend(&guardas);
    let mut cena = PortaQueMuda {
        porta,
        de: (30.0, 30.0),
        para: (0.0, 0.0),
        quando: 5,
    };
    let pt = com_a_porta_a_mudar(&mut sim, &mut b, &mut cena, &quem, 30);
    let vigia_em = servido_em(&pt, 0).expect("o vigia acaba por ser servido");
    for i in 1..quem.len() {
        let g = servido_em(&pt, i).expect("cada guarda procura");
        assert!(
            g < vigia_em,
            "o guarda {i} (caminho partido) servido no tique {g}, o vigia no {vigia_em}"
        );
    }
}

/// ⭐⭐ **Um scrub para o MEIO da fila devolve a mesma corrida** — a dívida (`owed`) vai no anel com a
/// memória do agente; sem ela, o replay serviria outra fila e cada guarda tomaria o atalho noutro
/// tique.
#[test]
fn um_scrub_para_o_meio_da_fila_devolve_a_mesma_corrida() {
    const FIM: u64 = 60;
    let (mut sim, porta, quem) = atalho(true);
    let mut b = PhysicsBridge::new();
    b.set_nav_replan_budget(1);
    let mut cena = PortaQueMuda {
        porta,
        de: (0.0, 0.0),
        para: (30.0, 30.0),
        quando: 8,
    };
    let posicoes = |sim: &mut SimWorld| -> Vec<(f32, f32)> {
        quem.iter()
            .map(|&e| {
                let t = sim.world_mut().get::<Transform>(e).expect("o corpo");
                (t.translation.x, t.translation.y)
            })
            .collect()
    };
    let mut primeira = Vec::new();
    for t in 1..=FIM {
        b.dispatch_with_scene(&mut sim, true, t, &mut cena);
        primeira.push(posicoes(&mut sim));
    }
    // ⚠️ O anel guarda um âncora de 10 em 10 tiques (`STRIDE`): o scrub para o 13 SEMEIA do 10 e
    // replaya três. A porta muda no 8 e a fila serve um por tique a partir do 9 — os dois tiques
    // caem a meio dela. (A 1.ª redacção ia para o 9: sem âncora antes, o scrub refazia tudo do zero e
    // a mutação «o seed não devolve a dívida» SOBREVIVIA.)
    const MEIO: u64 = 13;
    let antes = quem
        .iter()
        .filter(|&&e| b.nav_agent(e).is_some_and(|r| r.owed > 0))
        .count();
    assert_eq!(antes, 0, "no fim a fila está vazia");
    b.dispatch_with_scene(&mut sim, false, MEIO, &mut cena);
    assert_eq!(posicoes(&mut sim), primeira[(MEIO - 1) as usize], "o scrub");
    let devidos = quem
        .iter()
        .filter(|&&e| b.nav_agent(e).is_some_and(|r| r.owed > 0))
        .count();
    assert!(
        devidos > 0,
        "o scrub para o MEIO da fila devolve a fila a meio ({devidos} devidos)"
    );
    let resto: Vec<_> = ((MEIO + 1)..=FIM)
        .map(|t| {
            b.dispatch_with_scene(&mut sim, true, t, &mut cena);
            posicoes(&mut sim)
        })
        .collect();
    assert_eq!(
        resto,
        primeira[MEIO as usize..].to_vec(),
        "o resto da corrida"
    );
}

/// Uma pedra num canto, longe de todos os caminhos, que pára num sítio e noutro de dois em dois
/// tiques: a malha muda sem parar, e nenhum caminho se parte.
struct PedraInquieta {
    pedra: Entity,
}

impl ph2d_physics_ecs::SceneAtTick for PedraInquieta {
    fn put(&mut self, sim: &mut SimWorld, tick: u64) -> bool {
        let y = if (tick / 2) % 2 == 0 { 5.3 } else { 4.6 };
        poe(sim, self.pedra, (-7.6, y));
        true
    }
}

/// ⭐ Uma malha que muda SEM PARAR não deixa ninguém à espera para sempre: quem espera envelhece e
/// passa à frente de quem a mudança seguinte volta a pôr na fila (sem isso, os primeiros na ordem
/// das entidades seriam servidos de novo a cada mudança e os últimos nunca).
#[test]
fn uma_malha_que_nao_para_de_mudar_serve_todos_a_vez() {
    let (mut sim, _porta, quem) = atalho(false);
    let pedra = corpo(
        &mut sim,
        "Pedra",
        BodyKind::Kinematic,
        (-7.6, 5.3),
        (0.2, 0.2),
    );
    let mut b = PhysicsBridge::new();
    b.set_nav_replan_budget(1);
    let mut cena = PedraInquieta { pedra };
    let mut por_tique = Vec::new();
    for t in 1..=40 {
        b.dispatch_with_scene(&mut sim, true, t, &mut cena);
        por_tique.push(procuras_de(&b, &quem));
    }
    for (i, &e) in quem.iter().enumerate() {
        assert!(
            por_tique[39][i] >= por_tique[9][i] + 2,
            "{e:?} procurou {} vezes entre os tiques 10 e 40",
            por_tique[39][i] - por_tique[9][i]
        );
    }
}
