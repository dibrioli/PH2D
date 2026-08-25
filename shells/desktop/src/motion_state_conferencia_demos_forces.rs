//! **O QUE UMA FORÇA NÃO SABIA DIZER** — a cena `=95` (doc 89, folha 02).
//!
//! Quatro pares. ⚠️ **Esta cena ANDA — carregue Play**: as quatro fileiras são simulação.
//!
//! | par | esquerda | direita |
//! |---|---|---|
//! | `force.attractor` | a rampa de sempre | **o PERFIL** — janela, pico e inversão |
//! | `force.wind` | `Force` | **`Target Velocity`** — o vento SATURA |
//! | `force.vortex` | `Force` | **`Target Velocity`** — o rodamoinho estabiliza num anel |
//! | `force.buoyancy` | uma onda | **quatro** — o mar deixa de ser uma senoide |
//!
//! ⚠️ **As oito bandas montam a topologia do integrador**, que é a única que faz uma força
//! mover alguma coisa: a fonte entra em `rest`, e a cadeia de forças vive DENTRO do laço
//! `pre`. Ver [`sim_chain`].

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O vão entre as duas colunas e entre as quatro linhas.
const GAP_X: f32 = 5.2;
const GAP_Y: f32 = 3.8;
/// O perfil que o atrator da direita autora.
const PEAK: f32 = 2.2;
const REVERSE: f32 = 1.2;
/// A resistência do ar que os dois modos-alvo autoram.
const AIR: f32 = 3.0;
/// O espectro que o mar da direita autora.
const WAVES: f32 = 4.0;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16, delayed: bool) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed,
    })
    .ok()
}

/// **O ARRASTO** que as fileiras de equilíbrio precisam.
///
/// ⚠️ **Um anel é um EQUILÍBRIO, e um equilíbrio precisa de dissipação.** Sem arrasto a
/// nuvem atravessa o centro com a energia que ganhou a cair para lá e volta a sair — medido:
/// o ponto mais perto do alvo ficava em `4,65` de um raio de influência de `4`, ou seja
/// TODA a nuvem já tinha escapado. *Uma cena que demonstra onde as coisas assentam tem de
/// deixá-las assentar.*
fn damped(g: &mut Graph, ey: f32, head: NodeId) -> Option<NodeId> {
    let d = g.add_node("force.drag");
    g.set_pos(d, Pos { x: 380.0, y: ey });
    g.set_param(d, "coefficient", 1.2);
    wire(g, head, 0, d, 0, false)?;
    Some(d)
}

/// **A CADEIA QUE SIMULA** — a fonte, o integrador, e a força DENTRO do laço `pre`.
///
/// ⚠️ Uma `force.*` é `Pure` e só acumula `accel`; **um** integrador a consome. Montada na
/// horizontal (`fonte → força → saída`) a cena fica parada **sem erro nenhum** — o app
/// conserta esse gesto sozinho quando o artista o faz (ADR-0155), e um documento montado em
/// código não passa por esse portão.
fn sim_chain(g: &mut Graph, ey: f32, src: NodeId, head: NodeId, tail: NodeId) -> Option<NodeId> {
    let integ = g.add_node("motion.integrate");
    g.set_pos(integ, Pos { x: 460.0, y: ey });
    wire(g, src, 0, integ, 0, false)?;
    wire(g, integ, 0, head, 0, true)?;
    wire(g, tail, 0, integ, 1, false)?;
    Some(integ)
}

/// Leva a banda ao quadrante, pinta-a e fecha.
fn finish(g: &mut Graph, head: NodeId, rgb: [f32; 3], at: [f32; 2], ey: f32) -> Option<NodeId> {
    let mv = g.add_node("motion.move");
    g.set_pos(mv, Pos { x: 700.0, y: ey });
    g.set_param(mv, "dx", at[0]);
    g.set_param(mv, "dy", at[1]);
    wire(g, head, 0, mv, 0, false)?;
    let tint = g.add_node("motion.tint");
    g.set_pos(tint, Pos { x: 840.0, y: ey });
    g.set_param(tint, "r", rgb[0]);
    g.set_param(tint, "g", rgb[1]);
    g.set_param(tint, "b", rgb[2]);
    wire(g, mv, 0, tint, 0, false)?;
    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: 980.0, y: ey });
    wire(g, tint, 0, out, 0, false)?;
    Some(out)
}

/// A nuvem que as quatro fileiras partilham.
fn cloud(g: &mut Graph, ey: f32, seed: f32) -> NodeId {
    let n = g.add_node("motion.scatter");
    g.set_pos(n, Pos { x: 120.0, y: ey });
    g.set_param(n, "count", 90.0);
    // ⚠️ **A nuvem cabe DENTRO do raio de influência** (4,0): com ela a transbordar,
    // os cantos ficavam fora e nenhuma das duas fileiras do atrator mostrava o que a
    // força faz — media-se o que ela NÃO alcança.
    g.set_param(n, "width", 3.2);
    g.set_param(n, "height", 3.2);
    g.set_param(n, "seed", seed);
    n
}

/// **O ATRATOR** — a rampa de sempre, ou o perfil com pico e inversão.
fn attractor(g: &mut Graph, ey: f32, profiled: bool) -> NodeId {
    let a = g.add_node("force.attractor");
    g.set_pos(a, Pos { x: 300.0, y: ey });
    g.set_param(a, "strength", 6.0);
    g.set_param(a, "radius", 4.0);
    if profiled {
        g.set_param(a, ph2d_node_force_attractor::PEAK, PEAK);
        g.set_param(a, ph2d_node_force_attractor::REVERSE, REVERSE);
    }
    a
}

/// **O VENTO** — aceleração, ou velocidade-alvo.
fn wind(g: &mut Graph, ey: f32, target: bool) -> NodeId {
    let w = g.add_node("force.wind");
    g.set_pos(w, Pos { x: 300.0, y: ey });
    g.set_param(w, "strength", 2.0);
    g.set_param(w, "angle", 0.0);
    g.set_param(w, "gust", 0.0);
    if target {
        g.set_param(w, ph2d_node_force_wind::MODE, 1.0);
        g.set_param(w, ph2d_node_force_wind::AIR_RESIST, AIR);
    }
    w
}

/// **O VÓRTICE** — idem.
fn vortex(g: &mut Graph, ey: f32, target: bool) -> NodeId {
    let v = g.add_node("force.vortex");
    g.set_pos(v, Pos { x: 300.0, y: ey });
    g.set_param(v, "strength", 8.0);
    g.set_param(v, "radius", 4.0);
    if target {
        g.set_param(v, ph2d_node_force_vortex::MODE, 1.0);
        g.set_param(v, ph2d_node_force_vortex::AIR_RESIST, AIR);
    }
    v
}

/// **O MAR** — uma onda, ou quatro.
fn buoyancy(g: &mut Graph, ey: f32, spectrum: bool) -> NodeId {
    let b = g.add_node("force.buoyancy");
    g.set_pos(b, Pos { x: 300.0, y: ey });
    g.set_param(b, "level", 0.0);
    g.set_param(b, "density", 12.0);
    g.set_param(b, "drag", 1.5);
    g.set_param(b, "wave_amplitude", 0.7);
    g.set_param(b, "wave_length", 3.0);
    g.set_param(b, "wave_speed", 1.2);
    if spectrum {
        g.set_param(b, ph2d_node_force_buoyancy::WAVES, WAVES);
    }
    b
}

/// Monta a cena. Devolve os oito sinks, em pares.
pub(crate) fn build_forces_demo_document(
    doc: &mut MotionDoc,
    registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let rgb = [
        [0.52, 0.76, 1.0],
        [1.0, 0.78, 0.4],
        [0.66, 1.0, 0.72],
        [0.95, 0.6, 0.78],
    ];
    let mut sinks = Vec::with_capacity(8);
    for (row, colour) in rgb.iter().enumerate() {
        for col in 0..2 {
            let ey = (row * 2 + col) as f32 * 260.0;
            let on = col == 1;
            let src = cloud(g, ey, 3.0 + row as f32);
            let force = match row {
                0 => attractor(g, ey, on),
                1 => wind(g, ey, on),
                2 => vortex(g, ey, on),
                _ => buoyancy(g, ey, on),
            };
            // ⚠️ As fileiras do ATRATOR levam arrasto: elas demonstram onde as peças
            // ASSENTAM, e sem dissipação nada assenta. As outras três não — o que elas
            // mostram é precisamente a velocidade, e um arrasto mascararia a saturação.
            let tail = if row == 0 {
                damped(g, ey, force)?
            } else {
                force
            };
            let head = sim_chain(g, ey, src, force, tail)?;
            let at = [
                if col == 0 { -GAP_X } else { GAP_X },
                GAP_Y * 1.5 - row as f32 * GAP_Y,
            ];
            sinks.push(finish(g, head, *colour, at, ey)?);
        }
    }
    g.validate(registry).ok()?;
    Some(sinks)
}

/// Os rótulos das oito bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "ATRATOR de sempre -- puxa mais forte no centro, e tudo acaba num ponto",
        "ATRATOR com PERFIL -- ele EMPURRA de perto e puxa de longe: forma-se um ANEL",
        "VENTO Force -- a aceleracao nunca para, e as pecas saem do ecra",
        "VENTO Target Velocity -- elas alcancam a velocidade do vento e ficam nela",
        "VORTICE Force -- o giro acelera sem fim e a nuvem espalha-se",
        "VORTICE Target Velocity -- ela estabiliza num rodamoinho de raio constante",
        "MAR de UMA onda -- todas as cristas a mesma distancia, como um desenho",
        "MAR de QUATRO ondas -- cristas de tamanhos diferentes: le^-se como agua",
    ]
    .into_iter()
    .enumerate()
}

/// **As fichas desta cena, no canvas** — função PURA, medida pelo gate da legenda.
pub(crate) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    band_labels()
        .map(|(k, label)| {
            let (row, col) = (k / 2, k % 2);
            let at = [
                if col == 0 { -GAP_X } else { GAP_X },
                GAP_Y * 1.5 - row as f32 * GAP_Y + GAP_Y * 0.44,
            ];
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
pub(crate) fn authored() -> (f32, f32, f32) {
    (PEAK, AIR, WAVES)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_forces_tests.rs"]
mod tests;
