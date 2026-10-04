//! **O DESVIO entre agentes de ponta a ponta** (plano 30, W5) — vários agentes com um
//! [`TopDownPlayer`] cada, pela porta do produto (`PhysicsBridge::dispatch`).
//!
//! A lei do ORCA tem gates próprios na folha (`ph2d-orca`: o Godot passo a passo e o banco de
//! cenários); aqui mede-se o que só a COSTURA pode partir: a ponte vê os outros corpos, as paredes da
//! malha do raio certo, a velocidade certa de cada um, escreve a intenção antes do mover — e o
//! scrub devolve a mesma multidão. Cada gate tem o CONTROLO com o desvio desligado ao lado: sem ele
//! não se sabe se é o desvio que faz passar.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, NavAgent, NavRegion, NavTarget, PhysicsBridge, RigidBody,
    TopDownPlayer,
};
use ph2d_topdown::{TopDownLaw, direction::DirectionMode};

pub(crate) const R: f32 = 0.3;

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

pub(crate) fn regiao(sim: &mut SimWorld) {
    sim.world_mut().spawn((
        Name::new("Região"),
        NavRegion {
            half_extents: [8.0, 6.0],
            obstacle_layers: u8::MAX,
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
}

pub(crate) fn mover() -> TopDownPlayer {
    TopDownPlayer::from_law(TopDownLaw {
        default_controls: false,
        direction: DirectionMode::Free,
        ..TopDownLaw::default()
    })
}

pub(crate) fn corpo() -> (RigidBody, Collider) {
    (
        RigidBody {
            kind: BodyKind::Kinematic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: R },
            ..Collider::default()
        },
    )
}

/// Um agente em `em` a ir para `alvo`, com ou sem desvio.
pub(crate) fn agente(
    sim: &mut SimWorld,
    nome: &str,
    em: (f32, f32),
    alvo: NavTarget,
    desvio: bool,
) -> Entity {
    let (rb, col) = corpo();
    sim.world_mut()
        .spawn((
            Name::new(nome),
            rb,
            col,
            mover(),
            NavAgent {
                target: alvo,
                arrive_distance: 0.1,
                avoidance: desvio,
                ..NavAgent::default()
            },
            Transform::from_translation(Vec2::new(em.0, em.1)),
        ))
        .id()
}

pub(crate) fn pos(sim: &SimWorld, e: Entity) -> (f32, f32) {
    let t = sim.world().get::<Transform>(e).expect("o corpo");
    (t.translation.x, t.translation.y)
}

pub(crate) fn dist(a: (f32, f32), b: (f32, f32)) -> f32 {
    ((a.0 - b.0) * (a.0 - b.0) + (a.1 - b.1) * (a.1 - b.1)).sqrt()
}

/// Corre `de..=ate` e devolve a posição de cada agente a cada tique.
pub(crate) fn corre(
    sim: &mut SimWorld,
    bridge: &mut PhysicsBridge,
    quem: &[Entity],
    de: u64,
    ate: u64,
) -> Vec<Vec<(f32, f32)>> {
    let mut out = Vec::new();
    for t in de..=ate {
        bridge.dispatch(sim, true, t);
        out.push(quem.iter().map(|&e| pos(sim, e)).collect());
    }
    out
}

struct Desfecho {
    /// O tique em que TODOS estavam a menos de `0,15 m` do alvo.
    chegaram: Option<usize>,
    /// A menor distância entre dois centros.
    min_par: f32,
}

fn desfecho(corrida: &[Vec<(f32, f32)>], alvos: &[(f32, f32)]) -> Desfecho {
    let mut d = Desfecho {
        chegaram: None,
        min_par: f32::INFINITY,
    };
    for (t, ps) in corrida.iter().enumerate() {
        for i in 0..ps.len() {
            for j in i + 1..ps.len() {
                d.min_par = d.min_par.min(dist(ps[i], ps[j]));
            }
        }
        if d.chegaram.is_none() && ps.iter().zip(alvos).all(|(&p, &a)| dist(p, a) < 0.15) {
            d.chegaram = Some(t + 1);
        }
    }
    d
}

/// Dois agentes frente a frente, no MESMO eixo — o empate que o Godot não desfaz.
fn frente_a_frente(desvio: bool) -> Desfecho {
    let mut sim = SimWorld::new();
    regiao(&mut sim);
    let alvos = [(4.0, 0.0), (-4.0, 0.0)];
    let a = agente(
        &mut sim,
        "A",
        (-4.0, 0.0),
        NavTarget::Point([4.0, 0.0]),
        desvio,
    );
    let b = agente(
        &mut sim,
        "B",
        (4.0, 0.0),
        NavTarget::Point([-4.0, 0.0]),
        desvio,
    );
    let corrida = corre(&mut sim, &mut PhysicsBridge::new(), &[a, b], 1, 600);
    desfecho(&corrida, &alvos)
}

/// ⚠️ **A folga de contacto**, em metros. O mover acelera em RAMPA e o desvio escolhe a velocidade
/// como se ela fosse instantânea, logo podia haver invasão — e medido NÃO há: o par mais perto a
/// `2r + 0,004` (frente a frente `+0,005`, porta `+0,004`) e o centro mais perto da parede a `r + 0,006`.
/// `1e-3` é o arredondamento do `f32` a `~5 m`, com folga; sem o desvio a invasão é `0,06` (o CONTROLO).
const FOLGA: f32 = 1e-3;

/// ⭐⭐⭐ **Frente a frente, os dois cruzam-se e chegam** — e sem o desvio NÃO.
#[test]
fn frente_a_frente_os_dois_cruzam_se_e_chegam() {
    let com = frente_a_frente(true);
    let sem = frente_a_frente(false);
    eprintln!(
        "com desvio: chegaram {:?}, par mín {:.4} · sem: chegaram {:?}, par mín {:.4}",
        com.chegaram, com.min_par, sem.chegaram, sem.min_par
    );
    assert!(com.chegaram.is_some(), "com desvio, não chegaram");
    assert!(
        com.min_par >= 2.0 * R - FOLGA,
        "invadiram-se: {}",
        com.min_par
    );
    // CONTROLO: sem o desvio, os dois corpos batem de frente e ficam.
    assert_eq!(
        sem.chegaram, None,
        "sem desvio chegaram — a fixtura não contém o fenómeno"
    );
}

/// Oito agentes à esquerda de uma parede com uma porta, cada um para o seu ponto à direita.
fn a_porta(desvio: bool) -> (Desfecho, Vec<Vec<(f32, f32)>>) {
    let mut sim = SimWorld::new();
    regiao(&mut sim);
    // A parede em `x = 0`, com uma porta de `1,6 m` (cabem dois corpos de `0,6`, folgados).
    parede(&mut sim, (0.0, 3.4), (0.2, 2.6));
    parede(&mut sim, (0.0, -3.4), (0.2, 2.6));
    let mut quem = Vec::new();
    let mut alvos = Vec::new();
    for k in 0..8 {
        let y = -2.8 + 0.8 * k as f32;
        let x = -4.0 - 1.0 * (k % 2) as f32;
        let alvo = (5.0, -2.8 + 0.8 * (7 - k) as f32);
        alvos.push(alvo);
        quem.push(agente(
            &mut sim,
            &format!("A{k}"),
            (x, y),
            NavTarget::Point([alvo.0, alvo.1]),
            desvio,
        ));
    }
    let corrida = corre(&mut sim, &mut PhysicsBridge::new(), &quem, 1, 1200);
    (desfecho(&corrida, &alvos), corrida)
}

/// ⭐⭐⭐ **Oito pela porta: todos passam, sem se invadirem e sem entrar na parede** — o smoke `=2`.
#[test]
fn oito_pela_porta_passam_todos() {
    let (com, corrida) = a_porta(true);
    let (sem, _) = a_porta(false);
    eprintln!(
        "com desvio: chegaram {:?}, par mín {:.4} · sem: chegaram {:?}, par mín {:.4}",
        com.chegaram, com.min_par, sem.chegaram, sem.min_par
    );
    assert!(com.chegaram.is_some(), "com desvio, nem todos passaram");
    assert!(
        com.min_par >= 2.0 * R - FOLGA,
        "invadiram-se: {}",
        com.min_par
    );
    // Nenhum centro a menos de `r` das paredes. ⚠️ A DISTÂNCIA ao rectângulo, nunca uma caixa
    // alargada: a quina da área recuada é REDONDA (a 1.ª régua acusou um centro a `0,311 m` da quina
    // — a lição da W4 a repetir-se).
    let mut perto = f32::INFINITY;
    for ps in &corrida {
        for &(x, y) in ps {
            let dx = (x.abs() - 0.2).max(0.0);
            let dy = (0.8 - y.abs()).max(0.0);
            perto = perto.min((dx * dx + dy * dy).sqrt());
        }
    }
    eprintln!("o centro mais perto de uma parede: {perto:.4} m (r = {R})");
    assert!(perto >= R - FOLGA, "um centro dentro da parede: {perto}");
    // CONTROLO: sem o desvio, os corpos ATRAVESSAM-SE na porta (os movers cinemáticos não se
    // bloqueiam uns aos outros a fundo) — é a invasão que o desvio evita.
    assert!(
        sem.min_par < 2.0 * R - FOLGA,
        "sem desvio ninguém se invadiu: {}",
        sem.min_par
    );
}

/// ⭐⭐ **O scrub devolve a mesma MULTIDÃO** — o desvio lê a velocidade de todos, e ela vai no anel
/// com o mover; um scrub que a esquecesse daria outra corrida.
#[test]
fn um_scrub_devolve_a_mesma_multidao() {
    const MEIO: u64 = 150;
    let mut sim = SimWorld::new();
    regiao(&mut sim);
    let quem: Vec<Entity> = [
        ((-4.0, 0.0), (4.0, 0.2)),
        ((4.0, 0.0), (-4.0, -0.2)),
        ((0.0, -4.0), (0.0, 4.0)),
        ((0.2, 4.0), (0.0, -4.0)),
    ]
    .iter()
    .enumerate()
    .map(|(k, &(de, para))| {
        agente(
            &mut sim,
            &format!("A{k}"),
            de,
            NavTarget::Point([para.0, para.1]),
            true,
        )
    })
    .collect();
    let mut bridge = PhysicsBridge::new();
    let primeira = corre(&mut sim, &mut bridge, &quem, 1, 300);
    bridge.dispatch(&mut sim, false, MEIO);
    let agora: Vec<(f32, f32)> = quem.iter().map(|&e| pos(&sim, e)).collect();
    assert_eq!(agora, primeira[(MEIO - 1) as usize], "o scrub");
    let resto = corre(&mut sim, &mut bridge, &quem, MEIO + 1, 300);
    assert_eq!(
        resto,
        primeira[MEIO as usize..].to_vec(),
        "o resto da corrida"
    );
    // A fixtura contém o fenómeno: os quatro cruzam-se no meio (alguém chega a menos de 1 m).
    let perto = primeira
        .iter()
        .any(|ps| (0..4).any(|i| (i + 1..4).any(|j| dist(ps[i], ps[j]) < 1.0)));
    assert!(perto, "ninguém se cruzou — o desvio não correu");
}

/// ⭐⭐ **O perseguidor não se desvia do PRÓPRIO alvo** — chega a encostar no herói.
#[test]
fn o_perseguidor_chega_ao_heroi_que_nao_se_desvia() {
    let mut sim = SimWorld::new();
    regiao(&mut sim);
    let (rb, col) = corpo();
    let heroi = sim
        .world_mut()
        .spawn((
            Name::new("Hero"),
            rb,
            col,
            mover(),
            Transform::from_translation(Vec2::new(3.0, 0.0)),
        ))
        .id();
    let quem = sim
        .world_mut()
        .spawn((
            Name::new("Perseguidor"),
            corpo().0,
            corpo().1,
            mover(),
            NavAgent {
                target: NavTarget::Named(stable_name_id("Hero")),
                arrive_distance: 2.0 * R + 0.02,
                ..NavAgent::default()
            },
            Transform::from_translation(Vec2::new(-3.0, 0.0)),
        ))
        .id();
    let corrida = corre(&mut sim, &mut PhysicsBridge::new(), &[quem, heroi], 1, 400);
    let chegou = corrida
        .iter()
        .position(|ps| dist(ps[0], ps[1]) < 2.0 * R + 0.03)
        .map(|t| t + 1);
    let fim = corrida.last().expect("a corrida");
    let encosto = dist(fim[0], fim[1]);
    eprintln!("perseguidor chegou no tique {chegou:?}, acabou a {encosto:.4} m do herói");
    assert!(chegou.is_some(), "não chegou ao herói");
    // ⚠️ A régua é o ENCOSTO, MEDIDO com a mutação ao lado (a ponte sem o «ignora o alvo»): ele acaba
    // a `2r + 0,006` a ignorar o herói e a `2r + 0,014` a desviar-se dele (chega no tique `81` contra
    // `92`, perto demais para ser régua). A barra fica no meio.
    assert!(
        encosto < 2.0 * R + 0.01,
        "desviou-se do próprio alvo: acabou a {encosto}"
    );
}

/// ⭐⭐ **Um corpo que anda e NÃO é agente também se evita** — o herói parado no caminho de quem não o
/// persegue. Sem desvio os movers cinemáticos invadem-se (o CONTROLO); com ele, o agente contorna-o.
#[test]
fn um_corpo_que_nao_e_agente_tambem_se_evita() {
    let corrida = |desvio: bool| {
        let mut sim = SimWorld::new();
        regiao(&mut sim);
        let (rb, col) = corpo();
        let heroi = sim
            .world_mut()
            .spawn((
                Name::new("Hero"),
                rb,
                col,
                mover(),
                Transform::from_translation(Vec2::new(0.0, 0.0)),
            ))
            .id();
        let quem = agente(
            &mut sim,
            "A",
            (-4.0, 0.0),
            NavTarget::Point([4.0, 0.0]),
            desvio,
        );
        let c = corre(&mut sim, &mut PhysicsBridge::new(), &[quem, heroi], 1, 400);
        let min = c
            .iter()
            .map(|ps| dist(ps[0], ps[1]))
            .fold(f32::INFINITY, f32::min);
        // Quanto ele já saiu do eixo quando chega a `1,5 m` do herói — ANTES de lhe tocar.
        let antes = c
            .iter()
            .find(|ps| dist(ps[0], ps[1]) < 1.5)
            .map_or(0.0, |ps| ps[0].1.abs());
        let fim = c.last().expect("a corrida")[0];
        (min, antes, dist(fim, (4.0, 0.0)))
    };
    let (com, com_antes, chegou) = corrida(true);
    let (sem, sem_antes, _) = corrida(false);
    eprintln!(
        "com desvio: mais perto {com:.4}, fora do eixo a 1,5 m {com_antes:.4}, chega a {chegou:.4} · \
         sem: mais perto {sem:.4}, fora do eixo {sem_antes:.4}"
    );
    assert!(com >= 2.0 * R - FOLGA, "invadiu o herói: {com}");
    assert!(chegou < 0.15, "não chegou: ficou a {chegou}");
    assert!(
        com_antes > 0.05,
        "não contornou ANTES de tocar: {com_antes}"
    );
    // CONTROLO: sem o desvio ele vai pelo eixo até bater (e o mover desliza à volta dele — parado,
    // o herói não é invadido nem assim).
    assert!(
        sem_antes < 1e-3,
        "sem desvio saiu do eixo — a fixtura não contém o fenómeno: {sem_antes}"
    );
}

/// ⭐⭐ **Um agente SEM desvio obriga o outro a fazer o desvio INTEIRO** — frente a frente, um com e
/// outro sem: o que desvia não pode contar com a metade do outro (ele vai a direito).
#[test]
fn quem_nao_desvia_obriga_o_outro_a_desviar_por_inteiro() {
    let mut sim = SimWorld::new();
    regiao(&mut sim);
    let a = agente(
        &mut sim,
        "A",
        (-4.0, 0.0),
        NavTarget::Point([4.0, 0.0]),
        true,
    );
    let b = agente(
        &mut sim,
        "B",
        (4.0, 0.05),
        NavTarget::Point([-4.0, 0.05]),
        false,
    );
    let c = corre(&mut sim, &mut PhysicsBridge::new(), &[a, b], 1, 600);
    let d = desfecho(&c, &[(4.0, 0.0), (-4.0, 0.05)]);
    eprintln!("misto: chegaram {:?}, par mín {:.4}", d.chegaram, d.min_par);
    assert!(d.chegaram.is_some(), "não chegaram");
    // ⚠️ A régua é NÃO SE ENCOSTAREM, medida com a mutação ao lado (a ponte a tratar quem não desvia
    // como se fizesse metade): encostados, os dois movers ficam a `2r + 0,0040` (o offset de fábrica
    // do controlador de personagem); com o desvio inteiro, a `2r + 0,0096`. O quanto `A` sai do eixo
    // NÃO distingue (`0,549` contra `0,566`). A barra fica no meio.
    assert!(d.min_par > 2.0 * R + 0.0068, "encostaram-se: {}", d.min_par);
}

/// ⭐⭐ **O desvio não empurra contra a parede** (Q1, a metade do desvio): `A` anda rente ao chão e `B`
/// vem de frente, por cima — o desvio de `A` (e o lado dele, a direita) aponta PARA o chão. Com as
/// paredes da malha no desvio, `A` aproxima-se do chão devagar e nunca lhe toca; sem elas, bate e
/// desliza (medido com a mutação — ver o relatório).
#[test]
fn o_desvio_nao_empurra_contra_a_parede() {
    let mut sim = SimWorld::new();
    regiao(&mut sim);
    // O chão: o topo em `y = −0,9`.
    parede(&mut sim, (0.0, -1.0), (6.0, 0.1));
    let y_a = -0.9 + R + 0.1;
    let a = agente(
        &mut sim,
        "A",
        (-4.0, y_a),
        NavTarget::Point([4.0, y_a]),
        true,
    );
    let b = agente(
        &mut sim,
        "B",
        (4.0, y_a + 0.2),
        NavTarget::Point([-4.0, y_a + 0.2]),
        true,
    );
    let c = corre(&mut sim, &mut PhysicsBridge::new(), &[a, b], 1, 600);
    let min = c
        .iter()
        .map(|ps| ps[0].1 - (-0.9) - R)
        .fold(f32::INFINITY, f32::min);
    let d = desfecho(&c, &[(4.0, y_a), (-4.0, y_a + 0.2)]);
    eprintln!(
        "A ao chão: folga mín {min:.5} m · par mín {:.4} · chegaram {:?}",
        d.min_par, d.chegaram
    );
    assert!(d.chegaram.is_some(), "não chegaram");
    assert!(d.min_par >= 2.0 * R - FOLGA, "invadiram-se: {}", d.min_par);
    // ⚠️ MEDIDA com a mutação ao lado (as paredes da malha fora do desvio): `A` desce até `0,0060 m`
    // do chão, contra `0,0275` com elas. A barra fica entre as duas.
    assert!(
        min >= 0.015,
        "o desvio empurrou A contra o chão: folga {min}"
    );
}

/// ⭐⭐ **Uma porta COMPRIDA a andar desvia-se pela FORMA, não por um círculo à volta dela** (o aberto
/// da W6): um agente passa rente a uma porta de `4 m × 0,2 m` que desliza devagar ao lado dele. Com
/// o círculo que a envolvia (`r ≈ 2 m`) ele era atirado para longe; com a fileira de discos ao
/// longo dela, segue quase a direito.
///
/// **Mutação que deve sangrar:** o corpo que anda voltar a ser um disco só.
#[test]
fn uma_porta_comprida_a_andar_desvia_se_pela_forma() {
    let mut sim = SimWorld::new();
    regiao(&mut sim);
    let porta = sim
        .world_mut()
        .spawn((
            Name::new("Porta"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 2.0,
                    half_y: 0.1,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(-1.0, 0.0)),
        ))
        .id();
    let y = 0.1 + R + 0.35;
    let quem = agente(&mut sim, "A", (-4.0, y), NavTarget::Point([4.0, y]), true);
    let mut bridge = PhysicsBridge::new();
    let mut fora = 0.0_f32;
    for t in 1..=300_u64 {
        // A porta desliza a `0,5 m/s` para a direita: um corpo que ANDA, não uma parede da malha.
        sim.world_mut()
            .get_mut::<Transform>(porta)
            .expect("a porta")
            .translation
            .x = -1.0 + 0.5 * t as f32 / 60.0;
        bridge.dispatch(&mut sim, true, t);
        fora = fora.max((pos(&sim, quem).1 - y).abs());
    }
    let fim = pos(&sim, quem);
    eprintln!("porta comprida: o mais longe do eixo {fora:.3} m, acaba em {fim:?}");
    assert!(
        fora < 0.25,
        "a porta desviou-o {fora} m — um círculo à volta dela, não a forma"
    );
    assert!(dist(fim, (4.0, y)) < 0.15, "não chegou: {fim:?}");
}
