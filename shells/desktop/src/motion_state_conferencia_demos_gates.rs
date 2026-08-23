//! **O PAINEL QUE ENCOLHE** (`PH2D_GPU_COOK_DEMO=82`) — a cena da cura dos knobs mortos
//! ([doc 90](../../../docs/Motion%20Nodes/90_caca_aos_knobs_mortos.md), 2026-08-22).
//!
//! ## O oráculo é CONTAR LINHAS, e é isso que a torna smokável sem saber nada
//!
//! Cada célula desenha o **mesmo controle duas vezes ao mesmo tempo**: uma cópia com ele no
//! MÍNIMO e outra com ele no MÁXIMO, sobre a mesma linha de base.
//!
//! - **ESQUERDA** — o modo em que o controle **não faz nada** (o modo em que o nó NASCE). As
//!   duas cópias coincidem ao bit ⇒ vê-se **UMA linha**.
//! - **DIREITA** — o modo em que ele **age**. As duas cópias separam-se ⇒ vêem-se **DUAS**.
//!
//! ⚠️ **É por isso que a cena não precisa de legenda para se julgar**: o defeito e a cura são a
//! mesma pergunta — *este controle muda alguma coisa aqui?* — e a resposta é uma contagem, não
//! uma apreciação. Uma cena que mostrasse só o modo bom provaria que o knob funciona; ela não
//! provaria que ele era **mudo** no outro, que é o defeito que o Enio viria conferir.
//!
//! ## E a metade que a IMAGEM não mostra
//!
//! A cura é o painel a **esconder** a linha do controle na metade esquerda. Isso não se desenha:
//! vê-se clicando no nó e olhando o painel. A mensagem do smoke manda fazê-lo, e os dois gates
//! (`param_gates_are_exact` no kernel, `params_visible_tests` no painel) provam-no sem olhos.
//!
//! ⚠️ **Quatro dos onze nós curados não estão aqui, e é uma escolha**: `motion.emitter` e
//! `motion.boids` só respondem com o relógio a andar, `force.wind` precisa de um integrador e o
//! `fx.rgb_split` é um efeito de TELA. Pô-los aqui obrigaria a cena a misturar «julga-se parada»
//! com «carrega Play», que é o que faz um smoke ficar ambíguo. Eles seguem a MESMA lei e estão
//! cobertos pelos dois gates — o que esta cena tem de provar é a lei, não o censo.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};

/// Quantas peças cada linha tem.
const COUNT: f32 = 34.0;
/// O tamanho das peças.
const DOT: f32 = 0.2;
/// A distância vertical entre linhas.
const ROW_GAP: f32 = 1.55;
/// A que distância do centro cada coluna vive.
const COL_X: f32 = 2.6;
/// A meia-largura de uma figura.
const R: f32 = 1.05;
/// A meia-altura útil de uma célula — folgada contra `ROW_GAP/2` para nada invadir a vizinha.
const H: f32 = 0.5;

/// Qual dos sete controles curados esta linha encena.
#[derive(Clone, Copy)]
enum Knob {
    /// `motion.stagger::ease_dir` — inerte com a curva em `Linear`.
    StaggerDir,
    /// `value.step::width` — inerte no braço `Hard`.
    StepWidth,
    /// `value.map_range::clamp` — inerte em `Smooth`/`Smoother`, que clampam sempre.
    MapClamp,
    /// `value.instance_field::seed` — inerte fora de `Random`.
    FieldSeed,
    /// `value.noise::roughness` — inerte com uma oitava.
    NoiseRoughness,
    /// `motion.wiggle::amp_mult` — inerte com uma oitava.
    WiggleAmpMult,
    /// `motion.tint::r2..a2` (o swatch `End`) — inerte em `Solid`.
    TintEnd,
}

struct Row {
    label: &'static str,
    /// A ficha que pousa sobre a metade ESQUERDA (a muda) e sobre a DIREITA (a viva).
    left: &'static str,
    right: &'static str,
    knob: Knob,
    /// O valor do SELETOR na metade esquerda (onde o controle é mudo) e na direita.
    mute: f32,
    live: f32,
}

static ROWS_TABLE: &[Row] = &[
    Row {
        label: "STAGGER — a DIREÇÃO do easing: muda em Linear? (não) · em Bounce? (sim)",
        left: "1 STAGGER · Linear: mudo",
        right: "1 STAGGER · Bounce: age",
        knob: Knob::StaggerDir,
        mute: 0.0, // ease_curve = Linear
        live: 7.0, // ease_curve = Bounce
    },
    Row {
        label: "STEP    — a LARGURA da banda: muda no corte Hard? (não) · em Smooth? (sim)",
        left: "2 DEGRAU · corte Hard: mudo",
        right: "2 DEGRAU · Smooth: age",
        knob: Knob::StepWidth,
        mute: 0.0, // mode = Hard
        live: 1.0, // mode = Smooth
    },
    Row {
        label: "FAIXA   — o CLAMP: muda em Smooth? (não, ele já prende) · em Linear? (sim)",
        left: "3 FAIXA · Smooth: mudo",
        right: "3 FAIXA · Linear: age",
        knob: Knob::MapClamp,
        mute: 2.0, // interpolation = Smooth
        live: 0.0, // interpolation = Linear
    },
    Row {
        label: "CAMPO   — a SEMENTE: muda numa rampa? (não) · no modo Random? (sim)",
        left: "4 CAMPO · em rampa: mudo",
        right: "4 CAMPO · em Random: age",
        knob: Knob::FieldSeed,
        mute: 1.0, // mode = Ramp
        live: 2.0, // mode = Random
    },
    Row {
        label: "RUÍDO   — a ASPEREZA: muda com UMA oitava? (não) · com seis? (sim)",
        left: "5 RUÍDO · 1 oitava: mudo",
        right: "5 RUÍDO · 6 oitavas: age",
        knob: Knob::NoiseRoughness,
        mute: 1.0,
        live: 6.0,
    },
    Row {
        label: "TREMOR  — o MULTIPLICADOR: muda com UMA oitava? (não) · com quatro? (sim)",
        left: "6 TREMOR · 1 oitava: mudo",
        right: "6 TREMOR · 4 oitavas: age",
        knob: Knob::WiggleAmpMult,
        mute: 1.0,
        live: 4.0,
    },
    Row {
        label: "TINTA   — a SEGUNDA COR: muda em Solid? (não) · em Gradient? (sim)",
        left: "7 TINTA · em Solid: mudo",
        right: "7 TINTA · em Gradient: age",
        knob: Knob::TintEnd,
        mute: 0.0, // mode = Solid
        live: 1.0, // mode = Gradient
    },
];

/// Os números que a cena AUTORA e que a mensagem do smoke cita.
pub(crate) fn authored() -> (usize, f32) {
    (ROWS_TABLE.len(), COUNT)
}

/// Os rótulos, para a mensagem numerada.
pub(crate) fn row_labels() -> impl Iterator<Item = (usize, &'static str)> {
    ROWS_TABLE.iter().enumerate().map(|(i, r)| (i, r.label))
}

/// **As fichas desta cena, no canvas** — função PURA, medida pelo gate
/// ([`crate::motion_demo_legend`]).
///
/// ⚠️ **Acima da figura, e com `H` de folga, porque aqui as linhas são APERTADAS** (`ROW_GAP`
/// é 1,55 contra os 3,0 da cena irmã): uma ficha centrada na linha taparia as duas cópias que a
/// cena existe para contar, que é o oposto de uma legenda.
pub(crate) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    let mut out = Vec::with_capacity(ROWS_TABLE.len() * 2);
    for (k, row) in ROWS_TABLE.iter().enumerate() {
        let y = (ROWS_TABLE.len() as f32 - 1.0) * 0.5 * ROW_GAP - k as f32 * ROW_GAP + H * 0.62;
        out.push(crate::motion_demo_legend::Caption::new(
            [-COL_X, y],
            row.left,
        ));
        out.push(crate::motion_demo_legend::Caption::new(
            [COL_X, y],
            row.right,
        ));
    }
    out
}

fn wire(
    g: &mut ph2d_nodegraph::graph::Graph,
    from: NodeId,
    fp: u16,
    to: NodeId,
    tp: u16,
) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// O documento da cena `=82` — uma sink por célula (duas por linha).
pub(crate) fn build_gates_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::new();
    for (k, row) in ROWS_TABLE.iter().enumerate() {
        let y = (ROWS_TABLE.len() as f32 - 1.0) * 0.5 * ROW_GAP - k as f32 * ROW_GAP;
        for (half, selector) in [(0usize, row.mute), (1, row.live)] {
            let lane = 100.0 + (k * 2 + half) as f32 * 300.0;
            let cell = build_cell(g, row.knob, selector, lane)?;
            let place = g.add_node("motion.transform");
            g.set_param(place, "offset_x", if half == 0 { -COL_X } else { COL_X });
            g.set_param(place, "offset_y", y);
            let out = g.add_node("motion.output");
            g.set_pos(place, Pos { x: 1400.0, y: lane });
            g.set_pos(out, Pos { x: 1600.0, y: lane });
            wire(g, cell, 0, place, 0)?;
            wire(g, place, 0, out, 0)?;
            sinks.push(out);
        }
    }
    g.validate(reg).ok()?;
    Some(sinks)
}

/// Uma fileira de [`COUNT`] peças, e a rampa `0..1` sobre ela — o berço de toda célula.
fn seed(g: &mut ph2d_nodegraph::graph::Graph, lane: f32) -> Option<(NodeId, NodeId)> {
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", COUNT);
    g.set_param(grid, "gap_x", 2.0 * R / (COUNT - 1.0));
    g.set_param(grid, "gap_y", 0.0);
    let dot = g.add_node("motion.scale");
    g.set_param(dot, "amount", DOT);
    wire(g, grid, 0, dot, 0)?;
    let ramp = g.add_node("value.instance_field");
    g.set_param(ramp, "mode", 1.0); // Ramp: i/(N−1)
    wire(g, dot, 0, ramp, 0)?;
    g.set_pos(grid, Pos { x: 80.0, y: lane });
    g.set_pos(dot, Pos { x: 260.0, y: lane });
    Some((dot, ramp))
}

/// A fileira com o Y **posto** pelo valor dado (nunca somado — a lane É o valor).
///
/// `from` é a faixa em que o valor chega. ⚠️ **Ela é um argumento e não um default por uma razão
/// que esta cena pagou:** o `value.map_range` nasce com `clamp = 1`, então o encanamento que
/// põe o valor no Y **fechava a faixa** — e a linha do CLAMP, cuja pergunta inteira é *o que
/// acontece fora da faixa*, saía igual dos dois lados. *A fixture continha o fenómeno e a
/// canalização a jusante apagava-o* — o irmão mais difícil de ver da lei da fixture fraca,
/// porque o defeito não está onde se está a olhar.
fn lane_of(
    g: &mut ph2d_nodegraph::graph::Graph,
    dot: NodeId,
    value: NodeId,
    scale: f32,
    from: (f32, f32),
) -> Option<NodeId> {
    let mr = g.add_node("value.map_range");
    g.set_param(mr, "in_lo", from.0);
    g.set_param(mr, "in_hi", from.1);
    g.set_param(mr, "out_lo", -scale);
    g.set_param(mr, "out_hi", scale);
    wire(g, value, 0, mr, 0)?;
    let d = g.add_node("motion.drive");
    g.set_param(d, "channel", 1.0); // Y
    g.set_param(d, "mode", 1.0); // Set
    wire(g, dot, 0, d, 0)?;
    wire(g, mr, 0, d, 1)?;
    Some(d)
}

/// A faixa normal de um valor deste catálogo — `[0,1]`.
const UNIT: (f32, f32) = (0.0, 1.0);

/// **As duas cópias, sobrepostas** — o oráculo da cena.
///
/// ⚠️ Elas são unidas por `motion.combine` e **não** desenhadas em duas linhas separadas: o que
/// se pergunta é se elas COINCIDEM, e duas curvas em sítios diferentes da tela não respondem
/// isso — o olho compararia formas em vez de contar linhas.
fn overlay(
    g: &mut ph2d_nodegraph::graph::Graph,
    a: NodeId,
    b: NodeId,
    lane: f32,
) -> Option<NodeId> {
    let c = g.add_node("motion.combine");
    wire(g, a, 0, c, 0)?;
    wire(g, b, 0, c, 1)?;
    g.set_pos(c, Pos { x: 1200.0, y: lane });
    Some(c)
}

fn build_cell(
    g: &mut ph2d_nodegraph::graph::Graph,
    knob: Knob,
    selector: f32,
    lane: f32,
) -> Option<NodeId> {
    // As duas cópias: o controle no MÍNIMO e no MÁXIMO da faixa que a UI permite.
    let mut copy = |extreme: bool| -> Option<NodeId> {
        let (dot, ramp) = seed(g, lane)?;
        match knob {
            Knob::StaggerDir => {
                let st = g.add_node("motion.stagger");
                g.set_param(st, "channel", 1.0); // Y
                g.set_param(st, "min", -H);
                g.set_param(st, "max", H);
                g.set_param(st, "ease_curve", selector);
                // O knob: In (0) contra In-Out (2).
                g.set_param(st, "ease_dir", if extreme { 2.0 } else { 0.0 });
                wire(g, dot, 0, st, 0)?;
                Some(st)
            }
            Knob::StepWidth => {
                let s = g.add_node("value.step");
                g.set_param(s, "threshold", 0.5);
                g.set_param(s, "mode", selector);
                g.set_param(s, "width", if extreme { 1.0 } else { 0.0 });
                wire(g, ramp, 0, s, 0)?;
                lane_of(g, dot, s, H, UNIT)
            }
            Knob::MapClamp => {
                // ⚠️ A entrada TEM de sair da faixa, senão o clamp não tem o que morder — e a
                // célula nasceria a dizer «não muda» dos dois lados, que é o modo de falha
                // desta cena: uma fixture que não contém o fenómeno.
                let wide = g.add_node("value.map_range");
                g.set_param(wide, "out_lo", -1.0);
                g.set_param(wide, "out_hi", 2.0);
                wire(g, ramp, 0, wide, 0)?;
                let m = g.add_node("value.map_range");
                g.set_param(m, "interpolation", selector);
                g.set_param(m, "clamp", if extreme { 1.0 } else { 0.0 });
                wire(g, wide, 0, m, 0)?;
                lane_of(g, dot, m, H, (-1.0, 2.0))
            }
            Knob::FieldSeed => {
                let f = g.add_node("value.instance_field");
                g.set_param(f, "mode", selector);
                g.set_param(f, "seed", if extreme { 9_000.0 } else { 1.0 });
                wire(g, dot, 0, f, 0)?;
                lane_of(g, dot, f, H, UNIT)
            }
            Knob::NoiseRoughness => {
                // ⚠️ **O `value.noise` recebe o STREAM, não o valor** — ele é uma FONTE de
                // valor (amostra o índice ou a posição), não um transformador. Ligar-lhe a
                // rampa é o erro de tipo que a primeira versão desta cena cometeu quatro vezes.
                let n = g.add_node("value.noise");
                g.set_param(n, "octaves", selector);
                g.set_param(n, "roughness", if extreme { 1.0 } else { 0.0 });
                wire(g, dot, 0, n, 0)?;
                lane_of(g, dot, n, H, UNIT)
            }
            Knob::WiggleAmpMult => {
                let w = g.add_node("motion.wiggle");
                g.set_param(w, "channel", 1.0); // Y
                g.set_param(w, "amplitude", H);
                g.set_param(w, "octaves", selector);
                g.set_param(w, "amp_mult", if extreme { 1.0 } else { 0.0 });
                wire(g, dot, 0, w, 0)?;
                Some(w)
            }
            Knob::TintEnd => {
                // As duas cópias ficam em Y distintos: uma COR não se compara sobreposta.
                let place = g.add_node("motion.transform");
                g.set_param(place, "offset_y", if extreme { H * 0.6 } else { -H * 0.6 });
                wire(g, dot, 0, place, 0)?;
                let t = g.add_node("motion.tint");
                g.set_param(t, "mode", selector);
                // A cor de partida é a MESMA nas duas; só a `End` muda.
                for (ch, v) in [("r", 0.05), ("g", 0.45), ("b", 0.95), ("a", 1.0)] {
                    g.set_param(t, ch, v);
                }
                let end = if extreme { 0.95 } else { 0.05 };
                for (ch, v) in [("r2", end), ("g2", 0.45), ("b2", 1.0 - end), ("a2", 1.0)] {
                    g.set_param(t, ch, v);
                }
                wire(g, place, 0, t, 0)?;
                Some(t)
            }
        }
    };
    let a = copy(false)?;
    let b = copy(true)?;
    overlay(g, a, b, lane)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_gates_tests.rs"]
mod tests;
