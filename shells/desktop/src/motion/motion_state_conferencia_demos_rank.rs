//! **QUATRO EXEMPLOS, UM POR LINHA** — a cena `=73` (doc 89: folha 08 inteira, duas
//! células; folha 10 inteira, duas células).
//!
//! Cada linha é **o mesmo grafo duas vezes**: à esquerda como era, à direita com o
//! knob novo. Os dois blocos de uma linha têm o mesmo número de peças e a mesma
//! forma — só a COR diz o que mudou. A linha é rotulada **no canvas**.
//!
//! ```text
//!            ANTES              DEPOIS
//!   [ · · · · · ]   CORTE   [ · · · · · ]     a rampa alcança o fim?
//!   [ ▦ ]           BANDA   [ ▦ ]             que peças acendem?
//!   [ · · · · · ]   RAMPA   [ · · · · · ]     onde a rampa recomeça?
//!   [ ▦ ]           FORMA   [ ▦ ]             cheio ou contorno?
//! ```
//!
//! ## ⚠️ A LEI QUE ESTA CENA PAGOU: **posicionar é UPSTREAM da máscara**
//!
//! A primeira versão desta cena saiu ilegível (Enio, 2026-08-21: *"tudo misturado e
//! bagunçado"*), e a causa **não era nenhuma das features**: era o `motion.move` que
//! eu usava para pôr cada banda no seu quadrante. **Todo comportamento desta
//! biblioteca é mascarado pelo `falloff`** — é o contrato §1.2, e é a razão de
//! existirem os `field.*`. Posto DEPOIS do campo, o deslocamento de colocação virava
//! `dx · falloff_i`: cada peça andava uma distância diferente, e a banda esticava-se
//! por cima das vizinhas.
//!
//! Medido pela sonda `measure_scene_layout` (o instrumento que nasceu deste smoke):
//! uma fileira de 16 peças a `gap = 0.5` — 7,5 de largura por construção — saiu com
//! **1,50**, porque a peça `i` andou `−6 · i/15` e o passo colapsou de `0,50` para
//! `0,10`. A banda do par 2 saiu com **8,94** de largura para uma grelha de 2,94.
//!
//! ⇒ **[`place`] corre imediatamente a seguir à fonte**, antes de existir coluna
//! `falloff` nenhuma. Nesse ponto a máscara ausente lê `1.0` e o deslocamento é
//! rígido. O gate `no_band_leaves_its_slot` mede a caixa de cada banda contra a
//! prevista e reprova se ela transbordar.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O centro de cada coluna. A da esquerda é sempre *como era*.
pub(crate) const COL_X: f32 = 2.6;
/// O centro de cada linha, de cima para baixo.
pub(crate) const ROW_Y: [f32; 4] = [4.4, 1.4, -1.6, -4.7];
/// Onde os dois rótulos de coluna assentam.
const HEADER_Y: f32 = 6.2;
/// O tamanho da letra dos rótulos.
const LABEL_SIZE: f32 = 0.42;

/// **A grelha de cada linha: `(colunas, linhas, passo, tamanho da peça)`.**
///
/// ⚠️ A CAIXA que a banda ocupa é **derivada** disto (`(n − 1) · passo`), nunca
/// escrita ao lado — ver [`footprint`]. Foi por não haver essa derivação que a versão
/// anterior desta cena pôs quatro bandas umas por cima das outras.
pub(crate) const BANDS: [(f32, f32, f32, f32); 4] = [
    (12.0, 1.0, 0.26, 0.17),
    (8.0, 8.0, 0.28, 0.14),
    (16.0, 1.0, 0.19, 0.13),
    (11.0, 11.0, 0.24, 0.11),
];

/// A largura e a altura que a banda da linha `k` **de facto** ocupa depois de cozida.
///
/// ⚠️ **A linha 1 é a excepção, e ela é derivada e não escrita:** o corte guarda o
/// PREFIXO (`round(KEEP · cols)` peças), então o que fica na tela é a metade, não a
/// grelha. Uma caixa prevista com a largura da grelha inteira faria o gate de layout
/// aceitar um bloco descentrado — que é o que se via antes de [`x_bias`] existir.
#[must_use]
pub(crate) fn footprint(k: usize) -> (f32, f32) {
    let (_, rows, gap, _) = BANDS[k];
    ((visible_cols(k) - 1.0) * gap, (rows - 1.0) * gap)
}

/// Quantas colunas sobrevivem na linha `k` — todas, menos na do corte.
fn visible_cols(k: usize) -> f32 {
    let cols = BANDS[k].0;
    if k == 0 { (cols * KEEP).round() } else { cols }
}

/// O quanto a colocação da linha `k` compensa o corte.
///
/// Uma grelha nasce centrada na origem; o prefixo que o corte guarda ocupa a METADE
/// ESQUERDA dela. Sem esta compensação a fileira sobrevivente encostaria na margem
/// esquerda do quadrante enquanto as outras três linhas ficam centradas — e uma
/// coluna que não se alinha lê-se como desalinho, não como *"aqui foi cortado"*.
fn x_bias(k: usize) -> f32 {
    let (cols, _, gap, _) = BANDS[k];
    // A meia-largura da grelha INTEIRA menos a da parte que sobrevive — derivado de
    // [`footprint`], que é a mesma conta que o gate de layout usa.
    (cols - 1.0) * gap * 0.5 - footprint(k).0 * 0.5
}

/// A cor de REPOUSO — o cinza-chumbo de quem não foi tocado.
///
/// ⚠️ **Ela existe porque o branco não servia.** O default do `motion.tint` é branco
/// opaco, então na versão anterior as peças NÃO acendidas eram a coisa mais clara da
/// tela e o padrão sumia dentro delas. A leitura de uma máscara é *escuro → aceso*.
const REST: [f32; 3] = [0.24, 0.25, 0.30];
/// A cor de quem ACENDE, por linha.
const LIT: [[f32; 3]; 4] = [
    [1.0, 0.85, 0.30],
    [0.35, 0.75, 1.00],
    [1.0, 0.45, 0.80],
    [0.45, 1.00, 0.60],
];
/// Os rótulos, cinza médio: eles orientam, não competem.
const LABEL_RGB: [f32; 3] = [0.62, 0.64, 0.70];

// ── Par 1 · CORTE ────────────────────────────────────────────────────────────
/// A fracção que o corte mantém — metade, para que a contagem mentida seja o DOBRO
/// da verdadeira e a rampa pare exactamente a meio.
pub(crate) const KEEP: f32 = 0.5;
/// As duas pontas do degradê da linha 1. ⚠️ A leitura é de **brilho**, não de matiz:
/// a fileira acende da esquerda para a direita, e a pergunta é *"ela chega ao fim?"*.
const RAMP_START: [f32; 3] = [0.20, 0.10, 0.06];
const RAMP_END: [f32; 3] = LIT[0];

// ── Par 2 · BANDA ────────────────────────────────────────────────────────────
const BAND_LO: f32 = 0.4;
const BAND_HI: f32 = 0.6;
const BAND_SOFT: f32 = 0.05;
/// `Order By = Attribute` (o valor do enum).
pub(crate) const ORDER_BY_ATTRIBUTE: f32 = 1.0;
/// A frequência do campo que serve de atributo — escolhida contra o passo da grelha,
/// para que peças vizinhas caiam em postos distantes (gate abaixo).
pub(crate) const ATTR_FREQ: f32 = 4.5;

// ── Par 3 · RAMPA ────────────────────────────────────────────────────────────
/// O contorno `Curve` do `field.remap` (o valor do enum).
pub(crate) const CONTOUR_CURVE: f32 = 4.0;
/// O deslocamento que a direita autora — pouco mais de um terço, para que a costura
/// caia bem dentro da fileira em vez de na ponta.
pub(crate) const CURVE_SHIFT: f32 = 0.35;
/// O modo `Ramp` do `value.instance_field`: o índice normalizado, `0..1`.
pub(crate) const FIELD_RAMP: f32 = 1.0;
/// O canal **Falloff** do `motion.drive`, e o modo **Set**.
pub(crate) const DRIVE_FALLOFF: f32 = 5.0;
pub(crate) const DRIVE_SET: f32 = 1.0;

// ── Par 4 · FORMA ────────────────────────────────────────────────────────────
pub(crate) const SHAPE_SIDES: f32 = 5.0;
pub(crate) const SHAPE_RADIUS: f32 = 0.72;
pub(crate) const SHAPE_DISTANCE: f32 = 0.20;

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

/// **PÔR A BANDA NO QUADRANTE DELA — e isto corre ANTES de qualquer campo.**
///
/// Ver o aviso no topo do módulo: um `motion.transform` (como o `motion.move`, como
/// todo comportamento desta biblioteca) multiplica o deslocamento pelo `falloff`.
/// Chamado aqui, logo a seguir à fonte, não existe coluna `falloff` — a máscara
/// ausente lê `1.0` e a colocação é rígida. ⛔ **Não mova esta chamada para o fim da
/// cadeia**: é literalmente o bug que deixou a cena ilegível.
fn place(g: &mut Graph, head: NodeId, at: [f32; 2], ey: f32) -> NodeId {
    push(
        g,
        head,
        "motion.transform",
        &[("offset_x", at[0]), ("offset_y", at[1])],
        ey,
        200.0,
    )
}

/// A semente de uma linha: a grelha, já posicionada e já pintada de repouso.
///
/// A ordem é **grelha → colocação → cor de repouso → (o campo) → cor acesa**, e ela é
/// a lei desta cena: tudo o que é LAYOUT acontece antes de existir máscara; tudo o que
/// a máscara decide acontece depois.
fn seed(g: &mut Graph, k: usize, right: bool, rest: bool) -> NodeId {
    let (cols, rows, gap, piece) = BANDS[k];
    let ey = (k * 2 + usize::from(right)) as f32 * 240.0;
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
    let scaled = push(g, n, "motion.scale", &[("amount", piece)], ey, 110.0);
    let cx = if right { COL_X } else { -COL_X };
    let placed = place(g, scaled, [cx + x_bias(k), ROW_Y[k]], ey);
    if !rest {
        return placed;
    }
    push(
        g,
        placed,
        "motion.tint",
        &[("r", REST[0]), ("g", REST[1]), ("b", REST[2])],
        ey,
        300.0,
    )
}

/// Liga a cauda de uma banda a uma saída nova.
fn out_of(g: &mut Graph, tail: NodeId, ey: f32) -> Option<NodeId> {
    let out = node(g, "motion.output", &[], ey, 1000.0);
    wire(g, tail, 0, out, 0)?;
    Some(out)
}

/// A cor ACESA, mascarada pelo campo que vem antes dela.
fn lit(g: &mut Graph, head: NodeId, k: usize, ey: f32) -> NodeId {
    push(
        g,
        head,
        "motion.tint",
        &[("r", LIT[k][0]), ("g", LIT[k][1]), ("b", LIT[k][2])],
        ey,
        820.0,
    )
}

/// Uma palavra no canvas. ⚠️ Ela **não** entra nos sinks das bandas: os gates de
/// layout medem bandas, e uma legenda não é uma banda.
fn label(g: &mut Graph, word: &str, at: [f32; 2], ey: f32) -> Option<NodeId> {
    let t = g.add_node("source.text");
    g.set_pos(t, Pos { x: 0.0, y: ey });
    g.set_text_param(t, ph2d_node_source_text::TEXT_KEY, word);
    g.set_param(t, ph2d_node_source_text::param::SIZE, LABEL_SIZE);
    // Centrado: o rótulo de linha vive no vão ENTRE as duas colunas, e um texto
    // alinhado à esquerda encostaria numa delas.
    g.set_param(t, ph2d_node_source_text::param::ALIGN, 1.0);
    let placed = place(g, t, at, ey);
    let tinted = push(
        g,
        placed,
        "motion.tint",
        &[
            ("r", LABEL_RGB[0]),
            ("g", LABEL_RGB[1]),
            ("b", LABEL_RGB[2]),
        ],
        ey,
        300.0,
    );
    out_of(g, tinted, ey)
}

/// **LINHA 1 · CORTE** — a renumeração do `motion.cull`. As duas metades cortam
/// igual; o que difere é a contagem que a lista ANUNCIA ao degradê.
fn cull_band(g: &mut Graph, right: bool) -> Option<NodeId> {
    let ey = f32::from(u8::from(right)) * 240.0;
    let base = seed(g, 0, right, false);
    let cu = push(
        g,
        base,
        "motion.cull",
        &[
            ("mode", 0.0), // Fraction
            ("amount", KEEP),
            ("reindex", f32::from(u8::from(right))),
        ],
        ey,
        420.0,
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
        620.0,
    );
    out_of(g, tint, ey)
}

/// **LINHA 2 · BANDA** — o posto por atributo. À direita a banda segue o VALOR de um
/// campo, e o conjunto fica exactamente onde estava.
fn rank_band(g: &mut Graph, right: bool) -> Option<NodeId> {
    let ey = (2 + usize::from(right)) as f32 * 240.0;
    let base = seed(g, 1, right, true);
    let ir = node(
        g,
        "field.index_range",
        &[
            ("start", BAND_LO),
            ("end", BAND_HI),
            ("soft", BAND_SOFT),
            ("curve", 0.0), // Linear
            ("key", if right { ORDER_BY_ATTRIBUTE } else { 0.0 }),
        ],
        ey,
        620.0,
    );
    wire(g, base, 0, ir, 0)?;
    if right {
        let noise = node(
            g,
            "value.noise",
            &[("frequency", ATTR_FREQ), ("amplitude", 1.0), ("speed", 0.0)],
            ey + 140.0,
            440.0,
        );
        wire(g, base, 0, noise, 0)?;
        wire(g, noise, 0, ir, 1)?;
    }
    let tint = lit(g, ir, 1, ey);
    out_of(g, tint, ey)
}

/// **LINHA 3 · RAMPA** — o deslocamento da curva. A máscara é a rampa `0..1` do
/// índice; o `field.remap` no contorno `Curve` (sem curva autorada = a identidade)
/// devolve-a tal e qual, e o deslocamento fá-la desfilar.
fn shift_band(g: &mut Graph, right: bool) -> Option<NodeId> {
    let ey = (4 + usize::from(right)) as f32 * 240.0;
    let base = seed(g, 2, right, true);
    let ramp = node(
        g,
        "value.instance_field",
        &[("mode", FIELD_RAMP)],
        ey + 140.0,
        420.0,
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
        560.0,
    );
    wire(g, base, 0, drv, 0)?;
    wire(g, ramp, 0, drv, 1)?;
    let rm = push(
        g,
        drv,
        "field.remap",
        &[
            ("contour", CONTOUR_CURVE),
            ("curve_offset", if right { CURVE_SHIFT } else { 0.0 }),
        ],
        ey,
        700.0,
    );
    let tint = lit(g, rm, 2, ey);
    out_of(g, tint, ey)
}

/// **LINHA 4 · FORMA** — o nó NOVO: uma geometria como campo. O mesmo pentágono dos
/// dois lados; só o *Path Mode* muda.
fn shape_band(g: &mut Graph, right: bool) -> Option<NodeId> {
    let ey = (6 + usize::from(right)) as f32 * 240.0;
    let base = seed(g, 3, right, true);
    // A FORMA, e ela é posicionada pelo MESMO `place` da banda — senão o pentágono
    // ficaria na origem enquanto a grelha dele está no quadrante.
    let ring = node(
        g,
        "motion.distribute_radial",
        &[
            ("count", SHAPE_SIDES),
            ("rings", 1.0),
            ("radius", SHAPE_RADIUS),
            ("inner", 0.0),
        ],
        ey + 140.0,
        0.0,
    );
    let pent = place(
        g,
        ring,
        [if right { COL_X } else { -COL_X }, ROW_Y[3]],
        ey + 140.0,
    );
    let fs = node(
        g,
        "field.shape",
        &[
            ("mode", f32::from(u8::from(right))),
            ("distance", SHAPE_DISTANCE),
            ("curve", 2.0), // Smooth
        ],
        ey,
        620.0,
    );
    wire(g, base, 0, fs, 0)?;
    wire(g, pent, 0, fs, 1)?;
    let tint = lit(g, fs, 3, ey);
    out_of(g, tint, ey)
}

/// Monta a cena. Devolve os oito sinks das BANDAS, em pares (as legendas têm sinks
/// próprios, fora desta lista — ver [`label`]).
pub(crate) fn build_rank_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(8);
    for right in [false, true] {
        sinks.push((0usize, cull_band(g, right)?));
    }
    for right in [false, true] {
        sinks.push((1usize, rank_band(g, right)?));
    }
    for right in [false, true] {
        sinks.push((2usize, shift_band(g, right)?));
    }
    for right in [false, true] {
        sinks.push((3usize, shape_band(g, right)?));
    }
    // As legendas, por último: elas não são bandas.
    label(g, "ANTES", [-COL_X, HEADER_Y], 2000.0)?;
    label(g, "DEPOIS", [COL_X, HEADER_Y], 2140.0)?;
    for (k, word) in ROW_LABELS.iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "quatro linhas")]
        let ey = 2280.0 + k as f32 * 140.0;
        label(g, word, [0.0, ROW_Y[k]], ey)?;
    }
    // Ordenados por LINHA e depois por lado, que é a ordem em que se lê a tela.
    sinks.sort_by_key(|(k, _)| *k);
    Some(sinks.into_iter().map(|(_, s)| s).collect())
}

/// O nome de cada linha, pintado no vão entre as duas colunas.
pub(crate) const ROW_LABELS: [&str; 4] = ["CORTE", "BANDA", "RAMPA", "FORMA"];

/// Os números que a mensagem do smoke cita, para ela não os repetir à mão.
pub(crate) fn authored() -> (f32, f32) {
    (CURVE_SHIFT, SHAPE_DISTANCE)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_rank_tests.rs"]
mod tests;
