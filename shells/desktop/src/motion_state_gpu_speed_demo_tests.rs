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

/// **E A CONSEQUÊNCIA É A QUE O ARTISTA VÊ: O QUADRO PARA, E O CONTROLE VOA EMBORA.**
///
/// ⚠️ Este gate substitui um que media o raio final aos 2 s e ficava **VERDE sobre a cena que o
/// Enio reprovou** (*"cada Play um resultado diferente"*, 2026-08-11): raio não é quietude — a
/// nuvem tinha raio estável e **orbitava para sempre** no teto de velocidade, então dois Plays
/// interrompidos em instantes diferentes mostravam fases diferentes da mesma órbita. O oráculo
/// certo é a CONVERGÊNCIA: o raio tem de assentar e FICAR.
///
/// Medido: com teto o raio cai 4,68 → 2,86 → **1,2000 e não sai mais** (os elementos encostados
/// no núcleo, `CORE_R`); sem teto ele vai a 24,8 → 76,6 → **200,6** e continua — a nuvem some de
/// quadro para sempre, que é exatamente o que um teto de velocidade existe para impedir.
#[test]
fn the_frame_settles_and_the_control_flies_away() {
    let far = |p: &[[f32; 2]]| {
        p.iter()
            .fold(0.0f32, |m, q| m.max((q[0] * q[0] + q[1] * q[1]).sqrt()))
    };
    // Assentou: o raio aos 3 s é o do núcleo, e aos 12 s continua o mesmo.
    let (_, at3) = run(LIMIT, 3.0);
    let (_, at12) = run(LIMIT, 12.0);
    assert!(
        (far(&at3) - CORE_R).abs() < 0.05,
        "a nuvem tem de repousar no núcleo: raio {} contra {CORE_R}",
        far(&at3)
    );
    assert!(
        (far(&at12) - far(&at3)).abs() < 0.05,
        "o quadro tem de FICAR: {} aos 3 s contra {} aos 12 s",
        far(&at3),
        far(&at12)
    );
    // E o controle prova que havia o que segurar: sem teto ela não volta nunca.
    let (_, free) = run(0.0, 12.0);
    assert!(
        far(&free) > 20.0 * CORE_R,
        "sem teto a nuvem tem de sumir de quadro: raio {}",
        far(&free)
    );
}

/// **SONDA — o ALCANCE ao longo da corrida**, que é o que o artista vê enquanto a nuvem cai.
/// O quadro FINAL é o mesmo com e sem teto (o núcleo captura os dois); a pergunta é se o
/// estilingue ainda leva a nuvem mais longe ANTES de assentar.
#[test]
#[ignore]
fn probe_reach_during_the_fall() {
    for limit in [LIMIT, 0.0] {
        let reg = registry();
        let mut doc = MotionDoc::default();
        let sinks = build_gpu_speed_demo_document(&mut doc, &reg, limit).expect("bem tipada");
        let mut cook = Cook::new();
        let mut worst = 0.0f32;
        for k in 0..=180u64 {
            let t = k as f64 / 60.0;
            let s = cook.cook(&doc.graph, &reg, sinks[0], t).expect("cozinha")[0]
                .as_stream()
                .clone();
            if let Some(Column::Vec2(v)) = s.get("P") {
                worst = v
                    .iter()
                    .fold(worst, |m, q| m.max((q[0] * q[0] + q[1] * q[1]).sqrt()));
            }
            cook.advance_tick(&doc.graph, &reg, t).expect("avança");
        }
        eprintln!("teto {limit}: alcance MÁXIMO durante a corrida {worst:.4}");
    }
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

/// **SONDA — O PUMP DA CPU REBOBINA PARA O TIQUE 0?** (a metade que a sonda do
/// `ph2d-gpu-cook` nao alcanca: o produto cai no pump sempre que a rota do dispositivo recusa.)
///
/// `cargo test -p ph2d-host-desktop --lib probe_cpu_pump_rewind -- --ignored --nocapture`
#[test]
#[ignore]
fn probe_cpu_pump_rewind() {
    use crate::render_loop::motion_bridge::ticks_owed;
    use ph2d_eval_motion::MotionCookPump;

    let go = |seq: &[u64]| -> Vec<[f32; 2]> {
        let reg = registry();
        let mut doc = MotionDoc::default();
        let sinks = build_gpu_speed_demo_document(&mut doc, &reg, LIMIT).expect("bem tipada");
        let scopes = ph2d_node_motion_time_remap::time_scopes(&doc.graph, &reg);
        let mut pump = MotionCookPump::new();
        for &target in seq {
            for tick in ticks_owed(pump.last_cooked_tick(), target) {
                pump.advance_or_scrub_scoped(
                    &doc.graph,
                    &reg,
                    &sinks,
                    tick,
                    |k| k as f64 / 60.0,
                    [0.0, 0.0, 1.0, 1.0],
                    [1.0, 1.0],
                    &scopes,
                );
            }
        }
        pump.instances.iter().map(|i| i.world_pos).collect()
    };

    let far = |p: &[[f32; 2]]| {
        p.iter()
            .fold(0.0f32, |m, q| m.max((q[0] * q[0] + q[1] * q[1]).sqrt()))
    };
    let delta = |a: &[[f32; 2]], b: &[[f32; 2]]| {
        a.iter()
            .zip(b)
            .flat_map(|(x, y)| (0..2).map(move |k| (x[k] - y[k]).abs()))
            .fold(0.0f32, f32::max)
    };

    let fresco = go(&[0]);
    let tocado: Vec<u64> = (0..=120).collect();
    let mut seq = tocado.clone();
    seq.push(0);
    let rebobinado = go(&seq);
    eprintln!(
        "fresco {:.4} do centro | rebobinado {:.4} | delta {:.9}",
        far(&fresco),
        far(&rebobinado),
        delta(&fresco, &rebobinado)
    );

    // E o 2o Play: 0..=120 outra vez, DEPOIS do rewind.
    let mut seq2 = seq.clone();
    seq2.extend(1..=120u64);
    let play2 = go(&seq2);
    let play1 = go(&tocado);
    eprintln!(
        "play1 {:.4} | play2 {:.4} | delta {:.9}",
        far(&play1),
        far(&play2),
        delta(&play1, &play2)
    );
}

/// **SONDA — A CENA ASSENTA?** Se a nuvem nunca para, dois Plays interrompidos em instantes
/// diferentes mostram quadros diferentes — e isso e' DESENHO DE CENA, nao motor.
#[test]
#[ignore]
fn probe_does_the_scene_settle() {
    let col = |limit: f32, secs: f64| -> (f32, f32, f32) {
        let (peak, p) = run(limit, secs);
        let (_, q) = run(limit, secs + 1.0 / 60.0);
        let mov = p
            .iter()
            .zip(&q)
            .flat_map(|(a, b)| (0..2).map(move |k| (a[k] - b[k]).abs()))
            .fold(0.0f32, f32::max);
        let far = p
            .iter()
            .fold(0.0f32, |m, r| m.max((r[0] * r[0] + r[1] * r[1]).sqrt()));
        (far, peak, mov)
    };
    for secs in [1.0f64, 2.0, 3.0, 5.0, 8.0, 12.0] {
        let (fa, pa, ma) = col(LIMIT, secs);
        let (fb, pb, mb) = col(0.0, secs);
        eprintln!(
            "t={secs:>5.2}s   TETO raio {fa:>7.4} pico {pa:>6.2} mov {ma:.6}                SEM raio {fb:>8.4} pico {pb:>7.2} mov {mb:.6}"
        );
    }
}
