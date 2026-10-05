//! ⭐⭐ (W15) **Toda procura paga do orçamento do tique, e a que não cabe pára a meio** (plano 30 §23):
//! a procura em fatias acaba no MESMO caminho, um scrub para o meio dela devolve a mesma corrida, e quem
//! replaneia por motivo próprio (o alvo andou) também espera pela vez. Os ajudantes são os de
//! [`super::nav_desvio`].

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, NavCostArea, NavTarget, PhysicsBridge, RigidBody,
    SceneAtTick,
};

use super::nav_desvio::{agente, pos, regiao};

/// Um corpo parado: uma caixa sólida, ou (sensor) uma poça de lama a peso 4.
fn caixa(sim: &mut SimWorld, em: (f32, f32), meia: f32, lama: bool) {
    let mut e = sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: meia,
                half_y: meia,
            },
            is_sensor: lama,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(em.0, em.1)),
    ));
    if lama {
        e.insert(NavCostArea {
            cost: 4.0,
            forbidden: false,
        });
    }
}

/// A região `16 × 12` coberta de poças em xadrez e de pedras entre elas: uma procura ponderada cara.
fn lamacal(sim: &mut SimWorld) {
    regiao(sim);
    for i in 0..9 {
        for j in 0..7 {
            let (x, y) = (-7.0 + 1.75 * i as f32, -5.25 + 1.75 * j as f32);
            caixa(sim, (x, y), 0.45, (i + j) % 2 == 0);
            if (i * 7 + j) % 5 == 0 {
                caixa(sim, (x + 0.875, y + 0.875), 0.15, false);
            }
        }
    }
}

/// Um agente que atravessa o lamaçal de canto a canto.
fn travessia() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    lamacal(&mut sim);
    let e = agente(
        &mut sim,
        "Viajante",
        (-7.6, -5.6),
        NavTarget::Point([7.6, 5.6]),
        false,
    );
    (sim, e)
}

/// O caminho e o trabalho da procura da travessia sem tecto (o CONTROLO): tudo no 1.º tique.
fn sem_tecto() -> (Vec<[f64; 2]>, u64) {
    let (mut sim, e) = travessia();
    let mut b = PhysicsBridge::new();
    b.set_nav_replan_budget(u64::MAX);
    b.dispatch(&mut sim, true, 1);
    let rt = b.nav_agent(e).expect("o agente");
    assert!(
        rt.a_meio.is_none() && !rt.path.is_empty(),
        "o CONTROLO acaba no 1.º tique"
    );
    (rt.path.clone(), rt.last_work)
}

/// ⭐ A procura que não cabe no orçamento PÁRA a meio, o agente espera PARADO, e no tique em que ela
/// acaba o caminho é o MESMO da procura de uma vez — com o caminho crítico de cada tique no orçamento
/// (mais um `pop`), em série e com o passo em paralelo (uma procura sozinha não se reparte: acaba no
/// mesmo tique nos dois).
#[test]
fn uma_procura_que_nao_cabe_para_a_meio_e_acaba_no_mesmo_caminho() {
    let (caminho, w) = sem_tecto();
    assert!(w >= 150, "a fixtura: a procura tem de ser cara ({w})");
    let orc = w / 5;
    let acaba = |paralelas: usize| {
        let (mut sim, e) = travessia();
        let partida = pos(&sim, e);
        let mut b = PhysicsBridge::new();
        b.set_nav_replan_budget(orc);
        b.set_nav_parallel(paralelas);
        let tecto = orc;
        let mut t = 0;
        while b.nav_agent(e).is_none_or(|r| r.path.is_empty()) {
            t += 1;
            assert!(t <= 10, "a procura nunca acabou");
            b.dispatch(&mut sim, true, t);
            let rt = b.nav_agent(e).expect("o agente");
            assert_eq!(rt.searches, 1, "uma procura, continuada");
            assert!(
                b.nav_search_critical_work() <= tecto + tecto / 10,
                "o tique {t} gastou {} de {tecto}",
                b.nav_search_critical_work()
            );
            if rt.path.is_empty() {
                assert!(rt.a_meio.is_some(), "a meio no tique {t}");
                assert_eq!(pos(&sim, e), partida, "à espera, parado");
            }
        }
        let rt = b.nav_agent(e).expect("o agente");
        assert_eq!(
            rt.path, caminho,
            "o mesmo caminho, ao bit ({paralelas} em paralelo)"
        );
        assert_eq!(rt.last_work, w, "o mesmo trabalho");
        t
    };
    let serie = acaba(0);
    assert!(
        (5..=6).contains(&serie),
        "em série acabou no tique {serie} ({w} em fatias de {orc})"
    );
    assert_eq!(acaba(16), serie, "com o passo em paralelo");
}

/// ⭐⭐ **Um scrub para o MEIO de uma procura em fatias devolve a mesma corrida** — no anel vai só a
/// descrição dela (`a_meio`), e a condução refá-la até ao mesmo trabalho.
#[test]
fn um_scrub_a_meio_de_uma_procura_em_fatias_devolve_a_mesma_corrida() {
    const FIM: u64 = 60;
    let (_, w) = sem_tecto();
    let (mut sim, e) = travessia();
    let mut b = PhysicsBridge::new();
    // A procura dura `~40` tiques: o âncora `10` (o anel guarda um de 10 em 10) cai a meio dela.
    b.set_nav_replan_budget(w / 40);
    let mut primeira = Vec::new();
    for t in 1..=FIM {
        b.dispatch(&mut sim, true, t);
        primeira.push((pos(&sim, e), b.nav_agent(e).cloned()));
    }
    const MEIO: u64 = 13;
    let (_, rt) = &primeira[(MEIO - 1) as usize];
    assert!(
        rt.as_ref().is_some_and(|r| r.a_meio.is_some()),
        "a fixtura: no tique {MEIO} a procura está a meio"
    );
    assert!(
        primeira[(FIM - 1) as usize]
            .1
            .as_ref()
            .is_some_and(|r| !r.path.is_empty()),
        "e acaba antes do fim"
    );
    b.dispatch(&mut sim, false, MEIO);
    assert_eq!(
        (pos(&sim, e), b.nav_agent(e).cloned()),
        primeira[(MEIO - 1) as usize]
    );
    for t in MEIO + 1..=FIM {
        b.dispatch(&mut sim, true, t);
        assert_eq!(
            (pos(&sim, e), b.nav_agent(e).cloned()),
            primeira[(t - 1) as usize],
            "o tique {t}"
        );
    }
    // E o scrub feito COM a procura ainda a meio: a que a ponte tem é a do futuro (do tique `25`), e
    // o scrub tem de a esquecer e refazer a do âncora.
    let (mut sim, e) = travessia();
    let mut b = PhysicsBridge::new();
    b.set_nav_replan_budget(w / 40);
    for t in 1..=25 {
        b.dispatch(&mut sim, true, t);
    }
    assert!(
        b.nav_agent(e).is_some_and(|r| r.a_meio.is_some()),
        "a fixtura: no tique 25 a procura ainda vai a meio"
    );
    b.dispatch(&mut sim, false, MEIO);
    for t in MEIO + 1..=FIM {
        b.dispatch(&mut sim, true, t);
        assert_eq!(
            (pos(&sim, e), b.nav_agent(e).cloned()),
            primeira[(t - 1) as usize],
            "o tique {t} (scrub com a procura a meio)"
        );
    }
}

/// O herói a saltar de um canto para o outro a cada tique: todos os que o seguem querem replanear.
struct HeroiInquieto {
    heroi: Entity,
}

impl SceneAtTick for HeroiInquieto {
    fn put(&mut self, sim: &mut SimWorld, tick: u64) -> bool {
        let x = if tick.is_multiple_of(2) { 7.0 } else { 4.0 };
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(self.heroi) {
            t.translation = Vec2::new(x, 5.0);
        }
        true
    }
}

/// ⭐ **Quem replaneia por motivo próprio também paga do orçamento** (o mecanismo 2 do §22.8): oito que
/// seguem um herói que salta a cada tique não gastam num tique mais que o orçamento — esperam pela vez
/// e andam o caminho que têm. CONTROLO: sem tecto, um tique gasta `8×` o orçamento. (Procuras que não
/// custam nada — o herói a direito — cabem quantas forem.)
#[test]
fn quem_segue_um_alvo_que_salta_espera_pela_vez() {
    let pior = |orc: u64| {
        let mut sim = SimWorld::new();
        lamacal(&mut sim);
        let heroi = sim
            .world_mut()
            .spawn((
                Name::new("Herói"),
                Transform::from_translation(Vec2::new(4.0, 5.0)),
            ))
            .id();
        let nome = ph2d_ecs::stable_name_id("Herói");
        let quem: Vec<Entity> = (0..8)
            .map(|i| {
                agente(
                    &mut sim,
                    &format!("S{i}"),
                    (-7.6, -5.6 + 0.4 * i as f32),
                    NavTarget::Named(nome),
                    false,
                )
            })
            .collect();
        let mut b = PhysicsBridge::new();
        let mut cena = HeroiInquieto { heroi };
        // Nascem sem tecto; o orçamento vale daí em diante.
        b.dispatch_with_scene(&mut sim, true, 1, &mut cena);
        b.set_nav_replan_budget(orc);
        let antes: u64 = quem
            .iter()
            .filter_map(|&e| b.nav_agent(e))
            .map(|r| r.searches)
            .sum();
        let (mut gasto, mut procuras) = (0u64, 0u64);
        for t in 2..=40 {
            let p0: u64 = quem
                .iter()
                .filter_map(|&e| b.nav_agent(e))
                .map(|r| r.searches)
                .sum();
            b.dispatch_with_scene(&mut sim, true, t, &mut cena);
            let p1: u64 = quem
                .iter()
                .filter_map(|&e| b.nav_agent(e))
                .map(|r| r.searches)
                .sum();
            gasto = gasto.max(b.nav_search_critical_work());
            procuras = procuras.max(p1 - p0);
        }
        let depois: u64 = quem
            .iter()
            .filter_map(|&e| b.nav_agent(e))
            .map(|r| r.searches)
            .sum();
        assert!(
            depois > antes + 8,
            "a fixtura: eles replaneiam ({antes} → {depois})"
        );
        (gasto, procuras)
    };
    let (gasto_livre, procuras_livre) = pior(u64::MAX);
    assert!(
        procuras_livre >= 6,
        "o CONTROLO: sem tecto um tique procura por {procuras_livre}"
    );
    let orc = gasto_livre / 8;
    let (gasto, _) = pior(orc);
    assert!(
        gasto <= orc + orc / 10,
        "nenhum tique passa o orçamento no caminho crítico: {gasto} de {orc} (sem tecto: {gasto_livre})"
    );
}

/// ⭐⭐ **O passo em paralelo dá o MESMO com uma thread e com oito** (plano 30 §23.4, critério 5): quem
/// avança e quanto decide-se antes de correr, logo o número de núcleos só muda o relógio. Vinte agentes
/// a atravessar o lamaçal com um orçamento curto (várias procuras a meio ao mesmo tempo).
#[test]
fn o_passo_em_paralelo_da_o_mesmo_com_uma_thread_e_com_oito() {
    let (_, w) = sem_tecto();
    let corrida = |threads: usize| {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("o pool");
        pool.install(|| {
            let mut sim = SimWorld::new();
            lamacal(&mut sim);
            let quem: Vec<Entity> = (0..20)
                .map(|i| {
                    let y = -5.6 + 0.55 * i as f32;
                    agente(
                        &mut sim,
                        &format!("V{i}"),
                        (-7.6, y),
                        NavTarget::Point([7.6, -y]),
                        true,
                    )
                })
                .collect();
            let mut b = PhysicsBridge::new();
            b.set_nav_replan_budget(w / 4);
            let (mut juntas, mut fotos) = (0usize, Vec::new());
            for t in 1..=60 {
                b.dispatch(&mut sim, true, t);
                juntas += usize::from(b.nav_search_work() > b.nav_search_critical_work());
                fotos.push(
                    quem.iter()
                        .map(|&e| (pos(&sim, e), b.nav_agent(e).cloned()))
                        .collect::<Vec<_>>(),
                );
            }
            (fotos, juntas)
        })
    };
    let (uma, juntas) = corrida(1);
    assert!(
        juntas >= 5,
        "a fixtura: o passo em paralelo correu várias juntas em {juntas} tiques"
    );
    let (oito, _) = corrida(8);
    for (t, (a, b)) in uma.iter().zip(&oito).enumerate() {
        assert_eq!(a, b, "o tique {}", t + 1);
    }
}

/// Uma pedra num canto do lamaçal, longe do caminho, que pára num sítio e noutro de dois em dois tiques
/// (um corpo que se move a cada tique ANDA, e não entra na malha): a malha muda sem parar.
struct PedraInquieta {
    pedra: Entity,
}

impl SceneAtTick for PedraInquieta {
    fn put(&mut self, sim: &mut SimWorld, tick: u64) -> bool {
        let y = if (tick / 2).is_multiple_of(2) {
            5.2
        } else {
            4.4
        };
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(self.pedra) {
            t.translation = Vec2::new(-7.4, y);
        }
        true
    }
}

/// ⭐ **Uma malha que nunca pára não deixa uma procura sem acabar**: cada mudança recomeça a procura a
/// meio, e cada recomeço DOBRA a fatia da seguinte (`fatia_depois_de`) — ela acaba, e o tique só passa
/// o orçamento depois de `k` recomeços seguidos, e por `2^k` no máximo. CONTROLO: com a pedra quieta,
/// acaba nas fatias de sempre, sem recomeço nenhum.
#[test]
fn uma_malha_que_nunca_para_nao_deixa_a_procura_sem_acabar() {
    let (_, w) = sem_tecto();
    let orc = w / 8;
    let corre = |inquieta: bool| {
        let (mut sim, e) = travessia();
        let pedra = sim
            .world_mut()
            .spawn((
                RigidBody {
                    kind: BodyKind::Kinematic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.2,
                        half_y: 0.2,
                    },
                    ..Collider::default()
                },
                Transform::from_translation(Vec2::new(-7.4, 4.4)),
            ))
            .id();
        let mut cena = PedraInquieta { pedra };
        let mut b = PhysicsBridge::new();
        b.set_nav_replan_budget(orc);
        let (mut t, mut pior, mut recomecos) = (0u64, 0u64, 0u32);
        while b.nav_agent(e).is_none_or(|r| r.path.is_empty()) {
            t += 1;
            assert!(t <= 40, "a procura nunca acabou (inquieta: {inquieta})");
            if inquieta {
                b.dispatch_with_scene(&mut sim, true, t, &mut cena);
            } else {
                b.dispatch(&mut sim, true, t);
            }
            pior = pior.max(b.nav_search_critical_work());
            recomecos = recomecos.max(b.nav_agent(e).map_or(0, |r| r.recomecos));
        }
        (t, pior, recomecos)
    };
    let (quieta, _, sem) = corre(false);
    assert!(
        quieta <= 10 && sem == 0,
        "o CONTROLO acaba em fatias ({quieta} tiques)"
    );
    let (t, pior, recomecos) = corre(true);
    assert!(
        recomecos >= 2,
        "a fixtura: a malha que muda recomeça a procura ({recomecos}×)"
    );
    let tecto = orc << recomecos;
    assert!(
        pior <= tecto + tecto / 10,
        "o tique pagou {pior} (o orçamento {orc} dobrado {recomecos}×; acabou no tique {t})"
    );
}

/// `n` viajantes a atravessar o lamaçal, um por linha.
fn viajantes(n: usize) -> (SimWorld, Vec<Entity>) {
    let mut sim = SimWorld::new();
    lamacal(&mut sim);
    let quem = (0..n)
        .map(|i| {
            let y = -5.6 + 0.7 * i as f32;
            agente(
                &mut sim,
                &format!("V{i}"),
                (-7.6, y),
                NavTarget::Point([7.6, -y]),
                false,
            )
        })
        .collect();
    (sim, quem)
}

/// ⭐ **Com poucas faixas, a procura mais ADIANTADA acaba primeiro** — o passo em paralelo dá a fatia
/// à que já andou mais; por ordem de espera, as procuras a meio revezavam-se e nenhuma acabava antes de
/// a malha seguinte as recomeçar todas (medido na sonda: plano 30 §23.5). Quatro viajantes, uma faixa.
#[test]
fn com_poucas_faixas_a_procura_mais_adiantada_acaba_primeiro() {
    let (_, w) = sem_tecto();
    let (mut sim, quem) = viajantes(4);
    let mut b = PhysicsBridge::new();
    b.set_nav_replan_budget(w / 4);
    b.set_nav_parallel(1);
    let mut primeiro = None;
    for t in 1..=40u64 {
        b.dispatch(&mut sim, true, t);
        if primeiro.is_none()
            && quem
                .iter()
                .any(|&e| b.nav_agent(e).is_some_and(|r| !r.path.is_empty()))
        {
            primeiro = Some(t);
        }
    }
    let primeiro = primeiro.expect("alguém acabou");
    assert!(
        primeiro <= 7,
        "a 1.ª procura acabou no tique {primeiro} (4 fatias)"
    );
    assert!(
        quem.iter()
            .all(|&e| b.nav_agent(e).is_some_and(|r| !r.path.is_empty())),
        "e todos acabam"
    );
}

/// ⭐ **Uma procura que ainda não começou não conta como recomeço** — perder uma procura a meio sem
/// trabalho feito não perde nada, e contá-la dobrava a fatia de quem não precisa (um pico). Seis
/// viajantes, uma faixa, a malha a mudar de dois em dois tiques: os que esperam sem ter começado têm
/// `recomecos = 0`.
#[test]
fn uma_procura_que_nao_comecou_nao_conta_como_recomeco() {
    let (_, w) = sem_tecto();
    let (mut sim, quem) = viajantes(6);
    let pedra = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.2,
                    half_y: 0.2,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(-7.4, 4.4)),
        ))
        .id();
    let mut cena = PedraInquieta { pedra };
    let mut b = PhysicsBridge::new();
    b.set_nav_replan_budget(w / 4);
    b.set_nav_parallel(1);
    let mut comecou = vec![false; quem.len()];
    let mut esperas = 0;
    for t in 1..=30u64 {
        b.dispatch_with_scene(&mut sim, true, t, &mut cena);
        for (i, &e) in quem.iter().enumerate() {
            let Some(rt) = b.nav_agent(e) else { continue };
            comecou[i] |= rt.a_meio.is_some_and(|a| a.trabalho > 0) || !rt.path.is_empty();
            if !comecou[i] {
                esperas += usize::from(rt.a_meio.is_some());
                assert_eq!(rt.recomecos, 0, "o {i}.º, sem ter começado, no tique {t}");
            }
        }
    }
    assert!(
        esperas >= 20,
        "a fixtura: procuras abertas à espera durante as mudanças ({esperas})"
    );
}

/// Uma parede que aparece no meio do lamaçal no tique 2 (e fica: entra na malha no 3).
struct ParedeQueAparece {
    parede: Entity,
}

impl SceneAtTick for ParedeQueAparece {
    fn put(&mut self, sim: &mut SimWorld, tick: u64) -> bool {
        let em = if tick >= 2 { (0.0, 0.0) } else { (40.0, 40.0) };
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(self.parede) {
            t.translation = Vec2::new(em.0, em.1);
        }
        true
    }
}

/// ⭐ **Uma procura a meio numa malha que MUDOU recomeça — nunca acaba com a malha velha**: o estado
/// dela fala dos polígonos de outra malha. Uma parede aparece a meio da procura do viajante; o caminho
/// que ele recebe anda-se na malha de agora.
#[test]
fn uma_procura_a_meio_numa_malha_que_mudou_recomeca() {
    let (_, w) = sem_tecto();
    let (mut sim, e) = travessia();
    let parede = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.3,
                    half_y: 4.0,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(40.0, 40.0)),
        ))
        .id();
    let mut cena = ParedeQueAparece { parede };
    let mut b = PhysicsBridge::new();
    b.set_nav_replan_budget(w / 5);
    let mut a_meio_na_mudanca = false;
    for t in 1..=30u64 {
        b.dispatch_with_scene(&mut sim, true, t, &mut cena);
        let rt = b.nav_agent(e).expect("o agente");
        a_meio_na_mudanca |= rt.recomecos > 0;
    }
    assert!(
        a_meio_na_mudanca,
        "a fixtura: a parede recomeça a procura a meio"
    );
    let rt = b.nav_agent(e).expect("o agente");
    assert!(!rt.path.is_empty(), "acabou");
    let (_, _, malha) = b.nav_meshes().next().expect("a malha");
    let p = pos(&sim, e);
    assert!(
        ph2d_nav::refresh::path_still_walkable(malha, rt, [f64::from(p.0), f64::from(p.1)], None),
        "o caminho anda-se na malha de agora: {:?}",
        rt.path
    );
}
