//! Gates da cena **O COMPASSO** (`PH2D_GPU_COOK_DEMO=25`).
//!
//! Os dois fatos que o olho tem de separar são medidos aqui separadamente, porque eles
//! falham de formas diferentes: um `carry` mal ligado dá **quatro vezes mais** crescimentos,
//! e um envelope ausente dá o número certo deles com **um quadro** de duração cada. Uma cena
//! que só afirmasse *"algo pisca"* passaria nos dois defeitos.
//!
//! ⚠️ **E o gate que carrega a wave conta um relógio contra o OUTRO**, não contra uma
//! constante que eu escrevi: `os_pulos_entre_dois_crescimentos_sao_quatro` pergunta
//! exatamente o que o olho pergunta na tela. O irmão que compara o espaçamento contra
//! `BEAT × DIVIDE_BY` fica, e não é redundante — ele pega o relógio inteiro derivar, que a
//! razão entre dois relógios derivando juntos **não vê**.

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

/// O que o olho vê num tique: o Size do primeiro ponto (o CRESCIMENTO) e o Y dele (o PULO).
#[derive(Copy, Clone)]
struct Frame {
    size: f32,
    y: f32,
}

/// Roda a cena por `secs` e devolve as duas grandezas a cada tique.
fn run(secs: f64) -> Vec<Frame> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_adsr_demo_document(&mut doc, &reg).expect("cena bem tipada");
    let sink = sinks[0];
    let mut cook = Cook::new();
    let mut frames = Vec::new();
    for k in 0..=((secs * FPS) as u64) {
        let t = k as f64 / FPS;
        let out = cook.cook(&doc.graph, &reg, sink, t).expect("cozinha");
        let s = out[0].as_stream();
        frames.push(Frame {
            size: first_size(s),
            y: first_y(s),
        });
        cook.advance_tick(&doc.graph, &reg, t)
            .expect("fecha o tique");
    }
    frames
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

/// O Y do primeiro ponto — a posição é `P`, também `Vec2`, pela mesma armadilha.
fn first_y(s: &ph2d_nodegraph::attr::Stream) -> f32 {
    match s.get("P") {
        Some(Column::Vec2(v)) if !v.is_empty() => v[0][1],
        _ => f32::NAN,
    }
}

/// Os intervalos (em tiques) em que uma grandeza está acima do repouso — um por evento.
///
/// ⚠️ **O repouso é MEDIDO** (o mínimo da corrida), nunca escrito à mão: o Y de repouso vem
/// da geometria da grade e mudar `SIDE` ou `GAP` moveria um literal daqui sem ninguém notar.
fn events(vals: &[f32], margin: f32) -> Vec<(usize, usize)> {
    let rest = vals.iter().cloned().filter(|v| v.is_finite()).fold(f32::MAX, f32::min);
    let live = |v: f32| v.is_finite() && v > rest + margin;
    let mut out = Vec::new();
    let mut start = None;
    for (i, &v) in vals.iter().enumerate() {
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
        out.push((a, vals.len()));
    }
    out
}

fn swells(frames: &[Frame]) -> Vec<(usize, usize)> {
    let sizes: Vec<f32> = frames.iter().map(|f| f.size).collect();
    events(&sizes, 0.02)
}

fn hops(frames: &[Frame]) -> Vec<(usize, usize)> {
    let ys: Vec<f32> = frames.iter().map(|f| f.y).collect();
    events(&ys, 0.02)
}

/// O tamanho de repouso, lido da própria cena em vez de reescrito aqui.
const DOT_REST: f32 = 0.16;

/// **O gate que a cena existe para poder falhar: entre dois crescimentos cabem exatamente
/// QUATRO pulos.** Ele conta um relógio contra o OUTRO — a mesma pergunta que o olho faz na
/// tela —, e por isso ele *é* o oráculo do smoke em vez de um espelho dele.
///
/// ⚠️ A versão anterior desta cena não podia ter este gate, porque **não desenhava o
/// relógio de entrada**: sem os pulos não existe nada contra o que contar, e a única
/// afirmação possível era contra uma constante que eu mesmo escrevi.
#[test]
fn os_pulos_entre_dois_crescimentos_sao_quatro() {
    let frames = run(6.0);
    let grow = swells(&frames);
    let hop = hops(&frames);
    assert!(
        grow.len() >= 3,
        "6 s dão 3+ compassos para ter 2+ intervalos, medido {} ({grow:?})",
        grow.len()
    );
    for w in grow.windows(2) {
        let (a, b) = (w[0].0, w[1].0);
        let n = hop.iter().filter(|(s, _)| *s >= a && *s < b).count();
        assert_eq!(
            n, DIVIDE_BY as usize,
            "entre os crescimentos em {a} e {b} cabem {DIVIDE_BY} pulos, contados {n} \
             (pulos: {hop:?})"
        );
    }
}

/// O irmão do gate acima, e **não é redundante**: a razão entre dois relógios é cega ao par
/// inteiro derivar junto (um `period` errado move os dois e a razão continua 4). Este ancora
/// o espaçamento no TEMPO autorado.
#[test]
fn o_carry_divide_o_metronomo_por_quatro() {
    let frames = run(4.0);
    let bumps = swells(&frames);
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

/// **O pulo é a BATIDA, não o compasso** — a metade que torna o gate da razão honesto: se os
/// pulos fossem raros, "quatro entre dois crescimentos" continuaria verdade por acidente.
#[test]
fn o_pulo_marca_a_batida_crua() {
    let frames = run(4.0);
    let hop = hops(&frames);
    let beat_ticks = BEAT as f64 * FPS;
    assert!(
        hop.len() >= 9,
        "10 batidas em 4 s dão ~10 pulos, medido {} ({hop:?})",
        hop.len()
    );
    for w in hop.windows(2) {
        let gap = (w[1].0 - w[0].0) as f64;
        assert!(
            (gap - beat_ticks).abs() <= 2.0,
            "o intervalo entre pulos é a batida ({beat_ticks} tiques), medido {gap}"
        );
    }
    // E o pulo é CURTO — ele tem de caber quatro vezes dentro de um crescimento sem virar
    // um tremor contínuo.
    for (a, b) in &hop {
        assert!(
            b - a < (beat_ticks as usize) / 2,
            "um pulo dura menos de meia batida, medido {} tiques",
            b - a
        );
    }
}

/// **O crescimento tem DURAÇÃO, não é uma piscada** — o envelope. Sem ele o `carry` acenderia
/// o ponto por um tique, e a cena mostraria o número certo de eventos com nada visível.
#[test]
fn o_envelope_da_duracao_ao_disparo() {
    let frames = run(4.0);
    let bumps = swells(&frames);
    assert!(!bumps.is_empty(), "algum crescimento aconteceu");
    for (a, b) in &bumps {
        let ticks = b - a;
        assert!(
            ticks > 20,
            "um envelope dura dezenas de tiques, não {ticks} (a piscada de um `carry` cru)"
        );
    }
    // E a FORMA: dentro do crescimento o tamanho sobe até um pico e volta — não é um degrau.
    let (a, b) = bumps[0];
    let peak = frames[a..b]
        .iter()
        .map(|f| f.size)
        .fold(f32::MIN, f32::max);
    assert!(
        peak > DOT_REST + SWELL * 0.8,
        "o pico chega perto do topo: {peak}"
    );
    assert!(
        frames[b - 1].size < peak * 0.9,
        "e ele DESCE antes de acabar ({} contra o pico {peak})",
        frames[b - 1].size
    );
}

/// **O CONTROLE da cena:** sem o fio do `carry` nada cresce — **e os pulos CONTINUAM.**
///
/// ⚠️ A segunda metade é o controle positivo do controle: cortar o fio errado (ou matar o
/// documento inteiro) também deixaria o Size plano, e o gate ficaria verde sobre uma cena
/// morta. Exigir que o outro relógio sobreviva é o que prova que foi o `carry` que caiu.
#[test]
fn sem_o_fio_do_carry_o_ponto_nao_cresce_mas_ainda_pula() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_adsr_demo_document(&mut doc, &reg).expect("cena bem tipada");
    let sink = sinks[0];
    // ⚠️ Identificado pelo que ele SIGNIFICA (o envelope alimentado pelo contador), nunca
    // pela ordem em que a cena constrói os nós: a cena tem DOIS `pulse.adsr`, e um
    // `position()` pelo nome mudaria de alvo em silêncio se o construtor os reordenasse —
    // deixando este gate verde por cortar o ramo errado.
    let counters: Vec<_> = doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == "pulse.counter")
        .map(|n| n.id)
        .collect();
    let env = doc
        .graph
        .nodes()
        .iter()
        .find(|n| {
            n.type_name == "pulse.adsr"
                && doc
                    .graph
                    .input_edge(n.id, 0)
                    .is_some_and(|(src, _, _)| counters.contains(&src))
        })
        .map(|n| n.id)
        .expect("a cena tem um pulse.adsr alimentado pelo pulse.counter");
    doc.graph.disconnect(env, 0).expect("a cena fia o carry");

    let mut cook = Cook::new();
    let mut ys = Vec::new();
    for k in 0..=240u64 {
        let t = k as f64 / FPS;
        let out = cook.cook(&doc.graph, &reg, sink, t).expect("cozinha");
        let s = out[0].as_stream();
        let size = first_size(s);
        assert!(
            (size - DOT_REST).abs() < 0.02,
            "sem gatilho o ponto fica no repouso (tique {k}: {size})"
        );
        ys.push(first_y(s));
        cook.advance_tick(&doc.graph, &reg, t)
            .expect("fecha o tique");
    }
    let hop = events(&ys, 0.02);
    assert!(
        hop.len() >= 9,
        "o metrônomo sobrevive ao corte do carry: {} pulos em 4 s ({hop:?})",
        hop.len()
    );
}

/// Sonda: o retrato da cena em números, para a mensagem de smoke MEDIR em vez de estimar.
///
/// `cargo test -p ph2d-host-desktop --bins probe_o_compasso -- --ignored --nocapture`
#[test]
#[ignore = "sonda de diagnóstico: imprime, não afirma"]
fn probe_o_compasso() {
    let frames = run(6.0);
    let grow = swells(&frames);
    let hop = hops(&frames);
    println!(
        "[compasso] batida {BEAT}s / divisor {DIVIDE_BY} => compasso {}s",
        BEAT * DIVIDE_BY
    );
    println!("  pulos em 6 s: {} (o relogio cru)", hop.len());
    println!("  crescimentos em 6 s: {} (o dividido)", grow.len());
    for w in grow.windows(2) {
        let (a, b) = (w[0].0, w[1].0);
        let n = hop.iter().filter(|(s, _)| *s >= a && *s < b).count();
        println!("    entre os crescimentos em {a} e {b}: {n} pulos");
    }
    for (a, b) in &grow {
        let peak = frames[*a..*b].iter().map(|f| f.size).fold(f32::MIN, f32::max);
        println!(
            "    crescimento {a}..{b} ({} = {:.2}s) pico {peak:.3} contra repouso {DOT_REST}",
            b - a,
            (b - a) as f32 / 60.0
        );
    }
    if let Some((a, b)) = hop.first() {
        let peak = frames[*a..*b].iter().map(|f| f.y).fold(f32::MIN, f32::max);
        let rest = frames.iter().map(|f| f.y).fold(f32::MAX, f32::min);
        println!(
            "    pulo {a}..{b} ({} tiques) sobe {:.3} (repouso {rest:.3})",
            b - a,
            peak - rest
        );
    }
}
