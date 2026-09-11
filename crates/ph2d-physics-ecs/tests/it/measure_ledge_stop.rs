//! **QUANTO DA BEIRADA ELE JÁ PISA** — a sonda que abre a wave **G**
//! (`bCanWalkOffLedges`, §3.G da auditoria).
//!
//! A auditoria descreve o item numa linha (*"um veredito a mais no `footing`
//! quando o leque de pés vê o chão acabar"*), e a §0 manda medir antes de
//! escrever qualquer lei. As três perguntas, por esta ordem:
//!
//! 1. **onde é que ele larga o chão hoje**, em relação à quina — porque é isso
//!    que uma trava teria de mudar, e o número diz de quanto;
//! 2. **o leque JÁ vê a quina antes de a lei a sentir?** — se sim, a wave é um
//!    veredito; se não, é um sensor novo, que é outra wave;
//! 3. **quanto é que a resposta depende da VELOCIDADE** — uma trava que só
//!    funciona a passo lento não é uma trava.
//!
//! Rodar:
//! `ph2d-run cargo test -p ph2d-physics-ecs --release --test measure_ledge_stop -- --ignored --nocapture`

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerInput,
    RigidBody,
};

/// Altura de repouso, e o topo da laje está em `y = 0`.
const FLOAT: f32 = 0.9;
/// Meia-largura do corpo — a cápsula tem raio 0,2, logo ele mede 0,40 m.
const HALF_W: f32 = 0.2;
/// A QUINA. A laje acaba aqui, e todo `x` desta sonda é medido contra ela.
const LEDGE_X: f32 = 0.0;

/// O que uma travessia de beirada devolve.
struct Walk {
    /// O traço `(x, y)` do centro, um par por tique.
    trace: Vec<(f32, f32)>,
}

impl Walk {
    /// **O último `x` em que o chão ainda o segurava** — lido do traço com a
    /// tolerância MEDIDA da mola (ver [`measure_the_ripple_on_flat_ground`]), e
    /// não com um limiar escolhido.
    ///
    /// ⚠️ **O primeiro corte desta sonda perguntava *"onde ele já caiu 5 cm?"*, e
    /// isso mede a GRAVIDADE:** cair 5 cm leva 0,10 s, que a 8 m/s são 0,8 m de
    /// avanço — a tabela subia com a velocidade porque o oráculo tinha um relógio
    /// dentro, não porque o sensor visse a quina mais tarde.
    fn last_held(&self, ripple: f32) -> f32 {
        self.trace
            .iter()
            .take_while(|(_, y)| *y >= FLOAT - ripple)
            .map(|(x, _)| *x)
            .last()
            .unwrap_or(f32::NAN)
    }

    /// **Quanto do corpo passou da quina** no último tique segurado — o número
    /// que o artista vê, porque é a borda do desenho que fica no ar, não o centro.
    fn overhang(&self, ripple: f32) -> f32 {
        self.last_held(ripple) + HALF_W - LEDGE_X
    }
}

/// Anda para a direita até cair, e devolve onde. `trava` ARMA o `bCanWalkOffLedges`.
fn walk_off_with(foot_samples: u16, speed: f32, foot_spread: f32, trava: bool) -> Walk {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Slab"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 10.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(LEDGE_X - 10.0, -0.5)),
    ));

    let player = sim
        .world_mut()
        .spawn((
            Name::new("Player"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: 0.3,
                    radius: HALF_W,
                },
                ..Collider::default()
            },
            LockRotation,
            PlatformPlayer {
                float_height: FLOAT,
                foot_samples,
                foot_spread,
                speed,
                walk_off_ledges: !trava,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(LEDGE_X - 3.0, FLOAT)),
        ))
        .id();

    let mut bridge = PhysicsBridge::new();
    let mut trace = Vec::new();
    for i in 1..=400u64 {
        bridge.set_player_input(
            player,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, i);
        let t = sim.world().get::<Transform>(player).expect("transform");
        trace.push((t.translation.x, t.translation.y));
        // Já caiu abaixo do topo da laje: não há mais nada a aprender.
        if t.translation.y < -1.0 {
            break;
        }
    }
    Walk { trace }
}

/// A travessia SEM trava — o mundo que já shipava.
fn walk_off(foot_samples: u16, speed: f32, foot_spread: f32) -> Walk {
    walk_off_with(foot_samples, speed, foot_spread, false)
}

/// **O tremor da mola em chão plano** — a tolerância que o [`Walk::last_held`]
/// usa, medida em vez de escolhida.
fn flat_ripple(foot_samples: u16, speed: f32) -> f32 {
    // A mesma cena, mas com a quina longe: os primeiros 100 tiques correm todos
    // sobre laje cheia.
    let w = walk_off(foot_samples, speed, 0.9);
    w.trace
        .iter()
        .take_while(|(x, _)| *x < LEDGE_X - 1.0)
        .map(|(_, y)| (FLOAT - y).abs())
        .fold(0.0f32, f32::max)
}

#[test]
#[ignore = "sonda"]
fn measure_the_ripple_on_flat_ground() {
    println!("\n=== O TREMOR DA MOLA EM CHAO PLANO (a tolerancia do oraculo) ===");
    println!("  pes  vel    tremor (m)");
    for samples in [1u16, 3, 5] {
        for speed in [1.0f32, 4.0, 8.0] {
            println!(
                "  {samples:>3}  {speed:>3.0}    {:>10.6}",
                flat_ripple(samples, speed)
            );
        }
    }
}

#[test]
#[ignore = "sonda"]
fn measure_where_he_leaves_the_ledge() {
    let ripple = 0.005;
    println!("\n=== ONDE ELE LARGA A QUINA (quina em x=0, meia-largura {HALF_W}) ===");
    println!("  o pe' de fora nasce a {:.4} m do centro", HALF_W * 0.9);
    println!("  tolerancia do oraculo: {ripple} m\n");
    println!("  pes  vel    ultimo seguro   saliencia");
    for samples in [1u16, 3, 5] {
        for speed in [1.0f32, 4.0, 8.0] {
            let w = walk_off(samples, speed, 0.9);
            println!(
                "  {samples:>3}  {speed:>3.0}    {:>13.4}   {:>9.4}",
                w.last_held(ripple),
                w.overhang(ripple)
            );
        }
    }
}

#[test]
#[ignore = "sonda"]
fn measure_what_the_spread_buys() {
    let ripple = 0.005;
    println!("\n=== E O QUE O `foot_spread` MUDA (5 pes, 4 m/s) ===");
    println!("  spread   pe' de fora   ultimo seguro   saliencia");
    for spread in [0.0f32, 0.25, 0.5, 0.75, 0.9, 1.0] {
        let w = walk_off(5, 4.0, spread);
        println!(
            "  {spread:>6.2}   {:>11.4}   {:>13.4}   {:>9.4}",
            HALF_W * spread,
            w.last_held(ripple),
            w.overhang(ripple)
        );
    }
}

#[test]
#[ignore = "sonda"]
fn measure_the_trace_across_the_ledge() {
    println!("\n=== O TRACO, tique a tique, a atravessar a quina (5 pes, 4 m/s) ===");
    println!("  (x do centro, y; o repouso e' {FLOAT}, a quina e' x=0)\n");
    let w = walk_off(5, 4.0, 0.9);
    for (x, y) in w.trace.iter().filter(|(x, _)| (-0.6..1.2).contains(x)) {
        println!("  x={x:>8.4}   y={y:>8.4}   dy={:>8.4}", y - FLOAT);
    }
}

/// **A FENDA** — a pergunta que decide se o veredito basta, ou se a wave precisa
/// de um sensor que olhe ALÉM do corpo.
///
/// O leque só amostra dentro da pegada, então *"o chão acaba"* e *"há um buraco
/// que eu atravesso"* podem chegar-lhe idênticos. Esta sonda mede quantos
/// tiques a quina fica acesa a atravessar fendas que o corpo de facto vence — e
/// o CONTROLE é a mesma travessia com a quina de verdade.
fn brink_ticks_over_gap(gap: f32) -> (usize, usize, bool) {
    let mut sim = SimWorld::new();
    let mut slab = |name: &str, cx: f32, half_x: f32| {
        sim.world_mut().spawn((
            Name::new(name),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x,
                    half_y: 0.5,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(cx, -0.5)),
        ));
    };
    slab("Left", LEDGE_X - 10.0, 10.0);
    slab("Right", LEDGE_X + gap + 10.0, 10.0);

    let player = sim
        .world_mut()
        .spawn((
            Name::new("Player"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: 0.3,
                    radius: HALF_W,
                },
                ..Collider::default()
            },
            LockRotation,
            PlatformPlayer {
                float_height: FLOAT,
                speed: 4.0,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(LEDGE_X - 2.0, FLOAT)),
        ))
        .id();

    let mut bridge = PhysicsBridge::new();
    let (mut lit, mut ticks_over) = (0usize, 0usize);
    let mut crossed = false;
    for i in 1..=240u64 {
        bridge.set_player_input(
            player,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, i);
        let x = sim
            .world()
            .get::<Transform>(player)
            .expect("transform")
            .translation
            .x;
        if (LEDGE_X - HALF_W..LEDGE_X + gap + HALF_W).contains(&x) {
            ticks_over += 1;
            if bridge
                .player_view(player)
                .is_some_and(|v| v.brink.toward(1.0))
            {
                lit += 1;
            }
        }
        if x > LEDGE_X + gap + 0.5 {
            crossed = true;
            break;
        }
    }
    (lit, ticks_over, crossed)
}

#[test]
#[ignore = "sonda"]
fn measure_the_brink_over_a_gap_the_body_spans() {
    println!("\n=== A QUINA ACENDE NUMA FENDA QUE O CORPO ATRAVESSA? (corpo 0,40 m) ===");
    println!("  fenda    tiques acesos / sobre a fenda   atravessou?");
    for gap in [0.05f32, 0.10, 0.20, 0.30, 0.40, 0.60] {
        let (lit, over, crossed) = brink_ticks_over_gap(gap);
        println!(
            "  {gap:>5.2}    {lit:>5} / {over:<5}                    {}",
            if crossed { "sim" } else { "NAO" }
        );
    }
}

#[test]
#[ignore = "sonda"]
fn measure_where_the_trava_stops_him() {
    // A aceleração do ponto de partida, que a sonda não muda.
    const ACCEL: f32 = 60.0;
    println!(
        "\n=== COM A TRAVA ARMADA: ONDE ELE PARA (quina x=0, borda do corpo em x+{HALF_W}) ==="
    );
    println!("  (o alcance e' DERIVADO: v^2/(2a), com a = {ACCEL} m/s^2)\n");
    println!("  vel   look derivado   parou em   borda do corpo   caiu?");
    for speed in [1.0f32, 2.0, 4.0, 6.0, 8.0, 12.0] {
        let w = walk_off_with(3, speed, 0.9, true);
        let last = w.trace.last().map_or(f32::NAN, |(x, _)| *x);
        let lowest = w.trace.iter().map(|(_, y)| *y).fold(f32::MAX, f32::min);
        println!(
            "  {speed:>3.0}   {:>12.4}   {last:>8.4}   {:>14.4}   {}",
            speed * speed / (2.0 * ACCEL),
            last + HALF_W,
            if lowest < 0.0 { "SIM" } else { "nao" }
        );
    }
}

#[test]
#[ignore = "sonda"]
fn measure_that_the_trava_still_crosses_a_gap() {
    println!("\n=== COM A TRAVA ARMADA: ele ATRAVESSA uma fenda que a perna vence? ===");
    println!("  (o leque de 3 pes cobre +-0,18 m, entao ele cai numa fenda > ~0,36 m)");
    println!("  fenda   atravessou?   parou em");
    for gap in [0.05f32, 0.15, 0.30, 0.50] {
        let (crossed, last) = gap_walk(gap, true);
        println!(
            "  {gap:>5.2}    {:>11}   {last:>8.4}",
            if crossed { "sim" } else { "NAO" }
        );
    }
    println!("\n  CONTROLE (sem trava):");
    for gap in [0.05f32, 0.15, 0.30, 0.50] {
        let (crossed, last) = gap_walk(gap, false);
        println!(
            "  {gap:>5.2}    {:>11}   {last:>8.4}",
            if crossed { "sim" } else { "NAO" }
        );
    }
}

/// Anda para a direita sobre duas lajes separadas por `gap`; devolve
/// *(atravessou?, último x)*.
fn gap_walk(gap: f32, trava: bool) -> (bool, f32) {
    let mut sim = SimWorld::new();
    let mut slab = |name: &str, cx: f32, half_x: f32| {
        sim.world_mut().spawn((
            Name::new(name),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x,
                    half_y: 0.5,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(cx, -0.5)),
        ));
    };
    slab("Left", LEDGE_X - 10.0, 10.0);
    slab("Right", LEDGE_X + gap + 10.0, 10.0);

    let player = sim
        .world_mut()
        .spawn((
            Name::new("Player"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: 0.3,
                    radius: HALF_W,
                },
                ..Collider::default()
            },
            LockRotation,
            PlatformPlayer {
                float_height: FLOAT,
                speed: 4.0,
                walk_off_ledges: !trava,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(LEDGE_X - 2.0, FLOAT)),
        ))
        .id();

    let mut bridge = PhysicsBridge::new();
    let mut last = f32::NAN;
    for i in 1..=300u64 {
        bridge.set_player_input(
            player,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, i);
        let t = sim.world().get::<Transform>(player).expect("transform");
        last = t.translation.x;
        if t.translation.y < -1.0 {
            return (false, last);
        }
    }
    (last > LEDGE_X + gap + 0.3, last)
}

#[test]
#[ignore = "sonda"]
fn measure_that_he_walks_off_at_all() {
    // ⚠️ O CONTROLE: sem ele a tabela acima podia estar a medir um personagem
    // que nunca sai do lugar, e toda a saliência seria zero por vácuo.
    let w = walk_off(5, 4.0, 0.9);
    let held = w.last_held(0.005);
    let lowest = w.trace.iter().map(|(_, y)| *y).fold(f32::MAX, f32::min);
    println!(
        "\n=== CONTROLE: ele ANDA e CAI ===\n  ultimo seguro x={held:.4}, fundo y={lowest:.4}"
    );
    assert!(
        held.is_finite() && held > LEDGE_X - 1.0,
        "o personagem tem de CHEGAR a' quina, senao a sonda mede o nada"
    );
    assert!(
        lowest < -1.0,
        "ele tem de CAIR, senao nao ha' saliencia a medir"
    );
}

#[test]
#[ignore = "sonda"]
fn measure_the_armed_trace() {
    for speed in [2.0f32, 4.0] {
        println!("\n=== TRACO COM A TRAVA ARMADA, {speed} m/s (quina x=0) ===");
        let w = walk_off_with(3, speed, 0.9, true);
        for (x, y) in w.trace.iter().filter(|(x, _)| *x > -0.9) {
            println!("  x={x:>9.4}   y={y:>9.4}");
            if *y < 0.0 {
                break;
            }
        }
    }
}
