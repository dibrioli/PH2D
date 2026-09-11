//! **O METRÓNOMO** (`PH2D_GPU_COOK_DEMO=80`) — a cena do grupo de 2026-08-22
//! (doc 89, folha 12, que FECHOU por inteiro): a régua, a fase por-linha, a janela
//! de atividade e a referência por-elemento.
//!
//! ## Um pulso é um EVENTO, e o olho não vê eventos — vê o que eles deixam
//!
//! Cada fileira é uma linha de [`COLS`] peças ligada a um `pulse.counter` que
//! **sobe um degrau por batida**, e o degrau é a altura da peça. Uma fileira parada
//! é um metrónomo que não bate; uma fileira que sobe em bloco é um metrónomo
//! uniforme; uma que sobe em ONDA é um metrónomo escalonado.
//!
//! ⚠️ **O contador está em `Clamp`, não em `Wrap`, e isso é o que torna a janela
//! legível.** Com `Wrap` toda fileira volta ao chão de tempos a tempos, e uma que
//! PAROU fica indistinguível de uma que deu a volta. Com `Clamp` a escada que parou
//! fica parada, e a diferença é permanente na tela.
//!
//! ## Os quatro pares
//!
//! 1. **A RÉGUA** — `0,5 s` contra `120 BPM`. É o mesmo número em duas unidades,
//!    então as duas escadas têm de subir **em lock-step**. Qualquer divergência é a
//!    conversão errada.
//! 2. **A FASE POR LINHA** — a mesma batida, e à direita cada peça um degrau atrás
//!    da vizinha: a escada vira uma rampa que percorre a fileira.
//! 3. **A JANELA** — à esquerda o metrónomo é eterno; à direita ele dá [`WINDOW`]
//!    batidas e **para**.
//! 4. **A REFERÊNCIA** — um limiar por-elemento contra um limiar único. ⚠️ É o par
//!    que mostra um bug **silencioso**: um fio ligado ao *param* colapsa na linha 0
//!    (`driven_value` é `xs.first()`), e desenha um limiar plausível e errado.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};

/// Quantas peças por fileira.
pub(crate) const COLS: f32 = 24.0;
/// O passo horizontal entre peças.
const GAP_X: f32 = 0.30;
/// A distância vertical entre fileiras.
const ROW_GAP: f32 = 1.20;
/// Quantas fileiras a cena empilha.
const ROWS: usize = 8;
/// O tamanho das peças.
const DOT: f32 = 0.30;

/// O período do metrónomo dos pares 1, 3 e 4, em segundos — e o BPM que é o MESMO
/// número, que a fileira 2 usa para provar a régua.
pub(crate) const PERIOD: f32 = 0.5;
pub(crate) const BPM: f32 = 60.0 / PERIOD;

/// O período do par da FASE — mais lento, para o degrau a percorrer a fileira ser
/// legível em vez de um borrão.
const SLOW: f32 = 1.0;
/// O atraso de cada peça em relação à vizinha, no par da fase.
const STAGGER: f32 = 0.06;

/// Quantas batidas a janela deixa passar.
pub(crate) const WINDOW: f32 = 4.0;

/// A altura de um degrau, em unidades de mundo.
const STEP_H: f32 = 0.07;
/// O teto do contador. `Clamp` ⇒ a escada plateia aqui em vez de dar a volta.
const COUNT_MAX: f32 = 9.0;

/// Os limiares por-elemento do último par — o padrão que o limiar único não sabe
/// desenhar.
///
/// ⚠️ **`REF_B` está ACIMA do pico do sinal, de propósito.** O sinal varre
/// `[SIGNAL_LOW .. SIGNAL_HIGH]`, então as peças de limiar `REF_A` cruzam a cada
/// volta e as de `REF_B` **nunca** cruzam — a fileira sobe *bolinha sim, bolinha
/// não*. Com os dois limiares abaixo do pico as duas metades subiriam igual, e o
/// par não diria nada.
const REF_A: f32 = 0.25;
const REF_B: f32 = 0.75;

/// O limiar ÚNICO da fileira de controle — abaixo do pico, logo TODA peça cruza.
const SINGLE_RISE: f32 = 0.5;
/// A largura da histerese das duas fileiras de comparação.
const HYSTERESIS: f32 = 0.05;

/// A onda que alimenta o par da comparação: um seno lento que **percorre** a
/// fileira, varrendo `[0,10 .. 0,60]`.
///
/// ⚠️ **Ele existe porque a 1ª versão desta cena usou um sinal ESTÁTICO** (uma
/// rampa por índice) — e um sinal estático faz cada peça armar **uma vez** e ficar.
/// As duas fileiras subiam um degrau no primeiro quadro e nunca mais se mexiam.
/// Smoke reprovado (Enio, 2026-08-22: *"as duas últimas fileiras de baixo não se
/// movem"*), e o gate que existia media *"subiu alguma coisa?"* — que é verdade de
/// uma fileira morta que saltou uma vez.
const SIGNAL_PERIOD: f32 = 1.0;
const SIGNAL_LOW: f32 = 0.10;
const SIGNAL_HIGH: f32 = 0.60;
/// O atraso da onda de peça para peça — ela PERCORRE a fileira em vez de piscar.
const SIGNAL_STAGGER: f32 = 0.03;

/// Que fileira é esta.
#[derive(Clone, Copy)]
enum Kind {
    /// Um `pulse.beat` autorado — o período em segundos, ou o mesmo em BPM.
    Beat {
        bpm: bool,
        period: f32,
        stagger: f32,
        count: f32,
    },
    /// Um `pulse.compare` sobre uma rampa: limiar único ou por-elemento.
    Compare { per_element: bool },
}

struct Row {
    label: &'static str,
    kind: Kind,
}

static ROWS_TABLE: &[Row] = &[
    Row {
        label: "regua em SEGUNDOS -- meio segundo por degrau",
        kind: Kind::Beat {
            bpm: false,
            period: PERIOD,
            stagger: 0.0,
            count: 0.0,
        },
    },
    Row {
        label: "regua em BPM -- o MESMO numero, e a escada sobe junto",
        kind: Kind::Beat {
            bpm: true,
            period: PERIOD,
            stagger: 0.0,
            count: 0.0,
        },
    },
    Row {
        label: "fase por linha DESLIGADA -- a fileira sobe em bloco",
        kind: Kind::Beat {
            bpm: false,
            period: SLOW,
            stagger: 0.0,
            count: 0.0,
        },
    },
    Row {
        label: "fase por linha LIGADA -- o degrau PERCORRE a fileira",
        kind: Kind::Beat {
            bpm: false,
            period: SLOW,
            stagger: STAGGER,
            count: 0.0,
        },
    },
    Row {
        label: "sem janela -- o metronomo e' eterno (o controle)",
        kind: Kind::Beat {
            bpm: false,
            period: PERIOD,
            stagger: 0.0,
            count: 0.0,
        },
    },
    Row {
        label: "com janela -- ele da' 4 batidas e PARA",
        kind: Kind::Beat {
            bpm: false,
            period: PERIOD,
            stagger: 0.0,
            count: WINDOW,
        },
    },
    Row {
        label: "limiar UNICO -- a fileira INTEIRA sobe",
        kind: Kind::Compare { per_element: false },
    },
    Row {
        label: "limiar POR-ELEMENTO -- sobe bolinha SIM, bolinha NAO",
        kind: Kind::Compare { per_element: true },
    },
];

/// Os números que a cena AUTORA e que a mensagem do smoke cita.
pub(crate) fn authored() -> (usize, f32, f32, f32) {
    (ROWS_TABLE.len(), PERIOD, BPM, WINDOW)
}

/// O documento da cena `=80` — uma sink por fileira.
pub(crate) fn build_pulse_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::new();

    for (k, row) in ROWS_TABLE.iter().enumerate() {
        let lane = 100.0 + k as f32 * 240.0;
        let y = (ROWS as f32 - 1.0) * 0.5 * ROW_GAP - k as f32 * ROW_GAP;

        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", 1.0);
        g.set_param(grid, "cols", COLS);
        g.set_param(grid, "gap_x", GAP_X);
        g.set_param(grid, "gap_y", GAP_X);
        let dot = g.add_node("motion.scale");
        g.set_param(dot, "amount", DOT);
        wire(g, grid, 0, dot, 0)?;

        let pulse = build_pulse(g, row.kind, dot)?;

        // O CONTADOR: um degrau por batida. `Clamp` para a escada que PAROU ficar
        // parada — com `Wrap` ela voltaria ao chão e o par da janela não se leria.
        let counter = g.add_node("pulse.counter");
        g.set_param(counter, "count_max", COUNT_MAX);
        g.set_param(counter, "mode", 1.0); // Clamp
        wire(g, pulse, 0, counter, 0)?;
        // A memória de borda: a saída volta como `state` pelo laço `pre`.
        wire_pre(g, counter, counter, 1)?;

        let drive = g.add_node("motion.drive");
        g.set_param(drive, "channel", 1.0); // Y
        g.set_param(drive, "mode", 0.0); // Add
        g.set_param(drive, "scale", STEP_H);
        wire(g, dot, 0, drive, 0)?;
        wire(g, counter, 0, drive, 1)?;

        let place = g.add_node("motion.transform");
        g.set_param(place, "offset_y", y);
        let out = g.add_node("motion.output");
        wire(g, drive, 0, place, 0)?;
        wire(g, place, 0, out, 0)?;

        for (i, n) in [grid, dot, counter, drive, place, out]
            .into_iter()
            .enumerate()
        {
            g.set_pos(
                n,
                Pos {
                    x: 80.0 + i as f32 * 180.0,
                    y: lane,
                },
            );
        }
        sinks.push(out);
    }

    g.validate(reg).ok()?;
    Some(sinks)
}

/// Monta a fonte de PULSO da fileira.
fn build_pulse(g: &mut ph2d_nodegraph::graph::Graph, kind: Kind, src: NodeId) -> Option<NodeId> {
    Some(match kind {
        Kind::Beat {
            bpm,
            period,
            stagger,
            count,
        } => {
            let b = g.add_node("pulse.beat");
            if bpm {
                g.set_param(b, "time_mode", 1.0);
                g.set_param(b, "bpm", 60.0 / period);
            } else {
                g.set_param(b, "period", period);
            }
            g.set_param(b, "phase_stagger", stagger);
            g.set_param(b, "count", count);
            wire(g, src, 0, b, 0)?;
            wire_pre(g, b, b, 1)?;
            b
        }
        Kind::Compare { per_element } => {
            // O SINAL: uma onda lenta que PERCORRE a fileira, varrendo
            // `[SIGNAL_LOW .. SIGNAL_HIGH]`. ⚠️ Ele tem de ser ANIMADO — ver o doc
            // de [`SIGNAL_PERIOD`] e o smoke que a versão estática reprovou.
            let sig = g.add_node("value.lfo");
            g.set_param(sig, "period", SIGNAL_PERIOD);
            g.set_param(sig, "amplitude", (SIGNAL_HIGH - SIGNAL_LOW) * 0.5);
            g.set_param(sig, "offset", (SIGNAL_HIGH + SIGNAL_LOW) * 0.5);
            g.set_param(sig, "phase_stagger", SIGNAL_STAGGER);
            wire(g, src, 0, sig, 0)?;
            let c = g.add_node("pulse.compare");
            g.set_param(c, "rise", SINGLE_RISE);
            g.set_param(c, "fall", SINGLE_RISE - HYSTERESIS);
            wire(g, sig, 0, c, 0)?;
            wire_pre(g, c, c, 1)?;
            if per_element {
                // O limiar POR-ELEMENTO: um padrão que alterna entre dois valores,
                // um ABAIXO e outro ACIMA do pico da onda.
                // ⚠️ É a coisa que um fio ligado ao PARAM não sabe fazer — ele
                // colapsaria na linha 0 e desenharia a fileira inteira igual.
                let pat = g.add_node("value.pattern");
                g.set_param(pat, "steps", 2.0);
                g.set_param(pat, "v0", REF_A);
                g.set_param(pat, "v1", REF_B);
                wire(g, src, 0, pat, 0)?;
                wire(g, pat, 0, c, 2)?;
            }
            c
        }
    })
}

fn wire(
    g: &mut ph2d_nodegraph::graph::Graph,
    a: NodeId,
    ap: u16,
    b: NodeId,
    bp: u16,
) -> Option<()> {
    g.connect(Edge {
        from: (a, ap),
        to: (b, bp),
        delayed: false,
    })
    .ok()
}

/// Uma aresta ATRASADA — o laço `pre` que dá memória de borda a um nó de pulso.
fn wire_pre(g: &mut ph2d_nodegraph::graph::Graph, from: NodeId, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, 0),
        to: (to, tp),
        delayed: true,
    })
    .ok()
}

/// O que a cena anuncia — as fileiras, na ordem em que estão na tela.
pub(crate) fn row_labels() -> impl Iterator<Item = (usize, &'static str)> {
    ROWS_TABLE.iter().enumerate().map(|(i, r)| (i, r.label))
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_pulso_tests.rs"]
mod tests;
