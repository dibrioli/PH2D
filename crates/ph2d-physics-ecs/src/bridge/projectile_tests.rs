//! Gates da ponte do projéctil — a lei contra a `rapier` de verdade.

use super::*;
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_projectile::ProjectileLaw;

const DT: f32 = 1.0 / 60.0;

fn cena(law: ProjectileLaw, alvo: u64, em: Vec2, angulo: f32) -> (SimWorld, PhysicsBridge, Entity) {
    let mut sim = SimWorld::new();
    let mut t = Transform::from_translation(em);
    t.rotation = angulo;
    let quem = sim
        .world_mut()
        .spawn((
            Name::new("Bala"),
            crate::RigidBody {
                kind: crate::BodyKind::Kinematic,
            },
            crate::Collider {
                shape: crate::ColliderShape::Ball { radius: 0.1 },
                ..crate::Collider::default()
            },
            ProjectileMotion::from_law(law, alvo),
            t,
        ))
        .id();
    (sim, PhysicsBridge::new(), quem)
}

fn parede(sim: &mut SimWorld, em: Vec2, meio: (f32, f32)) {
    sim.world_mut().spawn((
        Name::new("Parede"),
        crate::RigidBody {
            kind: crate::BodyKind::Static,
        },
        crate::Collider {
            shape: crate::ColliderShape::Cuboid {
                half_x: meio.0,
                half_y: meio.1,
            },
            ..crate::Collider::default()
        },
        Transform::from_translation(em),
    ));
}

fn pos(sim: &SimWorld, quem: Entity) -> Vec2 {
    sim.world()
        .get::<Transform>(quem)
        .expect("a bala")
        .translation
}

fn corre(sim: &mut SimWorld, bridge: &mut PhysicsBridge, tiques: u64) {
    for t in 1..=tiques {
        bridge.dispatch(sim, true, t);
    }
}

/// ⭐⭐ **Ele voa sozinho, para onde o corpo está virado.**
#[test]
fn ele_voa_sozinho_para_onde_o_corpo_esta_virado() {
    let (mut sim, mut bridge, quem) = cena(
        ProjectileLaw {
            initial_speed: 6.0,
            ..ProjectileLaw::default()
        },
        0,
        Vec2::new(0.0, 0.0),
        0.0,
    );
    corre(&mut sim, &mut bridge, 30);
    let p = pos(&sim, quem);
    assert!(
        (p.x - 3.0).abs() < 0.05,
        "meio segundo a 6 m/s ⇒ 3 m: {p:?}"
    );
    assert!(p.y.abs() < 1.0e-3, "sem gravidade ele nao cai: {p:?}");
}

/// ⭐⭐⭐ **Ele RICOCHETEIA numa parede, e o orçamento do tique atravessa o salto.**
#[test]
fn ele_ricocheteia_na_parede() {
    let (mut sim, mut bridge, quem) = cena(
        ProjectileLaw {
            initial_speed: 6.0,
            max_bounces: 4,
            bounciness: 1.0,
            ..ProjectileLaw::default()
        },
        0,
        Vec2::new(0.0, 0.0),
        0.0,
    );
    // Uma parede vertical com a face em x = 2.
    parede(&mut sim, Vec2::new(3.0, 0.0), (1.0, 20.0));
    corre(&mut sim, &mut bridge, 60);
    let p = pos(&sim, quem);
    assert!(
        p.x < 0.0,
        "ele tinha de voltar para tras depois do salto: x = {}",
        p.x
    );
}

/// ⭐⭐⭐ **O ALCANCE mata o voo, e conta METROS — não tempo.**
///
/// ⚠️ É a metade que separa este componente do `Lifetime` do #12. O gate corre **duas** balas com
/// rapidezes diferentes e exige que elas morram na mesma DISTÂNCIA, em tiques diferentes.
#[test]
fn o_alcance_conta_metros_e_nao_tempo() {
    let mut onde = Vec::new();
    for rapidez in [4.0_f32, 12.0] {
        let (mut sim, mut bridge, quem) = cena(
            ProjectileLaw {
                initial_speed: rapidez,
                range: 2.0,
                ..ProjectileLaw::default()
            },
            0,
            Vec2::new(0.0, 0.0),
            0.0,
        );
        // Corre até a ponte anunciar o fim, ou 300 tiques.
        let mut t = 0;
        while t < 300 && bridge.projectile_done().is_empty() {
            t += 1;
            bridge.dispatch(&mut sim, true, t);
        }
        assert!(
            !bridge.projectile_done().is_empty(),
            "a {rapidez} m/s ele nunca acabou o voo"
        );
        onde.push((rapidez, t, pos(&sim, quem).x));
    }
    for (rapidez, _, x) in &onde {
        assert!(
            (*x - 2.0).abs() < 0.25,
            "a {rapidez} m/s ele morreu em x = {x} e o alcance e' 2 m"
        );
    }
    assert!(
        onde[0].1 > onde[1].1 * 2,
        "a bala LENTA tem de demorar MUITO mais tiques a percorrer os mesmos metros: {onde:?}"
    );
}

/// ⚠️ **O que o mundo não deixou andar não conta para o alcance** — uma bala barrada não percorreu
/// nada, e cobrar-lhe alcance mata-a mais cedo por ter batido.
#[test]
fn uma_bala_barrada_nao_gasta_alcance() {
    let (mut sim, mut bridge, _) = cena(
        ProjectileLaw {
            initial_speed: 6.0,
            range: 100.0,
            // ⚠️ Sem saltos ela ACABA ao bater — o que se mede aqui é o alcance, então o voo tem
            // de sobreviver ao toque: um tecto alto e uma parede à frente.
            max_bounces: 200,
            bounciness: 0.0,
            ..ProjectileLaw::default()
        },
        0,
        Vec2::new(0.0, 0.0),
        0.0,
    );
    parede(&mut sim, Vec2::new(1.5, 0.0), (1.0, 20.0));
    corre(&mut sim, &mut bridge, 120);
    assert!(
        bridge.projectile_done().is_empty(),
        "ela morreu de alcance encostada a uma parede — o alcance esta' a contar o pedido, nao o andado"
    );
}

/// ⭐⭐ **A flecha aponta para onde voa** — e num arco ela desce com a trajectória.
#[test]
fn a_flecha_aponta_para_onde_voa() {
    let (mut sim, mut bridge, quem) = cena(
        ProjectileLaw {
            initial_speed: 6.0,
            gravity: 20.0,
            face_velocity: true,
            ..ProjectileLaw::default()
        },
        0,
        Vec2::new(0.0, 0.0),
        0.0,
    );
    corre(&mut sim, &mut bridge, 30);
    let a = sim.world().get::<Transform>(quem).expect("a bala").rotation;
    assert!(
        a < -0.3,
        "no arco a flecha tem de apontar para BAIXO: {a} rad"
    );
}

/// ⭐⭐ **Ele persegue um alvo NOMEADO.**
///
/// ⚠️ O alvo é o `stable_name_id`, e é isso que o faz sobreviver a um `Ctrl+Z`.
///
/// ⚠️⚠️ **A régua é a APROXIMAÇÃO contra um CONTROLO, e não a posição num instante.** A 1.ª
/// redacção media `y > 1` ao fim de 40 tiques e lia `−0,017` sobre um homing que estava a
/// funcionar: com aceleração forte a bala **ultrapassa o alvo e volta**, então uma fotografia num
/// instante mede a fase da órbita, não a perseguição. *O que a perseguição É: chegar mais perto do
/// que a mesma bala sem ela.*
#[test]
fn ele_persegue_um_alvo_nomeado() {
    const ALVO: Vec2 = Vec2::new(3.0, 6.0);
    let perto_de = |accel: f32| {
        let (mut sim, mut bridge, quem) = cena(
            ProjectileLaw {
                initial_speed: 6.0,
                homing_accel: accel,
                ..ProjectileLaw::default()
            },
            ph2d_ecs::stable_name_id("Alvo"),
            Vec2::new(0.0, 0.0),
            0.0,
        );
        sim.world_mut()
            .spawn((Name::new("Alvo"), Transform::from_translation(ALVO)));
        let mut minima = f32::INFINITY;
        for t in 1..=60 {
            bridge.dispatch(&mut sim, true, t);
            let p = pos(&sim, quem);
            minima = minima.min(((p.x - ALVO.x).powi(2) + (p.y - ALVO.y).powi(2)).sqrt());
        }
        minima
    };
    let com = perto_de(300.0);
    let sem = perto_de(0.0);
    assert!(
        com < sem * 0.5,
        "a perseguicao tinha de o levar MUITO mais perto: {com:.3} m com, {sem:.3} m sem"
    );
    // ⚠️ **A barra é MEDIDA, não escolhida** (§0.0): nesta fixtura a aproximação mínima é
    // `0,635 m` — a bala chega ao alvo em arco e o ponto mais perto é onde ela vira. A folga até
    // `1,0` é o que separa *«ela chega lá»* de *«ela passa ao largo»* (sem perseguição são
    // `5,89 m`), e apertá-la abaixo do medido seria calibrar no ruído da fixtura.
    assert!(com < 1.0, "e perto de verdade: {com:.3} m (medido 0,635)");
}

/// ⚠️ **Um alvo que não existe não parte nada** — o campo fica escrito no ficheiro quando alguém
/// apaga o objecto, e um `unwrap` ali mataria o quadro.
#[test]
fn um_alvo_que_nao_existe_nao_parte_nada() {
    let (mut sim, mut bridge, quem) = cena(
        ProjectileLaw {
            initial_speed: 6.0,
            homing_accel: 400.0,
            ..ProjectileLaw::default()
        },
        ph2d_ecs::stable_name_id("NinguemComEsteNome"),
        Vec2::new(0.0, 0.0),
        0.0,
    );
    corre(&mut sim, &mut bridge, 30);
    let p = pos(&sim, quem);
    assert!(p.x.is_finite() && p.y.is_finite() && p.x > 2.0, "{p:?}");
}

/// ⭐⭐⭐ **A memória ATRAVESSA os tiques** — sem isso a bala re-nasce a cada quadro e nunca cai.
#[test]
fn a_memoria_do_voo_atravessa_os_tiques() {
    let (mut sim, mut bridge, quem) = cena(
        ProjectileLaw {
            initial_speed: 6.0,
            gravity: 20.0,
            ..ProjectileLaw::default()
        },
        0,
        Vec2::new(0.0, 0.0),
        0.0,
    );
    corre(&mut sim, &mut bridge, 30);
    let p = pos(&sim, quem);
    // Meio segundo a −20 m/s² ⇒ ~−2,5 m. Com a memória perdida a cada tique a queda seria
    // `30 × ½·g·dt²` ≈ −0,08 m: duas ordens de grandeza abaixo.
    assert!(
        p.y < -1.5,
        "a queda foi {}, logo a velocidade nao esta' a acumular entre tiques",
        p.y
    );
    let _ = DT;
}
