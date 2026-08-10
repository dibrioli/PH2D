//! Gates da cena **O COMPASSO** (`PH2D_GPU_COOK_DEMO=25`).
//!
//! Os dois fatos que o olho tem de separar são medidos aqui separadamente, porque eles
//! falham de formas diferentes: um `carry` mal ligado dá **quatro vezes mais** inchaços, e
//! um envelope ausente dá o número certo deles com **um quadro** de duração cada. Uma cena
//! que só afirmasse *"algo pisca"* passaria nos dois defeitos.

use super::*;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

const FPS: f64 = 60.0;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");
    reg
}

/// Roda a cena por `secs` e devolve o Size do primeiro ponto a cada tique — a grandeza que
/// o envelope de fato dirige.
fn run(secs: f64) -> Vec<f32> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_adsr_demo_document(&mut doc, &reg).expect("cena bem tipada");
    let sink = sinks[0];
    let mut cook = Cook::new();
    let mut sizes = Vec::new();
    for k in 0..=((secs * FPS) as u64) {
        let t = k as f64 / FPS;
        let out = cook.cook(&doc.graph, &reg, sink, t).expect("cozinha");
        sizes.push(first_size(out[0].as_stream()));
        cook.advance_tick(&doc.graph, &reg, t)
            .expect("fecha o tique");
    }
    sizes
}

/// O Size do primeiro ponto. ⚠️ **`size` é `Vec2`, não `Scalar`** — um ponto tem largura e
/// altura —, e ler o variant errado num `if let` não falha: ele simplesmente **não casa**, o
/// corpo nunca roda e o gate fica VERDE afirmando nada. Foi o que aconteceu com a primeira
/// versão do controle desta cena, e é por isso que este helper existe num lugar só.
fn first_size(s: &ph2d_nodegraph::attr::Stream) -> f32 {
    match s.get("size") {
        Some(Column::Vec2(v)) if !v.is_empty() => v[0][0],
        _ => f32::NAN,
    }
}

/// Os intervalos (em tiques) em que o ponto está inchado — um por inchaço.
fn swells(sizes: &[f32]) -> Vec<(usize, usize)> {
    let live = |v: f32| v.is_finite() && v > DOT_REST + 0.02;
    let mut out = Vec::new();
    let mut start = None;
    for (i, &v) in sizes.iter().enumerate() {
        match (start, live(v)) {
            (None, true) => start = Some(i),
            (Some(a), false) => {
                out.push((a, i));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(a) = start {
        out.push((a, sizes.len()));
    }
    out
}

/// O tamanho de repouso, lido da própria cena em vez de reescrito aqui.
const DOT_REST: f32 = 0.16;

/// **O inchaço acontece uma vez a cada QUATRO batidas** — o divisor de relógio, que é a
/// razão de o `carry` existir. ⚠️ O CONTROLE é a aritmética do metrônomo: em 4 s cabem 10
/// batidas e têm de caber **2 ou 3** inchaços, não 10.
#[test]
fn o_carry_divide_o_metronomo_por_quatro() {
    let sizes = run(4.0);
    let bumps = swells(&sizes);
    let period_ticks = (BEAT * DIVIDE_BY) as f64 * FPS;
    assert!(
        (2..=3).contains(&bumps.len()),
        "10 batidas em 4 s dão 2-3 compassos, não {} ({bumps:?})",
        bumps.len()
    );
    // E o espaçamento entre eles É o compasso, não a batida.
    for w in bumps.windows(2) {
        let gap = (w[1].0 - w[0].0) as f64;
        assert!(
            (gap - period_ticks).abs() <= 2.0,
            "o intervalo é o compasso ({period_ticks} tiques), medido {gap}"
        );
    }
}

/// **O inchaço tem DURAÇÃO, não é uma piscada** — o envelope. Sem ele o `carry` acenderia o
/// ponto por um tique, e a cena mostraria o número certo de eventos com nada visível.
#[test]
fn o_envelope_da_duracao_ao_disparo() {
    let sizes = run(4.0);
    let bumps = swells(&sizes);
    assert!(!bumps.is_empty(), "algum inchaço aconteceu");
    for (a, b) in &bumps {
        let ticks = b - a;
        assert!(
            ticks > 20,
            "um envelope dura dezenas de tiques, não {ticks} (a piscada de um `carry` cru)"
        );
    }
    // E a FORMA: dentro do inchaço o tamanho sobe até um pico e volta — não é um degrau.
    let (a, b) = bumps[0];
    let peak = sizes[a..b].iter().cloned().fold(f32::MIN, f32::max);
    assert!(
        peak > DOT_REST + SWELL * 0.8,
        "o pico chega perto do topo: {peak}"
    );
    assert!(
        sizes[b - 1] < peak * 0.9,
        "e ele DESCE antes de acabar ({} contra o pico {peak})",
        sizes[b - 1]
    );
}

/// **O CONTROLE da cena:** sem o fio do `carry` nada incha. É o que separa *"a cena
/// funciona"* de *"a cena desenharia isso de qualquer jeito"* — o Size de repouso vem do
/// `motion.scale`, e um drive desligado o deixaria plano.
#[test]
fn sem_o_fio_do_carry_o_ponto_fica_parado() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_adsr_demo_document(&mut doc, &reg).expect("cena bem tipada");
    let sink = sinks[0];
    let env = doc
        .graph
        .nodes()
        .iter()
        .position(|n| n.type_name == "pulse.adsr")
        .map(|i| ph2d_nodegraph::graph::NodeId(i as u32))
        .expect("a cena tem um pulse.adsr");
    doc.graph.disconnect(env, 0).expect("a cena fia o carry");

    let mut cook = Cook::new();
    for k in 0..=240u64 {
        let t = k as f64 / FPS;
        let out = cook.cook(&doc.graph, &reg, sink, t).expect("cozinha");
        let size = first_size(out[0].as_stream());
        assert!(
            (size - DOT_REST).abs() < 0.02,
            "sem gatilho o ponto fica no repouso (tique {k}: {size})"
        );
        cook.advance_tick(&doc.graph, &reg, t)
            .expect("fecha o tique");
    }
}

/// Sonda: o retrato da cena em números, para a mensagem de smoke MEDIR em vez de estimar.
///
/// `cargo test -p ph2d-host-desktop --bins probe_o_compasso -- --ignored --nocapture`
#[test]
#[ignore = "sonda de diagnóstico: imprime, não afirma"]
fn probe_o_compasso() {
    let sizes = run(4.0);
    let bumps = swells(&sizes);
    println!(
        "[compasso] batida {BEAT}s / divisor {DIVIDE_BY} => compasso {}s",
        BEAT * DIVIDE_BY
    );
    println!("  inchacos em 4 s: {}", bumps.len());
    for (a, b) in &bumps {
        let peak = sizes[*a..*b].iter().cloned().fold(f32::MIN, f32::max);
        println!(
            "    tiques {a}..{b} ({} = {:.2}s) pico {peak:.3} contra repouso {DOT_REST}",
            b - a,
            (b - a) as f32 / 60.0
        );
    }
}
