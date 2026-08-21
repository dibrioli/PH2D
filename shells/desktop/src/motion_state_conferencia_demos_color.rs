//! **A FAMÍLIA DA COR (+ o pareamento do `motion.step`)** — a cena `=72`
//! (doc 89: folha 09 inteira, três células; folha 07, a célula do pareamento).
//!
//! Três pares. O mesmo grafo dos dois lados; só o número novo muda.
//!
//! | par | esquerda | direita |
//! |---|---|---|
//! | `motion.tint` | `Blend = Mix` — a cor SUBSTITUI o que estava lá | **`Blend = Multiply`** — ela MODULA, e o degradê por baixo sobrevive |
//! | `motion.color_array` | o `Offset` desligado — listras regulares por índice | **um CAMPO no `Offset`** — cada peça escolhe a fatia dela |
//! | `motion.step` | um batimento POR PEÇA — a fileira sobe em bloco | **um batimento GLOBAL (uma linha só)** — e tem de subir em bloco na mesma |
//!
//! ⚠️ **O par 3 é o único cujos dois lados têm de ficar IGUAIS, e é essa a
//! leitura.** Antes desta wave o batimento global chegava **apenas ao elemento
//! 0**: a peça da esquerda subia e as outras cinco ficavam paradas para sempre.
//! O modo de falha a nomear no smoke não é *"as duas fileiras diferem"* — é *"só
//! a primeira peça da fileira de baixo anda"*.
//!
//! ⚠️ **A quarta célula da folha 09 não tem lado nenhum: é PARIDADE.** O
//! `motion.color_array` ganhou kernel de GPU (era o único dos quatro nós de cor
//! sem um, e um grafo que o usasse perdia a aceleração inteira). O que prova isso
//! numa cena é a AUSÊNCIA de diferença: rodar de novo com `PH2D_GPU_COOK=0` tem
//! de dar a mesma imagem. Está escrito na mensagem do smoke.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O vão entre as duas colunas e entre as três linhas.
const GAP_X: f32 = 6.0;
const GAP_Y: f32 = 5.2;

/// A cor que o par 1 aplica por cima do degradê — um laranja quente, para que a
/// diferença entre substituir e modular seja de MATIZ e de BRILHO ao mesmo tempo.
const WARM: [f32; 3] = [1.0, 0.55, 0.15];

/// O modo `Gradient` do `motion.tint` (o valor do enum, não um índice de linha).
const TINT_GRADIENT: f32 = 1.0;
/// Os dois modos de `blend` que o par 1 encena.
const BLEND_MIX: f32 = 0.0;
const BLEND_MULTIPLY: f32 = 3.0;

/// A paleta que o par 2 cicla — quatro cores bem separadas em matiz, para que uma
/// fatia trocada seja visível a um metro da tela.
fn palette_text() -> String {
    ph2d_color::serialize_palette(&[
        [0.95, 0.25, 0.30, 1.0],
        [0.30, 0.85, 0.45, 1.0],
        [0.35, 0.55, 1.00, 1.0],
        [1.00, 0.85, 0.25, 1.0],
    ])
}

/// A frequência do campo que dirige o `Offset` do par 2.
///
/// ⚠️ **Ela é escolhida contra o PASSO DA GRELHA, não à toa:** com `gap = 0.42` e
/// esta frequência, peças vizinhas caem a ~1,3 de distância no espaço do ruído, o
/// que as descorrelaciona. Uma frequência baixa daria manchas suaves — bonitas, e
/// indistinguíveis de um deslocamento global, que é exactamente a lei ANTIGA.
const FIELD_FREQ: f32 = 3.0;
/// A excursão do campo, em fatias de paleta: `±2` cobre o ciclo inteiro de quatro.
const FIELD_AMP: f32 = 2.0;

/// O período do metrônomo do par 3, em segundos — um compasso que se conta
/// olhando.
const BEAT: f32 = 0.5;
/// Quanto cada degrau levanta a peça, e quantos degraus a escada tem.
const STEP_RISE: f32 = 0.5;
const STEP_COUNT: f32 = 8.0;
/// `Zigzag` — a escada sobe e volta, em vez de saltar para casa. Num smoke que se
/// olha por dez segundos, um triângulo lê e um dente de serra pisca.
const STEP_ZIGZAG: f32 = 2.0;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// Um `pre` self-loop — a memória que a família de pulso e o `motion.step`
/// carregam. O editor plumba-o ao SOLTAR o nó; um documento montado à mão
/// escreve-o.
fn wire_pre(g: &mut Graph, n: NodeId, port: u16) -> Option<()> {
    g.connect(Edge {
        from: (n, 0),
        to: (n, port),
        delayed: true,
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

/// Uma grelha `rows × cols` de peças pequenas, já escalada.
fn grid(g: &mut Graph, rows: f32, cols: f32, gap: f32, piece: f32, ey: f32) -> NodeId {
    let n = g.add_node("motion.grid");
    g.set_pos(n, Pos { x: 0.0, y: ey });
    g.set_param(n, "rows", rows);
    g.set_param(n, "cols", cols);
    g.set_param(n, "gap_x", gap);
    g.set_param(n, "gap_y", gap);
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
        860.0,
    );
    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: 1000.0, y: ey });
    wire(g, placed, 0, out, 0)?;
    Some(out)
}

/// **PAR 1** — o `Blend` do `motion.tint`. Por baixo, um degradê branco→preto que
/// o `Mix` apaga e o `Multiply` preserva.
fn blend_band(g: &mut Graph, ey: f32, at: [f32; 2], blend: f32) -> Option<NodeId> {
    let base = grid(g, 6.0, 6.0, 0.5, 0.17, ey);
    // A cor DE BAIXO: o degradê que existe para ser (ou não) apagado.
    let under = push(
        g,
        base,
        "motion.tint",
        &[("mode", TINT_GRADIENT)],
        ey,
        320.0,
    );
    // A cor DE CIMA, e o único número que difere entre os dois lados.
    let over = push(
        g,
        under,
        "motion.tint",
        &[
            ("r", WARM[0]),
            ("g", WARM[1]),
            ("b", WARM[2]),
            ("blend", blend),
        ],
        ey,
        520.0,
    );
    finish(g, over, at, ey)
}

/// **PAR 2** — o `Offset` do `motion.color_array`. À direita ele vem de um CAMPO,
/// e é o campo que a lei antiga descartava (ela lia `.first()`).
fn array_band(g: &mut Graph, ey: f32, at: [f32; 2], field: bool) -> Option<NodeId> {
    let base = grid(g, 8.0, 8.0, 0.42, 0.15, ey);
    let ca = g.add_node("motion.color_array");
    g.set_pos(ca, Pos { x: 520.0, y: ey });
    g.set_text_param(ca, "palette", palette_text());
    wire(g, base, 0, ca, 0)?;
    if field {
        let noise = g.add_node("value.noise");
        g.set_pos(
            noise,
            Pos {
                x: 320.0,
                y: ey + 140.0,
            },
        );
        g.set_param(noise, "frequency", FIELD_FREQ);
        g.set_param(noise, "amplitude", FIELD_AMP);
        // ⚠️ PARADO de propósito: o que esta banda tem de mostrar é que cada PEÇA
        // tem o seu índice, e um campo a tremeluzir esconde exactamente isso
        // debaixo do movimento.
        g.set_param(noise, "speed", 0.0);
        wire(g, base, 0, noise, 0)?;
        wire(g, noise, 0, ca, 1)?;
    }
    finish(g, ca, at, ey)
}

/// **PAR 3** — o batimento do `motion.step`. À direita ele é GLOBAL (uma linha
/// só), e os dois lados têm de se comportar igual.
fn step_band(g: &mut Graph, ey: f32, at: [f32; 2], global: bool) -> Option<NodeId> {
    let base = grid(g, 1.0, 6.0, 0.7, 0.24, ey);
    // De onde o metrônomo conta as suas linhas: as SEIS peças, ou UMA só.
    let clock_src = if global {
        let one = g.add_node("motion.grid");
        g.set_pos(
            one,
            Pos {
                x: 160.0,
                y: ey + 140.0,
            },
        );
        g.set_param(one, "rows", 1.0);
        g.set_param(one, "cols", 1.0);
        one
    } else {
        base
    };
    let beat = g.add_node("pulse.beat");
    g.set_pos(
        beat,
        Pos {
            x: 340.0,
            y: ey + 140.0,
        },
    );
    g.set_param(beat, "period", BEAT);
    wire(g, clock_src, 0, beat, 0)?;
    wire_pre(g, beat, 1)?;

    let st = g.add_node("motion.step");
    g.set_pos(st, Pos { x: 520.0, y: ey });
    g.set_param(st, "channel", 1.0); // Y
    g.set_param(st, "step", STEP_RISE);
    g.set_param(st, "count_max", STEP_COUNT);
    g.set_param(st, "mode", STEP_ZIGZAG);
    wire(g, base, 0, st, 0)?;
    wire(g, beat, 0, st, 1)?;
    // A escada é ESTADO: sem este `pre` ela nunca passa do primeiro degrau.
    wire_pre(g, st, 2)?;

    let tint = push(
        g,
        st,
        "motion.tint",
        &[("r", 0.55), ("g", 0.95), ("b", 0.7)],
        ey,
        700.0,
    );
    finish(g, tint, at, ey)
}

/// Monta a cena. Devolve os seis sinks, em pares.
pub(crate) fn build_color_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(6);
    for (row, right) in (0..3).flat_map(|r| [(r, false), (r, true)]) {
        let ey = (row * 2 + usize::from(right)) as f32 * 240.0;
        let at = [
            if right { GAP_X } else { -GAP_X },
            GAP_Y - row as f32 * GAP_Y,
        ];
        let sink = match row {
            0 => blend_band(g, ey, at, if right { BLEND_MULTIPLY } else { BLEND_MIX })?,
            1 => array_band(g, ey, at, right)?,
            _ => step_band(g, ey, at, right)?,
        };
        sinks.push(sink);
    }
    Some(sinks)
}

/// Os rótulos das seis bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "TINT Blend = Mix -- o laranja SUBSTITUI, e o degrade' por baixo some",
        "TINT Blend = Multiply -- o laranja MODULA, e o degrade' sobrevive nele",
        "PALETA com o Offset desligado -- listras regulares, uma fatia por indice",
        "PALETA com um CAMPO no Offset -- cada peca escolhe a fatia dela",
        "ESCADA com um batimento POR PECA -- as seis sobem em bloco",
        "ESCADA com um batimento GLOBAL -- e tem de subir em bloco na mesma",
    ]
    .into_iter()
    .enumerate()
}

/// Os números que a mensagem do smoke cita, para ela não os repetir à mão.
pub(crate) fn authored() -> (f32, f32) {
    (BEAT, STEP_COUNT)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_color_tests.rs"]
mod tests;
