//! **O gate do `motion.delay`** (doc 63 §5) — a afirmação do PRODUTO, medida.
//!
//! Irmão de `motion_state_tests` (cap de 600 LOC do shell, HR-18). Declarado pelo `motion_state`
//! como `#[path]`, então `super` é `motion_state`.
//!
//! Ele existe porque a versão anterior desta afirmação era **FALSA** — e o gate que a "provava"
//! estava **verde**. Ver [[feedback_a_correct_number_can_carry_a_false_story]].

use super::strobe::{SEA_LEVEL, SEA_WAVE_AMP};
use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// **O `motion.delay` tira o TREMOR e deixa o MOVIMENTO** (doc 63 §5) — a afirmação do produto,
/// medida na cena do smoke (`PH2D_MOTION_DELAY_SMOKE=1`).
///
/// Este gate existe porque a versão anterior dela era **FALSA**: eu tinha posto o nó no documento de
/// boot dizendo que ele *"tirava o tremor da neve"* — e a neve **não treme** (o `gust` modula a
/// magnitude de uma força que aponta reto pra baixo: o floco cai em linha reta, mais rápido ou mais
/// devagar; desvio de aceleração medido: **0,1% de um floco**, deriva lateral **zero**). O número
/// que eu tinha medido era certo; a **história** que contei em cima dele era errada.
///
/// Um suavizador precisa de algo trêmulo. Então o gate mede o que o nó de fato faz, no que de fato
/// treme — e **falha nos dois sentidos**: se ele deixar de suavizar, e se ele suavizar até matar o
/// gesto (uma janela grande demais transforma o wiggle numa linha reta, o que também não é a
/// feature).
#[test]
fn the_ease_kills_the_twitch_and_keeps_the_motion() {
    use ph2d_nodegraph::graph::{Edge, Graph};
    let st = MotionState::new();
    let reg = &st.registry;

    // `grid(1) → wiggle(f=8, Y) [→ delay(Blend, 6)]` — a MESMA cadeia, com e sem o nó.
    let walk = |ease: f32| -> (f32, f32) {
        let mut g = Graph::new();
        let grid = g.add_node("motion.grid");
        let wig = g.add_node("motion.wiggle");
        let dly = g.add_node("motion.delay");
        g.connect(Edge {
            from: (grid, 0),
            to: (wig, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (wig, 0),
            to: (dly, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (dly, 0),
            to: (dly, 1),
            delayed: true,
        })
        .unwrap();
        g.set_param(grid, "rows", 1.0);
        g.set_param(grid, "cols", 1.0);
        g.set_param(wig, "channel", 1.0);
        g.set_param(wig, "amplitude", 0.5);
        g.set_param(wig, "frequency", 8.0);
        g.set_param(dly, "mode", 2.0);
        g.set_param(dly, "ticks", ease); // 0 = o nó é transparente (o ponto neutro)
        let mut cook = Cook::new();
        let mut ys = Vec::new();
        for k in 0..180u64 {
            let t = k as f64 / 60.0;
            let c = cook.cook(&g, reg, dly, t).unwrap();
            if let Some(Column::Vec2(p)) = c[0].as_stream().get("P") {
                ys.push(p[0][1]);
            }
            cook.advance_tick(&g, reg, t).unwrap();
        }
        // tremor = o pico da 2ª diferença (o que o olho lê como sacudida, num movimento que não
        // tem aceleração constante pra mascará-la — ao contrário da neve, e essa é a diferença).
        let twitch = ys
            .windows(3)
            .map(|w| (w[2] - 2.0 * w[1] + w[0]).abs())
            .fold(0.0f32, f32::max);
        let (lo, hi) = ys
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), y| (a.min(*y), b.max(*y)));
        (twitch, hi - lo)
    };

    let (raw_twitch, raw_span) = walk(0.0);
    let (eased_twitch, eased_span) = walk(6.0);

    // O wiggle a f=8 treme DE VERDADE: mais de 40% da largura do objeto (0.18), por tick. Sem isto,
    // o resto do gate estaria medindo a suavização de nada — que é exatamente o erro que ele corrige.
    assert!(
        raw_twitch > 0.07,
        "o fixture tem que TREMER, senão o gate não prova nada: {raw_twitch}"
    );
    assert!(
        eased_twitch < raw_twitch * 0.5,
        "a ease tem que cortar o tremor pela metade: {eased_twitch} vs {raw_twitch}"
    );
    assert!(
        eased_span > raw_span * 0.85,
        "…e o MOVIMENTO tem que sobreviver: excursão {eased_span} vs {raw_span}. Um suavizador que \
         achata o gesto não é um suavizador, é um mute"
    );
}

/// **A neve VAGUEIA — e ela não sacode** (doc 63 §6). As duas metades, medidas sobre a **população
/// inteira**, não sobre um floco.
///
/// Este gate existe por causa de dois erros meus, no mesmo dia:
///
/// 1. **Eu medi UM floco (`id = 0`) e concluí sobre a nevasca.** O `gust` é por-floco (cada um lê a
///    fileira de ruído dele), então um floco não é uma amostra — é uma anedota.
/// 2. **Eu misturei as duas fases do voo.** Filtrando por "acima da linha d'água" em vez de CORTAR no primeiro
///    contato, o floco que mergulha e volta a boiar reentrava na trilha — e eu media a queda colada
///    com a bóia. *"A neve que treme não é a mesma que cai"* (Enio): é a mesma neve, em outra fase,
///    e o que treme é o **splash**.
///
/// O que o gate afirma agora, sobre TODOS os flocos e SÓ na fase de queda:
/// - **eles vagueiam** (o `force.curl` existe e faz efeito — sem ele a deriva lateral é EXATAMENTE
///   zero, e uma neve que cai reta é chuva);
/// - **eles não sacodem** (curl noise é um campo SUAVE: o tremor por tick fica em ~1% da largura de
///   um floco). Snow drifts; it does not judder.
#[test]
fn the_snow_wanders_and_it_does_not_judder() {
    use std::collections::BTreeMap;
    let state = MotionState::new();
    let zone = state
        .doc
        .graph
        .nodes()
        .iter()
        .find(|n| n.type_name == "sim.zone")
        .expect("the snow is a zone")
        .id;
    let mut cook = Cook::new();
    // A trajetória de CADA floco, **cortada no primeiro contato com a água** — a queda, e só ela.
    let mut fall: BTreeMap<u64, Vec<[f32; 2]>> = BTreeMap::new();
    let mut wet: BTreeMap<u64, bool> = BTreeMap::new();
    for k in 0..=260u64 {
        let t = k as f64 / 60.0;
        let c = cook
            .cook(&state.doc.graph, &state.registry, zone, t)
            .unwrap();
        let s = c[0].as_stream();
        if let (Some(Column::Vec2(p)), Some(Column::Scalar(ids))) = (s.get("P"), s.get("id")) {
            for (i, d) in ids.iter().enumerate() {
                let id = d.to_bits() as u64;
                let w = wet.entry(id).or_insert(false);
                if *w {
                    continue; // já tocou a água: a QUEDA acabou. Não volte pra trilha.
                }
                if p[i][1] <= SEA_LEVEL + SEA_WAVE_AMP {
                    *w = true;
                    continue;
                }
                fall.entry(id).or_default().push(p[i]);
            }
        }
        cook.advance_tick(&state.doc.graph, &state.registry, t)
            .unwrap();
    }

    const FLAKE: f32 = 0.18; // a largura do quad — a unidade em que um número quer dizer algo
    let (mut worst_drift, mut worst_twitch, mut n) = (0.0f32, 0.0f32, 0);
    for tr in fall.values() {
        if tr.len() < 30 {
            continue; // nasceu tarde demais pra medir uma queda
        }
        n += 1;
        let (lo, hi) = tr
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), p| (a.min(p[0]), b.max(p[0])));
        worst_drift = worst_drift.max(hi - lo);
        // tremor = o desvio da aceleração em torno da média (a gravidade É a média)
        let steps: Vec<f32> = tr.windows(2).map(|w| w[0][1] - w[1][1]).collect();
        let d2: Vec<f32> = steps.windows(2).map(|w| w[1] - w[0]).collect();
        let mean = d2.iter().sum::<f32>() / d2.len().max(1) as f32;
        worst_twitch = worst_twitch.max(d2.iter().map(|x| (x - mean).abs()).fold(0.0f32, f32::max));
    }

    assert!(n > 40, "a amostra é a nevasca, não um floco: {n} flocos");
    assert!(
        worst_drift > 3.0 * FLAKE,
        "a neve tem que VAGUEAR (o `force.curl` está lá pra isso). Sem ele a deriva é exatamente \
         zero, e uma neve que cai reta é CHUVA. Medido: {worst_drift} ({:.0}% de um floco)",
        worst_drift / FLAKE * 100.0
    );
    assert!(
        worst_twitch < 0.15 * FLAKE,
        "…e ela NÃO pode sacudir: curl noise é um campo SUAVE, e neve vagueia sem tremer. Medido: \
         {worst_twitch} ({:.0}% de um floco por tick)",
        worst_twitch / FLAKE * 100.0
    );
}
