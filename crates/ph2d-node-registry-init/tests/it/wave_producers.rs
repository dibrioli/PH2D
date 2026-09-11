//! **N PRODUTORES NO `motion.wave` — a composição já os exprime, e este gate é o
//! que impede a célula de renascer.**
//!
//! A folha 06 linha 35 marcava `P0/P1`: *"**NÃO** — dois `motion.wave` montam duas
//! grades (`rows×cols` próprios) e não somam no mesmo campo; a fonte é a célula do
//! centro, e só ela"*. ⚠️ A célula é de **2026-08-10** e o **Grupo P**
//! (`motion.drive(Custom…)`, 2026-08-16) mudou o catálogo **seis dias depois** — a
//! sétima célula desta conferência a envelhecer antes de alguém voltar a ela.
//!
//! O produtor novo é uma CADEIA, não um nó:
//!
//! ```text
//! wave.out --pre--> field.box --> value.attribute("falloff") -->
//!     motion.drive(Custom "wave_h", Add) --> wave.state
//! ```
//!
//! ⚠️ **O `pre` mora na aresta que ENTRA na cadeia**, nunca na que volta ao `state`:
//! é ela que quebra o ciclo, e os três nós intermediários são `Effect::Pure` ⇒ não
//! carimbam `sim_t`, então a onda ainda vê o `dt` do próprio relógio no tique
//! seguinte (o precedente exacto é o `rope_thickness`).
//!
//! ⇒ **`P0/P1` → `P2`**: não falta capacidade, falta o GESTO — hoje são quatro nós e
//! três arestas à mão, e o artista tem de saber que a coluna se chama `wave_h`, um
//! nome de ESTADO que nenhum picker oferece.
//!
//! ⛔ **MEDIDO E REJEITADO, não refaça: encadear `wave A --> wave B.state`.** É a
//! tentativa natural e é um **no-op SILENCIOSO** — B lê o `sim_t` que A acabou de
//! carimbar, `dt` dá zero, o ramo de *hold* devolve o campo intacto e o `drive` de B
//! nunca é aplicado. Medido com o drive de B **cinco vezes mais forte**: as duas
//! saídas saem **bit a bit idênticas** (`measure_wave_producers`, rota 2).
//!
//! Os números desta cadeia vivem na sonda irmã
//! (`measure_wave_producers.rs`, `-- --ignored --nocapture`).

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::value::CookValue;

const SIDE: usize = 21;
const SPACING: f32 = 0.5;
const TICKS: usize = 240;
const DT: f64 = 1.0 / 60.0;

/// O centro do produtor injectado, em MUNDO. A grade vai de `-5` a `+5` em x, então
/// isto é a coluna 4 da linha do meio — seis células à esquerda da fonte embutida.
const BOX_X: f32 = -3.0;

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

/// O campo, com a fonte embutida (uma `value.lfo` no centro) e — se `inject` — um
/// SEGUNDO produtor por composição.
fn field(inject: bool) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let w = g.add_node("motion.wave");
    for (k, v) in [
        ("rows", SIDE as f32),
        ("cols", SIDE as f32),
        ("spacing", SPACING),
        ("speed", 0.35),
        ("damping", 0.02),
    ] {
        g.set_param(w, k, v);
    }
    let lfo = g.add_node("value.lfo");
    g.set_param(lfo, "period", 0.5);
    g.set_param(lfo, "amplitude", 1.0);
    wire(&mut g, lfo, 0, w, 0, false);

    if !inject {
        wire(&mut g, w, 0, w, 1, true);
        return (g, w);
    }
    let bx = g.add_node("field.box");
    g.set_param(bx, "width", 0.8);
    g.set_param(bx, "height", 0.8);
    g.set_param(bx, "soft", 0.3);
    g.set_param(bx, "center_x", BOX_X);
    g.set_param(bx, "center_y", 0.0);
    wire(&mut g, w, 0, bx, 0, true);

    let rd = g.add_node("value.attribute");
    g.set_param(rd, "mode", 0.0); // a coluna escalar, crua
    g.set_text_param(rd, "attr", "falloff");
    wire(&mut g, bx, 0, rd, 0, false);

    let dr = g.add_node("motion.drive");
    g.set_param(dr, "channel", 9.0); // Custom
    g.set_param(dr, "mode", 0.0); // Add
    g.set_param(dr, "scale", 0.6);
    g.set_text_param(dr, "column", "wave_h");
    wire(&mut g, bx, 0, dr, 0, false);
    wire(&mut g, rd, 0, dr, 1, false);

    wire(&mut g, dr, 0, w, 1, false);
    (g, w)
}

/// A pose no fim de `TICKS`. ⚠️ O `advance_tick` é o que faz a aresta `pre` CARREGAR
/// estado — sem ele a cadeia nunca integra e todo gate de feedback fica verde por
/// vácuo (a lição que o `rope_thickness` pagou duas vezes).
fn settle(g: &Graph, reg: &NodeRegistry, node: NodeId) -> (Vec<f32>, Vec<[f32; 2]>) {
    let mut cook = Cook::new();
    let mut last = Stream::new(0);
    for k in 0..TICKS {
        let playhead = k as f64 * DT;
        cook.advance_tick(g, reg, playhead).expect("o tique avanca");
        let out = cook.cook(g, reg, node, playhead).expect("o campo coze");
        let CookValue::Instances(s) = &out[0] else {
            panic!("a saida e' um stream")
        };
        last = s.clone();
    }
    let h = match last.get("wave_h") {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    };
    let p = match last.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    };
    (h, p)
}

fn peak(h: &[f32]) -> usize {
    let mut best = 0usize;
    for (i, z) in h.iter().enumerate() {
        if z.abs() > h[best].abs() {
            best = i;
        }
    }
    best
}

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1]];
    (d[0] * d[0] + d[1] * d[1]).sqrt()
}

/// **O TESTE DECISIVO.** Um segundo produtor, numa posição que o nó não tem knob
/// para exprimir, sai da composição — e o CONTROLE (o MESMO campo sem a cadeia) é a
/// metade que torna o número legível.
///
/// Os três oráculos são **categóricos**, não limiares calibrados: *onde* está o pico
/// (um índice), e *se* o canto oposto se moveu.
#[test]
fn a_second_producer_comes_from_composition() {
    let reg = registry();

    let (g_bare, w_bare) = field(false);
    let (bare, pos) = settle(&g_bare, &reg, w_bare);
    let (g_two, w_two) = field(true);
    let (two, _) = settle(&g_two, &reg, w_two);

    assert_eq!(bare.len(), SIDE * SIDE, "o campo nu tem a grade toda");
    assert_eq!(
        two.len(),
        SIDE * SIDE,
        "a cadeia NAO muda a contagem -- se mudar, o feedback quebrou a grade"
    );

    let centre = [0.0f32, 0.0];
    let box_at = [BOX_X, 0.0];
    let bare_peak = pos[peak(&bare)];
    let two_peak = pos[peak(&two)];
    eprintln!(
        "onda: pico nu em ({:+.2}, {:+.2})   pico composto em ({:+.2}, {:+.2})   caixa em ({BOX_X:+.2}, +0.00)",
        bare_peak[0], bare_peak[1], two_peak[0], two_peak[1]
    );

    // (1) O CONTROLE: sem a cadeia a UNICA fonte e' a celula do centro, e o campo
    // tem o pico la'. Sem isto, (2) passaria sobre um campo que ja' tinha um pico
    // deslocado por outro motivo.
    assert!(
        dist(bare_peak, centre) < 1.0,
        "o CONTROLE tem uma fonte so', no centro: pico a {:.2} dele",
        dist(bare_peak, centre)
    );

    // (2) A ENTREGA: com a cadeia o pico muda-se para a CAIXA -- existe um segundo
    // produtor, numa posicao que nenhum param do no' sabe dizer.
    assert!(
        dist(two_peak, box_at) < 1.0,
        "o produtor injetado domina onde foi posto: pico a {:.2} da caixa",
        dist(two_peak, box_at)
    );

    // (3) Ele PROPAGA -- um bump que nao se espalha e' tinta no campo de altura, nao
    // uma fonte. O oraculo e' o canto mais LONGE da caixa: se ele se moveu, a
    // perturbacao atravessou a grade inteira.
    let far = pos
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| {
            dist(**a, box_at)
                .partial_cmp(&dist(**b, box_at))
                .expect("distancias finitas")
        })
        .map(|(i, _)| i)
        .expect("a grade nao e' vazia");
    let delta = (two[far] - bare[far]).abs();
    eprintln!(
        "  canto mais distante ({:+.2}, {:+.2}) a {:.2} da caixa: |composto - nu| = {delta:.6}",
        pos[far][0],
        pos[far][1],
        dist(pos[far], box_at)
    );
    assert!(
        delta > 1e-3,
        "a ondulacao injetada atravessou a grade ate' o canto oposto: {delta:.6}"
    );

    // (4) Um laco que explode nao e' um campo com dois produtores.
    for z in two.iter().chain(bare.iter()) {
        assert!(z.is_finite(), "altura finita: {z}");
    }
}

/// **A ROTA QUE NÃO FUNCIONA, pinada para ninguém a tentar duas vezes.**
/// `wave A --> wave B.state` é um no-op silencioso: B devolve o campo de A **bit a
/// bit**, com o `drive` próprio de B — aqui cinco vezes mais forte — engolido pelo
/// ramo de *hold* (`dt == 0`, porque A acabou de carimbar `sim_t = playhead`).
///
/// ⚠️ Este gate afirma o DEFEITO de propósito: enquanto a rota existir e for legal
/// de ligar, uma nota em prosa não impede ninguém de a tentar. Se um dia ela passar
/// a fazer alguma coisa, é aqui que se descobre — e a mudança tem de ser deliberada
/// (dar um passo em B correria a física ao **dobro** da velocidade).
#[test]
fn chaining_two_waves_swallows_the_second_drive() {
    let reg = registry();

    let mut g = Graph::new();
    let a = g.add_node("motion.wave");
    let b = g.add_node("motion.wave");
    for n in [a, b] {
        for (k, v) in [
            ("rows", SIDE as f32),
            ("cols", SIDE as f32),
            ("spacing", SPACING),
            ("speed", 0.35),
            ("damping", 0.02),
        ] {
            g.set_param(n, k, v);
        }
    }
    let lfo_a = g.add_node("value.lfo");
    g.set_param(lfo_a, "period", 0.5);
    g.set_param(lfo_a, "amplitude", 1.0);
    wire(&mut g, lfo_a, 0, a, 0, false);
    wire(&mut g, a, 0, a, 1, true);

    let lfo_b = g.add_node("value.lfo");
    g.set_param(lfo_b, "period", 0.3);
    g.set_param(lfo_b, "amplitude", 5.0);
    wire(&mut g, lfo_b, 0, b, 0, false);
    wire(&mut g, a, 0, b, 1, false);

    let (ha, _) = settle(&g, &reg, a);
    let (hb, _) = settle(&g, &reg, b);
    assert_eq!(
        ha, hb,
        "o encadeamento e' um no-op: B devolve A bit a bit, com o proprio drive engolido"
    );
    // E o CONTROLE que impede o gate de ser vacuo: A de facto ondula, entao a
    // igualdade acima e' sobre um campo VIVO, nao sobre dois campos planos.
    assert!(
        ha.iter().any(|z| z.abs() > 0.05),
        "o campo de A esta' vivo -- senao a igualdade acima nao diria nada"
    );
}
