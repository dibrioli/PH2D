//! **O desvio contra CORPOS que não são agentes** (os abertos da W5): a velocidade que um corpo empurrado
//! leva, e a forma inteira de um corpo composto. Os ajudantes são os de [`super::nav_desvio`].

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, NavTarget, PhysicsBridge, RigidBody};

use super::nav_desvio::{R, agente, corpo, corre, dist, mover, pos, regiao};

/// ⭐ (o aberto da W5) **Um corpo EMPURRADO por um golpe é visto a andar** — o desvio lê a velocidade que o
/// mover de vista de cima vai seguir, e essa é o comando MAIS o empurrão. Um herói acima do caminho
/// leva um golpe que o atira para baixo, ATRAVÉS do caminho do agente, e acaba longe do outro lado: o
/// agente segue a direito. Visto parado em cada tique (sem o empurrão), o herói parecia um obstáculo
/// no caminho. CONTROLO: o empurrão a zero (o herói fica parado fora do caminho).
#[test]
fn um_corpo_empurrado_por_um_golpe_e_visto_a_andar() {
    const Y0: f32 = 1.5;
    const K: f32 = 14.0;
    use ph2d_physics_ecs::{Damage, Health};
    let corrida = |k: f32| {
        let mut sim = SimWorld::new();
        regiao(&mut sim);
        let (rb, _) = corpo();
        let heroi = sim
            .world_mut()
            .spawn((
                Name::new("Hero"),
                rb,
                Collider {
                    shape: ColliderShape::Ball { radius: 0.4 },
                    ..Collider::default()
                },
                mover(),
                Health::default(),
                Transform::from_translation(Vec2::new(0.0, Y0)),
            ))
            .id();
        // A fonte do golpe, sensor, logo ACIMA do herói: o empurrão sai do centro dela para o dele.
        sim.world_mut().spawn((
            Name::new("Golpe"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.4 },
                is_sensor: true,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, Y0 + 0.4)),
            Damage {
                amount: 1.0,
                knockback: k,
                ..Damage::default()
            },
        ));
        let quem = agente(
            &mut sim,
            "A",
            (-4.0, 0.0),
            NavTarget::Point([5.0, 0.0]),
            true,
        );
        corre(&mut sim, &mut PhysicsBridge::new(), &[quem, heroi], 1, 90)
    };
    let (com, sem) = (corrida(K), corrida(0.0));
    let fora = |c: &[Vec<(f32, f32)>]| c.iter().map(|ps| ps[0].1.abs()).fold(0.0f32, f32::max);
    let (f_com, f_sem) = (fora(&com), fora(&sem));
    eprintln!("fora do eixo: com o golpe {f_com:.4} · sem {f_sem:.4}");
    // A fixtura contém o fenómeno: o herói CRUZA o caminho à frente do agente e acaba longe dele.
    let cruza = com
        .iter()
        .position(|ps| ps[1].1 < 0.0)
        .expect("o herói cruza o caminho");
    assert!(
        com[cruza][0].0 < -2.5 && com.last().expect("a corrida")[1].1 < -2.0,
        "o herói não cruzou à frente do agente: {:?}",
        com[cruza]
    );
    // Visto PARADO em cada tique, o herói que cruza parecia um obstáculo no caminho: o agente desviava
    // `0,80 m` para nada (medido, sem o empurrão na velocidade). A andar, ele já vai a sair.
    assert!(f_com < 0.05, "desviou-se de quem já ia a sair: {f_com}");
    assert!(f_sem < 0.05, "o CONTROLO desviou-se: {f_sem}");
}

/// ⭐ (o aberto da W5) **Um corpo COMPOSTO desvia-se pela forma inteira** — a peça (um filho só com
/// `Collider`) faz parte dele. Um carrinho que vem de frente com o corpo FORA do caminho e um braço
/// atravessado nele: o agente guarda distância do braço enquanto o carrinho anda (sem as peças no
/// desvio ENCOSTAVA nele), e chega quando ele pára. CONTROLO: o mesmo carrinho sem o braço.
#[test]
fn um_corpo_composto_desvia_se_pela_forma_inteira() {
    let corrida = |com_braco: bool, perseguido: bool| {
        let mut sim = SimWorld::new();
        regiao(&mut sim);
        let carro = sim
            .world_mut()
            .spawn((
                Name::new("Carro"),
                RigidBody {
                    kind: BodyKind::Kinematic,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.2 },
                    ..Collider::default()
                },
                Transform::from_translation(Vec2::new(2.0, 1.2)),
            ))
            .id();
        if com_braco {
            sim.world_mut().spawn((
                Name::new("Braço"),
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.1,
                        half_y: 0.6,
                    },
                    ..Collider::default()
                },
                Transform::from_translation(Vec2::new(0.0, -1.2)),
                ChildOf(carro),
            ));
        }
        if perseguido {
            // O ALVO de alguém é UM disco para o desvio — o que envolve o corpo E as peças.
            agente(
                &mut sim,
                "B",
                (-4.0, 4.5),
                NavTarget::Named(stable_name_id("Carro")),
                true,
            );
        }
        let quem = agente(
            &mut sim,
            "A",
            (-4.0, 0.0),
            NavTarget::Point([4.0, 0.0]),
            true,
        );
        let mut bridge = PhysicsBridge::new();
        let mut c = Vec::new();
        for t in 1..=400_u64 {
            // O carrinho vem de frente a `0,3 m/s` e PÁRA ao fim de 2 s (parado, vira parede da malha).
            sim.world_mut()
                .get_mut::<Transform>(carro)
                .expect("o carro")
                .translation
                .x = 2.0 - 0.3 * t.min(120) as f32 / 60.0;
            bridge.dispatch(&mut sim, true, t);
            c.push((pos(&sim, quem), pos(&sim, carro)));
        }
        // A folga ao braço (o rectângulo `0,2 × 1,2` debaixo do carro) enquanto o carro ANDA.
        let folga = c
            .iter()
            .take(120)
            .map(|&((ax, ay), (bx, by))| {
                let dx = ((ax - bx).abs() - 0.1).max(0.0);
                let dy = ((ay - (by - 1.2)).abs() - 0.6).max(0.0);
                (dx * dx + dy * dy).sqrt() - R
            })
            .fold(f32::INFINITY, f32::min);
        // Quanto ele já saiu do eixo quando chega a `1 m` (em x) do braço — ANTES de lhe tocar.
        let antes = c
            .iter()
            .find(|(a, b)| b.0 - a.0 < 1.0)
            .map_or(0.0, |(a, _)| a.1.abs());
        let fim = c.last().expect("a corrida").0;
        (folga, antes, dist(fim, (4.0, 0.0)))
    };
    let (folga, _, chegou) = corrida(true, false);
    let (_, antes_alvo, chegou_alvo) = corrida(true, true);
    let (_, _, chegou_sem_braco) = corrida(false, false);
    eprintln!(
        "composto: folga ao braço {folga:.4}, chega a {chegou:.4} · perseguido: fora do eixo a 1 m \
         {antes_alvo:.4}, chega a {chegou_alvo:.4} · sem braço chega a {chegou_sem_braco:.4}"
    );
    // Sem as peças no desvio ele ENCOSTAVA no braço (folga `0,006 m`, medido).
    assert!(folga > 0.2, "encostou no braço: folga {folga}");
    // Perseguido, o carro é UM disco: o que envolve o braço já o afasta do eixo antes de lhe tocar.
    assert!(
        antes_alvo > 0.3,
        "o disco do alvo esqueceu o braço: {antes_alvo}"
    );
    assert!(
        chegou < 0.15 && chegou_alvo < 0.15,
        "não chegou: {chegou} · {chegou_alvo}"
    );
    assert!(
        chegou_sem_braco < 0.15,
        "o CONTROLO não chegou: {chegou_sem_braco}"
    );
}
