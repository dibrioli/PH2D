//! **A CADEIA QUE O `motion.velocity` EXISTE PARA SERVIR** (doc 89, folha 07).
//!
//! Os gates da própria crate provam a LEI (o número, o pareamento, o one-pole) e são **cegos ao
//! catálogo**: eles não sabem dizer se a coluna que o nó escreve é a que alguém lê. Esta crate é
//! a única que vê os 124 nós de uma vez, e é aqui que a wave se prova.
//!
//! ⚠️ **Cada gate traz o CONTROLE ao lado**, e ele é a metade que importa: *o campo tem valores*
//! é satisfeito por qualquer coisa que não seja zero, enquanto *o campo é ZERO sem este nó e
//! deixa de ser com ele* nomeia exactamente o buraco que a wave fecha — o canal **Speed** é
//! oferecido no picker do `value.attribute` e, num stream que nenhum simulador tocou, devolve
//! zeros no comprimento inteiro, indistinguível de um nome mal digitado.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

fn wire(g: &mut Graph, from: NodeId, to: NodeId) {
    g.connect(Edge {
        from: (from, 0),
        to: (to, 0),
        delayed: false,
    })
    .expect("edge");
}

/// Uma grade que se MEXE — sem movimento não há velocidade a medir, e uma fixture parada
/// deixaria os dois lados do controle em zero.
fn moving_grid(g: &mut Graph) -> NodeId {
    let seed = g.add_node("motion.grid");
    g.set_param(seed, "rows", 6.0);
    g.set_param(seed, "cols", 6.0);
    let osc = g.add_node("motion.oscillator");
    g.set_param(osc, "amplitude", 3.0);
    g.set_param(osc, "frequency", 0.7);
    wire(g, seed, osc);
    osc
}

/// Cozinha alguns ticks (o `pre` self-loop precisa de estado) e devolve a saída do alvo.
fn run(g: &Graph, reg: &NodeRegistry, target: NodeId, ticks: u32) -> Stream {
    let mut cook = Cook::new();
    let mut last = Stream::new(0);
    for t in 0..ticks {
        let t = f64::from(t);
        last = cook.cook(g, reg, target, t).expect("cooks")[0]
            .as_stream()
            .clone();
        cook.advance_tick(g, reg, t).expect("advance");
    }
    last
}

/// O `mode` do `value.attribute` e um `i32` (as reducoes crescem para BAIXO, as lanes para
/// cima) e o param que o grafo guarda e `f32` — a conversao mora aqui, uma vez, para os gates
/// nomearem o degrau em vez de o escreverem como literal.
#[allow(clippy::cast_precision_loss)]
fn mode_of(m: i32) -> f32 {
    m as f32
}

fn scalars(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// Monta `<fonte> → [motion.velocity] → value.attribute(mode)` e devolve o campo `v`.
///
/// `with_velocity = false` é o CONTROLE: a MESMA cadeia sem o nó novo.
fn read_channel(reg: &NodeRegistry, mode: f32, with_velocity: bool) -> Vec<f32> {
    let mut g = Graph::new();
    let src = moving_grid(&mut g);
    let head = if with_velocity {
        let vn = g.add_node("motion.velocity");
        wire(&mut g, src, vn);
        g.connect(Edge {
            from: (vn, 0),
            to: (vn, 1),
            delayed: true,
        })
        .expect("ring");
        vn
    } else {
        src
    };
    let attr = g.add_node("value.attribute");
    g.set_param(attr, "mode", mode);
    // ⚠️ O nome da coluna é TEXT PARAM (o canal do `motion.expression`), não um `f32` do
    // manifesto: é assim que o `value.attribute` lê qualquer coluna sem bumpar contrato.
    g.set_text_param(attr, ph2d_node_value_attribute::ATTR_KEY, "vel");
    wire(&mut g, head, attr);
    scalars(&run(&g, reg, attr, 8), "v")
}

/// **O CANAL `Speed` DEIXA DE DEVOLVER ZEROS.**
///
/// ⚠️ O controle é a metade que prova a wave: sem este nó, *"colore pela velocidade"* — a frase
/// mais ordinária de motion graphics — devolve um campo de zeros num stream cinemático, e o
/// artista não tem como distinguir isso de ter digitado o nome errado.
#[test]
fn the_speed_channel_was_zeros_and_this_node_fills_it() {
    let reg = registry();
    let control = read_channel(&reg, mode_of(ph2d_node_value_attribute::MODE_LENGTH), false);
    let with = read_channel(&reg, mode_of(ph2d_node_value_attribute::MODE_LENGTH), true);

    assert!(!control.is_empty() && control.len() == with.len());
    assert!(
        control.iter().all(|s| *s == 0.0),
        "CONTROLE: sem o `motion.velocity` o canal Speed tem de ser zeros — se ja tinha valores, \
         a fixture nao contem o fenomeno que a wave fecha"
    );
    let moving = with.iter().filter(|s| **s > 1e-3).count();
    assert!(
        moving > with.len() / 2,
        "com o no, a maioria dos elementos tem velocidade; {moving} de {}",
        with.len()
    );
}

/// **E O CANAL `Direction` TAMBEM** — o *align to velocity*, a linha que cinco famílias citaram.
///
/// ⚠️ O oráculo não é *"tem números"*: um campo de zeros passaria por isso, e zero é um ângulo
/// válido. A pergunta é se as direções **DIFEREM entre si** — um oscilador move cada elemento
/// para lados diferentes, e o `atan2` tem de as separar.
#[test]
fn the_direction_channel_reads_where_each_element_is_going() {
    let reg = registry();
    let control = read_channel(&reg, mode_of(ph2d_node_value_attribute::MODE_ANGLE), false);
    let with = read_channel(&reg, mode_of(ph2d_node_value_attribute::MODE_ANGLE), true);

    assert!(
        control.iter().all(|a| *a == 0.0),
        "CONTROLE: sem `vel` nao ha direcao — o `atan2(0,0)` do modo Direction devolve 0"
    );
    let (lo, hi) = with
        .iter()
        .fold((f32::MAX, f32::MIN), |(lo, hi), a| (lo.min(*a), hi.max(*a)));
    assert!(
        hi - lo > 30.0,
        "as direcoes tinham de diferir entre si; a faixa medida e [{lo}, {hi}]"
    );
}

/// **O NO E O PONTO FIXO DE UM CONJUNTO PARADO.**
///
/// ⚠️ O gate que separa *medir* de *inventar*: uma grade imóvel tem velocidade zero em TODO
/// elemento e em todo tick — uma diferença finita sobre um conjunto parado não pode produzir
/// número nenhum, e um pareamento errado (posicional sobre ids embaralhados, um `dt` mal lido)
/// apareceria aqui como movimento fantasma.
#[test]
fn a_still_set_has_no_velocity_at_any_tick() {
    let reg = registry();
    let mut g = Graph::new();
    let seed = g.add_node("motion.grid");
    g.set_param(seed, "rows", 5.0);
    g.set_param(seed, "cols", 5.0);
    let vn = g.add_node("motion.velocity");
    wire(&mut g, seed, vn);
    g.connect(Edge {
        from: (vn, 0),
        to: (vn, 1),
        delayed: true,
    })
    .expect("ring");

    let out = run(&g, &reg, vn, 12);
    let vel = match out.get("vel") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("a coluna `vel` tem de existir"),
    };
    assert!(
        vel.iter().all(|v| v[0] == 0.0 && v[1] == 0.0),
        "uma grade parada nao tem velocidade: {vel:?}"
    );
}
