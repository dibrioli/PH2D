//! ⭐ (o achado da W13) **Um corpo LARGO que anda** — o desvio local (ORCA) escolhe a velocidade mais
//! perto da pedida e não contorna; a malha só o vê PARADO (plano 30 §22.5).

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, NavTarget, PhysicsBridge, RigidBody};

use super::nav_desvio::{R, agente, dist, pos, regiao};

/// Uma barreira cinemática `2·hx × 2·hy` em `c`.
fn barreira(sim: &mut SimWorld, c: (f32, f32), hx: f32, hy: f32) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new("Barreira"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: hx,
                    half_y: hy,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(c.0, c.1)),
        ))
        .id()
}

/// A corrida: o agente de `(-6, 0)` para `(6, 0)`, a barreira (`0,3 × 2·hy`) a andar a `vx` m/s a
/// partir de `x0`. Devolve o tique em que chegou (`None` = não chegou), a menor folga à barreira
/// ENQUANTO ela anda (a distância ao rectângulo menos o raio) e o pior recuo atrás da partida.
fn corrida(hy: f32, x0: f32, vx: f32, desvio: bool, contorno: bool) -> (Option<u64>, f32, f32) {
    let mut sim = SimWorld::new();
    regiao(&mut sim);
    let b = barreira(&mut sim, (x0, 0.0), 0.15, hy);
    let quem = agente(
        &mut sim,
        "A",
        (-6.0, 0.0),
        NavTarget::Point([6.0, 0.0]),
        desvio,
    );
    let mut bridge = PhysicsBridge::new();
    bridge.set_nav_detour(contorno);
    let (mut chegou, mut folga, mut recuo) = (None, f32::INFINITY, 0.0f32);
    for t in 1..=900_u64 {
        let bx = x0 + vx * t as f32 / 60.0;
        sim.world_mut()
            .get_mut::<Transform>(b)
            .expect("a barreira")
            .translation
            .x = bx;
        bridge.dispatch(&mut sim, true, t);
        let p = pos(&sim, quem);
        let (dx, dy) = (
            ((p.0 - bx).abs() - 0.15).max(0.0),
            (p.1.abs() - hy).max(0.0),
        );
        folga = folga.min((dx * dx + dy * dy).sqrt() - R);
        recuo = recuo.max(-6.0 - p.0);
        if chegou.is_none() && dist(p, (6.0, 0.0)) < 0.15 {
            chegou = Some(t);
        }
    }
    (chegou, folga, recuo)
}

/// ⭐ (W14) **Um corpo LARGO que vem de frente é contornado** — o desvio vê-o em POLÍGONO com a
/// velocidade dele, e o agente desliza à volta, com a folga do que ele anda num horizonte. Medido antes
/// (a fileira de discos da W6): nenhum dos três chegava — colado à frente da barreira, ou empurrado `6 m`.
/// ⭐ (plano 30 §25, B) E o CAMINHO contorna-o pela tangente: os três chegam em `≤ 2×` o CONTROLO
/// parado (a mesma barreira quieta é parede da malha). Medido: `194 · 200 · 207` contra `194 · 198`;
/// sem o contorno (o CONTROLO da cura) `213 · 578 · 311` — a de `3 m` lenta passa os `2×`.
/// CONTROLO: o mesmo agente SEM desvio fica preso (a cena contém a armadilha). E à frente, no mesmo
/// sentido, continua a passar.
#[test]
fn um_corpo_largo_que_vem_de_frente_e_contornado() {
    let parado = |hy| corrida(hy, 3.0, 0.0, true, true).0.expect("o CONTROLO parado chega");
    let mut lenta_sem_contorno = None;
    for (nome, hy, vx) in [
        ("1,2 m a 0,3 m/s", 0.6, -0.3),
        ("3 m a 0,3 m/s", 1.5, -0.3),
        ("3 m a 1 m/s", 1.5, -1.0),
    ] {
        let controlo = parado(hy);
        let (chegou, folga, recuo) = corrida(hy, 3.0, vx, true, true);
        let (sem_contorno, _, _) = corrida(hy, 3.0, vx, true, false);
        eprintln!(
            "de frente, {nome}: chegou {chegou:?} (sem o contorno {sem_contorno:?}, parado {controlo}), \
             folga {folga:.3}, recuo {recuo:.3}"
        );
        assert!(
            chegou.is_some_and(|t| t <= 2 * controlo),
            "{nome}: chegou {chegou:?}, o CONTROLO parado {controlo}"
        );
        assert!(folga > 0.05, "{nome}: roçou a barreira (folga {folga})");
        assert!(recuo <= 0.0, "{nome}: empurrado {recuo} m para trás");
        if hy == 1.5 && vx == -0.3 {
            lenta_sem_contorno = sem_contorno.map(|t| (t, controlo));
        }
    }
    // A fixtura contém o fenómeno: sem o contorno do caminho, a barreira larga e lenta passa os `2×`.
    let (t, controlo) = lenta_sem_contorno.expect("sem o contorno também chega");
    assert!(t > 2 * controlo, "sem o contorno: {t} contra {controlo}");
    let (sem, _, _) = corrida(1.5, 3.0, -0.3, false, true);
    assert_eq!(sem, None, "o CONTROLO sem desvio passou");
    let (a_frente, _, _) = corrida(1.5, -3.0, 0.3, true, true);
    assert!(a_frente.is_some_and(|t| t <= 230), "à frente: {a_frente:?}");
}

/// Um cinemático com a forma `forma`, em `c`.
fn corpo_com(sim: &mut SimWorld, c: (f32, f32), forma: ColliderShape) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new("Corpo"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: forma,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(c.0, c.1)),
        ))
        .id()
}

/// A distância de `p` ao segmento `a`–`b`.
fn ao_segmento(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let t = (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
    dist(p, (a.0 + t * dx, a.1 + t * dy))
}

/// ⭐ (W14) **A forma inteira de uma CÁPSULA que anda, e um TORNIQUETE que roda** — a cápsula entra no
/// desvio pelo octógono circunscrito de cada ponta (contém-na), e a velocidade de cada ponto de um corpo
/// que roda é `ω ×` o braço (as pontas de uma barra que gira andam mais que o centro, parado). A folga
/// medida é à forma EXACTA.
#[test]
fn uma_capsula_que_anda_e_um_torniquete_que_roda_sao_contornados_pela_forma() {
    // A cápsula deitada em `y` (meia-altura `1,2`, raio `0,3`), a vir de frente a `0,3 m/s`.
    let mut sim = SimWorld::new();
    regiao(&mut sim);
    let cap = corpo_com(
        &mut sim,
        (3.0, 0.0),
        ColliderShape::Capsule {
            half_height: 1.2,
            radius: 0.3,
        },
    );
    let quem = agente(
        &mut sim,
        "A",
        (-6.0, 0.0),
        NavTarget::Point([6.0, 0.0]),
        true,
    );
    let mut b = PhysicsBridge::new();
    let (mut chegou, mut folga) = (None, f32::INFINITY);
    for t in 1..=900_u64 {
        let x = 3.0 - 0.3 * t as f32 / 60.0;
        sim.world_mut()
            .get_mut::<Transform>(cap)
            .expect("a cápsula")
            .translation
            .x = x;
        b.dispatch(&mut sim, true, t);
        let p = pos(&sim, quem);
        folga = folga.min(ao_segmento(p, (x, -1.2), (x, 1.2)) - 0.3 - R);
        if chegou.is_none() && dist(p, (6.0, 0.0)) < 0.15 {
            chegou = Some(t);
        }
    }
    eprintln!("a cápsula: chegou {chegou:?}, folga {folga:.3}");
    assert!(chegou.is_some(), "a cápsula: não chegou");
    // A folga do que ela anda num horizonte é `0,3 m`: o octógono contém-na, logo não se come.
    assert!(folga > 0.2, "a cápsula: folga {folga}");

    // O torniquete: uma barra `3 m` a rodar no sítio a `0,5 rad/s` no meio do caminho.
    let mut sim = SimWorld::new();
    regiao(&mut sim);
    let barra = barreira(&mut sim, (0.0, 0.0), 0.15, 1.5);
    let quem = agente(
        &mut sim,
        "A",
        (-6.0, 0.0),
        NavTarget::Point([6.0, 0.0]),
        true,
    );
    let mut b = PhysicsBridge::new();
    let (mut chegou, mut folga) = (None, f32::INFINITY);
    for t in 1..=1_200_u64 {
        let ang = 0.5 * t as f32 / 60.0;
        sim.world_mut()
            .get_mut::<Transform>(barra)
            .expect("a barra")
            .rotation = ang;
        b.dispatch(&mut sim, true, t);
        let p = pos(&sim, quem);
        let (s, c) = (libm::sinf(ang), libm::cosf(ang));
        // A barra deitada em `y`, rodada de `ang`: o segmento do eixo dela, e a meia-espessura.
        let (a, z) = ((1.5 * s, -1.5 * c), (-1.5 * s, 1.5 * c));
        folga = folga.min(ao_segmento(p, a, z) - 0.15 - R);
        if chegou.is_none() && dist(p, (6.0, 0.0)) < 0.15 {
            chegou = Some(t);
        }
    }
    eprintln!("o torniquete: chegou {chegou:?}, folga {folga:.3}");
    assert!(chegou.is_some(), "o torniquete: não chegou");
    assert!(folga > 0.0, "o torniquete bateu no agente: folga {folga}");
}
