//! **O TETO DO `motion.verlet_rope` É ONDE O KERNEL DEIXA DE HONRÁ-LO** (doc 88 B2 · folha 03).
//!
//! Números e tabelas vivem no doc-comment do `PARAM_HARD_MAX` da crate; a sonda é a
//! `measure_rope_ceiling`. Estes gates afirmam a PROPRIEDADE, dirigindo a corda pela porta do
//! produto no teto e acima dele.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::node::NodeTypeId;
use ph2d_nodegraph::value::CookValue;

const ROPE: NodeTypeId = NodeTypeId::of("motion.verlet_rope");
/// O pior passo que o `eval` admite (o clamp `MAX_DT` da crate).
const WORST_DT: f64 = 0.1;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

/// ⚠️ **A porta de estado é a 2** — as 0 e 1 são `anchor_x`/`anchor_y`. Ligar o `pre` na 1 é
/// aceito pelo `connect` **sem reclamar**, e a corda então nunca integra: a 1ª fixture da sonda
/// mediu a pose de repouso em toda a varredura por isso, com tudo byte-idêntico e nada a acusar.
fn rope(length: f32, gravity: f32, iterations: f32) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let n = g.add_node("motion.verlet_rope");
    g.set_param(n, "count", 24.0);
    g.set_param(n, "length", length);
    g.set_param(n, "gravity", gravity);
    g.set_param(n, "iterations", iterations);
    g.connect(Edge {
        from: (n, 0),
        to: (n, 2),
        delayed: true,
    })
    .expect("o self-loop de estado");
    (g, n)
}

/// A queda da ponta ao fim de 60 tiques. ⚠️ Em **f64**: o quadrado de uma coordenada `f32`
/// estoura em ~1e19, e uma régua que estoura acusa a corda de um defeito que é dela própria.
fn tail_drop(g: &Graph, reg: &NodeRegistry, node: NodeId) -> f32 {
    let mut cook = Cook::new();
    let mut last: Vec<[f32; 2]> = Vec::new();
    for t in 0..60u64 {
        let playhead = t as f64 * WORST_DT;
        cook.advance_tick(g, reg, playhead).expect("tick");
        let out = cook.cook(g, reg, node, playhead).expect("cook");
        let CookValue::Instances(s) = &out[0] else {
            panic!("a saida da corda e um stream")
        };
        last = match Stream::get(s, "P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => Vec::new(),
        };
    }
    let tail = *last.last().expect("a corda tem pontos");
    f64::from(tail[0]).hypot(f64::from(tail[1])) as f32
}

fn ceiling(param: &str) -> f32 {
    registry()
        .param_hard_max(ROPE, param)
        .unwrap_or_else(|| panic!("a corda declara um teto digitavel para `{param}`"))
}

/// **A LEI: um teto digitável não pode passar do que o kernel HONRA.**
///
/// O `eval` clampa as passadas de relaxação, então acima do clamp o número do artista é jogado
/// fora — e o gate mede a consequência (o resultado é **byte a byte** o do teto) em vez de a
/// afirmar contra uma constante.
#[test]
fn above_the_iterations_ceiling_the_rope_returns_the_ceilings_own_answer() {
    let reg = registry();
    let cap = ceiling("iterations");
    let (gc, nc) = rope(6.0, 9.0, cap);
    let at_cap = tail_drop(&gc, &reg, nc);
    for over in [cap + 1.0, cap * 4.0, 100_000.0] {
        let (g, n) = rope(6.0, 9.0, over);
        assert_eq!(
            tail_drop(&g, &reg, n).to_bits(),
            at_cap.to_bits(),
            "com iterations = {over} a corda devolve a resposta do teto ({cap}) byte a byte"
        );
    }
    // ⚠️ **E a metade que PINA o teto ao clamp é esta, na FRONTEIRA:** em `cap − 1` a resposta
    // tem de DIFERIR. Sem ela o gate é verde por construção — uma mutação que suba o teto para
    // 512 deixa `512`, `513` e `100.000` todos a cair no mesmo clamp de 128, e comparar duas
    // respostas do clamp uma com a outra dá identidade trivialmente. (Medido: a mutação
    // sobreviveu ao gate inteiro até esta linha existir.)
    let (g, n) = rope(6.0, 9.0, cap - 1.0);
    let just_below = tail_drop(&g, &reg, n);
    assert_ne!(
        just_below.to_bits(),
        at_cap.to_bits(),
        "em {} a corda tem de responder DIFERENTE de {cap} -- se nao responde, o teto declarado \
         ja esta acima do que o kernel honra, e o gate de cima nao pode ve-lo",
        cap - 1.0
    );
}

/// **`gravity` e `length` param na MESMA parede, e o modo de falha é o SILENCIOSO.**
///
/// ⚠️ O que estoura não é o parâmetro, é a **posição** da corda em `f32` — os dois são estradas
/// para a mesma coordenada grande. E em `1e21` a queda é **exactamente zero**: a corda não
/// explode à vista, ela **desaparece**. Um gate que só olhasse magnitude chamaria isso de calmo.
#[test]
fn at_the_ceiling_the_rope_is_still_there_and_just_past_it_it_vanishes() {
    let reg = registry();
    for param in ["gravity", "length"] {
        let cap = ceiling(param);
        let at = |v: f32| {
            let (g, n) = if param == "gravity" {
                rope(6.0, v, 24.0)
            } else {
                rope(v, 9.0, 24.0)
            };
            tail_drop(&g, &reg, n)
        };
        let alive = at(cap);
        assert!(
            alive.is_finite() && alive > 0.0,
            "no teto de `{param}` ({cap}) a corda ainda tem de estar la (queda {alive})"
        );
        let past = at(cap * 10.0);
        assert!(
            !(past.is_finite() && past > 0.0),
            "dez vezes o teto de `{param}` tem de a fazer DESAPARECER (queda {past}) -- e o teto \
             existe por causa desse silencio, nao por causa do custo"
        );
    }
}

/// As duas paredes são **a mesma**, e o gate diz isso — para ninguém "afinar" uma sem a outra.
#[test]
fn the_gravity_and_length_ceilings_are_the_same_wall() {
    assert!(
        (ceiling("gravity") - ceiling("length")).abs() < f32::EPSILON * ceiling("gravity"),
        "os dois tetos sao a mesma parede de representacao (a posicao em f32), alcancada por \
         dois caminhos: gravity {} contra length {}",
        ceiling("gravity"),
        ceiling("length")
    );
}
