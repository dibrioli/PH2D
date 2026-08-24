//! **O CORPO MOLE TEM RAIO DE PARTÍCULA — a célula dizia *"não rodei"*, e agora rodou.**
//!
//! O gêmeo exacto do [`rope_thickness`]. A folha 03 do doc 89 fecha a célula do
//! `motion.soft_body` com uma frase e nenhum número:
//!
//! > *"**A MESMA cadeia do rope** (`soft_body.out --pre--> motion.collide → soft_body.state`) —
//! > `motion.collide` é Pure e repassa `sb_vel`/`sim_t`. **Não rodei**"*
//!
//! ⚠️ **Uma célula que diz *"não rodei"* não é uma omissão nem uma recusa: é uma pergunta
//! sem resposta**, e o veredito dela (`omissão/ergonomia — o teste decide`) estava escrito à
//! espera de uma medição. Medido aqui: **funciona**. A cadeia corre, o corpo continua a
//! simular, e as partículas param de se aproximar mais que o diâmetro que o colisor pede.
//!
//! ⇒ **não falta capacidade, falta o GESTO** — hoje são dois nós e duas arestas ligadas à mão,
//! exactamente como na corda. É a mesma conclusão e pelo mesmo mecanismo.
//!
//! ⚠️ **A régua NÃO é a distância entre vizinhos de malha.** Um corpo mole segura os vizinhos
//! a `spacing` de propósito, e `spacing` é menor que o diâmetro do colisor — medir todos os
//! pares acusaria o corpo de se atravessar exactamente onde as molas dele estão a trabalhar.
//! A régua é a **extensão**: com o colisor no laço, as partículas não podem ficar mais perto
//! que o diâmetro, então o corpo **INCHA** — que é o que um *particle radius* faz.
//!
//! ⚠️ **E o `advance_tick` é o que faz a aresta `pre` CARREGAR estado.** Sem ele a cadeia coze
//! o mesmo tique para sempre e as duas versões saem bit a bit idênticas — a armadilha que
//! reprovou duas fixtures do gate da corda antes de ele medir alguma coisa.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::value::CookValue;

const ROWS: f32 = 5.0;
const COLS: f32 = 5.0;
/// O passo da malha em repouso — e o número contra o qual o raio do colisor é grande.
const SPACING: f32 = 0.30;
/// ⚠️ **O raio é MAIOR que meio passo de propósito:** um colisor que já coubesse entre as
/// partículas não teria nada que empurrar, e o gate ficaria verde por vácuo.
const RADIUS: f32 = 0.28;
const TICKS: usize = 180;
const DT: f64 = 1.0 / 60.0;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16, delayed: bool) {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed,
    })
    .expect("wire");
}

/// Um corpo mole a balançar, com ou sem o colisor no laço de estado.
///
/// ⚠️ **A porta de estado é a `2`** — as `0`/`1` são `anchor_x`/`anchor_y` e a `3` é a `shape`.
/// Ligar o `pre` na porta errada é aceite pelo `connect` sem uma palavra.
fn body(collide: bool) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let sb = g.add_node("motion.soft_body");
    g.set_param(sb, "rows", ROWS);
    g.set_param(sb, "cols", COLS);
    g.set_param(sb, "spacing", SPACING);
    g.set_param(sb, "gravity", 9.8);
    g.set_param(sb, "stiffness", 0.6);
    g.set_param(sb, "damping", 0.02);

    // A âncora varre de lado: um corpo pendurado e PARADO nunca se dobra sobre si próprio, e
    // foi essa a 1ª fixture reprovada do gate da corda.
    let lfo = g.add_node("value.lfo");
    g.set_param(lfo, "period", 0.7);
    g.set_param(lfo, "amplitude", 2.0);
    wire(&mut g, lfo, 0, sb, 0, false);

    if collide {
        let col = g.add_node("motion.collide");
        g.set_param(col, "radius", RADIUS);
        g.set_param(col, "iterations", 8.0);
        // ⚠️ O `pre` mora na aresta que ENTRA no colisor: é ela que quebra o ciclo, e o
        // colisor é `Pure` ⇒ não carimba `sim_t`, então o corpo ainda vê o `dt` do próprio
        // relógio no tique seguinte.
        wire(&mut g, sb, 0, col, 0, true);
        wire(&mut g, col, 0, sb, 2, false);
    } else {
        wire(&mut g, sb, 0, sb, 2, true);
    }
    (g, sb)
}

/// A pose no fim de `TICKS`.
fn settle(g: &Graph, reg: &NodeRegistry, node: NodeId) -> Vec<[f32; 2]> {
    let mut cook = Cook::new();
    let mut last = Stream::new(0);
    for k in 0..TICKS {
        let playhead = k as f64 * DT;
        cook.advance_tick(g, reg, playhead)
            .expect("o tique avanca o `pre`");
        let out = cook.cook(g, reg, node, playhead).expect("o corpo coze");
        let CookValue::Instances(s) = &out[0] else {
            panic!("a saida e um stream")
        };
        last = s.clone();
    }
    match last.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// A menor distância entre DUAS partículas quaisquer.
fn closest_pair(p: &[[f32; 2]]) -> f32 {
    let mut d = f32::MAX;
    for i in 0..p.len() {
        for j in (i + 1)..p.len() {
            d = d.min((p[i][0] - p[j][0]).hypot(p[i][1] - p[j][1]));
        }
    }
    d
}

/// A maior distância entre duas partículas — o quanto o corpo ocupa.
fn extent(p: &[[f32; 2]]) -> f32 {
    let mut d = 0.0_f32;
    for a in p {
        for b in p {
            d = d.max((a[0] - b[0]).hypot(a[1] - b[1]));
        }
    }
    d
}

/// ⭐ **A CADEIA CORRE, e o corpo INCHA** — a resposta que a célula esperava.
#[test]
fn the_soft_body_takes_a_particle_radius_through_the_collide_loop() {
    let reg = registry();
    let (g0, sb0) = body(false);
    let (g1, sb1) = body(true);
    let bare = settle(&g0, &reg, sb0);
    let thick = settle(&g1, &reg, sb1);

    assert_eq!(
        bare.len(),
        (ROWS * COLS) as usize,
        "o corpo nu tem uma particula por celula"
    );
    assert_eq!(
        thick.len(),
        bare.len(),
        "o colisor no laco NAO pode mudar a contagem -- ele empurra, nao apaga"
    );

    // ⚠️ **O CONTROLE primeiro:** as duas poses TÊM de diferir. Sem isto o gate ficaria verde
    // sobre uma cadeia que nunca integrou (a 2ª fixture reprovada do gate da corda).
    let moved = bare
        .iter()
        .zip(&thick)
        .map(|(a, b)| (a[0] - b[0]).abs().max((a[1] - b[1]).abs()))
        .fold(0.0_f32, f32::max);
    assert!(
        moved > 1e-3,
        "CONTROLE: as duas poses sairam iguais ({moved:.6}) -- a cadeia nao esta' a correr"
    );

    // A afirmação: o colisor AFASTA as partículas.
    let (d0, d1) = (closest_pair(&bare), closest_pair(&thick));
    assert!(
        d1 > d0,
        "o raio de particula tinha de afastar o par mais proximo: {d0:.4} sem, {d1:.4} com"
    );
    // E o corpo ocupa mais espaço, que é o que um *particle radius* faz.
    let (e0, e1) = (extent(&bare), extent(&thick));
    assert!(
        e1 > e0,
        "o corpo tinha de INCHAR: {e0:.4} sem o colisor, {e1:.4} com"
    );
    println!(
        "soft_body + collide: par mais proximo {d0:.4} -> {d1:.4} · extensao {e0:.4} -> {e1:.4}"
    );
}

/// ⚠️ **E o corpo continua a SIMULAR com o colisor no laço** — a metade que o `moved` acima
/// não cobre: uma cadeia pode diferir da nua e ainda assim estar congelada.
#[test]
fn the_body_keeps_simulating_with_the_collider_in_the_loop() {
    let reg = registry();
    let (g, sb) = body(true);
    let mut cook = Cook::new();
    let mut poses: Vec<Vec<[f32; 2]>> = Vec::new();
    for k in 0..TICKS {
        let playhead = k as f64 * DT;
        cook.advance_tick(&g, &reg, playhead).expect("avanca");
        let out = cook.cook(&g, &reg, sb, playhead).expect("coze");
        let CookValue::Instances(s) = &out[0] else {
            panic!("stream")
        };
        if k == TICKS / 2 || k == TICKS - 1 {
            match s.get("P") {
                Some(Column::Vec2(v)) => poses.push(v.clone()),
                _ => panic!("P"),
            }
        }
    }
    assert_eq!(poses.len(), 2, "duas fotografias");
    let d = poses[0]
        .iter()
        .zip(&poses[1])
        .map(|(a, b)| (a[0] - b[0]).abs().max((a[1] - b[1]).abs()))
        .fold(0.0_f32, f32::max);
    assert!(
        d > 1e-3,
        "o corpo congelou com o colisor no laco (maior desvio entre metade e fim: {d:.6})"
    );
    // E nada explodiu.
    for (i, q) in poses[1].iter().enumerate() {
        assert!(
            q[0].is_finite() && q[1].is_finite() && q[0].abs() < 1e3 && q[1].abs() < 1e3,
            "particula {i} fugiu para {q:?}"
        );
    }
}
