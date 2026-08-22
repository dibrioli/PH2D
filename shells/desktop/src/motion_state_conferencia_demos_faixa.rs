//! **A FAIXA QUE O NOME PROMETE** (`PH2D_GPU_COOK_DEMO=79`) — a cena do grupo de
//! 2026-08-22 (doc 89, folha 06): três animadores que passam a dizer **onde a
//! saída cai**, e um defeito que essa entrega curou.
//!
//! ## A cena desenha a faixa PEDIDA, e pergunta se o movimento a alcança
//!
//! Cada fileira é uma linha de [`COLS`] peças cuja **posição Y é a saída do nó** —
//! o idioma da `=41` e da `=78`. O que esta cena acrescenta são **duas marcas** à
//! esquerda de cada fileira, exactamente em `min` e em `max`: a faixa que o painel
//! diz. A leitura é uma pergunta só — *o movimento encosta nas duas?*
//!
//! ⚠️ **A régua torna o defeito VISÍVEL, e é a razão de ela existir.** Sem as
//! marcas, uma fileira que use só a metade de cima da faixa desenha um movimento
//! perfeitamente plausível: ela oscila, tem forma, e nada na tela diz que ela
//! deveria descer mais. *Um erro de metade de faixa não parece um erro — parece
//! uma escolha.*
//!
//! ## Os quatro pares
//!
//! Os três primeiros são o MESMO pedido (`[min, max]`) escrito de duas maneiras: à
//! esquerda pela aritmética que o artista faz de cabeça (`amplitude = (max−min)/2`,
//! `offset = (min+max)/2`), à direita pela régua `Min / Max`. Nas formas
//! **bipolares** as duas coincidem — e o par existe precisamente para mostrar isso,
//! senão a cena estaria a acusar o artista de um erro que ele não comete. Nas
//! **unipolares** (o `Spike` do oscilador, o `Ridged` do ruído) a conta de cabeça
//! levanta o piso ao **centro** da faixa, e só a régua nova encosta nas duas marcas.
//!
//! O quarto par é outro assunto — os modos apendados ao `motion.drive` —, e está
//! aqui porque é o resto do mesmo grupo.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// Quantas peças por fileira — a resolução do gráfico.
pub(crate) const COLS: f32 = 26.0;
/// O passo horizontal entre peças.
const GAP_X: f32 = 0.28;
/// A distância vertical entre fileiras.
const ROW_GAP: f32 = 1.15;
/// Quantas fileiras a cena empilha.
const ROWS: usize = 8;

/// A faixa que TODAS as fileiras de faixa pedem — um número só, e é ele que as
/// marcas desenham.
///
/// ⚠️ **Assimétrico de propósito** (`[−0,18 .. +0,42]`, centro em `+0,12`): uma
/// faixa centrada no zero esconderia metade do defeito, porque um piso levantado
/// ao centro cairia exactamente em `0` e leria como *"a onda está no sítio"*.
pub(crate) const BAND_MIN: f32 = -0.18;
pub(crate) const BAND_MAX: f32 = 0.42;

/// Onde as marcas da régua ficam, à esquerda da fileira.
const TICK_X: f32 = -0.45;
/// O tamanho das peças da fileira e o das marcas — as marcas são MAIORES, senão
/// perdem-se no meio do movimento.
const DOT: f32 = 0.26;
const TICK: f32 = 0.42;

/// O valor constante que o par do `motion.drive` injecta no canal.
const DRIVE_VALUE: f32 = 0.2;

/// Que fileira é esta.
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    /// `motion.oscillator` numa forma dada, com a régua escolhida.
    Osc { wave: i32, by_range: bool },
    /// `motion.noise` num tipo dado, com a régua escolhida.
    Noise { ty: i32, by_range: bool },
    /// `motion.stagger` seguido de um `motion.drive` no modo dado.
    Drive { mode: f32 },
}

struct Row {
    label: &'static str,
    kind: Kind,
    /// Se esta fileira desenha as marcas da faixa. O par do `drive` não pede faixa
    /// nenhuma — pôr-lhe marcas seria uma régua a medir outra coisa.
    ticks: bool,
}

static ROWS_TABLE: &[Row] = &[
    Row {
        label: "onda BIPOLAR (Sine), conta de cabeca -- ela ACERTA",
        kind: Kind::Osc {
            wave: 0,
            by_range: false,
        },
        ticks: true,
    },
    Row {
        label: "onda BIPOLAR (Sine), regua Min/Max -- igual a de cima",
        kind: Kind::Osc {
            wave: 0,
            by_range: true,
        },
        ticks: true,
    },
    Row {
        label: "onda UNIPOLAR (Spike), conta de cabeca -- so' a METADE de cima",
        kind: Kind::Osc {
            wave: 4,
            by_range: false,
        },
        ticks: true,
    },
    Row {
        label: "onda UNIPOLAR (Spike), regua Min/Max -- encosta nas DUAS marcas",
        kind: Kind::Osc {
            wave: 4,
            by_range: true,
        },
        ticks: true,
    },
    Row {
        label: "ruido RETIFICADO (Ridged), conta de cabeca -- piso levantado",
        kind: Kind::Noise {
            ty: 2,
            by_range: false,
        },
        ticks: true,
    },
    Row {
        label: "ruido RETIFICADO (Ridged), regua Min/Max -- desce ate' a marca",
        kind: Kind::Noise {
            ty: 2,
            by_range: true,
        },
        ticks: true,
    },
    Row {
        label: "drive no modo Add -- a rampa inteira SOBE (o controle)",
        kind: Kind::Drive { mode: 0.0 },
        ticks: false,
    },
    Row {
        label: "drive no modo Min -- a MESMA rampa bate num TECTO e achata",
        kind: Kind::Drive { mode: 5.0 },
        ticks: false,
    },
];

/// Os números que a cena AUTORA e que a mensagem do smoke cita — derivados da
/// tabela, nunca escritos duas vezes.
pub(crate) fn authored() -> (usize, f32, f32, f32) {
    (ROWS_TABLE.len(), BAND_MIN, BAND_MAX, DRIVE_VALUE)
}

/// O documento da cena `=79`.
pub(crate) fn build_band_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::Pos;
    let g = &mut doc.graph;
    let mut sinks = Vec::new();

    for (k, row) in ROWS_TABLE.iter().enumerate() {
        let lane = 100.0 + k as f32 * 210.0;
        let y = (ROWS as f32 - 1.0) * 0.5 * ROW_GAP - k as f32 * ROW_GAP;

        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", 1.0);
        g.set_param(grid, "cols", COLS);
        g.set_param(grid, "gap_x", GAP_X);
        g.set_param(grid, "gap_y", GAP_X);

        let dot = g.add_node("motion.scale");
        g.set_param(dot, "amount", DOT);

        let moved = build_kind(g, row.kind, dot)?;

        let place = g.add_node("motion.transform");
        g.set_param(place, "offset_y", y);
        let out = g.add_node("motion.output");

        for (i, n) in [grid, dot, place, out].into_iter().enumerate() {
            g.set_pos(
                n,
                Pos {
                    x: 80.0 + i as f32 * 190.0,
                    y: lane,
                },
            );
        }

        wire(g, grid, 0, dot, 0)?;
        wire(g, moved, 0, place, 0)?;
        wire(g, place, 0, out, 0)?;
        sinks.push(out);

        if row.ticks {
            sinks.push(build_ticks(g, y, lane)?);
        }
    }

    g.validate(reg).ok()?;
    Some(sinks)
}

/// **AS DUAS MARCAS DA RÉGUA** — duas peças, uma em `BAND_MIN` e outra em
/// `BAND_MAX`, à esquerda da fileira.
///
/// ⚠️ **Uma grade de 2×1 com `gap_y` igual à LARGURA da faixa**, e não dois nós de
/// posição: assim o número que separa as marcas é literalmente o mesmo `f32` que a
/// fileira pede ao animador. Duas posições escritas à mão seriam uma segunda cópia
/// da faixa, a envelhecer no dia em que alguém afinasse a primeira.
fn build_ticks(g: &mut ph2d_nodegraph::graph::Graph, y: f32, lane: f32) -> Option<NodeId> {
    use ph2d_nodegraph::graph::Pos;
    let span = BAND_MAX - BAND_MIN;
    let bar = g.add_node("motion.grid");
    g.set_param(bar, "rows", 2.0);
    g.set_param(bar, "cols", 1.0);
    g.set_param(bar, "gap_y", span);
    g.set_param(bar, "gap_x", 0.0);
    let small = g.add_node("motion.scale");
    g.set_param(small, "amount", TICK);
    let place = g.add_node("motion.transform");
    g.set_param(place, "offset_x", TICK_X - (COLS - 1.0) * GAP_X * 0.5);
    // A grade de 2 linhas nasce centrada, então o seu centro vai ao MEIO da faixa.
    g.set_param(place, "offset_y", y + (BAND_MIN + BAND_MAX) * 0.5);
    let out = g.add_node("motion.output");
    for (i, n) in [bar, small, place, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 190.0,
                y: lane + 105.0,
            },
        );
    }
    wire(g, bar, 0, small, 0)?;
    wire(g, small, 0, place, 0)?;
    wire(g, place, 0, out, 0)?;
    Some(out)
}

/// Monta o nó que MOVE a fileira e devolve-o.
///
/// ⚠️ **A metade `by_range = false` usa exactamente a aritmética que a folha 06
/// registou como *"a resposta do artista"***, e não uma versão de palha: é ela que
/// tem de aparecer certa nas formas bipolares e errada nas unipolares. Escrevê-la
/// de outra maneira faria a cena provar outra coisa.
fn build_kind(g: &mut ph2d_nodegraph::graph::Graph, kind: Kind, src: NodeId) -> Option<NodeId> {
    let amp = (BAND_MAX - BAND_MIN) * 0.5;
    let centre = (BAND_MIN + BAND_MAX) * 0.5;
    Some(match kind {
        Kind::Osc { wave, by_range } => {
            let n = g.add_node("motion.oscillator");
            g.set_param(n, "channel", 1.0); // Y
            g.set_param(n, "wave", wave as f32);
            g.set_param(n, "frequency", 0.5);
            // Uma onda a PERCORRER a fileira: sem stagger as peças subiriam juntas
            // e a fileira seria uma linha a saltar, sem forma para ler.
            g.set_param(n, "phase_stagger", 0.04);
            if by_range {
                g.set_param(n, "range_mode", 1.0);
                g.set_param(n, "min", BAND_MIN);
                g.set_param(n, "max", BAND_MAX);
            } else {
                g.set_param(n, "amplitude", amp);
                g.set_param(n, "offset", centre);
            }
            wire(g, src, 0, n, 0)?;
            n
        }
        Kind::Noise { ty, by_range } => {
            let n = g.add_node("motion.noise");
            g.set_param(n, "channel", 1.0); // Y
            g.set_param(n, "type", ty as f32);
            g.set_param(n, "octaves", 3.0);
            g.set_param(n, "scale", 1.1);
            g.set_param(n, "speed", 0.35);
            if by_range {
                g.set_param(n, "range_mode", 1.0);
                g.set_param(n, "min", BAND_MIN);
                g.set_param(n, "max", BAND_MAX);
            } else {
                // ⚠️ O ruído não tem `offset`: a conta de cabeça só sabe dizer a
                // amplitude, e o centro fica onde calhar. É metade do que a célula
                // 22 chamava de *"o DC é um segundo nó"*.
                g.set_param(n, "amplitude", amp);
            }
            wire(g, src, 0, n, 0)?;
            n
        }
        Kind::Drive { mode } => {
            // Uma rampa POR ÍNDICE no canal Y — a coisa que o drive vai combinar.
            let ramp = g.add_node("motion.stagger");
            g.set_param(ramp, "channel", 1.0); // Y
            g.set_param(ramp, "min", -0.3);
            g.set_param(ramp, "max", 0.5);
            wire(g, src, 0, ramp, 0)?;
            // O valor constante que entra no canal — o oscilador de amplitude zero,
            // o mesmo truque da `=41`.
            let k = g.add_node("value.lfo");
            g.set_param(k, "amplitude", 0.0);
            g.set_param(k, "offset", DRIVE_VALUE);
            wire(g, src, 0, k, 0)?;
            let drive = g.add_node("motion.drive");
            g.set_param(drive, "channel", 1.0); // Y
            g.set_param(drive, "mode", mode);
            g.set_param(drive, "scale", 1.0);
            wire(g, ramp, 0, drive, 0)?;
            wire(g, k, 0, drive, 1)?;
            drive
        }
    })
}

/// Uma aresta. Função LIVRE e não closure: uma closure que captura `g` empresta-o
/// até ao fim do escopo.
fn wire(
    g: &mut ph2d_nodegraph::graph::Graph,
    a: NodeId,
    ap: u16,
    b: NodeId,
    bp: u16,
) -> Option<()> {
    g.connect(ph2d_nodegraph::graph::Edge {
        from: (a, ap),
        to: (b, bp),
        delayed: false,
    })
    .ok()
}

/// O que a cena anuncia — as fileiras, na ordem em que estão na tela.
pub(crate) fn row_labels() -> impl Iterator<Item = (usize, &'static str)> {
    ROWS_TABLE.iter().enumerate().map(|(i, r)| (i, r.label))
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_faixa_tests.rs"]
mod tests;
