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
//! ## ⭐⭐⭐ O QUE ENTRA NA PORTA `shape` É UM `source.shape` — e a 1.ª versão punha um `motion.grid`
//!
//! Report do Enio (2026-09-06): *«você colocou grid entrando em Shape de Duplicator! Essa
//! aplicação é correta?»* — e não era.
//!
//! Um `motion.grid` de uma célula **funciona**: ele emite uma instância, o carimbo replica-a e o
//! sink desenha o ladrilho de omissão. ⛔ **Mas ensina o idioma errado.** O que uma forma É
//! neste app é um [`source.shape`] (geometria vectorial viva, nítida em qualquer zoom, 43
//! silhuetas) ou um `source.object` (uma sprite da cena) — e o doc do próprio `source.shape`
//! nomeia esta composição à letra: *«cross it with a `motion.grid` through a
//! `motion.duplicator` and the shape is stamped, crisp, at every point — with none of a baked
//! tile's dead pixels»*. Uma cena de demonstração que põe uma GRELHA onde vai uma FORMA ensina
//! o artista a fazer o que ela faz, e ele leva o erro para o trabalho dele.
//!
//! ⭐⭐ **E a troca melhorou a cena que ela veio corrigir:** as três alternativas do `Pick` deixam
//! de ser três quadrados de cores diferentes e passam a ser um **círculo, uma estrela e um
//! coração**. A pergunta do `Pick` é *«qual FORMA pousa em qual ponto»*, e agora a figura
//! responde-a — com o `geometry_id` a ser o oráculo dos gates, que é literalmente *qual forma*,
//! em vez da cor, que era um substituto.
//!
//! ⚠️ **`source.shape` lê um EXTERNAL que a SHELL publica** (`motion_shape_gen::publish`): num
//! cook virgem ele emite ZERO. É por isso que os gates desta cena cozinham através de um
//! `MotionState` — medir num `Cook` nu seria a sonda a acusar-se a si própria.
//!
//! ## ⭐⭐⭐ UMA CADEIA POR SAÍDA
//!
//! Report do Enio (2026-09-06): *«o cenário que você construiu tem tantos nós interligados que
//! não pude entender. Crie uma cadeia de nós por output.»*
//!
//! A 1.ª versão construía as formas **uma vez** e partilhava-as entre as bandas de cada fileira,
//! para a comparação medir o modo e não a entrada. A propriedade estava certa e o **preço era o
//! grafo**: um nó de forma alimentava quatro carimbos em quatro alturas diferentes, e os fios
//! atravessavam a tela inteira. *Um grafo que ninguém consegue seguir não ensina nada, por mais
//! correcta que seja a corrente que ele desenha.*
//!
//! ⇒ hoje **cada banda é uma cadeia FECHADA**: monta as próprias formas, os próprios pontos e o
//! próprio carimbo, e não toca em nó nenhum de outra banda. O grafo passa a ser treze ilhas
//! empilhadas, cada uma a ler-se da esquerda para a direita — e há gate a prová-lo
//! (`each_output_is_its_own_closed_chain`, que conta as **componentes ligadas**).
//!
//! ⚠️ **A propriedade que a partilha comprava NÃO se perdeu, mudou de dono:** as três bandas do
//! `Pick` carimbam as mesmas três formas porque saem da MESMA função (`trio_shape`), não porque
//! partilhem um nó — e o gate `the_three_pick_bands_stamp_the_same_shapes` mede-o na saída, que
//! é onde a afirmação vive. *Uma igualdade por construção é mais forte que uma por referência, e
//! não custa um fio a atravessar a tela.*
//!
//! ⚠️ **A banda 2 e a banda 4 são a MESMA lei vista de dois lados** — «uma forma feita de três
//! peças» e «três formas alternativas» são o mesmo stream para este nó; o que as separa é o
//! `Pick`, que trata cada ELEMENTO da forma como uma candidata.

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

/// O RAIO de uma forma e o passo entre pontos, em unidades de mundo.
///
/// ⚠️ **Raio, não lado:** a geometria de um `source.shape` vive em **raio 1** e o `size`
/// escala-a, então a meia-extensão de uma cópia **é** o `size` — não metade dele, como seria num
/// ladrilho. O gate que mede a sobreposição das bandas depende disto.
const PIECE: f32 = 0.11;
const GAP: f32 = 0.32;
/// Quantos pontos tem a fila de uma banda normal.
const PIECES: f32 = 5.0;
/// A banda do GRUPO usa menos pontos e mais espaço: a forma dela é larga.
const COMET_POINTS: f32 = 3.0;
const COMET_GAP: f32 = 0.62;
/// E as do `Point Scale` também: a última engorda as cópias `POINT_MUL` vezes, e cinco delas
/// encavalitavam-se.
const SCALE_POINTS: f32 = 3.0;
const SCALE_GAP: f32 = 0.62;

/// O desnível entre as três formas alternativas — é o que faz o `Cycle` e o `Random` lerem-se
/// como uma escolha, e o que transforma o produto cartesiano numa grelha de 3 × 5.
const SPREAD: f32 = 0.26;
/// Quanto o último ponto da fila gira, em graus. ⛔ **Não 90:** a leitura tem de ser monótona da
/// esquerda para a direita, e uma forma com simetria de um quarto de volta fecharia o ciclo.
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
/// fora com facilidade, e uma banda «Random» que só mostra DUAS formas ensina menos do que
/// promete. O gate `the_random_band_shows_every_shape` defende a escolha.
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

/// ⭐⭐ **AS TRÊS ALTERNATIVAS DO `PICK`, por SILHUETA** — os índices do `kind` do `source.shape`
/// (`Circle`, `Star`, `Heart`).
///
/// ⚠️ **Silhueta e não cor**, e a diferença é a própria pergunta do `Pick`: *«qual FORMA pousa em
/// qual ponto»*. Com três quadrados de cores diferentes o artista lê *«ele pinta»*.
const TRIO_KINDS: [f32; 3] = [0.0, 5.0, 6.0];
/// O `kind` das bandas que não são sobre a escolha: um CÍRCULO, que não tem orientação e por
/// isso não compete com a leitura das outras.
const PLAIN_KIND: f32 = 0.0;
/// O `kind` das bandas do `Transfer`: uma ESTRELA, que mostra a cor no miolo e nas pontas.
const TRANSFER_KIND: f32 = 5.0;
/// A forma que GIRA (banda 3) tem de ter uma ponta: um círculo a rodar não se vê rodar.
const TURN_KIND: f32 = 10.0; // ArrowRight

/// As três peças do COMETA: `(offset x, raio)`. Uma forma que se lê como UMA coisa, para a
/// banda 2 mostrar que é a coisa inteira que pousa em cada ponto.
const COMET: [(f32, f32); 3] = [(0.0, 0.11), (0.17, 0.072), (0.29, 0.047)];

/// A cor da FORMA nas bandas do `Transfer`. ⚠️ Um cinzento a meio caminho, pela razão que o
/// `=98` já pagou: o branco é o NEUTRO do `Multiply`, e com ele aquela banda sairia igual à
/// vizinha.
const SHAPE_RGB: [f32; 3] = [0.55, 0.62, 0.45];

/// A altura, no GRAFO, do espaço reservado a uma banda — é ela que faz cada cadeia ler-se como
/// uma ILHA e não como parte da vizinha.
const BAND_H: f32 = 640.0;
/// O passo entre duas sub-fileiras DENTRO de uma banda.
const SUB: f32 = 120.0;

/// A `i`-ésima sub-fileira do grafo dentro da banda que começa em `ey`.
fn row(ey: f32, i: usize) -> f32 {
    ey + i as f32 * SUB
}

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

/// UMA forma: geometria vectorial viva, do tamanho pedido, na origem ou deslocada dela.
///
/// ⚠️ **Uma forma nua é BRANCA e não precisa de mais nó nenhum** — o `size` é param dela, então
/// uma banda neutra é *um* nó em vez dos quatro que o ladrilho pedia.
fn shape_at(g: &mut Graph, ey: f32, kind: f32, off: [f32; 2], size: f32) -> NodeId {
    let s = node(
        g,
        "source.shape",
        &[
            (ph2d_node_motion_shape::param::KIND, kind),
            (ph2d_node_motion_shape::param::SIZE, size),
        ],
        ey,
        80.0,
    );
    if off == [0.0, 0.0] {
        s
    } else {
        push(
            g,
            s,
            "motion.transform",
            &[("offset_x", off[0]), ("offset_y", off[1])],
            ey,
            240.0,
        )
    }
}

/// Junta até quatro formas numa só.
fn joined(g: &mut Graph, parts: &[NodeId], ey: f32) -> Option<NodeId> {
    let c = node(g, "motion.combine", &[], ey, 480.0);
    for (i, p) in parts.iter().enumerate() {
        wire(g, *p, 0, c, u16::try_from(i).ok()?)?;
    }
    Some(c)
}

/// A fila de `n` pontos, já posta no quadrante da banda.
///
/// ⚠️ **Sem `motion.scale`**: os pontos só dizem ONDE, e uma coluna `size` neles acenderia o
/// `point_scale` em bandas que não são sobre ele (o `apply_point_scale` sai cedo sem a coluna).
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
        240.0,
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

/// O COMETA: três formas de tamanhos diferentes numa linha, que se lê como UMA coisa.
fn comet_shape(g: &mut Graph, ey: f32) -> Option<NodeId> {
    let parts: Vec<NodeId> = COMET
        .iter()
        .enumerate()
        .map(|(i, (dx, size))| shape_at(g, row(ey, i), PLAIN_KIND, [*dx, 0.0], *size))
        .collect();
    joined(g, &parts, row(ey, 0))
}

/// O TRIO: três formas alternativas, cada uma com a sua SILHUETA e a sua altura — é entre elas
/// que o `Pick` escolhe.
///
/// ⚠️ **As três bandas do `Pick` chamam esta função**, e é daí que vem a igualdade entre elas:
/// por CONSTRUÇÃO, sem que um nó de uma banda alcance a outra.
fn trio_shape(g: &mut Graph, ey: f32) -> Option<NodeId> {
    let parts: Vec<NodeId> = TRIO_KINDS
        .iter()
        .enumerate()
        .map(|(i, kind)| {
            shape_at(
                g,
                row(ey, i),
                *kind,
                [0.0, (i as f32 - 1.0) * SPREAD],
                PIECE,
            )
        })
        .collect();
    joined(g, &parts, row(ey, 0))
}

/// **A banda 1** — o carimbo puro: uma forma, cinco pontos.
fn stamp_band(g: &mut Graph, at: [f32; 2], ey: f32) -> Option<NodeId> {
    let shape = shape_at(g, row(ey, 0), PLAIN_KIND, [0.0, 0.0], PIECE);
    let pts = points_row(g, at, row(ey, 3), PIECES, GAP);
    finish(g, shape, pts, row(ey, 0), &[])
}

/// **A banda 2** — a forma INTEIRA pousa em cada ponto: o `P` do ponto SOMA-SE ao da forma, e é
/// por isso que o cometa chega sem se deformar.
fn whole_band(g: &mut Graph, at: [f32; 2], ey: f32) -> Option<NodeId> {
    let shape = comet_shape(g, ey)?;
    let pts = points_row(g, at, row(ey, 3), COMET_POINTS, COMET_GAP);
    finish(g, shape, pts, row(ey, 0), &[])
}

/// **A banda 3** — cada ponto traz o `rot` dele, e o carimbo SOMA-O ao da forma.
fn turn_band(g: &mut Graph, at: [f32; 2], ey: f32) -> Option<NodeId> {
    let shape = shape_at(g, row(ey, 0), TURN_KIND, [0.0, 0.0], PIECE);
    let pts = points_row(g, at, row(ey, 3), PIECES, GAP);
    let t = node(
        g,
        "value.instance_field",
        &[("mode", FIELD_RAMP)],
        row(ey, 4),
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
        row(ey, 3),
        620.0,
    );
    wire(g, pts, 0, turned, 0)?;
    wire(g, t, 0, turned, 1)?;
    finish(g, shape, turned, row(ey, 0), &[])
}

/// **As bandas 4-6** — o `Pick` escolhe qual das três formas pousa em qual ponto.
fn pick_band(g: &mut Graph, at: [f32; 2], ey: f32, ps: &[(&str, f32)]) -> Option<NodeId> {
    let shape = trio_shape(g, ey)?;
    let pts = points_row(g, at, row(ey, 3), PIECES, GAP);
    finish(g, shape, pts, row(ey, 0), ps)
}

/// **As bandas 7-9** — os pontos trazem uma escala PRÓPRIA, e `t` diz quanto dela entra.
fn scale_band(g: &mut Graph, at: [f32; 2], ey: f32, t: f32) -> Option<NodeId> {
    let shape = shape_at(g, row(ey, 0), PLAIN_KIND, [0.0, 0.0], PIECE);
    let pts = points_row(g, at, row(ey, 3), SCALE_POINTS, SCALE_GAP);
    let sized = push(
        g,
        pts,
        "motion.scale",
        &[("amount", POINT_MUL)],
        row(ey, 3),
        620.0,
    );
    finish(g, shape, sized, row(ey, 0), &[("point_scale", t)])
}

/// **As bandas 10-13** — a rampa é autorada nos PONTOS e a forma tem cor própria, então a coluna
/// `tint` existe dos DOIS lados, que é a única situação em que o `Transfer` decide algo.
fn transfer_band(g: &mut Graph, at: [f32; 2], ey: f32, mode: f32) -> Option<NodeId> {
    let forma = shape_at(g, row(ey, 0), TRANSFER_KIND, [0.0, 0.0], PIECE);
    let shape = push(
        g,
        forma,
        "motion.tint",
        &[
            ("r", SHAPE_RGB[0]),
            ("g", SHAPE_RGB[1]),
            ("b", SHAPE_RGB[2]),
        ],
        row(ey, 0),
        400.0,
    );
    let pts = points_row(g, at, row(ey, 3), PIECES, GAP);
    let t = node(
        g,
        "value.instance_field",
        &[("mode", FIELD_RAMP)],
        row(ey, 4),
        400.0,
    );
    wire(g, pts, 0, t, 0)?;
    let ramp = node(g, "motion.color_ramp", &[], row(ey, 3), 620.0);
    wire(g, pts, 0, ramp, 0)?;
    wire(g, t, 0, ramp, 1)?;
    finish(g, shape, ramp, row(ey, 0), &[("transfer", mode)])
}

/// `(x, y)` do centro da banda `k` no MUNDO, DERIVADO do [`LAYOUT`].
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
///
/// ⚠️ **Nenhuma linha deste laço partilha um nó com outra iteração** — é a lei da cena (report do
/// Enio, 2026-09-06), e o gate `each_output_is_its_own_closed_chain` conta as componentes ligadas
/// do grafo para a provar.
pub(crate) fn build_dup_demo_document(
    doc: &mut MotionDoc,
    registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(BANDS);
    for k in 0..BANDS {
        let ey = k as f32 * BAND_H;
        let at = quadrant(k);
        let sink = match k {
            0 => stamp_band(g, at, ey)?,
            1 => whole_band(g, at, ey)?,
            2 => turn_band(g, at, ey)?,
            3 => pick_band(g, at, ey, &[("pick", PICK_OFF)])?,
            4 => pick_band(g, at, ey, &[("pick", PICK_CYCLE)])?,
            5 => pick_band(g, at, ey, &[("pick", PICK_RANDOM), ("seed", PICK_SEED)])?,
            6 => scale_band(g, at, ey, 0.0)?,
            7 => scale_band(g, at, ey, 0.5)?,
            8 => scale_band(g, at, ey, 1.0)?,
            9 => transfer_band(g, at, ey, SHAPE_WINS)?,
            10 => transfer_band(g, at, ey, POINT_WINS)?,
            11 => transfer_band(g, at, ey, ADD)?,
            _ => transfer_band(g, at, ey, MULTIPLY)?,
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
///
/// ⚠️ **A ficha começa pelo NÚMERO da banda**, e não é enfeite: o anúncio do terminal lista as
/// treze por número, e sem ele o artista tem de contar blocos da esquerda para a direita e de
/// cima para baixo para casar uma coisa com a outra.
pub(crate) fn captions() -> Vec<crate::motion::motion_demo_legend::Caption> {
    band_labels()
        .map(|(k, label)| {
            let at = quadrant(k);
            crate::motion::motion_demo_legend::Caption::new(
                [at[0], at[1] + 0.62],
                format!("{} · {}", k + 1, short_of(label)),
            )
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

/// Os gates sobre como a cena está LIGADA — irmão por responsabilidade (HR-18).
#[cfg(test)]
#[path = "motion_state_conferencia_demos_dup_graph_tests.rs"]
mod graph_tests;
