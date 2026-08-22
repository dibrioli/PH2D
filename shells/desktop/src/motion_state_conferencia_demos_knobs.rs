//! **OS KNOBS QUE FALTAVAM AO DOMÍNIO DE VALOR** (`PH2D_GPU_COOK_DEMO=78`) — a cena
//! do grupo de 2026-08-22 (doc 89, folha 15): nove controles apendados a oito nós,
//! cada um com o **nó sem ele** desenhado ao lado.
//!
//! Irmão de `motion_state_conferencia_demos` pelo teto de LOC da shell.
//!
//! ## A cena é um GRÁFICO — o mesmo idioma da `=41`, e de propósito
//!
//! Cada fileira é uma linha de [`COLS`] peças cuja **posição Y é o valor**, então a
//! fileira desenha o PERFIL da função. O Enio já leu este idioma na `=41` (a
//! aritmética do valor); repeti-lo custa zero aprendizagem e a comparação é a
//! mesma: *as duas fileiras do par desenham formas diferentes?*
//!
//! ⚠️ **NENHUMA fileira está sozinha, e é essa a única coisa que esta cena pode
//! provar.** Um knob novo cuja implementação não fosse lida produziria as duas
//! metades do par IDÊNTICAS — que é exactamente o modo de falha de um ramo de WGSL
//! nunca alcançado, e o único que um smoke de *"apareceu alguma coisa?"* deixa
//! passar. Todo par aqui é `sem o knob` / `com o knob`, sobre a MESMA entrada.
//!
//! ## As duas colunas
//!
//! A área visível de uma cena de demo mede cerca de `10 x 10` unidades de mundo
//! (medido na `=41`: `x [−5,17 .. 5,17]`), e dezoito fileiras empilhadas não cabem
//! nela. Duas colunas de [`COLS`] peças resolvem-no sem apertar a altura: cada
//! coluna mede `(COLS − 1) · GAP_X` de largura e vive a [`COL_X`] do centro.
//!
//! ⚠️ **A largura de uma fileira não está escrita em lado nenhum do grafo** — ela
//! sai de `(cols − 1) · gap_x`, três nós acima do `motion.transform` que a coloca.
//! É a lição que a sonda `measure_scene_layout` existe para não se repetir.
//!
//! ## O que a cena NÃO prova
//!
//! Ela não mede paridade CPU/GPU nem a neutralidade bit-a-bit dos defaults — isso
//! é `gpu_cpu_parity_arith` (cada par desta cena tem lá o seu gêmeo, medido no
//! device) e os gates de unidade de cada crate. Aqui responde-se só à pergunta que
//! o olho responde: *o botão novo muda a FORMA que o nome dele promete?*

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// Quantas peças por fileira — a resolução do gráfico. Metade da `=41`, porque
/// aqui cabem duas colunas lado a lado.
pub(crate) const COLS: f32 = 22.0;
/// O passo horizontal entre peças. `(COLS − 1) · GAP_X` = a largura de uma fileira.
const GAP_X: f32 = 0.20;
/// A distância vertical entre fileiras.
const ROW_GAP: f32 = 1.02;
/// Quantas fileiras a coluna MAIS ALTA empilha — a escada de Y que as duas colunas
/// partilham, para o topo delas se alinhar.
const LADDER: usize = 10;
/// A que distância do centro cada coluna vive.
const COL_X: f32 = 2.60;

/// Quanto vale o `distance` do `Smooth Min` na fileira que o exercita.
///
/// ⚠️ **Medido, não escolhido:** o mergulho máximo do polinómio é `k/6`, então com
/// `k = 0,6` a quina desce `0,10` num perfil que percorre `0,5` — **20%**, que é o
/// que o olho separa de longe. Com o `0,4` do gate de paridade seriam 13%, e a
/// diferença lia-se como ruído de renderização.
const SMIN_DISTANCE: f32 = 0.6;

/// Quantos segundos dura a rampa de entrada da LFO — longa o bastante para o Enio
/// VER o crescimento em vez de o apanhar já cheio.
const LFO_FADE: f32 = 3.0;

/// Que knob a fileira exercita. Cada variante carrega o VALOR do knob, e o membro
/// neutro do par é a mesma variante com o valor que o nó sempre teve.
#[derive(Clone, Copy)]
enum Knob {
    /// `value.step` — o espelho da máscara.
    Step { invert: f32 },
    /// `value.quantize` — a fase da grade.
    Quantize { offset: f32 },
    /// `value.pattern` — a fase do padrão e como ela se resolve.
    Pattern { offset: f32, interp: f32 },
    /// `value.switch` — saltar ou dissolver.
    Switch { blend: f32 },
    /// `value.curve` — quanto da curva fica.
    Curve { factor: f32 },
    /// `value.mix` — o clamp do RESULTADO (não o do factor).
    MixClamp { clamp_result: f32 },
    /// `value.math` op 14 — a terceira porta.
    MultiplyAdd { with_c: bool },
    /// `value.math` op 15 — a largura da mistura.
    SmoothMin { distance: f32 },
    /// `value.lfo` — a rampa de entrada.
    Lfo { fade_in: f32 },
}

struct Row {
    label: &'static str,
    knob: Knob,
    /// `0` = coluna da esquerda, `1` = da direita.
    col: usize,
    /// Quanto o valor levanta a fileira. Por-fileira: os alcances de saída diferem
    /// (uma máscara percorre 1, um switch de quatro vias percorre 3), e um número
    /// único deixaria metade dos perfis achatados e a outra metade a invadir a
    /// fileira de baixo.
    scale: f32,
}

static ROWS_TABLE: &[Row] = &[
    // ── Coluna da ESQUERDA ──────────────────────────────────────────────────
    Row {
        label: "step Smoother -- o S que sobe (o controle)",
        knob: Knob::Step { invert: 0.0 },
        col: 0,
        scale: 0.60,
    },
    Row {
        label: "step Smoother INVERTIDO -- o mesmo S, ao contrario",
        knob: Knob::Step { invert: 1.0 },
        col: 0,
        scale: 0.60,
    },
    Row {
        label: "quantize -- escada com um DEGRAU sobre o meio (o controle)",
        knob: Knob::Quantize { offset: 0.0 },
        col: 0,
        scale: 0.28,
    },
    Row {
        label: "quantize com FASE -- a mesma escada com uma QUINA sobre o meio",
        knob: Knob::Quantize { offset: 0.25 },
        col: 0,
        scale: 0.28,
    },
    Row {
        label: "pattern -- os tres degraus repetidos (o controle)",
        knob: Knob::Pattern {
            offset: 0.0,
            interp: 0.0,
        },
        col: 0,
        scale: 0.30,
    },
    Row {
        label: "pattern DESLIZADO meio passo e amaciado -- zigue-zague mais curto",
        knob: Knob::Pattern {
            offset: 0.5,
            interp: 1.0,
        },
        col: 0,
        scale: 0.30,
    },
    Row {
        label: "switch -- ESCADA de quatro degraus (o controle)",
        knob: Knob::Switch { blend: 0.0 },
        col: 0,
        scale: 0.20,
    },
    Row {
        label: "switch a DISSOLVER -- a mesma escolha vira uma RETA",
        knob: Knob::Switch { blend: 1.0 },
        col: 0,
        scale: 0.20,
    },
    Row {
        label: "curve -- a TENDA simetrica (o controle)",
        knob: Knob::Curve { factor: 1.0 },
        col: 0,
        scale: 0.60,
    },
    Row {
        label: "curve a MEIO -- a tenda cede metade do caminho a` entrada",
        knob: Knob::Curve { factor: 0.5 },
        col: 0,
        scale: 0.60,
    },
    // ── Coluna da DIREITA ───────────────────────────────────────────────────
    Row {
        label: "mix Add -- a soma TRANSBORDA e continua a subir (o controle)",
        knob: Knob::MixClamp { clamp_result: 0.0 },
        col: 1,
        scale: 0.55,
    },
    Row {
        label: "mix Add com CLAMP RESULT -- a mesma soma para de subir",
        knob: Knob::MixClamp { clamp_result: 1.0 },
        col: 1,
        scale: 0.55,
    },
    Row {
        label: "math Multiply Add SEM a 3a porta -- uma rampa (o controle)",
        knob: Knob::MultiplyAdd { with_c: false },
        col: 1,
        scale: 0.60,
    },
    Row {
        label: "math Multiply Add COM a 3a porta -- a MESMA rampa passa a DESCER",
        knob: Knob::MultiplyAdd { with_c: true },
        col: 1,
        scale: 0.60,
    },
    Row {
        label: "math Smooth Min a zero -- rampa com QUINA seca (o controle)",
        knob: Knob::SmoothMin { distance: 0.0 },
        col: 1,
        scale: 1.10,
    },
    Row {
        label: "math Smooth Min aberto -- a MESMA quina, arredondada",
        knob: Knob::SmoothMin {
            distance: SMIN_DISTANCE,
        },
        col: 1,
        scale: 1.10,
    },
    Row {
        label: "lfo -- a onda ja' nasce inteira (o controle) >>> PLAY",
        knob: Knob::Lfo { fade_in: 0.0 },
        col: 1,
        scale: 0.30,
    },
    Row {
        label: "lfo com FADE IN -- a MESMA onda cresce do nada >>> PLAY",
        knob: Knob::Lfo { fade_in: LFO_FADE },
        col: 1,
        scale: 0.30,
    },
];

/// Os números que a cena AUTORA e que a mensagem do smoke cita — derivados da
/// tabela, nunca escritos duas vezes.
pub(crate) fn authored() -> (usize, f32, f32) {
    (ROWS_TABLE.len(), SMIN_DISTANCE, LFO_FADE)
}

/// O documento da cena `=78` — uma sink por fileira.
pub(crate) fn build_knobs_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::Pos;
    let g = &mut doc.graph;
    let mut sinks = Vec::new();
    // Quantas fileiras já foram colocadas em cada coluna — é isto que dá a
    // ESCADA de Y, e não o índice na tabela (as duas colunas partilham o topo).
    let mut placed = [0usize; 2];

    for (k, row) in ROWS_TABLE.iter().enumerate() {
        let lane = 100.0 + k as f32 * 210.0;
        let r = placed[row.col];
        placed[row.col] += 1;
        // A fileira do topo de cada coluna é a PRIMEIRA dela na tabela — ler o
        // gráfico de cima para baixo tem de dar a mesma ordem que ler o código.
        let y = (LADDER as f32 - 1.0) * 0.5 * ROW_GAP - r as f32 * ROW_GAP;
        let x = if row.col == 0 { -COL_X } else { COL_X };

        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", 1.0);
        g.set_param(grid, "cols", COLS);
        g.set_param(grid, "gap_x", GAP_X);
        g.set_param(grid, "gap_y", GAP_X);

        // Peças pequenas: o que se lê é a CURVA que elas traçam, não cada uma.
        let dot = g.add_node("motion.scale");
        g.set_param(dot, "amount", 0.26);

        let ramp = g.add_node("value.instance_field");
        g.set_param(ramp, "mode", 1.0); // Ramp: i/(N−1) em [0,1]

        let value = build_value(g, row, dot, ramp)?;

        let drive = g.add_node("motion.drive");
        g.set_param(drive, "channel", 1.0); // Y
        g.set_param(drive, "mode", 0.0); // Add
        g.set_param(drive, "scale", row.scale);

        let place = g.add_node("motion.transform");
        g.set_param(place, "offset_x", x);
        g.set_param(place, "offset_y", y);
        let out = g.add_node("motion.output");

        for (i, n) in [grid, dot, drive, place, out].into_iter().enumerate() {
            g.set_pos(
                n,
                Pos {
                    x: 80.0 + i as f32 * 190.0,
                    y: lane,
                },
            );
        }

        wire(g, grid, 0, dot, 0)?;
        wire(g, dot, 0, ramp, 0)?;
        wire(g, dot, 0, drive, 0)?;
        wire(g, value, 0, drive, 1)?;
        wire(g, drive, 0, place, 0)?;
        wire(g, place, 0, out, 0)?;
        sinks.push(out);
    }

    g.validate(reg).ok()?;
    Some(sinks)
}

/// Monta a cadeia de VALOR de uma fileira e devolve o nó terminal dela.
///
/// ⚠️ **O membro NEUTRO de cada par usa exactamente o mesmo código**, com o knob no
/// valor que o nó sempre teve. Duas funções — uma "antiga" e uma "nova" — seriam
/// dois caminhos a divergir, e o par deixaria de comparar o que diz comparar.
fn build_value(
    g: &mut ph2d_nodegraph::graph::Graph,
    row: &Row,
    dot: NodeId,
    ramp: NodeId,
) -> Option<NodeId> {
    Some(match row.knob {
        Knob::Step { invert } => {
            let s = g.add_node("value.step");
            g.set_param(s, "threshold", 0.5);
            // A banda cobre a rampa inteira: uma banda estreita desenharia um
            // degrau quase duro, e o espelho leria como dois degraus quaisquer.
            g.set_param(s, "width", 1.0);
            g.set_param(s, "mode", 2.0); // Smoother
            g.set_param(s, "invert", invert);
            wire(g, ramp, 0, s, 0)?;
            s
        }
        Knob::Quantize { offset } => {
            // ⚠️ A entrada é ASSINADA e o passo é `0,5`, para o **meio da fileira**
            // cair sobre um DEGRAU no controle e sobre uma QUINA na fase. Numa
            // rampa `[0,1]` as duas escadas teriam a mesma silhueta deslocada
            // verticalmente — e um deslocamento vertical entre fileiras DIFERENTES
            // é invisível.
            let signed = stretch(g, ramp, -1.1, 1.1)?;
            let q = g.add_node("value.quantize");
            g.set_param(q, "step", 0.5);
            g.set_param(q, "offset", offset);
            wire(g, signed, 0, q, 0)?;
            q
        }
        Knob::Pattern { offset, interp } => {
            let p = g.add_node("value.pattern");
            g.set_param(p, "steps", 3.0);
            g.set_param(p, "v0", 0.0);
            g.set_param(p, "v1", 1.0);
            g.set_param(p, "v2", 2.0);
            g.set_param(p, "offset", offset);
            g.set_param(p, "interp", interp);
            // O padrão lê o stream pela CONTAGEM, não pelo valor.
            wire(g, dot, 0, p, 0)?;
            p
        }
        Knob::Switch { blend } => {
            let sel = stretch(g, ramp, 0.0, 3.0)?;
            let sw = g.add_node("value.switch");
            g.set_param(sw, "blend", blend);
            wire(g, sel, 0, sw, 0)?;
            for (port, v) in [(1u16, 0.0f32), (2, 1.0), (3, 2.0), (4, 3.0)] {
                let c = constant(g, dot, v)?;
                wire(g, c, 0, sw, port)?;
            }
            sw
        }
        Knob::Curve { factor } => {
            let c = g.add_node("value.curve");
            // Uma tenda: a forma que nenhum remap escalar faz.
            g.set_text_param(c, "curve", "c1 0:0:L 0.5:1:L 1:0:L".to_string());
            g.set_param(c, "factor", factor);
            wire(g, ramp, 0, c, 0)?;
            c
        }
        Knob::MixClamp { clamp_result } => {
            let m = g.add_node("value.mix");
            g.set_param(m, "factor", 1.0); // o modo aparece INTEIRO
            g.set_param(m, "blend", 1.0); // Add
            g.set_param(m, "clamp_result", clamp_result);
            wire(g, ramp, 0, m, 0)?;
            // ⚠️ `0,5` e não `0,9`: o tecto tem de ser atingido **no MEIO** da
            // fileira. Com `0,9` a metade travada subia 10% e ficava plana o
            // resto — e uma fileira quase plana não se distingue, no olho de quem
            // smoka, de uma cadeia que não produziu nada. Assim ela sobe metade
            // do caminho e PARA, que é a forma que o nome do botão promete.
            let b = constant(g, dot, 0.5)?;
            wire(g, b, 0, m, 1)?;
            m
        }
        Knob::MultiplyAdd { with_c } => {
            let m = g.add_node("value.math");
            g.set_param(m, "op", 14.0); // Multiply Add
            wire(g, ramp, 0, m, 0)?;
            let b = constant(g, dot, 1.0)?;
            wire(g, b, 0, m, 1)?;
            if with_c {
                // ⚠️ O `c` é a rampa INVERTIDA, e não uma constante: uma constante
                // levantaria o perfil sem lhe mudar a FORMA, e entre duas fileiras
                // diferentes um levantamento é invisível.
                //
                // ⚠️ E ela desce de `1,5`, não de `1,0`. Com `1,0` a soma daria
                // `a + (1 − a) = 1` — uma reta horizontal PERFEITA, que é
                // exactamente o que uma cadeia partida também desenha. Com `1,5` a
                // soma é `1,5 − 0,5·a`: o perfil DESCE onde o controle SOBE, que
                // se lê de longe e não se confunde com nada.
                let c = stretch(g, ramp, 1.5, 0.0)?;
                wire(g, c, 0, m, 2)?;
            }
            m
        }
        Knob::SmoothMin { distance } => {
            let m = g.add_node("value.math");
            g.set_param(m, "op", 15.0); // Smooth Min
            g.set_param(m, "distance", distance);
            wire(g, ramp, 0, m, 0)?;
            // O tecto contra o qual a rampa bate — a quina cai no MEIO da fileira.
            let b = constant(g, dot, 0.5)?;
            wire(g, b, 0, m, 1)?;
            m
        }
        Knob::Lfo { fade_in } => {
            let k = g.add_node("value.lfo");
            g.set_param(k, "period", 0.6);
            // Uma onda a PERCORRER a fileira: sem stagger as 22 peças subiriam e
            // desceriam juntas e a fileira seria uma linha a saltar.
            g.set_param(k, "phase_stagger", 0.05);
            g.set_param(k, "fade_in", fade_in);
            wire(g, dot, 0, k, 0)?;
            k
        }
    })
}

/// A rampa esticada para `[lo, hi]` — PLUMBING, com a interpolação no default.
fn stretch(g: &mut ph2d_nodegraph::graph::Graph, ramp: NodeId, lo: f32, hi: f32) -> Option<NodeId> {
    let mr = g.add_node("value.map_range");
    g.set_param(mr, "out_lo", lo);
    g.set_param(mr, "out_hi", hi);
    wire(g, ramp, 0, mr, 0)?;
    Some(mr)
}

/// Um campo CONSTANTE — o oscilador de amplitude zero, o mesmo truque da `=41`.
fn constant(g: &mut ph2d_nodegraph::graph::Graph, dot: NodeId, v: f32) -> Option<NodeId> {
    let k = g.add_node("value.lfo");
    g.set_param(k, "amplitude", 0.0);
    g.set_param(k, "offset", v);
    wire(g, dot, 0, k, 0)?;
    Some(k)
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
#[path = "motion_state_conferencia_demos_knobs_tests.rs"]
mod tests;
