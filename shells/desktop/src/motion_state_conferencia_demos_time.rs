//! **O RUÍDO E O RELÓGIO** (`PH2D_GPU_COOK_DEMO=42`) — a cena do **grupo B** da
//! conferência (doc 89, folha 15): os dois geradores TEMPORAIS do domínio de
//! valor, com os quatro params que faltavam.
//!
//! Irmão de `motion_state_conferencia_demos_arith` (a cena do grupo A), e com a
//! mesma forma: cada fileira é uma linha de `{cols}` peças cuja **posição Y é o
//! valor**, então a fileira desenha o PERFIL do campo. ⚠️ E **nenhuma fileira
//! está sozinha** — cada param novo tem o seu CONTROLE imediatamente acima, sobre
//! a MESMA entrada, porque a pergunta não é *"apareceu alguma coisa?"* e sim
//! *"apareceu coisa DIFERENTE?"*.
//!
//! ## ⚠️ Esta cena tem uma leitura que só o PLAY responde
//!
//! O grupo A inteiro se julgava numa foto. Aqui não: **o laço é uma propriedade
//! do TEMPO**, e uma foto de um campo que fecha é indistinguível de uma foto de
//! um campo que não fecha. As fileiras 1 e 2 são o par, e o que se olha é se a
//! segunda **volta à mesma forma** a cada 2 segundos enquanto a primeira nunca
//! repete. É por isso que elas são as únicas que se movem — as outras oito têm
//! `speed = 0` de propósito, para que uma comparação de FORMA não seja também uma
//! comparação de instante.
//!
//! ## As quatro leituras
//!
//! - **O LAÇO** (1-2, em movimento): a de baixo repete a cada 2 s; a de cima
//!   nunca. É o item de maior valor da família — *uma ferramenta de motion design
//!   cujo ruído não fecha o laço não faz um GIF*.
//! - **A LACUNARITY** (3-4, congeladas, cinco oitavas): a mesma pilha com o
//!   multiplicador de frequência em 2 e em 4 — a de baixo tem detalhe visivelmente
//!   mais fino sobre o mesmo esqueleto.
//! - **O PAN, nos seus DOIS eixos** (5-7, congeladas): a 6 é a 5 **DESLIZADA** ao
//!   longo da fila (as mesmas feições, 0,4 de célula adiante) e a 7 é outra
//!   **FATIA** do campo.
//!
//!   ⚠️ **A primeira versão desta leitura era FALSA, e a medição que a derrubou
//!   era minha:** eu escrevi *"pan contra seed — deslize contra re-sorteio"*, e o
//!   gate `a_pan_of_one_is_a_seed_of_one` prova, **byte a byte**, que `pan_y` e
//!   `seed` são o MESMO eixo. Eles não podem desenhar coisas diferentes. O que os
//!   separa é o **GESTO** (passo 1 e widget `Seed` contra passo 0,01 e slider),
//!   e um gesto não se fotografa.
//!
//!   O que a cena mostra, e é verdade, é que o pan é um **VETOR**: `pan_y` desliza
//!   ao longo do eixo que a fileira desenha, e `pan_x` — o eixo do TEMPO, onde
//!   **não existe seed nenhum** — escolhe outra fatia com o relógio parado. É essa
//!   metade que nenhum param anterior alcançava, e é ela que justifica o par.
//! - **O BPM** (8-10, em movimento): a 9 tem de andar em **LOCK-STEP** com a 8 —
//!   0,5 s por ciclo e 120 BPM são o MESMO número em duas réguas —, e a 10, a 180
//!   BPM, visivelmente mais rápida.
//!
//! ## O que a cena NÃO prova
//!
//! Ela não mostra a costura fechar **ao bit**; o olho não resolve 1e-4. Quem o
//! prova é `the_loop_seam_closes_on_the_device` (medido: `0e0` no device, contra
//! uma deriva de 5,22 no controle sem laço). A cena responde à pergunta que só o
//! olho responde: *o campo volta ao mesmo lugar, e o artista vê isso acontecer?*

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// Quantas peças por fileira — a resolução do gráfico.
pub(crate) const COLS: f32 = 48.0;
/// Quantas fileiras a cena empilha.
pub(crate) const ROWS: usize = 10;
/// A distância vertical entre fileiras. A amplitude de cada perfil cabe DENTRO
/// dela: dois perfis que se cruzam deixariam de ser dois gráficos.
const ROW_GAP: f32 = 1.15;
/// O comprimento do laço das fileiras 1-2, em segundos. Curto o bastante para o
/// artista assistir a uma volta inteira sem esperar.
const LOOP_SECONDS: f32 = 2.0;

/// Que nó a fileira exercita, e com que param.
#[derive(Clone, Copy)]
enum Kind {
    /// `value.noise` — o gerador de campo. `speed` decide se a fileira anda.
    Noise(Noise),
    /// `value.lfo` — o oscilador. `time_mode` 0 lê `period`, 1 lê `bpm`.
    Lfo {
        time_mode: f32,
        period: f32,
        bpm: f32,
    },
}

/// Os knobs de um `value.noise` de fileira. ⚠️ É um STRUCT e não campos soltos do
/// enum porque as fileiras 3-7 são variações de UMA linha de base
/// ([`FROZEN_STACK`]) — e a sintaxe `..base` só existe para structs, que é
/// precisamente o que faz cada fileira declarar *o que ela muda* em vez de
/// repetir seis números onde um errado passaria despercebido.
#[derive(Clone, Copy)]
struct Noise {
    speed: f32,
    octaves: f32,
    lacunarity: f32,
    loop_period: f32,
    pan_x: f32,
    pan_y: f32,
    seed: f32,
}

/// O ruído das fileiras 3-7: congelado, cinco oitavas, sem laço.
const FROZEN_STACK: Noise = Noise {
    speed: 0.0,
    octaves: 5.0,
    lacunarity: 2.0,
    loop_period: 0.0,
    pan_x: 0.0,
    pan_y: 0.0,
    seed: 3.0,
};

struct Row {
    label: &'static str,
    kind: Kind,
    /// Quanto o valor levanta a fileira. Por-fileira porque os alcances diferem
    /// (o ruído percorre `[-1,1]`, o LFO também, mas com amplitudes autoradas
    /// diferentes) e um número único achataria metade dos perfis.
    scale: f32,
}

static ROWS_TABLE: &[Row] = &[
    Row {
        label: "noise SEM laco -- o campo nunca repete (PLAY)",
        kind: Kind::Noise(Noise {
            speed: 1.0,
            octaves: 2.0,
            ..FROZEN_STACK
        }),
        scale: 0.45,
    },
    Row {
        label: "noise COM laco 2s -- volta a MESMA forma a cada 2s (PLAY)",
        kind: Kind::Noise(Noise {
            speed: 1.0,
            octaves: 2.0,
            loop_period: LOOP_SECONDS,
            ..FROZEN_STACK
        }),
        scale: 0.45,
    },
    Row {
        label: "noise lacunarity 2 -- cinco oitavas, o CONTROLE",
        kind: Kind::Noise(FROZEN_STACK),
        scale: 0.45,
    },
    Row {
        label: "noise lacunarity 4 -- o mesmo esqueleto, detalhe mais FINO",
        kind: Kind::Noise(Noise {
            lacunarity: 4.0,
            ..FROZEN_STACK
        }),
        scale: 0.45,
    },
    Row {
        label: "noise pan 0 -- o CONTROLE do par abaixo",
        kind: Kind::Noise(FROZEN_STACK),
        scale: 0.45,
    },
    Row {
        label: "noise pan_y +0,4 -- o MESMO perfil, DESLIZADO ao longo da fila",
        kind: Kind::Noise(Noise {
            pan_y: 0.4,
            ..FROZEN_STACK
        }),
        scale: 0.45,
    },
    Row {
        label: "noise pan_x +2,0 -- outra FATIA do campo (o eixo que so' o pan alcanca)",
        kind: Kind::Noise(Noise {
            pan_x: 2.0,
            ..FROZEN_STACK
        }),
        scale: 0.45,
    },
    Row {
        label: "lfo Seconds 0,5s -- o CONTROLE da regua (PLAY)",
        kind: Kind::Lfo {
            time_mode: 0.0,
            period: 0.5,
            bpm: 120.0,
        },
        scale: 0.45,
    },
    Row {
        label: "lfo BPM 120 -- a MESMA regua: anda em LOCK-STEP com a de cima",
        kind: Kind::Lfo {
            time_mode: 1.0,
            period: 0.5,
            bpm: 120.0,
        },
        scale: 0.45,
    },
    Row {
        label: "lfo BPM 180 -- visivelmente mais rapida (PLAY)",
        kind: Kind::Lfo {
            time_mode: 1.0,
            period: 0.5,
            bpm: 180.0,
        },
        scale: 0.45,
    },
];

/// `grid → scale → [<nó> → drive(Y)] → transform → output`, dez vezes.
/// Devolve os DEZ sinks.
pub(super) fn build_time_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::Pos;
    let g = &mut doc.graph;
    let mut sinks = Vec::new();

    for (k, row) in ROWS_TABLE.iter().enumerate() {
        let lane = 100.0 + k as f32 * 210.0;
        // A fileira do topo é a PRIMEIRA da tabela — ler o gráfico de cima para
        // baixo tem de dar a mesma ordem que ler a tabela no código.
        let y = (ROWS as f32 - 1.0) * 0.5 * ROW_GAP - k as f32 * ROW_GAP;

        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", 1.0);
        g.set_param(grid, "cols", COLS);
        g.set_param(grid, "gap_x", 0.22);
        g.set_param(grid, "gap_y", 0.22);

        // Peças pequenas: o que se lê é a CURVA que elas traçam, não cada uma.
        let dot = g.add_node("motion.scale");
        g.set_param(dot, "amount", 0.30);

        let value = build_value(g, row, dot)?;

        let drive = g.add_node("motion.drive");
        g.set_param(drive, "channel", 1.0); // Y
        g.set_param(drive, "mode", 0.0); // Add
        g.set_param(drive, "scale", row.scale);

        let place = g.add_node("motion.transform");
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
        wire(g, dot, 0, drive, 0)?;
        wire(g, value, 0, drive, 1)?;
        wire(g, drive, 0, place, 0)?;
        wire(g, place, 0, out, 0)?;
        sinks.push(out);
    }

    g.validate(reg).ok()?;
    Some(sinks)
}

/// Monta o gerador de uma fileira e devolve o nó terminal dele. ⚠️ Ele lê a
/// GEOMETRIA para a CONTAGEM (a lei de contagem dos dois nós), então o campo tem
/// um valor por peça em vez de um valor global espalhado.
fn build_value(g: &mut ph2d_nodegraph::graph::Graph, row: &Row, geom: NodeId) -> Option<NodeId> {
    Some(match row.kind {
        Kind::Noise(n) => {
            let vn = g.add_node("value.noise");
            // Baixa o bastante para o campo LER como campo (vizinhos correlacionados
            // ao longo da fileira) e alta o bastante para haver várias feições nas
            // 48 peças — sem isso a fileira desenharia meia onda e nenhum detalhe.
            g.set_param(vn, "frequency", 0.13);
            g.set_param(vn, "speed", n.speed);
            g.set_param(vn, "octaves", n.octaves);
            g.set_param(vn, "roughness", 0.55);
            g.set_param(vn, "lacunarity", n.lacunarity);
            g.set_param(vn, "loop_period", n.loop_period);
            g.set_param(vn, "pan_x", n.pan_x);
            g.set_param(vn, "pan_y", n.pan_y);
            g.set_param(vn, "seed", n.seed);
            wire(g, geom, 0, vn, 0)?;
            vn
        }
        Kind::Lfo {
            time_mode,
            period,
            bpm,
        } => {
            let lfo = g.add_node("value.lfo");
            g.set_param(lfo, "period", period);
            g.set_param(lfo, "bpm", bpm);
            g.set_param(lfo, "time_mode", time_mode);
            g.set_param(lfo, "amplitude", 1.0);
            // A onda viaja pela fileira: sem stagger as 48 peças subiriam e
            // desceriam juntas e a fileira seria uma barra, não um gráfico.
            g.set_param(lfo, "phase_stagger", 0.021);
            wire(g, geom, 0, lfo, 0)?;
            lfo
        }
    })
}

/// Uma aresta. Função LIVRE e não closure: uma closure que captura `g` o empresta
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

/// O comprimento do laço que a cena arma — para o anúncio dizer o número que o
/// artista tem de cronometrar, em vez de um literal repetido na mensagem.
pub(crate) const fn loop_seconds() -> f32 {
    LOOP_SECONDS
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_time_tests.rs"]
mod tests;
