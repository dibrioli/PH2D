//! **O ENVELOPE** (`PH2D_GPU_COOK_DEMO=46`) — a cena do **grupo F** da conferência
//! (doc 89, folha 07).
//!
//! ## Uma pergunta, dois nós
//!
//! *Que FORMA tem uma coisa que acende e apaga?* O `motion.strobe` responde a
//! partir de um **pulso** (o flash), o `motion.delay` a partir de um **degrau** (o
//! smoother). Os dois governavam a forma com um número só — queda exponencial
//! fixa num, uma régua para as duas direções no outro — e a folha 07 marcava cada
//! metade.
//!
//! ## A leitura é por PARES, e é isso que a torna barata
//!
//! Cada par é **o mesmo rig com um knob de diferença**, empilhado adjacente. A
//! pergunta nunca é *"apareceu alguma coisa?"* — é *"as duas fileiras fazem coisas
//! DIFERENTES, e a diferença é a que o knob promete?"*.
//!
//! 1. **ATTACK** — a de cima POPA no pulso, a de baixo INCHA ao longo de meio
//!    segundo. É o trecho que o nó não tinha: ele subia sempre em um tick.
//! 2. **HOLD** — a de baixo fica GRANDE por meio segundo antes de começar a cair.
//! 3. **SHAPE** — a mesma queda, uma exponencial e a outra através de um DEGRAU
//!    desenhado: ela fica cheia e **CORTA**, o que nenhuma exponencial faz.
//! 4. **PROBABILITY** — a de cima acende a fileira INTEIRA em toda batida; a de
//!    baixo acende ~um terço das peças, e **peças diferentes a cada batida**.
//! 5. **RISE ≠ FALL** (o `motion.delay`) — as duas seguem o MESMO degrau; a de
//!    cima sobe e desce no mesmo tempo, a de baixo **salta e escorre**.
//!
//! ⚠️ **Esta cena julga-se com PLAY.** Um envelope é uma forma no TEMPO: uma foto
//! de um instante mostra dois tamanhos e não diz nada sobre como se chegou a eles.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Quantas peças por fileira.
pub(crate) const COLS: f32 = 24.0;
/// Quantas fileiras a cena empilha (cinco PARES).
pub(crate) const BANDS: usize = 10;
/// A distância vertical entre fileiras.
const BAND_GAP: f32 = 1.0;
/// O tamanho de repouso de uma peça.
const DOT: f32 = 0.19;
/// Quanto o flash a engorda no pico — grande, para a forma se ler de longe.
const BOOST: f32 = 3.0;

/// O compasso: dois segundos entre batidas, tempo de sobra para ver uma subida de
/// meio segundo e uma queda inteira antes da próxima.
pub(crate) const BEAT: f32 = 2.0;
/// Meio segundo a 60 Hz — a régua de todos os trechos autorados desta cena.
pub(crate) const HALF_SECOND: f32 = 30.0;
/// A cauda comum: um terço de segundo.
const DECAY: f32 = 20.0;
/// A probabilidade da banda 8. ⚠️ Nem tão baixa que a fileira pareça morta, nem
/// tão alta que *"quase todas"* e *"todas"* sejam a mesma foto.
///
/// ⚠️ **E a CONTAGEM balança de batida para batida, de propósito.** Um sorteio
/// por-peça sobre 24 peças tem desvio-padrão `√(24·p·(1−p)) ≈ 2,1`; as quatro
/// primeiras batidas desta cena medem `8 · 10 · 11 · 8` contra uma média de 6.
///
/// ⚠️ **Isso foi MEDIDO até ao fim antes de a cena ser afinada, e a conclusão foi
/// NÃO a afinar.** As quatro pistas são individualmente justas (`0,2505 · 0,2450 ·
/// 0,2452 · 0,2505` sobre 4000 linhas cada) e a média global fecha em **0,2523**
/// sobre 4000 batidas — o que a cena calha é um **canto de 1 em 2000**: varrendo
/// as janelas `linhas base..base+24 × pistas 0..3`, só **uma em duas mil** acende
/// 37 ou mais das 96. Alargar a fileira até o número *parecer* o pedido seria
/// afinar a demonstração para lisonjear a feature; o que um sorteio por-peça de
/// facto entrega é uma contagem que balança, e é isso que a cena mostra e a
/// mensagem diz. O gate afirma a MÉDIA, com folga para o balanço e nenhuma para
/// os dois modos de falha reais (acender tudo, ou não acender nada).
pub(crate) const SOME: f32 = 0.25;
/// Um degrau desenhado em `ph2d-curve`: cheio até meio caminho, **zero** depois.
/// É a forma que uma exponencial não tem — ela desvanece, nunca corta.
pub(crate) const CLIFF: &str = "c1 0:0:H 0.5:1:H 1:1:H";

/// As duas réguas do par do `motion.delay`.
pub(crate) const FAST: f32 = 2.0;
pub(crate) const SLOW: f32 = 40.0;

/// O que uma fileira desenha.
#[derive(Clone, Copy)]
enum Kind {
    /// Um strobe com os quatro knobs do envelope.
    Strobe {
        attack: f32,
        hold: f32,
        curve: Option<&'static str>,
        probability: f32,
    },
    /// Um degrau quadrado passado por um smoother com as duas réguas.
    Smoother { rise: f32, fall: f32 },
}

static LANES: [Kind; BANDS] = [
    // 1-2 · ATTACK
    Kind::Strobe {
        attack: 0.0,
        hold: 0.0,
        curve: None,
        probability: 1.0,
    },
    Kind::Strobe {
        attack: HALF_SECOND,
        hold: 0.0,
        curve: None,
        probability: 1.0,
    },
    // 3-4 · HOLD
    Kind::Strobe {
        attack: 0.0,
        hold: 0.0,
        curve: None,
        probability: 1.0,
    },
    Kind::Strobe {
        attack: 0.0,
        hold: HALF_SECOND,
        curve: None,
        probability: 1.0,
    },
    // 5-6 · SHAPE
    Kind::Strobe {
        attack: 0.0,
        hold: 0.0,
        curve: None,
        probability: 1.0,
    },
    Kind::Strobe {
        attack: 0.0,
        hold: 0.0,
        curve: Some(CLIFF),
        probability: 1.0,
    },
    // 7-8 · PROBABILITY
    Kind::Strobe {
        attack: 0.0,
        hold: 0.0,
        curve: None,
        probability: 1.0,
    },
    Kind::Strobe {
        attack: 0.0,
        hold: 0.0,
        curve: None,
        probability: SOME,
    },
    // 9-10 · RISE ≠ FALL
    Kind::Smoother {
        rise: FAST,
        fall: 0.0,
    },
    Kind::Smoother {
        rise: FAST,
        fall: SLOW,
    },
];

/// O que a cena anuncia — uma linha por fileira, na ordem em que estão na tela.
pub(crate) const BAND_LABELS: [&str; BANDS] = [
    "1  STROBE  attack 0                -- \\",
    "2  STROBE  attack 30 ticks         -- /  POPA contra INCHA",
    "3  STROBE  hold 0                  -- \\",
    "4  STROBE  hold 30 ticks           -- /  cai ja' contra fica CHEIA e depois cai",
    "5  STROBE  shape identidade        -- \\",
    "6  STROBE  shape DEGRAU            -- /  desvanece contra CORTA",
    "7  STROBE  probability 1.00        -- \\",
    "8  STROBE  probability 0.25        -- /  a fileira TODA contra ~um quarto, sorteado",
    "9  DELAY   rise 2  fall = rise     -- \\",
    "10 DELAY   rise 2  fall 40         -- /  simetrico contra SALTA e escorre",
];

/// Monta as dez fileiras e devolve os sinks.
///
/// ⚠️ `pub(crate)` como a irmã da cena `=45`: o gate mora ao lado, e uma cena que
/// só o roteador alcança não pode ser medida.
pub(crate) fn build_envelope_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::new();

    for (k, kind) in LANES.iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "BANDS e' pequeno")]
        let row = 100.0 + k as f32 * 230.0;
        // A fileira do topo é a PRIMEIRA da tabela — ler o gráfico de cima para
        // baixo tem de dar a mesma ordem que ler a lista no log.
        #[expect(clippy::cast_precision_loss, reason = "BANDS e' pequeno")]
        let y = (BANDS as f32 - 1.0) * 0.5 * BAND_GAP - k as f32 * BAND_GAP;

        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", 1.0);
        g.set_param(grid, "cols", COLS);
        g.set_param(grid, "gap_x", 0.34);
        g.set_param(grid, "gap_y", 0.34);

        let dot = g.add_node("motion.scale");
        g.set_param(dot, "amount", DOT);

        let tail = build_lane(g, *kind, dot)?;

        let place = g.add_node("motion.transform");
        g.set_param(place, "offset_y", y);
        let out = g.add_node("motion.output");

        for (i, n) in [grid, dot, tail, place, out].into_iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "poucos nos por fileira")]
            let x = 80.0 + i as f32 * 210.0;
            g.set_pos(n, Pos { x, y: row });
        }

        wire(g, grid, 0, dot, 0)?;
        wire(g, tail, 0, place, 0)?;
        wire(g, place, 0, out, 0)?;
        sinks.push(out);
    }

    g.validate(reg).ok()?;
    Some(sinks)
}

/// Monta a cadeia de uma fileira sobre a geometria `geom` e devolve o nó terminal.
fn build_lane(g: &mut Graph, kind: Kind, geom: NodeId) -> Option<NodeId> {
    match kind {
        Kind::Strobe {
            attack,
            hold,
            curve,
            probability,
        } => {
            let beat = g.add_node("pulse.beat");
            g.set_param(beat, "period", BEAT);

            let st = g.add_node("motion.strobe");
            g.set_param(st, "decay", DECAY);
            g.set_param(st, "size_boost", BOOST);
            g.set_param(st, "attack", attack);
            g.set_param(st, "hold", hold);
            g.set_param(st, "probability", probability);
            if let Some(c) = curve {
                g.set_text_param(st, "curve", c);
            }

            // ⚠️ O metrônomo LÊ a geometria — ele é per-instância e tira dali a
            // contagem de linhas. Sem esta aresta ele emite ZERO linhas, o
            // `scalar_col` do strobe redimensiona com `0.0`, e a cena fica parada
            // sem um erro: um pulso que nunca chega é indistinguível de um pulso
            // que não acende.
            wire(g, geom, 0, beat, 0)?;
            wire(g, geom, 0, st, 0)?;
            wire(g, beat, 0, st, 1)?;
            // ⚠️ Os `pre` self-loops são escritos à MÃO: o editor os plumba ao
            // SOLTAR um nó, e um documento montado por `add_node` não os ganha.
            // Sem eles nem a memória de borda do `pulse.beat` nem o envelope do
            // strobe existem, e a cena fica PARADA.
            self_loop(g, beat, 1)?;
            self_loop(g, st, 2)?;
            Some(st)
        }
        Kind::Smoother { rise, fall } => {
            // O degrau: uma onda QUADRADA em [0,1], o sinal cuja subida e descida
            // um smoother tem de tratar de formas diferentes.
            let sq = g.add_node("value.lfo");
            g.set_param(sq, "wave", 2.0); // Square
            g.set_param(sq, "period", BEAT);
            g.set_param(sq, "amplitude", 0.5);
            g.set_param(sq, "offset", 0.5);

            let drive = g.add_node("motion.drive");
            g.set_param(drive, "channel", 3.0); // Size
            g.set_param(drive, "mode", 0.0); // Add
            g.set_param(drive, "scale", DOT * BOOST);

            let dly = g.add_node("motion.delay");
            g.set_param(dly, "channel", 3.0); // Size
            g.set_param(dly, "mode", 2.0); // Blend
            g.set_param(dly, "ticks", rise);
            g.set_param(dly, "ticks_down", fall);

            wire(g, geom, 0, drive, 0)?;
            wire(g, sq, 0, drive, 1)?;
            wire(g, drive, 0, dly, 0)?;
            self_loop(g, dly, 1)?;
            Some(dly)
        }
    }
}

/// Uma aresta.
fn wire(g: &mut Graph, a: NodeId, ap: u16, b: NodeId, bp: u16) -> Option<()> {
    g.connect(Edge {
        from: (a, ap),
        to: (b, bp),
        delayed: false,
    })
    .ok()
}

/// O `pre` self-loop de um nó sequencial: a saída dele de volta na porta de estado.
fn self_loop(g: &mut Graph, n: NodeId, port: u16) -> Option<()> {
    g.connect(Edge {
        from: (n, 0),
        to: (n, port),
        delayed: true,
    })
    .ok()
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_envelope_tests.rs"]
mod tests;
