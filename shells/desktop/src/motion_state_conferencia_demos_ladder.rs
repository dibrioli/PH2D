//! **O QUE A SIMULAÇÃO E A CONTAGEM NÃO SABIAM DIZER** — a cena `=92` (doc 89, folhas 03 e 07).
//!
//! Quatro pares. ⚠️ **Estes JULGAM-SE A ANDAR** — ao contrário das cenas `=90` e `=91`: três
//! dos quatro são estado que evolui (a corda cai, a mola persegue, a cauda desbota), e o
//! quarto conta. **Carregue Play.**
//!
//! | par | esquerda | direita |
//! |---|---|---|
//! | `motion.verlet_rope` | repouso uniforme | **`Rest Start/End`** — a corda afina, e não encolhe |
//! | `motion.spring` | canal `X` — só um eixo persegue | **`Position XY`** — um nó, os dois eixos |
//! | `motion.trail` | a cauda nasce colada à cabeça | **`Tail Alpha Max`** — ela nasce a 40% |
//! | `motion.step` | conta `0,1,2…` para cima | **`Direction = Down`** — a mesma escada ao contrário |

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O vão entre as duas colunas e entre as quatro linhas.
const GAP_X: f32 = 5.4;
const GAP_Y: f32 = 4.0;
/// O afunilamento que o par da corda autora.
const REST_START: f32 = 2.2;
const REST_END: f32 = 0.3;
/// O teto que o par da cauda autora.
const TAIL_MAX: f32 = 0.4;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16, delayed: bool) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed,
    })
    .ok()
}

/// Leva a banda ao quadrante e fecha, **sem a pintar**.
///
/// ⚠️ **O par da CAUDA passa por aqui, e a razão é a própria célula que ele demonstra:** um
/// `motion.tint` a jusante ESCREVE a coluna `tint`, então ele apaga a rampa de alfa que o
/// `motion.trail` acabou de autorar — as duas bandas mediram `1,0000` contra `1,0000` na
/// primeira versão desta cena. *A cena encenou o defeito que a célula descreve, contra si
/// própria.*
fn place(g: &mut Graph, head: NodeId, at: [f32; 2], ey: f32) -> Option<NodeId> {
    let mv = g.add_node("motion.move");
    g.set_pos(mv, Pos { x: 700.0, y: ey });
    g.set_param(mv, "dx", at[0]);
    g.set_param(mv, "dy", at[1]);
    wire(g, head, 0, mv, 0, false)?;
    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: 980.0, y: ey });
    wire(g, mv, 0, out, 0, false)?;
    Some(out)
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

/// **A CORDA** — pendurada numa âncora que varre, com ou sem afunilamento.
fn rope(g: &mut Graph, ey: f32, tapered: bool) -> Option<NodeId> {
    let r = g.add_node("motion.verlet_rope");
    g.set_pos(r, Pos { x: 400.0, y: ey });
    g.set_param(r, "count", 18.0);
    g.set_param(r, "length", 3.2);
    g.set_param(r, "gravity", 9.8);
    g.set_param(r, "iterations", 16.0);
    if tapered {
        g.set_param(r, "rest_start", REST_START);
        g.set_param(r, "rest_end", REST_END);
    }
    let lfo = g.add_node("value.lfo");
    g.set_pos(lfo, Pos { x: 120.0, y: ey });
    g.set_param(lfo, "period", 2.4);
    g.set_param(lfo, "amplitude", 1.6);
    wire(g, lfo, 0, r, 0, false)?;
    // O laço de estado — sem ele a corda coze o mesmo tique para sempre.
    wire(g, r, 0, r, 2, true)?;
    Some(r)
}

/// **A MOLA** — uma peça a perseguir um alvo que corre em CÍRCULO, num canal ou nos dois.
fn spring(g: &mut Graph, ey: f32, both_axes: bool) -> Option<NodeId> {
    let src = g.add_node("motion.grid");
    g.set_pos(src, Pos { x: 60.0, y: ey });
    g.set_param(src, "rows", 1.0);
    g.set_param(src, "cols", 1.0);
    let fit = g.add_node("motion.scale");
    g.set_pos(fit, Pos { x: 180.0, y: ey });
    g.set_param(fit, "amount", 0.45);
    wire(g, src, 0, fit, 0, false)?;

    // O ALVO corre em círculo: o `motion.orbit` move `P` em torno de um pivô.
    let orb = g.add_node("motion.orbit");
    g.set_pos(orb, Pos { x: 300.0, y: ey });
    g.set_param(orb, "speed", 90.0);
    g.set_param(orb, "pivot_x", -1.4);
    wire(g, fit, 0, orb, 0, false)?;

    let sp = g.add_node("motion.spring");
    g.set_pos(sp, Pos { x: 460.0, y: ey });
    g.set_param(sp, "channel", if both_axes { 4.0 } else { 0.0 });
    g.set_param(sp, "tension", 40.0);
    g.set_param(sp, "friction", 6.0);
    wire(g, orb, 0, sp, 0, false)?;
    wire(g, sp, 0, sp, 1, true)?;
    Some(sp)
}

/// **A CAUDA** — uma peça a orbitar, deixando eco, com ou sem teto.
fn trail(g: &mut Graph, ey: f32, capped: bool) -> Option<NodeId> {
    let src = g.add_node("motion.grid");
    g.set_pos(src, Pos { x: 60.0, y: ey });
    g.set_param(src, "rows", 1.0);
    g.set_param(src, "cols", 1.0);
    let fit = g.add_node("motion.scale");
    g.set_pos(fit, Pos { x: 180.0, y: ey });
    g.set_param(fit, "amount", 0.4);
    wire(g, src, 0, fit, 0, false)?;
    let orb = g.add_node("motion.orbit");
    g.set_pos(orb, Pos { x: 300.0, y: ey });
    g.set_param(orb, "speed", 120.0);
    g.set_param(orb, "pivot_x", -1.2);
    wire(g, fit, 0, orb, 0, false)?;

    let tr = g.add_node("motion.trail");
    g.set_pos(tr, Pos { x: 460.0, y: ey });
    g.set_param(tr, "length", 14.0);
    g.set_param(tr, "fade", 0.05);
    if capped {
        g.set_param(tr, "alpha_max", TAIL_MAX);
    }
    wire(g, orb, 0, tr, 0, false)?;
    wire(g, tr, 0, tr, 1, true)?;
    Some(tr)
}

/// **A ESCADA** — uma fileira que um pulso faz saltar, para cima ou para baixo.
fn ladder(g: &mut Graph, ey: f32, down: bool) -> Option<NodeId> {
    let src = g.add_node("motion.grid");
    g.set_pos(src, Pos { x: 60.0, y: ey });
    g.set_param(src, "rows", 1.0);
    g.set_param(src, "cols", 6.0);
    g.set_param(src, "gap_x", 0.7);
    let fit = g.add_node("motion.scale");
    g.set_pos(fit, Pos { x: 180.0, y: ey });
    g.set_param(fit, "amount", 0.3);
    wire(g, src, 0, fit, 0, false)?;

    // ⚠️ **O metrónomo é o `pulse.beat`, e ele toma um STREAM.** A primeira versão desta
    // banda ligava uma `value.lfo` a um `pulse.threshold` — e a porta `in` do limiar é
    // `INST_VEC2`, não um valor: o `validate` recusou com `TypeMismatch`. *O limiar
    // transforma um sinal POR ELEMENTO num pulso; quem produz um batimento é o beat.*
    let beat = g.add_node("pulse.beat");
    g.set_pos(
        beat,
        Pos {
            x: 300.0,
            y: ey + 110.0,
        },
    );
    g.set_param(beat, "period", 0.55);
    wire(g, fit, 0, beat, 0, false)?;
    wire(g, beat, 0, beat, 1, true)?;

    let st = g.add_node("motion.step");
    g.set_pos(st, Pos { x: 460.0, y: ey });
    g.set_param(st, "channel", 1.0); // Y
    g.set_param(st, "step", 0.45);
    g.set_param(st, "count_max", 6.0);
    if down {
        g.set_param(st, "direction", 1.0);
    }
    wire(g, fit, 0, st, 0, false)?;
    wire(g, beat, 0, st, 1, false)?;
    wire(g, st, 0, st, 2, true)?;
    Some(st)
}

/// Monta a cena. Devolve os oito sinks, em pares.
pub(crate) fn build_ladder_demo_document(
    doc: &mut MotionDoc,
    registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let rgb = [
        [0.46, 0.72, 1.0],
        [1.0, 0.74, 0.3],
        [0.62, 1.0, 0.66],
        [1.0, 0.6, 0.72],
    ];
    let mut sinks = Vec::with_capacity(8);
    for (row, colour) in rgb.iter().enumerate() {
        for col in 0..2 {
            let ey = (row * 2 + col) as f32 * 260.0;
            let on = col == 1;
            let head = match row {
                0 => rope(g, ey, on)?,
                1 => spring(g, ey, on)?,
                2 => trail(g, ey, on)?,
                _ => ladder(g, ey, on)?,
            };
            let at = [
                if col == 0 { -GAP_X } else { GAP_X },
                GAP_Y * 1.5 - row as f32 * GAP_Y,
            ];
            // A cauda fecha SEM tinta — ver [`place`].
            sinks.push(if row == 2 {
                place(g, head, at, ey)?
            } else {
                finish(g, head, *colour, at, ey)?
            });
        }
    }
    g.validate(registry).ok()?;
    Some(sinks)
}

/// Os rótulos das oito bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "CORDA uniforme -- todos os elos querem o mesmo comprimento",
        "CORDA afunilada -- os elos de cima largos, os de baixo curtos, e ela NAO encolheu",
        "MOLA num eixo so' -- ela persegue o alvo na horizontal e cola-se a ele na vertical",
        "MOLA Position XY -- um no' so', e ela persegue nos DOIS eixos: o atraso vira uma curva",
        "CAUDA sem teto -- o eco mais novo nasce colado a' cabeca",
        "CAUDA com Tail Alpha Max -- a cauda nasce ja' apagada, separada da cabeca",
        "ESCADA para cima -- os degraus sobem, e a contagem sobe com eles",
        "ESCADA Direction Down -- os mesmos degraus, percorridos ao contrario",
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
                GAP_Y * 1.5 - row as f32 * GAP_Y + GAP_Y * 0.42,
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
    (REST_START, REST_END, TAIL_MAX)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_ladder_tests.rs"]
mod tests;
