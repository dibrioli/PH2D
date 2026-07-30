//! **Dá para APONTAR uma corda no canvas?** (W-Pulley W1) — o item aberto que diz
//! *"a corda de uma roldana é escolhida só na CRIAÇÃO … um eyedropper de corda, se
//! vier, é o irmão exato do da §12"*.
//!
//! ⚠️ **E ele NÃO pode ser o irmão exato, e é por isso que esta sonda existe.** O
//! eyedropper da §12 resolve o alvo com `pick_sprites_at_world`, que exige um
//! **sprite** sob o cursor — um corpo tem; uma corda é uma **LINHA** e a entidade
//! dela não tem sprite nenhum. Medido: `pick_sprites_at_world` sobre um ponto em
//! cima da corda devolve **a lista vazia**.
//!
//! O que existe é melhor: a corda **é desenhada** pela `rope_route::route`, a mesma
//! função que o solver usa. Então ela é apontável **por onde ela é desenhada**, e a
//! pergunta *"que corda está sob o cursor?"* sai da MESMA porta que a desenha — não
//! de uma segunda geometria capaz de discordar do traço na tela.
//!
//! Esta sonda mede as três coisas que decidem a wave:
//!
//! 1. a distância do cursor à rota é o que a mão de fato controla (ela devolve o
//!    afastamento, não um booleano);
//! 2. duas cordas vizinhas: a mais próxima ganha, e a partir de que separação a
//!    escolha deixa de ser ambígua;
//! 3. o alcance em METROS que uma tolerância de tela vale, por zoom — o número que
//!    a wave precisa escrever.
//!
//! `cargo test -p ph2d-physics-ecs --test measure_rope_pick -- --ignored --nocapture --test-threads=1`

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics::world::rope_route;
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, PulleyWheel,
    RigidBody,
};

/// A tolerância de tela que o resto do editor usa para *"apontei nisto"*.
///
/// O mesmo 14 px do ímã de âncora (`SNAP_PX`) e do snap de pivô — um app onde dois
/// alvos de canvas respondem a distâncias diferentes é um app que se aprende duas
/// vezes.
const PICK_PX: f32 = 14.0;

/// Uma corda vertical simples: âncora morta no alto, carga embaixo, uma roldana no
/// caminho. `x` desloca o rig inteiro, para a sonda montar duas cordas paralelas.
fn rope(sim: &mut SimWorld, tag: &str, x: f32) {
    let mut ball = |name: String, y: f32, kind: BodyKind| {
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
    ball(format!("{tag} Dead"), 6.0, BodyKind::Static);
    ball(format!("{tag} Load"), 2.0, BodyKind::Dynamic);
    sim.world_mut().spawn((
        Name::new(format!("{tag} Rope")),
        PhysicsJoint {
            body_a: stable_name_id(&format!("{tag} Dead")),
            body_b: stable_name_id(&format!("{tag} Load")),
            kind: JointKind::Pulley,
            ..PhysicsJoint::of_kind(JointKind::Pulley)
        },
        Transform::from_translation(Vec2::new(x, 6.0)),
    ));
    sim.world_mut().spawn((
        Name::new(format!("{tag} Rope Wheel 1")),
        PulleyWheel {
            rope: stable_name_id(&format!("{tag} Rope")),
            order: 0,
            radius: 0.25,
            ..Default::default()
        },
        Transform::from_translation(Vec2::new(x + 0.5, 4.5)),
    ));
}

fn named(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entidade viva")
}

/// A distância de `p` à rota desenhada da corda `rope`, em metros — **pela MESMA
/// porta que a desenha** (`rope_route::route`), nunca por uma segunda geometria.
///
/// `None` quando a corda não roteia (degenerada) — e nesse caso o desenho também
/// é uma reta, então a sonda não a estaria medindo de verdade.
fn distance_to_rope(
    bridge: &PhysicsBridge,
    sim: &SimWorld,
    rope: Entity,
    p: [f32; 2],
) -> Option<f32> {
    let v = bridge.joint_views().find(|v| v.entity == rope)?;
    let wheels: Vec<_> = bridge.rope_wheels(rope).map(|(_, w)| w).collect();
    let mut segs = Vec::new();
    let r = rope_route::route(v.anchor_a, v.anchor_b, &wheels, &mut segs)?;
    let _ = (r, sim);
    // A rota é uma polilinha: âncora A -> (from, to) de cada trecho -> âncora B.
    let mut pts = vec![v.anchor_a];
    for t in &segs {
        pts.push(t.from);
        pts.push(t.to);
    }
    pts.push(v.anchor_b);
    let mut best = f32::INFINITY;
    for pair in pts.windows(2) {
        best = best.min(point_to_segment(p, pair[0], pair[1]));
    }
    Some(best)
}

/// Distância de um ponto a um segmento — a aritmética que um hit-test de linha é.
fn point_to_segment(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let (ax, ay) = (b[0] - a[0], b[1] - a[1]);
    let len2 = ax * ax + ay * ay;
    let t = if len2 <= f32::EPSILON {
        0.0
    } else {
        (((p[0] - a[0]) * ax + (p[1] - a[1]) * ay) / len2).clamp(0.0, 1.0)
    };
    (p[0] - (a[0] + ax * t)).hypot(p[1] - (a[1] + ay * t))
}

/// **Um ponto SOBRE a rota**, na fração `t` da polilinha, e a NORMAL ali.
///
/// ⚠️ **A 1ª versão desta sonda cravou "(0, 5)" como *em cima da corda* e mediu
/// 0,46 m de afastamento**, com a coluna saindo NÃO-MONOTÔNICA (0,4622 · 0,3736 ·
/// **0,1962** · 0,4245): a corda não passa por `x=0` — ela desvia para a roldana em
/// `(0,5; 4,5)`. A medição estava certa e a premissa, errada. *Um cursor "sobre a
/// linha" tem de ser PERGUNTADO à linha, nunca adivinhado da fixture.*
fn point_on_route(bridge: &PhysicsBridge, rope: Entity, t: f32) -> Option<([f32; 2], [f32; 2])> {
    let v = bridge.joint_views().find(|v| v.entity == rope)?;
    let wheels: Vec<_> = bridge.rope_wheels(rope).map(|(_, w)| w).collect();
    let mut segs = Vec::new();
    rope_route::route(v.anchor_a, v.anchor_b, &wheels, &mut segs)?;
    let mut pts = vec![v.anchor_a];
    for s in &segs {
        pts.push(s.from);
        pts.push(s.to);
    }
    pts.push(v.anchor_b);
    // Comprimento acumulado, para `t` medir ARCO e não índice de vértice.
    let total: f32 = pts
        .windows(2)
        .map(|p| (p[1][0] - p[0][0]).hypot(p[1][1] - p[0][1]))
        .sum();
    let want = total * t.clamp(0.0, 1.0);
    let mut acc = 0.0;
    for pair in pts.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let len = (b[0] - a[0]).hypot(b[1] - a[1]);
        if len <= f32::EPSILON {
            continue;
        }
        if acc + len >= want {
            let f = (want - acc) / len;
            let p = [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f];
            // A normal do trecho: é por ela que o afastamento é honesto (deslocar
            // AO LONGO da corda não afasta nada).
            let n = [-(b[1] - a[1]) / len, (b[0] - a[0]) / len];
            return Some((p, n));
        }
        acc += len;
    }
    pts.last().map(|&p| (p, [1.0, 0.0]))
}

fn rig(taps: &[(&str, f32)]) -> (SimWorld, PhysicsBridge) {
    let mut sim = SimWorld::new();
    for (tag, x) in taps {
        rope(&mut sim, tag, *x);
    }
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    (sim, bridge)
}

#[test]
#[ignore = "measurement, not a gate"]
fn measure_that_a_rope_is_not_sprite_pickable() {
    println!("\n=== O eyedropper da §12 alcança uma corda? ===\n");
    let (mut sim, bridge) = rig(&[("A", 0.0)]);
    let r = named(&mut sim, "A Rope");
    // Um ponto EM CIMA da corda -- perguntado A ELA, no meio do percurso.
    let (on, _) = point_on_route(&bridge, r, 0.5).expect("a corda roteia");
    println!(
        "  ponto no meio da rota: ({:.3}, {:.3}) · distancia a ela: {:.5} m",
        on[0],
        on[1],
        distance_to_rope(&bridge, &sim, r, on).expect("a corda roteia")
    );
    println!(
        "\n  ⚠️ `pick_sprites_at_world` precisa de um SPRITE, e a entidade-corda nao\n  \
         tem nenhum -- entao o eyedropper da §12, copiado, resolveria SEMPRE `None`\n  \
         em cima da corda. A rota e o unico alvo que existe."
    );
}

#[test]
#[ignore = "measurement, not a gate"]
fn measure_how_far_off_the_cursor_can_be() {
    println!("\n=== A distancia do cursor a rota, e o que ela vale por zoom ===\n");
    let (mut sim, bridge) = rig(&[("A", 0.0)]);
    let r = named(&mut sim, "A Rope");
    let (on, n) = point_on_route(&bridge, r, 0.5).expect("roteia");
    println!(
        "  ancora: ({:.3}, {:.3}), normal ({:.3}, {:.3})\n",
        on[0], on[1], n[0], n[1]
    );
    println!("{:>18} | {:>12}", "afastamento", "dist (m)");
    for off in [0.0_f32, 0.05, 0.1, 0.3, 1.0] {
        let p = [on[0] + n[0] * off, on[1] + n[1] * off];
        println!(
            "{:>18} | {:>12.5}",
            format!("{off:.2} m pela normal"),
            distance_to_rope(&bridge, &sim, r, p).expect("roteia")
        );
    }
    println!("\n=== {PICK_PX} px em metros, por zoom (janela 1080) ===");
    println!("{:>14} | {:>12}", "height_world", "alcance (m)");
    for h in [4.0_f32, 8.0, 16.0, 32.0] {
        println!("{h:>14.1} | {:>12.4}", PICK_PX * h / 1080.0);
    }
}

#[test]
#[ignore = "measurement, not a gate"]
fn measure_when_two_ropes_become_ambiguous() {
    println!("\n=== Duas cordas paralelas: a mais proxima ganha? ===\n");
    println!(
        "{:>10} | {:>12} | {:>12} | {:>10}",
        "separacao", "dist A", "dist B", "vencedora"
    );
    for gap in [3.0_f32, 1.0, 0.4, 0.1, 0.02] {
        let (mut sim, bridge) = rig(&[("A", 0.0), ("B", gap)]);
        let (ra, rb) = (named(&mut sim, "A Rope"), named(&mut sim, "B Rope"));
        // O cursor SOBRE a corda A, deslocado um terço do vão na direção de B —
        // uma fixture que não põe o cursor na linha não mede pick nenhum.
        let (on, _) = point_on_route(&bridge, ra, 0.5).expect("roteia");
        let p = [on[0] + gap / 3.0, on[1]];
        let da = distance_to_rope(&bridge, &sim, ra, p).expect("roteia");
        let db = distance_to_rope(&bridge, &sim, rb, p).expect("roteia");
        println!(
            "{gap:>10.2} | {da:>12.4} | {db:>12.4} | {:>10}",
            if da < db { "A" } else { "B" }
        );
    }
    println!(
        "\n  A mais proxima ganha em toda separacao: a escolha nunca e ambigua, ela\n  \
         so fica FINA -- e a mao de quem aponta e que decide, como em todo pick."
    );
}
