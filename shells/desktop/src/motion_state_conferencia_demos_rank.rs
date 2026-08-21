//! **A IDENTIDADE, O POSTO, A CURVA E A FORMA** — a cena `=73` (doc 89: folha 08
//! inteira, duas células; folha 10 inteira, duas células).
//!
//! Quatro pares. O mesmo grafo dos dois lados; só o número novo muda.
//!
//! | par | esquerda | direita |
//! |---|---|---|
//! | `motion.cull` | como era — o degradê pára a meio | **`Renumber Survivors`** — ele volta a alcançar o fim |
//! | `field.index_range` | `Order By = Index` — a banda é um bloco da lista | **`Order By = Attribute`** — a banda é o POSTO, e o conjunto não se mexe |
//! | `field.remap` | `Curve Offset = 0` | **um deslocamento** — a rampa desfila e reentra |
//! | `field.shape` (**nó novo**) | `Filled Path` — a forma é uma máscara sólida | **`Path Edges`** — ela é um contorno, e o miolo esvazia |
//!
//! ⚠️ **O par 1 é o mais fácil de ler errado.** As duas metades têm o MESMO número de
//! peças (metade da grelha, cortada igual): o que difere é a COR, porque o degradê
//! divide pela contagem que a lista anuncia. À esquerda ela anuncia 36 e só existem 18,
//! então a rampa pára a meio; à direita ela anuncia 18 e a rampa chega ao fim.
//!
//! ⚠️ **O par 3 não tem curva autorada, e isso é o desenho.** Sem curva, o `Curve`
//! contour é a identidade — então o que o deslocamento faz é visível NU: a rampa inteira
//! anda e reentra pelo começo. Uma curva autorada por cima só esconderia o mecanismo
//! atrás da forma dela.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O vão entre as duas colunas e entre as quatro linhas.
const GAP_X: f32 = 6.0;
const GAP_Y: f32 = 4.4;
/// A linha de cima (as quatro descem a partir daqui).
const TOP_Y: f32 = 6.6;

/// A fracção que o par 1 mantém — metade, para que a contagem mentida seja o DOBRO
/// da verdadeira e a rampa pare exactamente a meio.
const KEEP: f32 = 0.5;
/// As duas pontas do degradê do par 1 — amarelo → vermelho profundo. ⚠️ Escolhidas
/// para que **meio caminho** (o laranja) seja inconfundível a olho contra o fim.
const RAMP_START: [f32; 3] = [1.0, 0.92, 0.25];
const RAMP_END: [f32; 3] = [0.75, 0.05, 0.08];

/// A banda estreita que o par 2 acende, e a rampa dela.
const BAND_LO: f32 = 0.4;
const BAND_HI: f32 = 0.6;
const BAND_SOFT: f32 = 0.05;
/// `Order By = Attribute` (o valor do enum, não um índice de linha).
const ORDER_BY_ATTRIBUTE: f32 = 1.0;
/// A frequência do campo que serve de atributo — escolhida contra o passo da grelha
/// (0,42), para que peças vizinhas caiam em postos distantes.
const ATTR_FREQ: f32 = 3.0;

/// O contorno `Curve` do `field.remap` (o valor do enum).
const CONTOUR_CURVE: f32 = 4.0;
/// O deslocamento que o lado direito do par 3 autora — pouco mais de um terço, para
/// que a costura caia bem dentro da fileira em vez de na ponta.
const CURVE_SHIFT: f32 = 0.35;
/// O modo `Ramp` do `value.instance_field`: o índice normalizado, `0..1`.
const FIELD_RAMP: f32 = 1.0;
/// O canal **Falloff** do `motion.drive`, e o modo **Set**.
const DRIVE_FALLOFF: f32 = 5.0;
const DRIVE_SET: f32 = 1.0;

/// Quantos vértices tem a forma do par 4, e o raio dela — um pentágono, porque um
/// polígono ÍMPAR não tem simetria de meia volta e por isso não se confunde com um
/// círculo nem com a grelha por baixo.
const SHAPE_SIDES: f32 = 5.0;
const SHAPE_RADIUS: f32 = 1.5;
/// A penumbra do `field.shape`, em unidades de mundo.
const SHAPE_DISTANCE: f32 = 0.5;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

fn push(g: &mut Graph, head: NodeId, kind: &str, ps: &[(&str, f32)], ey: f32, x: f32) -> NodeId {
    let n = g.add_node(kind);
    g.set_pos(n, Pos { x, y: ey });
    for (k, v) in ps {
        g.set_param(n, *k, *v);
    }
    let _ = wire(g, head, 0, n, 0);
    n
}

fn node(g: &mut Graph, kind: &str, ps: &[(&str, f32)], ey: f32, x: f32) -> NodeId {
    let n = g.add_node(kind);
    g.set_pos(n, Pos { x, y: ey });
    for (k, v) in ps {
        g.set_param(n, *k, *v);
    }
    n
}

/// Uma grelha `rows × cols` de peças pequenas, já escalada.
fn grid(g: &mut Graph, rows: f32, cols: f32, gap: f32, piece: f32, ey: f32) -> NodeId {
    let n = node(
        g,
        "motion.grid",
        &[
            ("rows", rows),
            ("cols", cols),
            ("gap_x", gap),
            ("gap_y", gap),
        ],
        ey,
        0.0,
    );
    push(g, n, "motion.scale", &[("amount", piece)], ey, 160.0)
}

/// Fecha uma banda: posiciona-a no seu quadrante e liga a saída.
fn finish(g: &mut Graph, tail: NodeId, at: [f32; 2], ey: f32) -> Option<NodeId> {
    let placed = push(
        g,
        tail,
        "motion.move",
        &[("dx", at[0]), ("dy", at[1])],
        ey,
        900.0,
    );
    let out = node(g, "motion.output", &[], ey, 1040.0);
    wire(g, placed, 0, out, 0)?;
    Some(out)
}

/// **PAR 1** — a renumeração do `motion.cull`. O degradê é keyed pela contagem que a
/// lista ANUNCIA, e é por isso que este par se lê pela cor e não pelo número de peças.
fn cull_band(g: &mut Graph, ey: f32, at: [f32; 2], renumber: bool) -> Option<NodeId> {
    let base = grid(g, 6.0, 6.0, 0.45, 0.16, ey);
    let cu = push(
        g,
        base,
        "motion.cull",
        &[
            ("mode", 0.0), // Fraction
            ("amount", KEEP),
            ("reindex", f32::from(u8::from(renumber))),
        ],
        ey,
        330.0,
    );
    let tint = push(
        g,
        cu,
        "motion.tint",
        &[
            ("mode", 1.0), // Gradient
            ("r", RAMP_START[0]),
            ("g", RAMP_START[1]),
            ("b", RAMP_START[2]),
            ("r2", RAMP_END[0]),
            ("g2", RAMP_END[1]),
            ("b2", RAMP_END[2]),
        ],
        ey,
        520.0,
    );
    finish(g, tint, at, ey)
}

/// **PAR 2** — o posto por atributo. À direita a banda segue o VALOR do campo, e o
/// stream fica exactamente onde estava.
fn rank_band(g: &mut Graph, ey: f32, at: [f32; 2], by_attribute: bool) -> Option<NodeId> {
    let base = grid(g, 8.0, 8.0, 0.42, 0.15, ey);
    let ir = node(
        g,
        "field.index_range",
        &[
            ("start", BAND_LO),
            ("end", BAND_HI),
            ("soft", BAND_SOFT),
            ("curve", 0.0), // Linear
            (
                "key",
                if by_attribute {
                    ORDER_BY_ATTRIBUTE
                } else {
                    0.0
                },
            ),
        ],
        ey,
        520.0,
    );
    wire(g, base, 0, ir, 0)?;
    if by_attribute {
        // ⚠️ O MESMO campo dos dois lados seria impossível: à esquerda ele não é lido.
        // O que o par mantém igual é a BANDA; o que muda é o que a ordena.
        let noise = node(
            g,
            "value.noise",
            &[("frequency", ATTR_FREQ), ("amplitude", 1.0), ("speed", 0.0)],
            ey + 140.0,
            330.0,
        );
        wire(g, base, 0, noise, 0)?;
        wire(g, noise, 0, ir, 1)?;
    }
    // A máscara pinta: quem está na banda toma a cor, quem não está fica branco.
    let tint = push(
        g,
        ir,
        "motion.tint",
        &[("r", 0.25), ("g", 0.7), ("b", 1.0)],
        ey,
        700.0,
    );
    finish(g, tint, at, ey)
}

/// **PAR 3** — o deslocamento da curva. A máscara é a rampa `0..1` do índice; o
/// `field.remap` no contorno `Curve` (sem curva autorada = a identidade) devolve-a
/// tal e qual, e o deslocamento fá-la desfilar.
fn shift_band(g: &mut Graph, ey: f32, at: [f32; 2], shifted: bool) -> Option<NodeId> {
    let base = grid(g, 1.0, 16.0, 0.5, 0.19, ey);
    // A rampa do índice, escrita NO `falloff` pelo canal que existe para isso.
    let ramp = node(
        g,
        "value.instance_field",
        &[("mode", FIELD_RAMP)],
        ey + 140.0,
        330.0,
    );
    wire(g, base, 0, ramp, 0)?;
    let drv = node(
        g,
        "motion.drive",
        &[
            ("channel", DRIVE_FALLOFF),
            ("mode", DRIVE_SET),
            ("scale", 1.0),
        ],
        ey,
        470.0,
    );
    wire(g, base, 0, drv, 0)?;
    wire(g, ramp, 0, drv, 1)?;
    let rm = push(
        g,
        drv,
        "field.remap",
        &[
            ("contour", CONTOUR_CURVE),
            ("curve_offset", if shifted { CURVE_SHIFT } else { 0.0 }),
        ],
        ey,
        640.0,
    );
    let tint = push(
        g,
        rm,
        "motion.tint",
        &[("r", 0.95), ("g", 0.35), ("b", 0.75)],
        ey,
        780.0,
    );
    finish(g, tint, at, ey)
}

/// **PAR 4** — o nó NOVO: uma geometria como campo. O mesmo pentágono dos dois lados;
/// só o *Path Mode* muda.
fn shape_band(g: &mut Graph, ey: f32, at: [f32; 2], edges: bool) -> Option<NodeId> {
    // ⚠️ O passo `0.45` NÃO é estético: com `0.34` a grelha media 3,4 de largura e o
    // pentágono (raio 1,5 + penumbra 0,5 = 2,0) transbordava-a — as duas bandas do par
    // sairiam mascaradas por inteiro, IGUAIS, e o par estaria verde e mudo. Quem
    // apanhou isso foi o gate `the_pentagon_fits_inside_the_grid_it_masks`, que deriva
    // a conta da grelha em vez de a escrever à mão.
    let base = grid(g, 11.0, 11.0, 0.45, 0.15, ey);
    // A FORMA: cinco pontos num anel — um pentágono, pela porta `shape`.
    let pent = node(
        g,
        "motion.distribute_radial",
        &[
            ("count", SHAPE_SIDES),
            ("rings", 1.0),
            ("radius", SHAPE_RADIUS),
            ("inner", 0.0),
        ],
        ey + 140.0,
        330.0,
    );
    let fs = node(
        g,
        "field.shape",
        &[
            ("mode", f32::from(u8::from(edges))),
            ("distance", SHAPE_DISTANCE),
            ("curve", 2.0), // Smooth
        ],
        ey,
        520.0,
    );
    wire(g, base, 0, fs, 0)?;
    wire(g, pent, 0, fs, 1)?;
    let tint = push(
        g,
        fs,
        "motion.tint",
        &[("r", 0.45), ("g", 1.0), ("b", 0.55)],
        ey,
        700.0,
    );
    finish(g, tint, at, ey)
}

/// Monta a cena. Devolve os oito sinks, em pares.
pub(crate) fn build_rank_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(8);
    for (row, right) in (0..4).flat_map(|r| [(r, false), (r, true)]) {
        let ey = (row * 2 + usize::from(right)) as f32 * 240.0;
        let at = [
            if right { GAP_X } else { -GAP_X },
            TOP_Y - row as f32 * GAP_Y,
        ];
        let sink = match row {
            0 => cull_band(g, ey, at, right)?,
            1 => rank_band(g, ey, at, right)?,
            2 => shift_band(g, ey, at, right)?,
            _ => shape_band(g, ey, at, right)?,
        };
        sinks.push(sink);
    }
    Some(sinks)
}

/// Os rótulos das oito bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "CORTE como era -- o degrade' pa'ra a meio, porque a lista mente a contagem",
        "CORTE com `Renumber Survivors` -- a MESMA metade, e a rampa chega ao fim",
        "BANDA por indice -- ela acende um bloco contiguo da lista",
        "BANDA por ATRIBUTO -- ela acende o POSTO, e nada se reordena",
        "RAMPA sem deslocamento -- o degrade' de sempre",
        "RAMPA deslocada -- ela desfila e reentra pelo comeco",
        "FORMA como mascara SOLIDA (Filled Path) -- o pentagono cheio",
        "FORMA como CONTORNO (Path Edges) -- so' a borda, e o miolo esvazia",
    ]
    .into_iter()
    .enumerate()
}

/// Os números que a mensagem do smoke cita, para ela não os repetir à mão.
pub(crate) fn authored() -> (f32, f32) {
    (CURVE_SHIFT, SHAPE_DISTANCE)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_rank_tests.rs"]
mod tests;
