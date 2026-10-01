//! **A NAVEGAÇÃO de ponta a ponta** (plano 30, W3) — um agente com um [`TopDownPlayer`] a andar no
//! mundo da física, pela porta do produto (`PhysicsBridge::dispatch`).
//!
//! As leis da malha e do caminho têm gates próprios nas duas folhas (contra o oráculo exacto e
//! contra o Godot); aqui mede-se o que só a COSTURA pode partir: o agente lê o corpo certo, a região
//! certa, o raio certo, escreve a intenção no mover antes do passo, e um scrub devolve a mesma
//! perseguição.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_nav::{Event, Status};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, NavAgent, NavRegion, NavTarget, PhysicsBridge, RigidBody,
    TopDownPlayer,
};
use ph2d_topdown::{TopDownLaw, direction::DirectionMode};

fn parede(sim: &mut SimWorld, centro: (f32, f32), meio: (f32, f32)) {
    sim.world_mut().spawn((
        Name::new("Parede"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: meio.0,
                half_y: meio.1,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(centro.0, centro.1)),
    ));
}

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

/// Um agente de raio `r`, em `em`, a ir para `alvo`.
fn agente(sim: &mut SimWorld, nome: &str, em: (f32, f32), r: f32, alvo: NavTarget) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new(nome),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: r },
                ..Collider::default()
            },
            TopDownPlayer::from_law(TopDownLaw {
                default_controls: false,
                direction: DirectionMode::Free,
                ..TopDownLaw::default()
            }),
            NavAgent {
                target: alvo,
                arrive_distance: 0.1,
                ..NavAgent::default()
            },
            Transform::from_translation(Vec2::new(em.0, em.1)),
        ))
        .id()
}

fn pos(sim: &SimWorld, e: Entity) -> (f32, f32) {
    let t = sim.world().get::<Transform>(e).expect("o corpo");
    (t.translation.x, t.translation.y)
}

/// Corre `ate` tiques e devolve o caminho e os factos de navegação de cada dispatch.
fn corre(
    sim: &mut SimWorld,
    bridge: &mut PhysicsBridge,
    quem: Entity,
    de: u64,
    ate: u64,
) -> (Vec<(f32, f32)>, Vec<Event>) {
    let mut caminho = Vec::new();
    let mut factos = Vec::new();
    for t in de..=ate {
        bridge.dispatch(sim, true, t);
        caminho.push(pos(sim, quem));
        factos.extend(
            bridge
                .nav_events()
                .iter()
                .filter(|e| e.agent == quem)
                .map(|e| e.kind),
        );
    }
    (caminho, factos)
}

/// Uma parede de `y = −4` a `4` no meio da região: o caminho recto bate nela, o caminho certo dá a
/// volta por cima ou por baixo.
fn cena_parede() -> (SimWorld, PhysicsBridge, Entity) {
    let mut sim = SimWorld::new();
    regiao(&mut sim);
    parede(&mut sim, (0.0, 0.0), (0.3, 4.0));
    sim.world_mut().spawn((
        Name::new("Alvo"),
        Transform::from_translation(Vec2::new(4.0, 0.0)),
    ));
    let quem = agente(
        &mut sim,
        "Perseguidor",
        (-4.0, 0.0),
        0.3,
        NavTarget::Named(stable_name_id("Alvo")),
    );
    (sim, PhysicsBridge::new(), quem)
}

/// ⭐⭐⭐ **O agente contorna a parede e chega — UMA vez.**
#[test]
fn o_agente_contorna_a_parede_e_chega_uma_vez() {
    let (mut sim, mut bridge, quem) = cena_parede();
    let (caminho, factos) = corre(&mut sim, &mut bridge, quem, 1, 360);
    let fim = *caminho.last().expect("o caminho");
    assert!(
        (fim.0 - 4.0).hypot(fim.1) < 0.2,
        "não chegou ao alvo: acabou em {fim:?}"
    );
    let dobrou = caminho.iter().any(|p| p.1.abs() > 4.0);
    assert!(dobrou, "não deu a volta à parede");
    // Nunca atravessou: com a parede em `|x| < 0,3` e o corpo de raio `0,3`, o centro nunca entra em
    // `|x| < 0,55` com `|y| < 3,95`.
    for p in &caminho {
        assert!(
            !(p.0.abs() < 0.55 && p.1.abs() < 3.95),
            "atravessou a parede em {p:?}"
        );
    }
    assert_eq!(factos, vec![Event::Arrived], "os factos: {factos:?}");
    assert_eq!(
        bridge.nav_agent(quem).map(|a| a.status),
        Some(Status::Arrived)
    );
}

/// O CONTROLO: o mesmo agente sem região não tem malha — fica parado e diz porquê, uma vez.
#[test]
fn sem_regiao_o_agente_para_e_diz_porque() {
    let mut sim = SimWorld::new();
    parede(&mut sim, (0.0, 0.0), (0.3, 4.0));
    let quem = agente(
        &mut sim,
        "Perdido",
        (-4.0, 0.0),
        0.3,
        NavTarget::Point([4.0, 0.0]),
    );
    let mut bridge = PhysicsBridge::new();
    let (caminho, factos) = corre(&mut sim, &mut bridge, quem, 1, 60);
    assert_eq!(*caminho.last().expect("o caminho"), (-4.0, 0.0));
    assert_eq!(factos, vec![Event::NoPath]);
}

/// ⭐⭐ **Um vão de 1 m deixa passar um corpo de 0,6 m e não um de 1,2 m.**
#[test]
fn o_vao_estreito_deixa_passar_o_pequeno_e_nao_o_grande() {
    let cena = |r: f32| {
        let mut sim = SimWorld::new();
        regiao(&mut sim);
        // Duas paredes que fecham a região de cima a baixo, menos um vão de `y = −0,5` a `0,5`.
        parede(&mut sim, (0.0, 3.25), (0.3, 2.75));
        parede(&mut sim, (0.0, -3.25), (0.3, 2.75));
        let quem = agente(
            &mut sim,
            "Agente",
            (-4.0, 2.0),
            r,
            NavTarget::Point([4.0, 2.0]),
        );
        (sim, PhysicsBridge::new(), quem)
    };
    let (mut sim, mut bridge, pequeno) = cena(0.3);
    let (caminho, factos) = corre(&mut sim, &mut bridge, pequeno, 1, 360);
    let fim = *caminho.last().expect("o caminho");
    assert!(
        (fim.0 - 4.0).hypot(fim.1 - 2.0) < 0.2,
        "o pequeno não passou: {fim:?}"
    );
    assert_eq!(factos, vec![Event::Arrived]);

    let (mut sim, mut bridge, grande) = cena(0.6);
    let (caminho, factos) = corre(&mut sim, &mut bridge, grande, 1, 360);
    assert!(
        caminho.iter().all(|p| p.0 < 0.0),
        "o grande passou um vão mais estreito que ele"
    );
    assert_eq!(
        factos,
        vec![Event::NoPath],
        "o grande diz que não há caminho, uma vez"
    );
    assert_eq!(
        bridge.nav_agent(grande).map(|a| a.status),
        Some(Status::MovingPartial)
    );
    // E pára o mais perto que pode — encostado ao vão, do lado dele.
    let fim = *caminho.last().expect("o caminho");
    assert!(fim.0 > -1.2, "parou longe do vão: {fim:?}");
}

/// ⭐⭐⭐ **Um Reset devolve a MESMA perseguição** — três corridas iguais ao bit.
#[test]
fn um_reset_devolve_a_mesma_perseguicao() {
    let (mut sim, mut bridge, quem) = cena_parede();
    let primeira = corre(&mut sim, &mut bridge, quem, 1, 200).0;
    for corrida in 2..=3 {
        bridge.dispatch(&mut sim, false, 0);
        let outra = corre(&mut sim, &mut bridge, quem, 1, 200).0;
        assert_eq!(primeira, outra, "a corrida {corrida} divergiu");
    }
}

/// ⚠️ **Um replay não publica**: rebobinar para logo depois da chegada refaz o tique em que ela
/// aconteceu, e o «chegou» não pode sair outra vez — um scrub não é uma tempestade de sinais.
///
/// ⚠️⚠️ **A chegada tem de cair ENTRE dois checkpoints**, e a 1.ª redacção não o garantia: com o
/// agente em `(−4, 0)` ele chega no tique `180`, que é múltiplo do `STRIDE` do anel — o checkpoint
/// desse tique já tem o agente CHEGADO, todo rebobinar até `≥ 180` semeia dali, o tique da chegada
/// nunca é refeito e a mutação «o replay também publica» SOBREVIVIA. ⇒ o ponto de partida é
/// escolhido para a chegada não cair num múltiplo, e o gate afirma essa pré-condição.
#[test]
fn um_replay_nao_volta_a_dizer_que_chegou() {
    let stride = ph2d_physics::world::checkpoint::STRIDE;
    let mut achado = None;
    for dx in [0.0_f32, 0.07, 0.13, 0.21, 0.29] {
        let mut sim = SimWorld::new();
        regiao(&mut sim);
        parede(&mut sim, (0.0, 0.0), (0.3, 4.0));
        let quem = agente(
            &mut sim,
            "Ag",
            (-4.0 + dx, 0.0),
            0.3,
            NavTarget::Point([4.0, 0.0]),
        );
        let mut bridge = PhysicsBridge::new();
        let mut chegada = None;
        for t in 1..=360_u64 {
            bridge.dispatch(&mut sim, true, t);
            if bridge
                .nav_events()
                .iter()
                .any(|e| e.agent == quem && e.kind == Event::Arrived)
            {
                chegada = Some(t);
            }
        }
        if let Some(a) = chegada.filter(|a| a % stride != 0) {
            achado = Some((sim, bridge, a));
            break;
        }
    }
    let (mut sim, mut bridge, chegada) =
        achado.expect("a fixtura precisa de uma chegada que não caia num checkpoint");
    for alvo in [chegada, chegada + 1] {
        bridge.dispatch(&mut sim, false, alvo);
        assert!(
            bridge.nav_events().is_empty(),
            "o replay até {alvo} voltou a publicar: {:?}",
            bridge.nav_events()
        );
        bridge.dispatch(&mut sim, false, 360);
    }
}

/// Desligar um agente PÁRA o mover — a intenção dele é zerada, e não fica a última direcção presa
/// no canal.
#[test]
fn desligar_o_agente_para_o_mover() {
    let (mut sim, mut bridge, quem) = cena_parede();
    corre(&mut sim, &mut bridge, quem, 1, 30);
    if let Some(mut a) = sim.world_mut().get_mut::<NavAgent>(quem) {
        a.active = false;
    }
    corre(&mut sim, &mut bridge, quem, 31, 32);
    let parado = pos(&sim, quem);
    corre(&mut sim, &mut bridge, quem, 33, 90);
    assert_eq!(pos(&sim, quem), parado, "desligado e ainda a andar");
}

/// ⭐⭐⭐ **Um scrub para o MEIO devolve o tique exacto** — a memória do agente vai no anel.
///
/// ⚠️ **O alvo cai no meio e não no zero**: um rewind para o tique 0 replaya zero passos, e um
/// gate que só resetasse deixaria passar o laço de replay sem a navegação.
#[test]
fn um_scrub_para_o_meio_devolve_o_tique_exacto() {
    const MEIO: u64 = 90;
    let (mut sim, mut bridge, quem) = cena_parede();
    let primeira = corre(&mut sim, &mut bridge, quem, 1, 200).0;
    bridge.dispatch(&mut sim, false, MEIO);
    assert_eq!(pos(&sim, quem), primeira[(MEIO - 1) as usize], "o scrub");
    let resto = corre(&mut sim, &mut bridge, quem, MEIO + 1, 200).0;
    assert_eq!(
        resto,
        primeira[MEIO as usize..].to_vec(),
        "o resto da corrida"
    );
}

/// ⭐⭐ **A memória do agente vai no anel** — com um alvo que ANDA, o caminho em curso foi planeado
/// para onde o alvo ESTAVA (o recálculo só acorda depois de `repath_distance`), e um scrub que
/// esquecesse a memória replanearia para onde o alvo ESTÁ: outra perseguição.
///
/// ⚠️ É esta fixtura que torna a memória observável: com o alvo parado, replanear de um ponto do
/// caminho óptimo devolve o resto do mesmo caminho e o gate de cima não a distingue. ⚠️ E o alvo é
/// OUTRO AGENTE (a fugir por um caminho dele) porque a 1.ª redacção usou um projéctil que nunca saiu
/// do sítio — a sonda mostrou o «fugitivo» parado em `(4, −3)` os 240 tiques, logo a fixtura não
/// continha o fenómeno e as mutações da memória SOBREVIVERAM.
#[test]
fn com_um_alvo_que_anda_o_scrub_devolve_a_mesma_perseguicao() {
    let mut sim = SimWorld::new();
    regiao(&mut sim);
    parede(&mut sim, (0.0, 0.0), (0.3, 4.0));
    let fugitivo = agente(
        &mut sim,
        "Fugitivo",
        (4.0, -3.0),
        0.3,
        NavTarget::Point([-5.0, 5.0]),
    );
    if let Some(mut m) = sim.world_mut().get_mut::<TopDownPlayer>(fugitivo) {
        m.speed = 1.5;
    }
    let quem = agente(
        &mut sim,
        "Perseguidor",
        (-4.0, 0.0),
        0.3,
        NavTarget::Named(stable_name_id("Fugitivo")),
    );
    let mut bridge = PhysicsBridge::new();
    let primeira = corre(&mut sim, &mut bridge, quem, 1, 240).0;
    let fugiu = pos(&sim, fugitivo);
    assert!(
        (fugiu.0 - 4.0).hypot(fugiu.1 + 3.0) > 2.0,
        "a fixtura precisa de um alvo que ANDA: o fugitivo acabou em {fugiu:?}"
    );
    let buscas = bridge.nav_agent(quem).map_or(0, |a| a.searches);
    assert!(buscas >= 3, "a fixtura precisa de recálculos: {buscas}");
    for meio in [77_u64, 131, 199] {
        bridge.dispatch(&mut sim, false, meio);
        assert_eq!(
            pos(&sim, quem),
            primeira[(meio - 1) as usize],
            "o scrub para {meio}"
        );
        let resto = corre(&mut sim, &mut bridge, quem, meio + 1, 240).0;
        assert_eq!(
            resto,
            primeira[meio as usize..].to_vec(),
            "o resto depois de {meio}"
        );
    }
}

/// ⭐ **Chegar é um SINAL** — pela mesma porta da tabela de acções, uma vez, e um nome vazio cala.
#[test]
fn chegar_publica_o_sinal_uma_vez_pela_porta_da_casa() {
    let (mut sim, mut bridge, quem) = cena_parede();
    if let Some(mut a) = sim.world_mut().get_mut::<NavAgent>(quem) {
        a.on_arrived = "  chegou ".to_owned();
    }
    let tree = ph2d_tags::TagTree::default();
    let mut ouvidos = Vec::new();
    for t in 1..=360 {
        bridge.dispatch(&mut sim, true, t);
        ouvidos.extend(
            bridge
                .signal_events(&sim, &tree)
                .into_iter()
                .filter(|s| s.source == quem)
                .map(|s| s.name),
        );
    }
    assert_eq!(ouvidos, vec!["chegou".to_owned()]);
}

/// A malha de uma região COM a parede e SEM ela (a parede numa camada fora da máscara).
fn area_da_malha(mascara: u8, camada_da_parede: u8) -> (f64, f64) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Região"),
        NavRegion {
            half_extents: [8.0, 6.0],
            obstacle_layers: mascara,
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Parede"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.3,
                half_y: 4.0,
            },
            layer: camada_da_parede,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    agente(
        &mut sim,
        "Ag",
        (-4.0, 0.0),
        0.3,
        NavTarget::Point([4.0, 0.0]),
    );
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, true, 1);
    let malhas: Vec<(f64, f64)> = bridge
        .nav_meshes()
        .map(|(_, r, m)| (m.area(), f64::from(r)))
        .collect();
    assert_eq!(malhas.len(), 1, "uma região, um raio");
    malhas[0]
}

/// ⭐ **Só as camadas da máscara bloqueiam** — uma parede noutra camada não abre buraco na malha.
#[test]
fn so_as_camadas_da_mascara_bloqueiam() {
    let (com, _) = area_da_malha(0b0000_0001, 0);
    let (sem, r) = area_da_malha(0b0000_0001, 3);
    // ⚠️ O raio é o da CHAVE da malha (arredondado para cima a `1/256 m`), não o `0,3` do colisor.
    assert!((0.3..0.3 + 1.0 / 256.0).contains(&r), "o raio da chave: {r}");
    let livre = (16.0 - 2.0 * r) * (12.0 - 2.0 * r);
    assert!(
        (sem - livre).abs() < 1e-6,
        "a parede fora da máscara abriu buraco: {sem} contra {livre}"
    );
    assert!(
        com < livre - 3.0,
        "a parede dentro da máscara não abriu buraco: {com}"
    );
}

/// ⭐⭐ **Mudar os obstáculos esquece os caminhos** — tirar a parede a meio faz o agente seguir em
/// linha recta, em vez de continuar a dar a volta a uma parede que já não existe.
#[test]
fn tirar_a_parede_a_meio_endireita_o_caminho() {
    let mut sim = SimWorld::new();
    regiao(&mut sim);
    let parede_e = sim
        .world_mut()
        .spawn((
            Name::new("Parede"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.3,
                    half_y: 4.0,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    let quem = agente(
        &mut sim,
        "Ag",
        (-4.0, 0.0),
        0.3,
        NavTarget::Point([4.0, 0.0]),
    );
    let mut bridge = PhysicsBridge::new();
    let (antes, _) = corre(&mut sim, &mut bridge, quem, 1, 20);
    let fim = *antes.last().expect("o caminho");
    assert!(
        fim.1.abs() > 0.3,
        "a fixtura precisa de que ele já esteja a dar a volta: {fim:?}"
    );
    sim.world_mut().despawn(parede_e);
    let (depois, _) = corre(&mut sim, &mut bridge, quem, 21, 300);
    let chegou = *depois.last().expect("o caminho");
    assert!(
        (chegou.0 - 4.0).hypot(chegou.1) < 0.2,
        "não chegou: {chegou:?}"
    );
    let maior_y = depois.iter().fold(0.0_f32, |m, p| m.max(p.1.abs()));
    assert!(
        maior_y <= fim.1.abs() + 1e-3,
        "continuou a dar a volta a uma parede que já não existe (|y| foi a {maior_y})"
    );
}

/// ⚠️ **Um Reset esquece a memória do agente** — e o caso que o mostra é a LINHA RECTA.
///
/// Com a parede, o ponto de partida fica longe do último troço do caminho velho e o agente replaneia
/// no 1.º tique de qualquer forma; sem parede o caminho velho PASSA pelo ponto de partida, o agente
/// segue-o sem replanear, e o «melhor que falta» velho (perto de zero, da chegada) faz o relógio do
/// «preso» encher: um `Stuck` que nenhuma corrida limpa produz. ⇒ a fixtura é a recta.
#[test]
fn um_reset_numa_linha_recta_nao_inventa_um_preso() {
    let mut sim = SimWorld::new();
    regiao(&mut sim);
    let quem = agente(
        &mut sim,
        "Ag",
        (-4.0, 0.0),
        0.3,
        NavTarget::Point([4.0, 0.0]),
    );
    let mut bridge = PhysicsBridge::new();
    let (c1, f1) = corre(&mut sim, &mut bridge, quem, 1, 240);
    assert_eq!(f1, vec![Event::Arrived]);
    bridge.dispatch(&mut sim, false, 0);
    let (c2, f2) = corre(&mut sim, &mut bridge, quem, 1, 240);
    assert_eq!(f2, f1, "a 2.ª corrida disse outra coisa");
    assert_eq!(c2, c1, "a 2.ª corrida andou outro caminho");
}
