//! **A CORDA TEM ESPESSURA — e a resposta era a metade otimista da aposta.**
//!
//! O teste que a folha 03 chamou de *"o decisivo mais barato desta conferência"* e
//! declarou **não ter rodado**. A célula do `motion.verlet_rope` dizia:
//! *"**PROVAVELMENTE SIM, e ninguém sabe.** `rope.out --pre--> motion.collide.in` +
//! `motion.collide.out -> rope.state` é legal (feedback host genérico), e o bloqueio
//! conhecido não se aplica: `motion.collide` é `Effect::Pure` ⇒ não carimba `sim_t`
//! e repassa `rope_prev`. O Verlet lê o deslocamento como velocidade — que é como
//! toda colisão PBD entra num Verlet."*
//!
//! ⚠️ **Uma célula que diz *"provavelmente sim"* é uma aposta, e o veredito dela
//! dependia de qual metade era verdade** (`omissão` se falhasse, `ergonomia` se
//! funcionasse). Medido: **funciona** — a corda nua deixa nós não-vizinhos a
//! `0,6811` com o colisor a pedir `1,0` de diâmetro, e com o colisor no laço eles
//! ficam a `1,0516`. ⇒ **`P1` -> `P2`**: não falta capacidade, falta o gesto (hoje
//! são dois nós e duas arestas à mão).
//!
//! ⚠️ **Vizinhos são EXCLUÍDOS do oráculo por construção:** o Verlet os segura a
//! `length / (count − 1)` de propósito, e essa distância é menor que o diâmetro do
//! colisor — medir todos os pares acusaria a corda de se atravessar exactamente
//! onde ela está a fazer o trabalho dela.
//!
//! ⚠️ **E DUAS versões desta fixture mediram coisa nenhuma antes desta.** A 1ª
//! pendurava a corda: o controle reprovou com `min_far_pair = 1,0435`, que é
//! EXATAMENTE `4 × length/(count−1)` — a pose RETA (o nó tem uma âncora só, a
//! cabeça, e sob gravidade ela nunca se dobra). A 2ª chicoteava a âncora e **não
//! chamava `advance_tick`**: a corda ficou na pose de repouso a cozer o mesmo tique
//! para sempre (`tail = head + length`, `y = 0,0` sob gravidade 9,8) e as duas
//! cordas saíram **bit a bit idênticas** — uma cadeia que nunca integra faz todo
//! gate de feedback ser verde por vácuo.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::value::CookValue;

/// Nós mais próximos que isto na CADEIA são vizinhos estruturais, não uma dobra.
const NEIGHBOUR_SKIP: usize = 3;
const COUNT: usize = 24;
const RADIUS: f32 = 0.50;
/// Dois nós de raio `r` estão SOBREPOSTOS abaixo de `2r` — a régua do colisor é o
/// diâmetro, e comparar com o raio pediria o dobro da dobra.
const DIAMETER: f32 = 2.0 * RADIUS;
const TICKS: usize = 240;
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

/// Uma corda CHICOTEADA, e a fixture teve de ser esta (ver o cabeçalho): a cabeça
/// varre de lado num período curto, e a cauda — que chega atrasada — dobra por
/// cima do corpo.
///
/// Com `collide = true` o laço de estado passa pelo colisor:
/// `rope.out --pre--> collide.in` · `collide.out --> rope.state`.
fn whipped(collide: bool) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let rope = g.add_node("motion.verlet_rope");
    g.set_param(rope, "count", COUNT as f32);
    g.set_param(rope, "length", 6.0);
    g.set_param(rope, "gravity", 9.8);
    g.set_param(rope, "iterations", 16.0);

    let lfo = g.add_node("value.lfo");
    g.set_param(lfo, "period", 0.6);
    g.set_param(lfo, "amplitude", 3.0);
    wire(&mut g, lfo, 0, rope, 0, false);

    if collide {
        let col = g.add_node("motion.collide");
        g.set_param(col, "radius", RADIUS);
        g.set_param(col, "iterations", 8.0);
        // ⚠️ O `pre` mora na aresta que ENTRA no colisor, não na que sai dele: é ela
        // que quebra o ciclo, e o colisor é `Pure` ⇒ não carimba `sim_t`, então a
        // corda ainda vê o `dt` do próprio relógio no tique seguinte.
        wire(&mut g, rope, 0, col, 0, true);
        wire(&mut g, col, 0, rope, 2, false);
    } else {
        // ⚠️ A porta de estado é a **2** — as 0 e 1 são `anchor_x`/`anchor_y`, e
        // ligar o `pre` na 1 é aceito pelo `connect` sem reclamar.
        wire(&mut g, rope, 0, rope, 2, true);
    }
    (g, rope)
}

/// A pose no fim de `TICKS`. O `advance_tick` é o que faz a aresta `pre` CARREGAR
/// estado (ver o cabeçalho).
fn settle(g: &Graph, reg: &NodeRegistry, node: NodeId) -> Vec<[f32; 2]> {
    let mut cook = Cook::new();
    let mut last = Stream::new(0);
    for k in 0..TICKS {
        let playhead = k as f64 * DT;
        cook.advance_tick(g, reg, playhead)
            .expect("o tique avanca o `pre`");
        let out = cook.cook(g, reg, node, playhead).expect("a corda coze");
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

/// A menor distância entre dois nós que **não** são vizinhos na corda.
fn min_far_pair(pos: &[[f32; 2]]) -> f32 {
    let mut m = f32::MAX;
    for (i, a) in pos.iter().enumerate() {
        for b in pos.iter().skip(i + NEIGHBOUR_SKIP + 1) {
            let d = [a[0] - b[0], a[1] - b[1]];
            m = m.min((d[0] * d[0] + d[1] * d[1]).sqrt());
        }
    }
    m
}

/// **O TESTE DECISIVO.** Com o colisor no laço de estado a corda deixa de se
/// atravessar — e o CONTROLE (a MESMA corda sem ele) é a metade que torna o número
/// legível: sem ele, um limiar qualquer passaria numa corda que nunca dobrou.
#[test]
fn a_collider_in_the_state_loop_gives_the_rope_thickness() {
    let reg = registry();

    let (g_bare, rope_bare) = whipped(false);
    let bare = settle(&g_bare, &reg, rope_bare);
    let (g_thick, rope_thick) = whipped(true);
    let thick = settle(&g_thick, &reg, rope_thick);

    assert_eq!(bare.len(), COUNT, "a corda nua tem os nos todos");
    assert_eq!(
        thick.len(),
        COUNT,
        "a corda com colisor NAO perde nos -- se perder, o feedback quebrou a contagem"
    );

    let bare_min = min_far_pair(&bare);
    let thick_min = min_far_pair(&thick);
    eprintln!(
        "corda: min par distante  nua {bare_min:.4}  com colisor {thick_min:.4}  (diametro {DIAMETER:.4})"
    );

    // (1) O CONTROLE: a corda nua TEM de estar sobreposta, senao o gate e' verde
    // por vacuo -- foi assim que as duas primeiras fixtures reprovaram.
    assert!(
        bare_min < DIAMETER,
        "o CONTROLE contem o fenomeno: a corda nua se atravessa ({bare_min:.4} < {DIAMETER:.4})"
    );

    // (2) A ENTREGA: o colisor afasta os nao-vizinhos ate ~o proprio diametro.
    assert!(
        thick_min >= DIAMETER * 0.9,
        "o colisor da ESPESSURA a corda: {thick_min:.4} contra o diametro {DIAMETER:.4}"
    );

    // (3) O feedback nao pode DESTRUIR a corda: um laco que explode nao e' uma
    // corda grossa. (Um `NaN` faria (2) passar por acidente de comparacao.)
    for q in thick.iter().chain(bare.iter()) {
        assert!(q[0].is_finite() && q[1].is_finite(), "pose finita: {q:?}");
    }
}
