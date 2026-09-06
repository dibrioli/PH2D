//! ⭐⭐⭐ **TODO O DUPLICATOR NUMA CENA** (`=110`) — pedido do Enio, 2026-09-05, a seguir ao
//! report *«em duplicator não vejo o efeito de Pick, Point Scale e Transfer»*.
//!
//! A medição daquele report (`measure_what_each_param_needs`, na crate do nó) mostrou que os
//! três estão **vivos** e que cada um precisa de algo na ENTRADA — e um param cujo sujeito não
//! existe naquela cena lê-se exactamente como um param morto. ⇒ esta cena é a **entrada
//! montada**: cada banda traz o que aquele controlo precisa para falar.
//!
//! ## As treze bandas, por fileira
//!
//! | fileira | o que ela responde |
//! |---|---|
//! | 1 | **o que o carimbo É** — a forma inteira pousa em cada ponto, e `P`/`rot` SOMAM |
//! | 2 | **`Pick`** — qual forma pousa em qual ponto (`Off` · `Cycle` · `Random`) |
//! | 3 | **`Point Scale`** — a escala do PONTO compõe-se com a da forma (`0` · `0,5` · `1`) |
//! | 4 | **`Transfer`** — de quem é a cor quando a coluna existe dos DOIS lados |
//!
//! ⚠️ **As três bandas do `Pick` carimbam as MESMAS formas, e as quatro do `Transfer` a mesma
//! forma cinzenta** — a régua é a lei do `=98`: se as entradas diferissem, o olho atribuiria a
//! diferença às entradas e não ao modo. Por isso as formas são construídas **uma vez** e
//! partilhadas: mexer numa muda o trio inteiro, que é o que faz a comparação continuar honesta.
//!
//! ⚠️ **Os pontos não trazem `size` nas bandas que não são do `Point Scale`**, de propósito: o
//! `apply_point_scale` sai cedo quando a coluna não existe, então uma coluna posta por comodidade
//! ligaria aquele controlo em bandas que não são sobre ele.
//!
//! ⚠️ **A banda 2 e a banda 4 são a MESMA lei vista de dois lados** — «uma forma feita de três
//! peças» e «três formas alternativas» são o mesmo stream para este nó; o que as separa é o
//! `Pick`, que trata cada ELEMENTO da forma como uma candidata. Está escrito no anúncio.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Quantas bandas tem cada fileira — e esta lista **é a fonte**: o [`quadrant`] deriva dela e o
/// número de bandas conta-se dela, então acrescentar um caso é mudar UM número.
const LAYOUT: [usize; 4] = [3, 3, 3, 4];
/// Quantas bandas a cena monta, CONTADAS do `LAYOUT`.
const BANDS: usize = LAYOUT[0] + LAYOUT[1] + LAYOUT[2] + LAYOUT[3];
/// A distância entre os centros de duas bandas vizinhas da mesma fileira.
const COL_STEP: f32 = 2.5;
/// O centro de cada fileira, de cima para baixo.
const ROW_Y: [f32; 4] = [2.7, 0.9, -0.9, -2.7];

/// O tamanho de uma peça e o passo entre pontos, em unidades de mundo.
const PIECE: f32 = 0.2;
const GAP: f32 = 0.34;
/// Quantos pontos tem a fila de uma banda normal.
const PIECES: f32 = 5.0;
/// A banda do GRUPO usa menos pontos e mais espaço: a forma dela é larga.
const COMET_POINTS: f32 = 3.0;
const COMET_GAP: f32 = 0.72;

/// O desnível entre as três formas alternativas — é o que faz o `Cycle` e o `Random` lerem-se
/// mesmo em cinzento, e o que transforma o produto cartesiano numa grelha de 3 × 5.
const SPREAD: f32 = 0.26;
/// Quanto o último ponto da fila gira, em graus. ⛔ **Não 90:** um quadrado a 90° lê-se igual a
/// um quadrado a 0°, e a banda pareceria começar e acabar no mesmo sítio.
const ROT_SPAN: f32 = 60.0;
/// O multiplicador de escala que os PONTOS carregam nas bandas do `Point Scale`. Eles nunca são
/// desenhados: este número existe só para o `point_scale` ter o que compor.
const POINT_MUL: f32 = 2.2;

/// A escada do param `pick` (a ordem do `ParamWidget::Enum` do nó).
const PICK_OFF: f32 = 0.0;
const PICK_CYCLE: f32 = 1.0;
const PICK_RANDOM: f32 = 2.0;
/// A semente da banda `Random` — fixa, para a cena abrir sempre igual, e **escolhida por
/// medição** (`which_seed_shows_every_shape`): cinco sorteios sobre três formas deixam uma de
/// fora com facilidade, e uma banda «Random» que só mostra DUAS cores ensina menos do que
/// promete. ⚠️ Das 24 primeiras sementes, **nove** deixam uma forma de fora. O gate
/// `the_random_band_shows_every_shape` defende esta escolha.
const PICK_SEED: f32 = 9.0;
/// A escada do param `transfer`.
const SHAPE_WINS: f32 = 0.0;
const POINT_WINS: f32 = 1.0;
const ADD: f32 = 2.0;
const MULTIPLY: f32 = 3.0;
/// A escada do param `mode` do `value.instance_field` — `1` é a RAMPA (`i/(N−1)`).
const FIELD_RAMP: f32 = 1.0;
/// O canal `Rotation` e o modo `Set` do `motion.drive`.
const CH_ROTATION: f32 = 2.0;
const DRIVE_SET: f32 = 1.0;

/// A cor neutra das formas que não são sobre cor.
const NEUTRAL: [f32; 3] = [0.72, 0.74, 0.78];
/// As três formas alternativas do `Pick` — bem separadas no matiz, senão o `Random` lê-se como
/// ruído em vez de como escolha.
const TRIO_RGB: [[f32; 3]; 3] = [[0.85, 0.35, 0.25], [0.35, 0.70, 0.40], [0.30, 0.45, 0.85]];
/// A cor da FORMA nas bandas do `Transfer`. ⚠️ Um cinzento a meio caminho, pela razão que o
/// `=98` já pagou: o branco é o NEUTRO do `Multiply`, e com ele aquela banda sairia igual à
/// vizinha.
const SHAPE_RGB: [f32; 3] = [0.55, 0.62, 0.45];

/// As três peças do COMETA: `(offset x, tamanho)`. Uma forma que se lê como UMA coisa, para a
/// banda 2 mostrar que é a coisa inteira que pousa em cada ponto.
const COMET: [(f32, f32); 3] = [(0.0, 0.20), (0.17, 0.13), (0.30, 0.08)];

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

fn node(g: &mut Graph, kind: &str, ps: &[(&str, f32)], ey: f32, x: f32) -> NodeId {
    let n = g.add_node(kind);
    g.set_pos(n, Pos { x, y: ey });
    for (k, v) in ps {
        g.set_param(n, *k, *v);
    }
    n
}

fn push(g: &mut Graph, head: NodeId, kind: &str, ps: &[(&str, f32)], ey: f32, x: f32) -> NodeId {
    let n = node(g, kind, ps, ey, x);
    let _ = wire(g, head, 0, n, 0);
    n
}

/// UMA peça: na origem (ou deslocada dela), do tamanho pedido e com a cor pedida.
fn piece(g: &mut Graph, ey: f32, rgb: [f32; 3], off: [f32; 2], size: f32) -> NodeId {
    let one = node(g, "motion.grid", &[("rows", 1.0), ("cols", 1.0)], ey, 80.0);
    let placed = if off == [0.0, 0.0] {
        one
    } else {
        push(
            g,
            one,
            "motion.transform",
            &[("offset_x", off[0]), ("offset_y", off[1])],
            ey,
            220.0,
        )
    };
    let sized = push(g, placed, "motion.scale", &[("amount", size)], ey, 360.0);
    push(
        g,
        sized,
        "motion.tint",
        &[("r", rgb[0]), ("g", rgb[1]), ("b", rgb[2])],
        ey,
        500.0,
    )
}

/// Junta até quatro peças numa forma só.
fn joined(g: &mut Graph, parts: &[NodeId], ey: f32) -> Option<NodeId> {
    let c = node(g, "motion.combine", &[], ey, 660.0);
    for (i, p) in parts.iter().enumerate() {
        wire(g, *p, 0, c, u16::try_from(i).ok()?)?;
    }
    Some(c)
}

/// A fila de `n` pontos, já posta no quadrante da banda.
///
/// ⚠️ **Sem `motion.scale`**, ao contrário da fila do `=98`: ali os pontos também eram
/// desenhados; aqui eles só dizem ONDE, e uma coluna `size` neles acenderia o `point_scale` em
/// bandas que não são sobre ele.
fn points_row(g: &mut Graph, at: [f32; 2], ey: f32, n: f32, gap: f32) -> NodeId {
    let grid = node(
        g,
        "motion.grid",
        &[("rows", 1.0), ("cols", n), ("gap_x", gap)],
        ey,
        80.0,
    );
    push(
        g,
        grid,
        "motion.transform",
        &[("offset_x", at[0]), ("offset_y", at[1])],
        ey,
        220.0,
    )
}

/// Fecha uma banda: o carimbo (com os params dela) e o sink.
fn finish(
    g: &mut Graph,
    shape: NodeId,
    points: NodeId,
    ey: f32,
    ps: &[(&str, f32)],
) -> Option<NodeId> {
    let dup = node(g, "motion.duplicator", ps, ey, 900.0);
    wire(g, shape, 0, dup, 0)?;
    wire(g, points, 0, dup, 1)?;
    let out = node(g, "motion.output", &[], ey, 1060.0);
    wire(g, dup, 0, out, 0)?;
    Some(out)
}

/// Uma banda que só carimba: a forma, a fila, e os params pedidos.
fn plain_band(
    g: &mut Graph,
    shape: NodeId,
    at: [f32; 2],
    ey: f32,
    n: f32,
    gap: f32,
    ps: &[(&str, f32)],
) -> Option<NodeId> {
    let pts = points_row(g, at, ey, n, gap);
    finish(g, shape, pts, ey, ps)
}

/// A banda do GIRO: cada ponto traz o `rot` dele, e o carimbo SOMA-O ao da forma.
fn turn_band(g: &mut Graph, shape: NodeId, at: [f32; 2], ey: f32) -> Option<NodeId> {
    let pts = points_row(g, at, ey, PIECES, GAP);
    let t = node(
        g,
        "value.instance_field",
        &[("mode", FIELD_RAMP)],
        ey + 120.0,
        400.0,
    );
    wire(g, pts, 0, t, 0)?;
    let turned = node(
        g,
        "motion.drive",
        &[
            ("channel", CH_ROTATION),
            ("mode", DRIVE_SET),
            ("scale", ROT_SPAN),
        ],
        ey,
        620.0,
    );
    wire(g, pts, 0, turned, 0)?;
    wire(g, t, 0, turned, 1)?;
    finish(g, shape, turned, ey, &[])
}

/// Uma banda do `Point Scale`: os pontos trazem uma escala PRÓPRIA, e `t` diz quanto dela entra.
fn scale_band(g: &mut Graph, shape: NodeId, at: [f32; 2], ey: f32, t: f32) -> Option<NodeId> {
    let pts = points_row(g, at, ey, PIECES, GAP);
    let sized = push(g, pts, "motion.scale", &[("amount", POINT_MUL)], ey, 620.0);
    finish(g, shape, sized, ey, &[("point_scale", t)])
}

/// Uma banda do `Transfer`: a rampa é autorada nos PONTOS e a forma tem cor própria, então a
/// coluna `tint` existe dos DOIS lados — que é a única situação em que este param decide algo.
fn transfer_band(g: &mut Graph, shape: NodeId, at: [f32; 2], ey: f32, mode: f32) -> Option<NodeId> {
    let pts = points_row(g, at, ey, PIECES, GAP);
    let t = node(
        g,
        "value.instance_field",
        &[("mode", FIELD_RAMP)],
        ey + 120.0,
        400.0,
    );
    wire(g, pts, 0, t, 0)?;
    let ramp = node(g, "motion.color_ramp", &[], ey, 620.0);
    wire(g, pts, 0, ramp, 0)?;
    wire(g, t, 0, ramp, 1)?;
    finish(g, shape, ramp, ey, &[("transfer", mode)])
}

/// `(x, y)` do centro da banda `k`, DERIVADO do [`LAYOUT`].
fn quadrant(k: usize) -> [f32; 2] {
    let mut i = k;
    for (r, n) in LAYOUT.iter().enumerate() {
        if i < *n {
            let x = (i as f32 - (*n as f32 - 1.0) * 0.5) * COL_STEP;
            return [x, ROW_Y[r]];
        }
        i -= *n;
    }
    [0.0, 0.0]
}

/// Monta a cena. Devolve um sink por banda.
pub(crate) fn build_dup_demo_document(
    doc: &mut MotionDoc,
    registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    // ⚠️ **AS FORMAS SÃO CONSTRUÍDAS UMA VEZ E PARTILHADAS** — as três bandas do `Pick` têm de
    // carimbar as MESMAS formas e as quatro do `Transfer` a mesma forma, senão a comparação
    // mede as entradas em vez do modo. Elas vivem ACIMA das bandas no grafo (`y` negativo),
    // para a coluna de cada banda continuar a ler-se de cima a baixo.
    let plain = piece(g, -560.0, NEUTRAL, [0.0, 0.0], PIECE);
    let comet_parts: Vec<NodeId> = COMET
        .iter()
        .enumerate()
        .map(|(i, (dx, size))| piece(g, -440.0 + i as f32 * 120.0, NEUTRAL, [*dx, 0.0], *size))
        .collect();
    let comet = joined(g, &comet_parts, -440.0)?;
    let trio_parts: Vec<NodeId> = TRIO_RGB
        .iter()
        .enumerate()
        .map(|(i, rgb)| {
            let dy = (i as f32 - 1.0) * SPREAD;
            piece(g, -80.0 + i as f32 * 120.0, *rgb, [0.0, dy], PIECE)
        })
        .collect();
    let trio = joined(g, &trio_parts, -80.0)?;
    let grey = piece(g, 280.0, SHAPE_RGB, [0.0, 0.0], PIECE);

    let mut sinks = Vec::with_capacity(BANDS);
    for k in 0..BANDS {
        let ey = 440.0 + k as f32 * 260.0;
        let at = quadrant(k);
        let sink = match k {
            0 => plain_band(g, plain, at, ey, PIECES, GAP, &[])?,
            1 => plain_band(g, comet, at, ey, COMET_POINTS, COMET_GAP, &[])?,
            2 => turn_band(g, plain, at, ey)?,
            3 => plain_band(g, trio, at, ey, PIECES, GAP, &[("pick", PICK_OFF)])?,
            4 => plain_band(g, trio, at, ey, PIECES, GAP, &[("pick", PICK_CYCLE)])?,
            5 => plain_band(
                g,
                trio,
                at,
                ey,
                PIECES,
                GAP,
                &[("pick", PICK_RANDOM), ("seed", PICK_SEED)],
            )?,
            6 => scale_band(g, plain, at, ey, 0.0)?,
            7 => scale_band(g, plain, at, ey, 0.5)?,
            8 => scale_band(g, plain, at, ey, 1.0)?,
            9 => transfer_band(g, grey, at, ey, SHAPE_WINS)?,
            10 => transfer_band(g, grey, at, ey, POINT_WINS)?,
            11 => transfer_band(g, grey, at, ey, ADD)?,
            _ => transfer_band(g, grey, at, ey, MULTIPLY)?,
        };
        sinks.push(sink);
    }
    g.validate(registry).ok()?;
    Some(sinks)
}

/// Os rótulos das bandas, na ordem em que a cena as monta. A ficha do canvas é o que está
/// ANTES do primeiro ` --`; a frase inteira sai no terminal.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "O carimbo -- uma forma, cinco pontos: uma copia em cada",
        "A forma INTEIRA -- as tres pecas do cometa pousam juntas, sem perder a forma",
        "O ponto da' o GIRO -- o `rot` do ponto SOMA-SE ao da forma, e a fila torce",
        "Pick = Off -- o PRODUTO: as tres formas em cada ponto, 15 copias",
        "Pick = Cycle -- UMA por ponto, revezando na ordem",
        "Pick = Random -- UMA por ponto, sorteada pela semente",
        "Point Scale = 0 -- a escala do PONTO e' deitada fora (o de sempre)",
        "Point Scale = 0,5 -- meio caminho entre a forma e o ponto",
        "Point Scale = 1 -- a escala do ponto inteira: as copias engordam",
        "Transfer = Shape Wins -- a cor autorada no ARRANJO some (o de sempre)",
        "Transfer = Point Wins -- a rampa do arranjo chega",
        "Transfer = Add -- as duas somadas: a rampa CLAREIA",
        "Transfer = Multiply -- a rampa TINGIDA pela cor da forma",
    ]
    .into_iter()
    .enumerate()
}

/// **As fichas desta cena, no canvas** — função PURA, medida pelo gate da legenda.
pub(crate) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    band_labels()
        .map(|(k, label)| {
            let at = quadrant(k);
            crate::motion_demo_legend::Caption::new([at[0], at[1] + 0.62], short_of(label))
        })
        .collect()
}

/// A ficha curta: o que está ANTES do primeiro ` --`.
fn short_of(label: &'static str) -> &'static str {
    match label.find(" --") {
        Some(i) => &label[..i],
        None => label,
    }
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_dup_tests.rs"]
mod tests;
