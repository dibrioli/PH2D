//! **A cena 74, medida HEADLESS antes de a mensagem ser escrita.**
//!
//! A política do plano: *toda wave ganha cena com números MEDIDOS*, porque
//! nesta linha duas cenas já afirmaram coisas que a medição desmentiu. Este
//! arquivo é a sonda e o gate ao mesmo tempo — ele dirige os quatro rigs pelas
//! portas do produto e afirma o que a mensagem promete ao artista.

use super::*;
use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::{IkOptions, PhysicsBridge};

fn scene() -> (SimWorld, PhysicsBridge) {
    let mut sim = SimWorld::new();
    build_lead_scene(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    (sim, bridge)
}

fn named(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("a cena tem de conter '{name}'"))
}

fn pos(sim: &SimWorld, e: Entity) -> [f32; 2] {
    sim.world()
        .get::<Transform>(e)
        .map(|t| [t.translation.x, t.translation.y])
        .expect("corpo tem Transform")
}

fn write(sim: &mut SimWorld, poses: &[(Entity, [f32; 2], f32)]) {
    for &(e, t, r) in poses {
        if let Some(mut tr) = sim.world_mut().get_mut::<Transform>(e) {
            tr.translation = Vec2::new(t[0], t[1]);
            tr.rotation = r;
        }
    }
}

/// **A CORDA: pegar a cabeça leva o rig inteiro, e a cauda chega por último.**
///
/// Medido nesta cena: com 20 cm de puxada o perfil é
/// **0,200 · 0,102 · 0,020 · 0,020** — a cauda mal se mexe. Com 2 m já é
/// **2,000 · 1,562 · 1,102 · 1,030**, e é por isso que o oráculo tem de ser o
/// INSTANTE: no fim tudo andou.
#[test]
fn the_rope_trails_behind_the_hand() {
    let (mut sim, mut b) = scene();
    let e: Vec<Entity> = (1..=4)
        .map(|i| named(&mut sim, &format!("Rope {i}")))
        .collect();
    let before: Vec<_> = e.iter().map(|&x| pos(&sim, x)).collect();

    assert!(b.ik_begin(e[0]), "pegar a cabeca da corda abre o gesto");
    assert_eq!(b.posing_bodies().len(), 4, "a corda inteira vem junto");

    let head = before[0];
    for k in 1..=4i16 {
        let t = [head[0], head[1] + 0.2 * f32::from(k) / 4.0];
        let poses = b.ik_move(t, 0.0, IkOptions::default());
        write(&mut sim, &poses);
        b.dispatch(&mut sim, false, 0);
    }
    let d: Vec<f32> = (0..4)
        .map(|i| {
            let (a, c) = (before[i], pos(&sim, e[i]));
            (c[0] - a[0]).hypot(c[1] - a[1])
        })
        .collect();
    for i in 0..3 {
        assert!(
            d[i] >= d[i + 1] - 1e-4,
            "o perfil tem de DECAIR da mao para a cauda: {d:?}"
        );
    }
    assert!(
        d[3] < d[0] * 0.25,
        "a cauda mal devia se mexer no comeco: {d:?}"
    );
}

/// **A PEÇA: o gesto que era morto agora leva os três elos, rígido.**
#[test]
fn the_welded_piece_travels_whole() {
    let (mut sim, mut b) = scene();
    let e: Vec<Entity> = (1..=3)
        .map(|i| named(&mut sim, &format!("Piece {i}")))
        .collect();
    let grab = pos(&sim, e[1]);

    assert!(
        b.fk_begin(&sim, e[1], grab),
        "uma peca soldada tem de abrir gesto -- antes desta wave nao abria"
    );
    assert_eq!(b.fk_bodies().len(), 3, "os tres elos viajam juntos");
    assert!(
        b.fk_session().expect("sessao").is_rigid(),
        "sem junta que dobre, o gesto e' uma TRANSLACAO"
    );

    let poses = b.fk_move([grab[0] + 1.0, grab[1] + 0.5]);
    for (_, t, r) in &poses {
        assert!(r.abs() < 1e-6, "uma translacao nao gira nada: {r}");
        let _ = t;
    }
}

/// **O CONTROLE: a cadeia presa à parede continua DOBRANDO.** Se o ramo novo
/// tivesse roubado o caso da dobradiça, os dois gates acima ficariam mais
/// verdes ainda e este seria o único a sangrar.
#[test]
fn the_arm_on_the_post_still_bends() {
    let (mut sim, mut b) = scene();
    let arm1 = named(&mut sim, "Arm 1");
    let grab = pos(&sim, arm1);
    assert!(b.fk_begin(&sim, arm1, grab));
    assert!(
        !b.fk_session().expect("sessao").is_rigid(),
        "ha' uma dobradica na parede: o elo GIRA, nao viaja"
    );
    let poses = b.fk_move([grab[0] - 0.5, grab[1] + 1.0]);
    assert!(
        poses.iter().any(|(_, _, r)| r.abs() > 0.2),
        "algum elo tem de ter girado: {poses:?}"
    );
}

/// **A PAREDE não anda.** O suporte soldado a um corpo estático não tem grau de
/// liberdade nenhum, e o gesto recusa — o defeito que a sonda achou no meio
/// desta wave.
#[test]
fn the_bracket_welded_to_the_wall_has_no_gesture() {
    let (mut sim, mut b) = scene();
    let br2 = named(&mut sim, "Bracket 2");
    let grab = pos(&sim, br2);
    assert!(
        !b.fk_begin(&sim, br2, grab),
        "soldado a uma parede nao ha' o que mover"
    );
    assert!(b.fk_bodies().is_empty());
}

/// **A cabeça é a MESMA seja qual elo a mão pegue** — o rig tem um "para cima",
/// e não um por gesto.
#[test]
fn the_ropes_head_does_not_depend_on_where_the_hand_lands() {
    let (mut sim, b) = scene();
    let e: Vec<Entity> = (1..=4)
        .map(|i| named(&mut sim, &format!("Rope {i}")))
        .collect();
    for &grabbed in &e {
        let plan = b.ik_plan(grabbed).expect("plano");
        assert_eq!(plan.root, e[0], "a cabeca autorada e' 'Rope 1'");
    }
}

/// **A sonda que alimenta a mensagem da cena.** Os números do `eprintln!` saem
/// daqui, e não de um fixture parecido: *quando o número vira decisão de
/// produto, ele TEM de sair da porta do produto*.
///
/// `cargo test -p ph2d-host-desktop --release --bins measure_the_lead_scene -- --ignored --nocapture`
#[test]
#[ignore = "sonda"]
fn measure_the_lead_scene() {
    let (mut sim, mut b) = scene();
    let e: Vec<Entity> = (1..=4)
        .map(|i| named(&mut sim, &format!("Rope {i}")))
        .collect();
    let before: Vec<_> = e.iter().map(|&x| pos(&sim, x)).collect();
    assert!(b.ik_begin(e[0]));
    let head = before[0];
    println!("\n=== A CORDA (Rope 1..4), cabeca puxada para CIMA ===");
    for k in 1..=20i16 {
        let up = 2.0 * f32::from(k) / 20.0;
        let poses = b.ik_move([head[0], head[1] + up], 0.0, IkOptions::default());
        write(&mut sim, &poses);
        b.dispatch(&mut sim, false, 0);
        if matches!(k, 1 | 2 | 4 | 20) {
            let d: Vec<String> = (0..4)
                .map(|i| {
                    let (a, c) = (before[i], pos(&sim, e[i]));
                    format!("{:>6.3}", (c[0] - a[0]).hypot(c[1] - a[1]))
                })
                .collect();
            println!("  apos {up:>4.2} m: {}", d.join(" "));
        }
    }
    b.ik_end();

    let (mut sim, mut b) = scene();
    let p2 = named(&mut sim, "Piece 2");
    let grab = pos(&sim, p2);
    println!("\n=== A PECA soldada (Piece 1..3) ===");
    println!("  fk_begin = {}", b.fk_begin(&sim, p2, grab));
    println!("  corpos que o gesto move = {}", b.fk_bodies().len());
    println!(
        "  rigido = {}",
        b.fk_session()
            .is_some_and(ph2d_physics_ecs::FkSession::is_rigid)
    );

    let (mut sim, mut b) = scene();
    let br = named(&mut sim, "Bracket 2");
    let grab = pos(&sim, br);
    println!("\n=== O SUPORTE soldado a' PAREDE ===");
    println!(
        "  fk_begin = {} (tem de ser false)",
        b.fk_begin(&sim, br, grab)
    );
}
