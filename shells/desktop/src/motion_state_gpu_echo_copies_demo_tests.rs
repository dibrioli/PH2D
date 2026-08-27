//! Gates da cena `=106` — as cópias atrasadas (folha 08, célula 41).
//!
//! ⚠️ **Estes gates montam o LEQUE à mão**, porque quem o monta em produção é o shell
//! (`motion_bridge`) e não a cena. Sem ele as duas fileiras saem idênticas — que é
//! exactamente o que o gate `the_scene_needs_the_shell_to_land_the_fan` afirma.

use super::*;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

const TICKS: usize = 90;

fn scene() -> (MotionDoc, NodeRegistry, Vec<NodeId>) {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_echo_copies_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

/// As posições de cada metade no último tique, com os leques do shell pousados.
fn run(
    doc: &MotionDoc,
    reg: &NodeRegistry,
    sinks: &[NodeId],
    with_fans: bool,
) -> Vec<Vec<[f32; 2]>> {
    let fans = if with_fans {
        ph2d_node_motion_clone::fan::time_fans(&doc.graph, reg, 1.0 / 60.0)
    } else {
        ph2d_nodegraph::cook::TimeFans::new()
    };
    let scopes = ph2d_nodegraph::cook::TimeScopes::new();
    let mut cook = Cook::new();
    let mut out = vec![Vec::new(); sinks.len()];
    for k in 0..TICKS {
        let t = k as f64 / 60.0;
        for (s, sink) in sinks.iter().enumerate() {
            let v = cook
                .cook_scoped_fanned(&doc.graph, reg, *sink, t, &scopes, &fans)
                .expect("coze");
            if k == TICKS - 1 {
                out[s] = match v[0].as_stream().get("P") {
                    Some(Column::Vec2(p)) => p.clone(),
                    _ => Vec::new(),
                };
            }
        }
        cook.advance_tick_fanned(&doc.graph, reg, t, &scopes, &fans)
            .expect("avanca");
    }
    out
}

/// A cena monta as duas metades, cada uma com as `COPIES` cópias.
#[test]
fn the_echo_scene_builds_both_rows() {
    let (doc, reg, sinks) = scene();
    assert_eq!(sinks.len(), 2, "duas fileiras");
    for (k, p) in run(&doc, &reg, &sinks, true).into_iter().enumerate() {
        assert_eq!(p.len(), COPIES as usize, "fileira {k}");
        assert!(
            p.iter().all(|q| q.iter().all(|x| x.is_finite())),
            "fileira {k}"
        );
    }
}

/// ⭐⭐ **A fileira de cima é UM ponto; a de baixo é um RASTRO.** A régua é a envergadura
/// interna: sem atraso as cópias estão empilhadas (distância zero), com atraso elas ocupam
/// pedaços diferentes da volta.
#[test]
fn only_the_delayed_row_spreads_over_the_orbit() {
    let (doc, reg, sinks) = scene();
    let r = run(&doc, &reg, &sinks, true);
    let spread = |p: &[[f32; 2]]| {
        let mut m = 0.0f32;
        for a in p {
            for b in p {
                m = m.max(((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt());
            }
        }
        m
    };
    let plain = spread(&r[0]);
    let echoed = spread(&r[1]);
    assert!(plain < 1e-4, "sem atraso as copias empilham: {plain}");
    assert!(
        echoed > 0.5,
        "com atraso elas espalham-se pela volta: {echoed}"
    );
}

/// ⚠️ **A CENA sozinha não faz nada — quem pousa o leque é o SHELL.** Sem ele as duas
/// fileiras são idênticas, e é por isso que o `motion_bridge` tem de chamar o terceiro
/// produtor ao lado dos outros dois.
#[test]
fn the_scene_needs_the_shell_to_land_the_fan() {
    let (doc, reg, sinks) = scene();
    let without = run(&doc, &reg, &sinks, false);
    // ⚠️ Comparo a FORMA e não a posição: as duas fileiras vivem em `y` diferentes por
    // construção (é assim que a cena as separa na tela), então subtraio a altura de cada uma.
    let shape =
        |p: &[[f32; 2]], y: f32| -> Vec<[f32; 2]> { p.iter().map(|q| [q[0], q[1] - y]).collect() };
    // ⚠️ Com TOLERÂNCIA, não `assert_eq`: subtrair `1,9` de um lado e `−1,9` do outro arredonda
    // diferente, e as duas formas saíam a **1 ULP** uma da outra. É a régua, não o produto.
    let (a, b) = (shape(&without[0], 1.9), shape(&without[1], -1.9));
    for (i, (p, q)) in a.iter().zip(&b).enumerate() {
        assert!(
            (p[0] - q[0]).abs() < 1e-5 && (p[1] - q[1]).abs() < 1e-5,
            "sem o leque as duas fileiras desenham a mesma coisa (copia {i}: {p:?} vs {q:?})"
        );
    }
    // E a fonte que prova que o shell o faz.
    let src = include_str!("render_loop/motion_bridge.rs");
    assert!(
        src.contains("ph2d_node_motion_clone::fan::time_fans"),
        "o shell tem de pousar o terceiro leque"
    );
}

/// ⚠️ **Os números que o anúncio cita vivem em `const`.**
#[test]
fn the_announcement_cites_the_numbers_the_scene_uses() {
    let src = include_str!("motion_state_demo_announce.rs");
    for k in [
        "gpu_echo_copies_demo::COPIES",
        "gpu_echo_copies_demo::OFFSET",
    ] {
        assert!(src.contains(k), "o anuncio tem de citar `{k}`");
    }
}
