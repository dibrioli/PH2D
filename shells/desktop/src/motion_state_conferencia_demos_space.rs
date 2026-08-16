//! **O PINO SEGUE A PRESCRIÇÃO, E O BANDO GANHA ESPAÇO PESSOAL** — a cena `=51`,
//! os dois reports do smoke da `=50`.
//!
//! ⚠️ **São dois defeitos DIFERENTES, e a cena os separa em dois pares.**
//!
//! **(1)** *"o que está pinado, se eu mover `spacing` em Soft Body, não muda de
//! posição até apertar reset"*. Medido, a afirmação geral era falsa (o `spacing`
//! movia o corpo **3,87** contra um controle de **0,0017**) e o exemplo era exacto:
//! **o pinado andava `0,0000`**. O pino genérico segurava *onde estava*, então nem
//! o `spacing` nem a ÂNCORA o alcançavam — uma bandeira num mastro que se move era
//! inexprimível. Hoje ele segue a mesma prescrição do pino intrínseco, e as duas
//! bandas de cima provam-no ao ficarem **iguais**.
//!
//! **(2)** *"não há parâmetro que estabeleça o grau de sobreposição entre os
//! indivíduos; ou a imagem desenhada é maior do que deveria"*. ⚠️ **A segunda
//! metade era a certa:** um agente desenha a **1,0** (o `SIZE_IDENTITY` do shell) e
//! o bando empacota-os a **0,795** de mediana — 70% sobrepostos. E o único knob era
//! um PESO: para zerar a sobreposição era preciso `separation = 51,2` contra um
//! slider que para em **6,0**. O `separation_radius` é a peça que o Reynolds tem e
//! este nó não tinha, e ele fecha o caso **dentro** dos sliders.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O vão vertical entre bandas.
const BAND_DY: f32 = 9.0;
/// A malha do corpo mole (6×6 = 36 partículas).
const BODY_SIDE: f32 = 6.0;
/// Quantas partículas o pino genérico segura — a primeira LINHA.
const BODY_PIN: f32 = 6.0;
/// Agentes por bando.
const FLOCK_COUNT: f32 = 40.0;
/// O tamanho com que cada agente é DESENHADO (o `SIZE_IDENTITY` do shell) — o
/// número contra o qual *"eles se sobrepõem"* se mede.
const DRAWN_SIZE: f32 = 1.0;
/// O espaço pessoal da banda 4, em unidades de mundo. ⚠️ **Ele é MAIOR que o
/// `radius` de propósito** — é o regime que o peso sozinho não alcançava, e o
/// preço está nomeado: um espaço pessoal que a grade da GPU não vê **recusa o
/// device** (o `applicable` do kernel), e esta banda cozinha na CPU.
const PERSONAL_SPACE: f32 = 4.0;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

fn wire_pre(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: true,
    })
    .ok()
}

/// Termina a banda num `motion.output`, deslocada para o seu lugar.
///
/// ⚠️ **O laço de render RE-RESOLVE os sinks a cada quadro** a partir dos nós de
/// saída do grafo, então uma cadeia sem Output cozinha certo, passa na suíte e
/// **desenha NADA na tela**.
fn place(g: &mut Graph, head: NodeId, dx: f32, dy: f32, x: f32, y: f32) -> Option<NodeId> {
    let mv = g.add_node("motion.move");
    g.set_pos(mv, Pos { x, y });
    g.set_param(mv, "dx", dx);
    g.set_param(mv, "dy", dy);
    wire(g, head, 0, mv, 0)?;
    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: x + 220.0, y });
    wire(g, mv, 0, out, 0)?;
    Some(out)
}

/// A BANDEIRA: um corpo mole cuja âncora é varrida por uma `value.lfo` — o mastro
/// que se move. `generic` escolhe por qual das duas espécies de pino a linha de
/// topo é pregada; **as duas têm de desenhar a mesma coisa**.
fn flag(g: &mut Graph, generic: bool, x: f32, y: f32) -> Option<NodeId> {
    let b = g.add_node("motion.soft_body");
    g.set_pos(b, Pos { x, y });
    g.set_param(b, "rows", BODY_SIDE);
    g.set_param(b, "cols", BODY_SIDE);
    g.set_param(b, "spacing", 0.7);
    g.set_param(b, "gravity", 9.8);
    g.set_param(b, "pin", if generic { 0.0 } else { 1.0 });

    // O MASTRO — a âncora é uma PORTA (`anchor_x`), nunca um param.
    let mast = g.add_node("value.lfo");
    g.set_pos(mast, Pos { x: x - 220.0, y });
    g.set_param(mast, "period", 2.4);
    g.set_param(mast, "amplitude", 2.5);
    wire(g, mast, 0, b, 0)?;

    if generic {
        let p = g.add_node("motion.pin_constraint");
        g.set_pos(
            p,
            Pos {
                x: x + 220.0,
                y: y + 90.0,
            },
        );
        g.set_param(p, "first", 0.0);
        g.set_param(p, "count", BODY_PIN);
        g.set_param(p, "strength", 1.0);
        wire_pre(g, b, 0, p, 0)?;
        wire(g, p, 0, b, 2)?;
    } else {
        wire_pre(g, b, 0, b, 2)?;
    }
    Some(b)
}

/// O BANDO, com ou sem espaço pessoal. As peças são DISCOS do tamanho desenhado —
/// a cena escreve `size` para que a sobreposição seja a que o artista vê, e não a
/// que uma fixture escolheu.
fn flock(g: &mut Graph, personal_space: bool, x: f32, y: f32) -> Option<NodeId> {
    let f = g.add_node("motion.boids");
    g.set_pos(f, Pos { x, y });
    g.set_param(f, "count", FLOCK_COUNT);
    g.set_param(f, "seed", 7.0);
    if personal_space {
        g.set_param(f, "separation_radius", PERSONAL_SPACE);
        // ⚠️ O peso continua a mandar em quão FIRME é o empurrão; o que o raio
        // mudou foi o ALCANCE, e é ele que o peso sozinho não alcançava.
        g.set_param(f, "separation", 6.0);
    }
    wire_pre(g, f, 0, f, 2)?;
    Some(f)
}

/// Monta a cena. Devolve os sinks, um por banda.
pub(crate) fn build_space_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(4);

    for (row, generic) in [false, true].into_iter().enumerate() {
        let gy = row as f32 * 300.0;
        let head = flag(g, generic, 0.0, gy)?;
        sinks.push(place(
            g,
            head,
            -8.0,
            BAND_DY - row as f32 * BAND_DY,
            440.0,
            gy,
        )?);
    }
    for (row, space) in [false, true].into_iter().enumerate() {
        let gy = (row + 2) as f32 * 300.0;
        let head = flock(g, space, 0.0, gy)?;
        sinks.push(place(
            g,
            head,
            10.0,
            -BAND_DY - row as f32 * BAND_DY,
            440.0,
            gy,
        )?);
    }
    Some(sinks)
}

/// Os rótulos das quatro bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "BANDEIRA, pino INTRINSECO -- o mastro varre e a bandeira acompanha",
        "BANDEIRA, pino GENERICO (motion.pin_constraint) -- TEM de fazer o mesmo",
        "BANDO, controle -- 40 agentes de tamanho 1,0 sobrepostos",
        "BANDO, espaco pessoal 4,0 -- eles se abrem",
    ]
    .into_iter()
    .enumerate()
}

/// O tamanho que a cena declara para cada agente, para a mensagem do smoke poder
/// citar a régua contra a qual *"eles se sobrepõem"* se mede.
pub(crate) fn drawn_size() -> f32 {
    DRAWN_SIZE
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_space_tests.rs"]
mod tests;
