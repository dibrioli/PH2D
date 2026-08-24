//! **A FORMA QUE SE DESENHA E A VARIAÇÃO QUE SE PEDE** — a cena `=94`.
//!
//! Cinco pares. ⚠️ **Esta cena ANDA — carregue Play**: o oscilador oscila e as quatro
//! fileiras de baixo são partículas.
//!
//! | par | esquerda | direita |
//! |---|---|---|
//! | `motion.oscillator` | `Sine` | **`Custom`** — a forma desenhada a conduzir de facto |
//! | `motion.randomize` | o penacho de sempre | **`Rotation`** — cada partícula com o seu ângulo |
//! | `motion.randomize` | idem | **`Opacity`** — umas mais apagadas que outras |
//! | `motion.randomize` | idem, pintado | **`Hue`** — cada uma com a sua cor |
//! | `motion.randomize` | idem | **`Size`** — cada uma com o seu tamanho |
//!
//! ⚠️ **A primeira fileira é o DEFEITO curado** (Enio, 2026-08-24, com foto): o editor
//! `Custom Wave` era oferecido em toda onda e só lido na `Custom`. Hoje ele só aparece na
//! onda que o lê — então a esquerda desta fileira **não tem editor de curva no painel**, e
//! é isso que se vai ver ao clicar nas duas.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O vão entre as duas colunas e entre as cinco linhas.
const GAP_X: f32 = 5.0;
const GAP_Y: f32 = 3.4;
/// A forma que a onda `Custom` da direita carrega — um V, que nenhuma onda enumerada faz.
const CUSTOM_WAVE: &str = "c1 0:1:L 0.5:0:L 1:1:L";
/// A dispersão que os quatro pares de partícula autoram.
const AMOUNT: f32 = 0.7;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// Leva a banda ao quadrante e fecha.
///
/// ⚠️ **Sem `motion.tint`**, ao contrário das outras cenas: três destas fileiras variam a
/// COR, e um nó de cor a jusante escreveria a coluna `tint` por cima do que o
/// `motion.randomize` acabou de dispersar. *É o mesmo defeito que a cena `=92` encenou
/// contra si própria.*
fn finish(g: &mut Graph, head: NodeId, at: [f32; 2], ey: f32) -> Option<NodeId> {
    let mv = g.add_node("motion.move");
    g.set_pos(mv, Pos { x: 760.0, y: ey });
    g.set_param(mv, "dx", at[0]);
    g.set_param(mv, "dy", at[1]);
    wire(g, head, 0, mv, 0)?;
    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: 980.0, y: ey });
    wire(g, mv, 0, out, 0)?;
    Some(out)
}

/// **O OSCILADOR** — uma fileira de peças a subir e descer, na onda escolhida.
fn oscillator(g: &mut Graph, ey: f32, custom: bool) -> Option<NodeId> {
    let src = g.add_node("motion.grid");
    g.set_pos(src, Pos { x: 120.0, y: ey });
    g.set_param(src, "rows", 1.0);
    g.set_param(src, "cols", 12.0);
    g.set_param(src, "gap_x", 0.32);
    let osc = g.add_node("motion.oscillator");
    g.set_pos(osc, Pos { x: 420.0, y: ey });
    g.set_param(osc, "channel", 1.0); // Y
    g.set_param(osc, "amplitude", 1.1);
    g.set_param(osc, "frequency", 0.5);
    if custom {
        g.set_param(osc, "wave", 5.0); // Custom
        g.set_text_param(osc, "curve", CUSTOM_WAVE.to_string());
    }
    wire(g, src, 0, osc, 0)?;
    Some(osc)
}

/// **O PENACHO** — o emissor que as quatro fileiras de baixo partilham.
fn plume(g: &mut Graph, ey: f32) -> NodeId {
    let em = g.add_node("motion.emitter");
    g.set_pos(em, Pos { x: 240.0, y: ey });
    g.set_param(em, "rate", 45.0);
    g.set_param(em, "life", 2.0);
    g.set_param(em, "speed", 0.9);
    g.set_param(em, "angle", 90.0);
    g.set_param(em, "spread", 140.0);
    g.set_param(em, "max", 192.0);
    g.set_param(em, "size", 0.16);
    g.set_param(em, "seed", 6.0);
    em
}

/// A cor de base das fileiras que variam cor — **antes** do `motion.randomize`, nunca
/// depois.
fn painted(g: &mut Graph, head: NodeId, ey: f32, rgb: [f32; 3]) -> Option<NodeId> {
    let t = g.add_node("motion.tint");
    g.set_pos(t, Pos { x: 420.0, y: ey });
    g.set_param(t, "r", rgb[0]);
    g.set_param(t, "g", rgb[1]);
    g.set_param(t, "b", rgb[2]);
    wire(g, head, 0, t, 0)?;
    Some(t)
}

/// **A VARIAÇÃO** — o nó novo, no canal pedido.
fn vary(g: &mut Graph, head: NodeId, ey: f32, channel: f32) -> Option<NodeId> {
    let r = g.add_node("motion.randomize");
    g.set_pos(r, Pos { x: 600.0, y: ey });
    g.set_param(r, "channel", channel);
    g.set_param(r, "amount", AMOUNT);
    g.set_param(r, "seed", 11.0);
    wire(g, head, 0, r, 0)?;
    Some(r)
}

/// Monta a cena. Devolve os dez sinks, em pares.
pub(crate) fn build_vary_demo_document(
    doc: &mut MotionDoc,
    registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    // Os canais do `motion.randomize` que cada fileira demonstra.
    let rows = [0.0_f32, 1.0, 2.0, 5.0]; // Rotation · Opacity · Hue · Size
    let rgb = [
        [0.55, 0.78, 1.0],
        [1.0, 0.78, 0.42],
        [0.95, 0.45, 0.55],
        [0.66, 1.0, 0.72],
    ];
    let mut sinks = Vec::with_capacity(10);
    for col in 0..2 {
        let ey = col as f32 * 260.0;
        let head = oscillator(g, ey, col == 1)?;
        let at = [if col == 0 { -GAP_X } else { GAP_X }, GAP_Y * 2.0];
        sinks.push(finish(g, head, at, ey)?);
    }
    for (row, channel) in rows.iter().enumerate() {
        for col in 0..2 {
            let ey = ((row + 1) * 2 + col) as f32 * 260.0;
            let em = plume(g, ey);
            let head = painted(g, em, ey, rgb[row])?;
            let head = if col == 1 {
                vary(g, head, ey, *channel)?
            } else {
                head
            };
            let at = [
                if col == 0 { -GAP_X } else { GAP_X },
                GAP_Y * 1.0 - row as f32 * GAP_Y,
            ];
            sinks.push(finish(g, head, at, ey)?);
        }
    }
    g.validate(registry).ok()?;
    Some(sinks)
}

/// Os rótulos das dez bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "ONDA Sine -- a onda de sempre, e no painel dela NAO ha' editor de curva",
        "ONDA Custom -- a forma DESENHADA (um V) a conduzir mesmo, e o editor aparece",
        "PENACHO de sempre -- toda particula com o mesmo angulo",
        "PENACHO + Randomize(Rotation) -- cada uma com o seu angulo",
        "PENACHO de sempre -- todas igualmente opacas",
        "PENACHO + Randomize(Opacity) -- umas mais apagadas que outras",
        "PENACHO de sempre -- todas da mesma cor",
        "PENACHO + Randomize(Hue) -- cada uma com a sua cor",
        "PENACHO de sempre -- todas do mesmo tamanho",
        "PENACHO + Randomize(Size) -- cada uma com o seu tamanho",
    ]
    .into_iter()
    .enumerate()
}

/// **As fichas desta cena, no canvas** — função PURA, medida pelo gate da legenda.
pub(crate) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    band_labels()
        .map(|(k, label)| {
            let (row, col) = (k / 2, k % 2);
            let y = if row == 0 {
                GAP_Y * 2.0
            } else {
                GAP_Y * 1.0 - (row - 1) as f32 * GAP_Y
            };
            let at = [if col == 0 { -GAP_X } else { GAP_X }, y + GAP_Y * 0.42];
            crate::motion_demo_legend::Caption::new(at, short_of(label))
        })
        .collect()
}

/// A ficha curta: o que está ANTES do primeiro `--`.
fn short_of(label: &'static str) -> &'static str {
    match label.find(" --") {
        Some(i) => &label[..i],
        None => label,
    }
}

/// Os números que a mensagem do smoke cita.
pub(crate) fn authored() -> (&'static str, f32) {
    (CUSTOM_WAVE, AMOUNT)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_vary_tests.rs"]
mod tests;
