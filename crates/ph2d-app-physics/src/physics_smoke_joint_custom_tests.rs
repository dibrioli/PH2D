//! A sonda da cena 79 + os gates que mantêm a mensagem dela honesta
//! (W-JointCustom).

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::{FrozenScene, PhysicsBridge};

fn staged() -> (SimWorld, PhysicsBridge) {
    let mut sim = SimWorld::new();
    build_joint_custom_scene(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    (sim, PhysicsBridge::new())
}

fn named(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("a cena 79 nao montou '{name}'"))
}

fn pose(sim: &mut SimWorld, name: &str) -> [f32; 3] {
    let e = named(sim, name);
    let t = sim.world().get::<Transform>(e).expect("transform");
    [t.translation.x, t.translation.y, t.rotation]
}

/// Toca `ticks` e devolve, para cada nome, `(excursão em X, |giro| acumulado,
/// queda em Y)` — as três grandezas que as três bancadas afirmam.
fn run(ticks: u64, watch: &[&str]) -> Vec<[f32; 3]> {
    let (mut sim, mut bridge) = staged();
    let start: Vec<[f32; 3]> = watch.iter().map(|n| pose(&mut sim, n)).collect();
    let mut spin: Vec<f32> = vec![0.0; watch.len()];
    let mut prev: Vec<f32> = start.iter().map(|p| p[2]).collect();
    let mut span: Vec<(f32, f32)> = start.iter().map(|p| (p[0], p[0])).collect();
    let mut frozen = FrozenScene;
    for t in 1..=ticks {
        bridge.dispatch_with_scene(&mut sim, true, t, &mut frozen);
        for (i, n) in watch.iter().enumerate() {
            let p = pose(&mut sim, n);
            // ⚠️ **O giro é acumulado por DIFERENÇA, nunca lido do `rotation`**:
            // ele wrapa em ±π, e uma pá que dá voltas devolve ruído — a lição do
            // W-AreaTorque, que esta cena reencontra na bancada de baixo.
            let mut d = p[2] - prev[i];
            if d > std::f32::consts::PI {
                d -= std::f32::consts::TAU;
            } else if d < -std::f32::consts::PI {
                d += std::f32::consts::TAU;
            }
            spin[i] += d.abs();
            prev[i] = p[2];
            span[i].0 = span[i].0.min(p[0]);
            span[i].1 = span[i].1.max(p[0]);
        }
    }
    watch
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let p = pose(&mut sim, n);
            [span[i].1 - span[i].0, spin[i], p[1] - start[i][1]]
        })
        .collect()
}

/// **A sonda.** `cargo test -p ph2d-host-desktop --release probe_smoke_79 --
/// --ignored --nocapture`
#[test]
#[ignore = "sonda de medição"]
fn probe_smoke_79() {
    let watch = ["RailCart", "CtrlCart", "SlotBlock", "SpinBar", "SlideBar"];
    let r = run(300, &watch);
    println!("\n=== cena 79 — o joint descrito por eixo (5 s) ===");
    println!(
        "  {:<12} {:>10} {:>10} {:>10}",
        "corpo", "curso X", "giro", "queda Y"
    );
    for (n, v) in watch.iter().zip(&r) {
        println!("  {n:<12} {:>10.2} {:>10.2} {:>10.2}", v[0], v[1], v[2]);
    }
}

/// **A cena monta as cinco máquinas que a mensagem nomeia.**
#[test]
fn the_scene_builds_the_machines_it_names() {
    let (mut sim, _) = staged();
    for n in ["RailCart", "CtrlCart", "SlotBlock", "SpinBar", "SlideBar"] {
        let _ = named(&mut sim, n);
    }
}

/// **O carrinho do Custom DESLIZA E GIRA; o Slider ao lado só desliza.**
///
/// O oráculo da wave inteira, e ele não conhece função nenhuma dela: dois
/// carrinhos, o mesmo curso, o mesmo motor — só a configuração de eixos difere.
/// Sem o controle, *"o carrinho girou"* seria satisfeito por qualquer joint que
/// não travasse nada.
#[test]
fn the_custom_cart_slides_and_spins_where_the_slider_only_slides() {
    let r = run(300, &["RailCart", "CtrlCart"]);
    let (custom, slider) = (r[0], r[1]);
    assert!(
        custom[0] > 1.0 && slider[0] > 1.0,
        "os dois carrinhos têm de deslizar: {custom:?} contra {slider:?}"
    );
    assert!(
        custom[1] > 1.0,
        "o eixo de rotação LIVRE tem de deixar o carrinho girar: {custom:?}"
    );
    assert!(
        slider[1] < 0.05,
        "um Slider proíbe o giro — é isso que o Custom acrescenta: {slider:?}"
    );
}

/// **O bloco da calha cai até o batente e para lá** — um eixo `Limited` na
/// vertical.
#[test]
fn the_slot_block_falls_to_its_stop_and_stays() {
    let r = run(300, &["SlotBlock"]);
    let drop = -r[0][2];
    assert!(
        (drop - 1.5).abs() < 0.15,
        "o batente de -1,5 m tem de segurar o bloco: caiu {drop:.3} m"
    );
    // E o eixo X TRAVADO não o deixa sair da calha.
    assert!(
        r[0][0] < 0.05,
        "X travado não pode deixar o bloco sair da calha: curso {:.3} m",
        r[0][0]
    );
}

/// **A configuração é IDÊNTICA e só o eixo do motor difere** — um gira, o outro
/// desliza.
///
/// É o gate da decisão de projeto: *"o motor dirige o primeiro eixo livre"*
/// daria X nos dois, e este par é o que torna essa mágica inexprimível.
#[test]
fn the_authored_motor_axis_decides_which_way_it_moves() {
    let r = run(300, &["SpinBar", "SlideBar"]);
    let (spun, slid) = (r[0], r[1]);
    assert!(
        spun[1] > 2.0,
        "o motor no eixo de rotação tem de girar a barra: {spun:?}"
    );
    // ⚠️ A barra deslizante percorre o CURSO INTEIRO que ela tem (±2,0 m, medido
    // 2,00) e para no batente — a barra do gate fica abaixo disso de propósito,
    // porque o número exato é o do batente e não uma propriedade do motor.
    assert!(
        slid[0] > 1.5,
        "o mesmo motor no eixo X tem de deslizar a barra: {slid:?}"
    );
    assert!(
        spun[0] < 0.5,
        "a barra girada não é a que desliza: {spun:?}"
    );
}
