//! Gates da cena `=31` — e a SONDA de onde saíram os números do anúncio.

use super::*;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// Cozinha a cena e devolve `(a maior velocidade vista em QUALQUER tique, as posições finais)`.
///
/// O pico é sobre a corrida inteira de propósito: um teto é sobre o que a sim **chega a fazer**,
/// e ler só o último quadro perderia justamente o estilingue.
fn run(limit: f32, secs: f64) -> (f32, Vec<[f32; 2]>) {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_speed_demo_document(&mut doc, &reg, limit).expect("a cena é bem tipada");
    let mut cook = Cook::new();
    let last = (secs * 60.0) as u64;
    let (mut peak, mut p) = (0.0f32, vec![]);
    for k in 0..=last {
        let t = k as f64 / 60.0;
        let s = cook.cook(&doc.graph, &reg, sinks[0], t).expect("cozinha")[0]
            .as_stream()
            .clone();
        if let Some(Column::Vec2(v)) = s.get("vel") {
            peak = v
                .iter()
                .fold(peak, |m, w| m.max((w[0] * w[0] + w[1] * w[1]).sqrt()));
        }
        if k == last {
            p = match s.get("P") {
                Some(Column::Vec2(v)) => v.clone(),
                _ => vec![],
            };
        }
        cook.advance_tick(&doc.graph, &reg, t).expect("avança");
    }
    (peak, p)
}

/// **A cena é bem tipada e a nuvem inteira está lá.**
#[test]
fn the_scene_builds_with_the_whole_cloud() {
    let (_, p) = run(LIMIT, 2.0);
    assert_eq!(p.len(), (ROWS * COLS) as usize);
}

/// **O TETO MORDE, E O CONTROLE PROVA QUE HAVIA O QUE MORDER.**
///
/// A MESMA cena e um único número diferente. Sem teto o atrator estilinga a nuvem a uma
/// velocidade muito acima do limite; com ele nada passa do limite, em tique nenhum.
///
/// FALSIFICADO por um teto que nunca alcança o passo: os dois braços mediriam o mesmo pico.
#[test]
fn the_ceiling_bites_and_the_control_shows_there_was_something_to_bite() {
    let (free, _) = run(0.0, 2.0);
    let (capped, _) = run(LIMIT, 2.0);
    assert!(
        free > LIMIT * 2.0,
        "o controle tem de estilingar de verdade: pico {free} contra um teto de {LIMIT}"
    );
    assert!(
        capped <= LIMIT + 1e-3,
        "nada pode passar do teto: pico {capped} contra {LIMIT}"
    );
}

/// **E a consequência é a que o artista VÊ: a nuvem FICA.**
///
/// Sem teto o estilingue joga elementos para longe; com ele a nuvem permanece contida. O oráculo
/// é a maior distância ao centro, que é o que "sumiu de quadro" quer dizer.
#[test]
fn the_cloud_stays_in_frame_when_the_ceiling_is_armed() {
    let far = |p: &[[f32; 2]]| {
        p.iter()
            .fold(0.0f32, |m, q| m.max((q[0] * q[0] + q[1] * q[1]).sqrt()))
    };
    let (_, free) = run(0.0, 2.0);
    let (_, capped) = run(LIMIT, 2.0);
    assert!(
        far(&free) > far(&capped) * 1.5,
        "sem teto {} contra {} com teto",
        far(&free),
        far(&capped)
    );
}

/// **A SONDA** — de onde saem os números do anúncio e do doc.
///
/// `cargo test -p ph2d-host-desktop --bins probe_speed_ceiling -- --ignored --nocapture`
#[test]
#[ignore]
fn probe_speed_ceiling() {
    for limit in [0.0f32, LIMIT, 20.0] {
        let (peak, p) = run(limit, 2.0);
        let far = p
            .iter()
            .fold(0.0f32, |m, q| m.max((q[0] * q[0] + q[1] * q[1]).sqrt()));
        eprintln!("teto {limit}: pico de {peak:.2} u/s, o mais longe a {far:.2} do centro");
    }
}
