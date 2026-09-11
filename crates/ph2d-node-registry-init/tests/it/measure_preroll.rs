//! **SONDA — o *Pre-roll* do AE é construível neste desenho?**
//!
//! A folha 06 linha 36 pede quatro coisas ao `motion.wave`, e o **Pre-roll** é a única
//! que nenhuma composição alcança: *um passo é um tique que ainda não aconteceu*. A
//! pergunta natural é então **construir o param** — e ela tem uma resposta que só a
//! medição dá.
//!
//! ⚠️ **A porta `drive` entrega UM NÚMERO POR TIQUE, não uma função do tempo.** Durante
//! um pre-roll o nó não pode re-avaliar a `value.lfo` a montante em instantes fictícios
//! (o cook lhe dá um valor, do tique corrente), então o valor fica **CONGELADO**. Um
//! pre-roll de `K` passos com a fonte congelada em `v` é, ao bit, **`K` tiques com uma
//! fonte constante `v`** — e é isso que esta sonda mede, sem precisar do param.
//!
//! O **CONTROLE** é o que torna a leitura legível: com a fonte **VIVA** o mesmo campo
//! constrói anéis. Sem ele, *"a fonte congelada faz um domo"* seria satisfeito também
//! por um motor quebrado.
//!
//! Ela **imprime e não afirma**. Rode com
//! `cargo test -p ph2d-node-registry-init --test measure_preroll -- --ignored --nocapture`.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::value::CookValue;

const SIDE: usize = 21;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

/// Um `motion.wave` com laço fechado e uma `value.lfo` de amplitude/offset escolhidos.
/// `amplitude = 0` é a **fonte congelada** (o que um pre-roll de facto veria).
fn wave(amp: f32, offset: f32) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let w = g.add_node("motion.wave");
    for (k, v) in [
        ("rows", SIDE as f32),
        ("cols", SIDE as f32),
        ("spacing", 0.5),
        ("speed", 0.35),
        ("damping", 0.02),
    ] {
        g.set_param(w, k, v);
    }
    let lfo = g.add_node("value.lfo");
    // ⚠️ Período CURTO de propósito: o comprimento de onda tem de caber no raio, senão
    // o CONTROLE não mostra anel nenhum e a leitura deixa de discriminar (medido na
    // sonda irmã `measure_wave_edges`: meia-onda 1,30 em 0,15 contra 4,4 em 0,50).
    g.set_param(lfo, "period", 0.15);
    g.set_param(lfo, "amplitude", amp);
    g.set_param(lfo, "offset", offset);
    for (from, to, port, delayed) in [(lfo, w, 0u16, false), (w, w, 1, true)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed,
        })
        .expect("wire");
    }
    (g, w)
}

/// Coze `ticks` tiques e devolve o campo de altura.
fn settle(g: &Graph, reg: &NodeRegistry, node: NodeId, ticks: usize) -> Vec<f32> {
    let mut cook = Cook::new();
    let mut out = Vec::new();
    for k in 0..ticks {
        let playhead = k as f64 / 60.0;
        cook.advance_tick(g, reg, playhead).expect("o tique avanca");
        let cooked = cook.cook(g, reg, node, playhead).expect("o campo coze");
        let CookValue::Instances(s) = &cooked[0] else {
            panic!("a saida e' um stream")
        };
        out = match s.get("wave_h") {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
    }
    out
}

/// O raio do centro para a direita, e quantas vezes ele cruza o zero.
/// **Cruzamentos = ANÉIS. Zero cruzamentos = um domo PARADO.**
fn read(h: &[f32]) -> (f32, usize, String) {
    let r = SIDE / 2;
    let ray: Vec<f32> = (r..SIDE).map(|c| h[r * SIDE + c]).collect();
    let peak = h.iter().fold(0.0f32, |m, x| m.max(x.abs()));
    let cross = ray
        .windows(2)
        .filter(|w| (w[0] < 0.0) != (w[1] < 0.0))
        .count();
    let txt = ray
        .iter()
        .take(7)
        .map(|x| format!("{x:+.3}"))
        .collect::<Vec<_>>()
        .join(" ");
    (peak, cross, txt)
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn a_frozen_source_pre_rolls_into_a_dome_and_a_live_one_into_rings() {
    let reg = registry();
    eprintln!("\n[pre-roll] o que uma fonte CONGELADA constroi (grade {SIDE}x{SIDE})");
    for (tag, amp, offset) in [
        // O caso COMUM: uma senoide vale ZERO no instante da semeadura.
        ("congelada em 0,0", 0.0f32, 0.0f32),
        ("congelada em 0,5", 0.0, 0.5),
        ("congelada em 1,0", 0.0, 1.0),
    ] {
        for ticks in [30usize, 120, 500] {
            let (g, w) = wave(amp, offset);
            let (peak, cross, ray) = read(&settle(&g, &reg, w, ticks));
            eprintln!(
                "  {tag:<18} {ticks:>4} passos: max |h| {peak:.6}  cruzamentos {cross}  raio {ray}"
            );
        }
    }

    eprintln!("\n  -- o CONTROLE: a MESMA grade com a fonte VIVA --");
    for ticks in [30usize, 120, 500] {
        let (g, w) = wave(1.0, 0.0);
        let (peak, cross, ray) = read(&settle(&g, &reg, w, ticks));
        eprintln!(
            "  {:<18} {ticks:>4} passos: max |h| {peak:.6}  cruzamentos {cross}  raio {ray}",
            "viva (senoide)"
        );
    }
    eprintln!(
        "  => se a congelada nunca cruza o zero e a viva cruza, o pre-roll nao constroi ANEIS."
    );
}
