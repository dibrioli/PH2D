//! A sonda da cena 58 + o gate que mantém a mensagem honesta.

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::{PhysicsBridge, PulleyWheel, WrapSide};

fn run(ticks: u64) -> (SimWorld, PhysicsBridge) {
    let mut sim = SimWorld::new();
    build(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    for t in 1..=ticks {
        bridge.dispatch(&mut sim, false, t);
    }
    (sim, bridge)
}

fn y_of(sim: &mut SimWorld, name: &str) -> f32 {
    let mut q = sim.world_mut().query::<(&Name, &Transform)>();
    q.iter(sim.world())
        .find(|(n, _)| n.as_str() == name)
        .map(|(_, t)| t.translation.y)
        .expect("corpo vivo")
}

/// A sonda: roda a cena e imprime o que ela de fato faz.
///
/// `cargo test -p ph2d-host-desktop --bins probe_smoke_58 -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate"]
fn probe_smoke_58() {
    let (mut sim, _) = run(180);
    println!("\n=== CENA 58 — o elevador (3 s) ===");
    for tag in ["Simple", "Zigzag"] {
        let load = START_Y - y_of(&mut sim, &format!("{tag} Load"));
        let cw = y_of(&mut sim, &format!("{tag} Counterweight")) - START_Y;
        println!("{tag:>8}: carga desceu {load:>7.4} m | contrapeso subiu {cw:>7.4} m");
    }
}

/// **A mensagem afirma os números que a simulação produz.**
///
/// O molde do irmão da cena 57: uma cena que diz *"a talha ergue a carga"* e uma
/// simulação que a deixa cair é uma demonstração que ensina o oposto do que a
/// wave construiu — e nada além deste gate reconferiria isso. Foi ele que pegou a
/// razão invertida da primeira versão desta cena.
#[test]
fn the_scene_message_states_the_numbers_the_sim_produces() {
    let (mut sim, _) = run(180);
    let simple_drop = START_Y - y_of(&mut sim, "Simple Load");
    let simple_rise = y_of(&mut sim, "Simple Counterweight") - START_Y;
    let zig_drop = START_Y - y_of(&mut sim, "Zigzag Load");

    for (got, said, what) in [
        (
            simple_drop,
            MEASURED_SIMPLE_LOAD_DROP,
            "queda da carga simples",
        ),
        (simple_rise, MEASURED_SIMPLE_CW_RISE, "subida do contrapeso"),
        (
            zig_drop,
            MEASURED_ZIGZAG_LOAD_DROP,
            "queda da carga no ziguezague",
        ),
    ] {
        assert!(
            (got - said).abs() < 0.05,
            "{what}: a mensagem diz {said:.2} m e a sim faz {got:.4} m"
        );
    }

    // **A corda é inextensível**, e é isso que o par de números do rig simples
    // diz junto — um afirmando 1,5 m sem o outro descreveria uma queda livre.
    assert!(
        (simple_drop - simple_rise).abs() < 0.05,
        "o que um lado desce o outro sobe: desceu {simple_drop:.4} e subiu {simple_rise:.4}"
    );

    // **E o ZIGUEZAGUE não muda a mecânica** — é a afirmação inteira do item 3 da
    // mensagem, e o que substituiu a talha falsa. Quatro roldanas em vez de duas,
    // raios diferentes, e o mesmo par de massas anda o mesmo tanto: numa corda
    // única a tensão é uniforme. Se este gate abrir, a cena voltou a ensinar que
    // roldanas dão vantagem mecânica.
    assert!(
        (simple_drop - zig_drop).abs() < 0.05,
        "mais roldanas não podem mudar o resultado: {simple_drop:.4} contra {zig_drop:.4}"
    );
}

/// **A corda passa na SUPERFÍCIE da roldana, não pelo centro** — o pedido (5) do
/// artista, afirmado sobre a rota que o SOLVER usa.
///
/// ⚠️ O oráculo é a DISTÂNCIA do centro da roda ao ponto onde a corda a toca: ela
/// tem de ser o raio, e não zero. Um gate que só perguntasse *"a rota existe?"*
/// ficaria verde sobre o modelo de ponto, que é exatamente o que esta wave
/// substituiu.
#[test]
fn the_rope_touches_the_rim_and_not_the_centre() {
    let (mut sim, bridge) = run(1);
    let rope = {
        let mut q = sim.world_mut().query::<(ph2d_ecs::Entity, &Name)>();
        q.iter(sim.world())
            .find(|(_, n)| n.as_str() == "Simple Rope")
            .map(|(e, _)| e)
            .expect("a corda existe")
    };
    let wheels: Vec<_> = bridge.rope_wheels(rope).map(|(_, w)| w).collect();
    assert_eq!(wheels.len(), 2, "a corda verde tem duas roldanas");
    let v = bridge
        .joint_views()
        .find(|v| v.entity == rope)
        .expect("a corda tem view");
    let mut segs = Vec::new();
    ph2d_physics_ecs::rope_route::route(v.anchor_a, v.anchor_b, &wheels, &mut segs)
        .expect("a rota existe");
    // O trecho `i` CHEGA na roldana `i`: o ponto de toque é o `to` dele.
    for (i, w) in wheels.iter().enumerate() {
        let t = segs[i].to;
        let d = (t[0] - w.centre[0]).hypot(t[1] - w.centre[1]);
        assert!(
            (d - w.radius).abs() < 1.0e-3,
            "roldana {i}: a corda toca a {d:.4} m do centro, e o raio é {:.4}",
            w.radius
        );
        assert!(w.radius > 0.1, "a cena tem de ter roda com tamanho visível");
    }
}

/// **O que forçar `Over`/`Under` numa roldana faz com a corda** (W-Pulley W1-E).
///
/// A §13 é o único gesto que alcança este campo, então a cena de smoke pede que
/// o artista o exercite — e o número que ela cita sai daqui.
///
/// `cargo test -p ph2d-host-desktop --bins probe_wrap_58 -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate"]
fn probe_wrap_58() {
    println!("\n=== CENA 58 — o lado da corda na 2a roldana do ziguezague ===");
    for wrap in [WrapSide::Auto, WrapSide::Over, WrapSide::Under] {
        let mut sim = SimWorld::new();
        build(sim.world_mut());
        {
            let mut q = sim.world_mut().query::<(&Name, &mut PulleyWheel)>();
            for (n, mut w) in q.iter_mut(sim.world_mut()) {
                if n.as_str() == "Zigzag Rope Wheel 2" {
                    w.wrap = wrap;
                }
            }
        }
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 1);
        let rope = {
            let mut q = sim.world_mut().query::<(ph2d_ecs::Entity, &Name)>();
            q.iter(sim.world())
                .find(|(_, n)| n.as_str() == "Zigzag Rope")
                .map(|(e, _)| e)
                .expect("a corda existe")
        };
        let side = bridge.rope_wheels(rope).nth(1).map(|(_, w)| w.side);
        let drop = START_Y - y_of(&mut sim, "Zigzag Load");
        println!("{wrap:>6?}: lado resolvido {side:?} | carga em 1 tick {drop:+.6} m");
    }
}
