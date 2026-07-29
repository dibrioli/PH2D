//! **O que uma alça de roldana MONTADA tem para colar** (W-Pulley W6) — o item
//! aberto que o plano nomeia como *"nenhuma alça de roldana tem ÍMÃ, e é decisão
//! herdada do W1"*.
//!
//! A isenção está escrita no produto (`joint_anchor_drag.rs:241`) e diz, em voz
//! alta, o porquê: *"o ímã cola a alça nos pontos do COLLIDER do corpo daquela
//! ponta, e uma roldana não pertence a corpo nenhum — não há a que colar"*.
//!
//! ⚠️ Essa frase era verdade quando foi escrita, e o **W3 a falsificou**: uma
//! roldana MONTADA pertence a um corpo (`PulleyWheel::body`), o eixo dela é
//! `corpo · local`, e aquele corpo tem collider — logo tem os nove pontos. É a
//! forma exata de *uma condição que enumera seus leitores*: ela enumerava os
//! donos de collider da época.
//!
//! Esta sonda mede as três coisas que decidem a wave, ANTES de escrever código:
//!
//! 1. o conjunto de candidatos de uma roldana MONTADA existe, e onde ele está;
//! 2. o que uma pontaria a olho custa — e que o erro é **congelado** no `local`,
//!    logo permanente e invisível;
//! 3. o CONTROLE: uma roldana de CENÁRIO segue sem nada a que colar, então a
//!    isenção continua certa para ela.
//!
//! `cargo test -p ph2d-physics-ecs --test measure_pulley_wheel_snap -- --ignored --nocapture`

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, PulleyWheel,
    RigidBody,
};

/// O alcance do ímã em pixels, o mesmo `SNAP_PX` da shell — e a régua que o
/// converte em mundo é `height_world / altura da janela`.
const SNAP_PX: f32 = 14.0;

/// Meias-extensões do bloco que carrega o eixo, e a pose dele. Declaradas AQUI
/// de propósito: os nove pontos esperados saem desta geometria, não de uma
/// chamada ao produto — um oráculo que usa a função sob teste é sempre verde.
const BLOCK_HALF: [f32; 2] = [0.60, 0.25];
const BLOCK_AT: [f32; 2] = [0.0, 2.0];

/// Um bloco RETANGULAR com uma roldana montada nele (o caso da *cadernal
/// móvel*), mais as duas pontas da corda.
///
/// A caixa é a única forma com **nove** pontos de encaixe — quinas E meios de
/// aresta —, e é exatamente onde um ímã tem o que fazer: numa bola os cardinais
/// são visíveis no contorno, numa caixa a quina é o ponto que a mão não acerta.
fn rig(mount: &str) -> SimWorld {
    let mut sim = SimWorld::new();
    let mut ball = |name: &str, x: f32, y: f32, kind: BodyKind| {
        sim.world_mut().spawn((
            Name::new(name),
            RigidBody { kind },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, y)),
        ));
    };
    ball("Dead", -1.0, 6.0, BodyKind::Static);
    ball("Haul", 1.5, 4.0, BodyKind::Dynamic);
    sim.world_mut().spawn((
        Name::new("Block"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: BLOCK_HALF[0],
                half_y: BLOCK_HALF[1],
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(BLOCK_AT[0], BLOCK_AT[1])),
    ));
    sim.world_mut().spawn((
        Name::new("Rope"),
        PhysicsJoint {
            body_a: stable_name_id("Dead"),
            body_b: stable_name_id("Haul"),
            kind: JointKind::Pulley,
            ..PhysicsJoint::of_kind(JointKind::Pulley)
        },
        Transform::from_translation(Vec2::new(-1.0, 6.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Rope Wheel 1"),
        PulleyWheel {
            rope: stable_name_id("Rope"),
            order: 0,
            radius: 0.3,
            body: stable_name_id(mount),
            ..Default::default()
        },
        // Fora do centro do bloco, senão o local semeado é `[0, 0]` e a fixture
        // não distingue *convertido* de *nunca convertido*.
        Transform::from_translation(Vec2::new(0.25, 2.15)),
    ));
    sim
}

fn entity_of(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entidade viva")
}

fn wheel_of(sim: &mut SimWorld) -> PulleyWheel {
    let mut q = sim.world_mut().query::<(&Name, &PulleyWheel)>();
    q.iter(sim.world())
        .find(|(n, _)| n.as_str() == "Rope Wheel 1")
        .map(|(_, w)| *w)
        .expect("a roldana existe")
}

/// Os nove pontos que a caixa do bloco oferece, em MUNDO, derivados da geometria
/// declarada nesta fixture (nunca do produto).
fn expected_points() -> Vec<(&'static str, [f32; 2])> {
    let (hx, hy) = (BLOCK_HALF[0], BLOCK_HALF[1]);
    let (cx, cy) = (BLOCK_AT[0], BLOCK_AT[1]);
    vec![
        ("centro", [cx, cy]),
        ("quina TL", [cx - hx, cy + hy]),
        ("quina TR", [cx + hx, cy + hy]),
        ("quina BL", [cx - hx, cy - hy]),
        ("quina BR", [cx + hx, cy - hy]),
        ("meio topo", [cx, cy + hy]),
        ("meio dir", [cx + hx, cy]),
        ("meio base", [cx, cy - hy]),
        ("meio esq", [cx - hx, cy]),
    ]
}

#[test]
#[ignore = "measurement, not a gate"]
fn measure_where_a_mounted_axle_can_be_aimed() {
    println!("\n=== Os pontos que o eixo de uma roldana MONTADA pode mirar ===");
    println!("bloco {BLOCK_HALF:?} de meia-extensão em {BLOCK_AT:?}\n");
    println!("{:>11} | {:>16}", "ponto", "mundo");
    for (name, p) in expected_points() {
        println!("{name:>11} | ({:>6.3}, {:>6.3})", p[0], p[1]);
    }
    println!(
        "\n  Nove pontos. A isenção do produto diz *nao ha a que colar* — e para\n  \
         esta roldana ha nove, porque o W3 lhe deu um corpo."
    );

    // O alcance do ímã em MUNDO, na régua que a shell usa.
    println!("\n=== O alcance do ima ({SNAP_PX} px) em metros, por zoom ===");
    println!("{:>14} | {:>12}", "height_world", "alcance (m)");
    for h in [4.0_f32, 8.0, 16.0, 32.0] {
        println!("{h:>14.1} | {:>12.4}", SNAP_PX * h / 1080.0);
    }
}

#[test]
#[ignore = "measurement, not a gate"]
fn measure_what_aiming_by_eye_costs() {
    println!("\n=== O que uma pontaria A OLHO custa, e por quanto tempo ===");
    // O artista quer o eixo na quina de cima à direita: (0.60, 2.25).
    let target = [BLOCK_AT[0] + BLOCK_HALF[0], BLOCK_AT[1] + BLOCK_HALF[1]];
    println!("alvo: quina TR = ({:.3}, {:.3})\n", target[0], target[1]);
    println!(
        "{:>10} | {:>18} | {:>18} | {:>10}",
        "erro (m)", "eixo local", "eixo depois do move", "desvio"
    );
    for miss in [0.0_f32, 0.02, 0.05, 0.10] {
        let mut sim = rig("Block");
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);

        // O gesto: solta a alça `miss` fora do alvo, pela MESMA porta que o
        // arrasto usa (escreve o `Transform` e re-abre o eixo derivado).
        let we = entity_of(&mut sim, "Rope Wheel 1");
        let dropped = [target[0] + miss, target[1] + miss];
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(we) {
            t.translation = Vec2::new(dropped[0], dropped[1]);
        }
        ph2d_physics_ecs::reseat_wheel_geometry(sim.world_mut(), we);
        bridge.dispatch(&mut sim, false, 0);
        let local = wheel_of(&mut sim).local;

        // E agora o bloco anda: o eixo VIAJA com o erro, porque o erro está
        // congelado no local. Um número errado que nada corrige depois.
        let be = entity_of(&mut sim, "Block");
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(be) {
            t.translation.x += 3.0;
            t.translation.y -= 1.0;
        }
        bridge.dispatch(&mut sim, false, 0);
        let t = *sim
            .world()
            .get::<Transform>(we)
            .expect("a roldana tem pose");
        // Onde a quina do bloco está agora, e onde o eixo ficou.
        let corner = [
            BLOCK_AT[0] + 3.0 + BLOCK_HALF[0],
            BLOCK_AT[1] - 1.0 + BLOCK_HALF[1],
        ];
        let drift =
            ((t.translation.x - corner[0]).powi(2) + (t.translation.y - corner[1]).powi(2)).sqrt();
        println!(
            "{miss:>10.3} | ({:>7.4}, {:>7.4}) | ({:>7.3}, {:>7.3}) | {drift:>10.4}",
            local[0], local[1], t.translation.x, t.translation.y
        );
    }
    println!(
        "\n  O desvio NAO decai: ele e o erro da mao, assado no `local` e carregado\n  \
         pelo bloco para sempre. E invisivel — o eixo acompanha o bloco\n  \
         corretamente, so nao esta na quina."
    );
}

#[test]
#[ignore = "measurement, not a gate"]
fn measure_that_a_scenery_wheel_has_nothing_to_snap_to() {
    println!("\n=== O CONTROLE: uma roldana de CENARIO ===");
    let mut sim = rig("Nao Existe");
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let w = wheel_of(&mut sim);
    println!(
        "  body = {} · mounted = {} · a arena diz body = {:?}",
        w.body,
        w.mounted,
        bridge
            .pulley_wheel_arena()
            .first()
            .map(|r| r.body.is_some())
    );
    println!(
        "\n  Sem corpo resolvido nao ha collider, logo nao ha candidato: a isencao\n  \
         do produto continua CERTA para ela, e o ima simplesmente nao abre."
    );
}
