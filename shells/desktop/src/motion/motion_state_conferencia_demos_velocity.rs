//! **PARA ONDE ISTO VAI** (`PH2D_GPU_COOK_DEMO=47`) — a cena do **grupo G** da
//! conferência (doc 89, folha 07).
//!
//! ## O problema da cena é que a wave é INVISÍVEL
//!
//! O `motion.velocity` escreve uma coluna. Uma coluna não se vê — ela só existe na
//! tela através de quem a LÊ, e é por isso que cada banda desta cena é uma cadeia
//! `velocity → value.attribute → motion.drive` em vez do nó sozinho.
//!
//! ## A leitura é por PARES, como no grupo F
//!
//! Cada par é **o mesmo rig com um nó (ou um knob) de diferença**. A pergunta nunca
//! é *"apareceu alguma coisa?"* — é *"as duas fileiras fazem coisas DIFERENTES, e a
//! diferença é a que a wave promete?"*.
//!
//! 1. **VELOCIDADE → TAMANHO** — as peças da de cima têm todas o mesmo tamanho; as
//!    de baixo **INCHAM onde vão depressa** e encolhem nas pontas do balanço, onde
//!    param para voltar. O canal `Speed` do `value.attribute` já existia e devolvia
//!    **zeros** nesta cadeia: a fileira de cima é literalmente o que ele mostrava.
//! 2. **DIREÇÃO → ROTAÇÃO** — o *align to velocity*, a linha que cinco famílias
//!    citaram. As peças são **TRAÇOS**, e as de baixo apontam para onde vão: numa
//!    órbita, a tangente. As de cima ficam todas no mesmo ângulo.
//! 3. **O `smooth`** — o mesmo rig com um driver TREMIDO. A de cima usa a diferença
//!    crua e o tamanho **pisca**; a de baixo passa pelo one-pole e **respira**.
//!
//! ⚠️ **Esta cena julga-se com PLAY.** Uma velocidade é uma diferença entre dois
//! instantes: uma foto mostra tamanhos e ângulos diferentes ao longo da fileira — o
//! que já é meio caminho, porque o `phase_stagger` põe cada peça num ponto distinto
//! do percurso — mas só o movimento mostra a peça a INCHAR quando acelera.
//!
//! ⚠️ **As peças da 2ª metade são traços de propósito** (a lição da cena `=38`): um
//! quadrado rodado 90° é o mesmo quadrado, e a cena não provaria nada.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Quantas peças por fileira.
pub(crate) const COLS: f32 = 18.0;
/// Quantas fileiras a cena empilha (três PARES).
pub(crate) const BANDS: usize = 6;
/// A distância vertical entre fileiras.
const BAND_GAP: f32 = 1.15;

/// O raio do percurso de cada peça — pequeno o bastante para a fileira não invadir
/// a vizinha, grande o bastante para a velocidade variar de forma visível.
const SWING: f32 = 0.28;
/// Voltas por segundo. Uma volta a cada dois segundos e meio: devagar o suficiente
/// para o olho seguir uma peça, rápido o suficiente para o inchaço ser óbvio.
const SPIN: f32 = 0.4;
/// O deslocamento de fase entre peças vizinhas, em ciclos. ⚠️ **Não é enfeite:** com
/// stagger zero as dezoito peças fariam exatamente a mesma coisa ao mesmo tempo, e a
/// fileira inteira teria um tamanho só — a cena mostraria o efeito e **não** que ele
/// é POR ELEMENTO.
const STAGGER: f32 = 0.055;

/// O tamanho de repouso de uma peça redonda (bandas 1-2 e 5-6).
const DOT: f32 = 0.17;
/// O comprimento de um traço (bandas 3-4) — a razão de aspecto é o que torna a
/// rotação legível.
const TRACE_LONG: f32 = 0.34;
const TRACE_SHORT: f32 = 0.075;

/// Quanto a velocidade engorda a peça, por unidade de mundo por segundo.
///
/// ⚠️ **É uma régua, não um gosto:** a velocidade de pico deste percurso é
/// `2π · SWING · SPIN ≈ 0,70 u/s`, então este ganho a converte em `~0,26` de
/// tamanho — uma vez e meia o `DOT`. Um número tirado do ar daria uma fileira ou
/// inerte ou saturada, e nos dois casos o par não diria nada.
const SPEED_TO_SIZE: f32 = 0.38;

/// A amplitude do tremor das bandas 5-6, em unidades de mundo.
///
/// ⚠️ **Ela é PEQUENA contra o percurso** (`SWING` é 0,28) e mesmo assim domina a
/// velocidade — e é esse o ponto da wave: uma diferença finita **amplifica** todo
/// tremor, porque ela divide por `dt`. Um sacolejo de dois centésimos num tick de
/// 1/60 s são **1,2 u/s**, quase o dobro da velocidade do próprio percurso.
const JITTER: f32 = 0.02;
/// A frequência do tremor: alta o bastante para mudar de direção a cada poucos
/// ticks, que é o que faz a diferença crua PISCAR.
const JITTER_HZ: f32 = 9.0;
/// A constante do one-pole da banda 6, em ticks.
const SMOOTH: f32 = 8.0;

/// Os canais compartilhados (`motion.oscillator` / `motion.wiggle` / `motion.drive`).
const CH_X: f32 = 0.0;
const CH_Y: f32 = 1.0;
const CH_ROTATION: f32 = 2.0;
const CH_SIZE: f32 = 3.0;
/// `motion.drive`: 0 Add · 1 Set · 2 Multiply.
const MODE_ADD: f32 = 0.0;
const MODE_SET: f32 = 1.0;

/// O que cada fileira demonstra.
#[derive(Clone, Copy)]
enum Kind {
    /// Tamanho: `None` é o controle (nada dirige), `Some(smooth)` liga a cadeia.
    Size { smooth: Option<f32>, jitter: bool },
    /// Rotação: `false` é o controle (ângulo fixo), `true` alinha à velocidade.
    Aim { align: bool },
}

/// As seis fileiras, de cima para baixo — a MESMA ordem que o log imprime.
const LANES: [Kind; BANDS] = [
    Kind::Size {
        smooth: None,
        jitter: false,
    },
    Kind::Size {
        smooth: Some(0.0),
        jitter: false,
    },
    Kind::Aim { align: false },
    Kind::Aim { align: true },
    Kind::Size {
        smooth: Some(0.0),
        jitter: true,
    },
    Kind::Size {
        smooth: Some(SMOOTH),
        jitter: true,
    },
];

/// Os rótulos que o roteador de smoke imprime — a lista que o artista lê antes de
/// olhar para a tela.
pub(crate) const BAND_LABELS: [&str; BANDS] = [
    "1  tamanho FIXO (controle: o canal Speed devolvia zeros)",
    "2  tamanho pela VELOCIDADE (incha onde acelera)",
    "3  angulo FIXO (controle)",
    "4  ALINHADO a velocidade (o traco aponta para onde vai)",
    "5  driver TREMIDO, diferenca crua (o tamanho pisca)",
    "6  o MESMO tremor, smooth 8 (o tamanho respira)",
];

/// Monta o documento da cena. `None` se um nó não registrar ou uma aresta recusar.
pub(crate) fn build_velocity_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::new();

    for (k, kind) in LANES.iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "BANDS e' pequeno")]
        let row = 100.0 + k as f32 * 240.0;
        // A fileira do topo é a PRIMEIRA da tabela — ler o gráfico de cima para
        // baixo tem de dar a mesma ordem que ler a lista no log.
        #[expect(clippy::cast_precision_loss, reason = "BANDS e' pequeno")]
        let y = (BANDS as f32 - 1.0) * 0.5 * BAND_GAP - k as f32 * BAND_GAP;

        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", 1.0);
        g.set_param(grid, "cols", COLS);
        g.set_param(grid, "gap_x", 0.42);
        g.set_param(grid, "gap_y", 0.42);

        let shape = g.add_node("motion.scale");
        let aiming = matches!(kind, Kind::Aim { .. });
        // Um traço nas bandas de rotação, um disco nas de tamanho.
        g.set_param(shape, "amount", if aiming { TRACE_LONG } else { DOT });
        g.set_param(shape, "amount_y", if aiming { TRACE_SHORT } else { DOT });

        let tail = build_lane(g, *kind, shape)?;

        let place = g.add_node("motion.transform");
        g.set_param(place, "offset_y", y);
        let out = g.add_node("motion.output");

        for (i, n) in [grid, shape, tail, place, out].into_iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "poucos nos por fileira")]
            let x = 80.0 + i as f32 * 210.0;
            g.set_pos(n, Pos { x, y: row });
        }

        wire(g, grid, 0, shape, 0)?;
        wire(g, tail, 0, place, 0)?;
        wire(g, place, 0, out, 0)?;
        sinks.push(out);
    }

    g.validate(reg).ok()?;
    Some(sinks)
}

/// **DUAS FORMAS DE PERCURSO, E A ESCOLHA E' MEDIDA — não estética.**
///
/// ⚠️ **A 1ª versão desta cena punha as SEIS bandas num círculo, e a sonda a
/// reprovou antes do smoke:** a velocidade sobre um círculo é constante em
/// MAGNITUDE, então a banda do tamanho media `[0,4341 … 0,4382]` — as dezoito peças
/// **do mesmo tamanho**, só maiores que o controle. A cena mostraria *"tudo
/// cresceu"* e não *"incha onde acelera"*, que é a frase que a mensagem promete.
///
/// - **`Swing`** (bandas 1-2 e 5-6) — um vaivém sobre X. A rapidez é máxima no meio
///   e **zero nas pontas**, onde a peça para para voltar: é a variação que a banda
///   do tamanho existe para mostrar.
/// - **`Circle`** (bandas 3-4) — dois osciladores em quadratura. ⚠️ **Um vaivém NÃO
///   serve para a direção**, e a razão é geometria: sobre X ela alterna entre `0°` e
///   `180°`, e um traço rodado de meia volta **é o mesmo traço** — a cena mostraria
///   o alinhamento a funcionar e o olho não o distinguiria de um ângulo fixo.
#[derive(Clone, Copy, PartialEq)]
enum Path {
    Swing,
    Circle,
}

fn build_path(g: &mut Graph, geom: NodeId, path: Path, jitter: bool) -> Option<NodeId> {
    let ox = g.add_node("motion.oscillator");
    g.set_param(ox, "channel", CH_X);
    g.set_param(ox, "amplitude", SWING);
    g.set_param(ox, "frequency", SPIN);
    g.set_param(ox, "phase_stagger", STAGGER);
    wire(g, geom, 0, ox, 0)?;

    let mut tail = ox;
    if path == Path::Circle {
        let oy = g.add_node("motion.oscillator");
        g.set_param(oy, "channel", CH_Y);
        g.set_param(oy, "amplitude", SWING);
        g.set_param(oy, "frequency", SPIN);
        g.set_param(oy, "phase_stagger", STAGGER);
        // Um quarto de ciclo — o `phase` do oscilador é medido em CICLOS
        // (`frac(t·cps + i·stagger + phase)`), então `0,25` é o que fecha o círculo.
        g.set_param(oy, "phase", 0.25);
        wire(g, ox, 0, oy, 0)?;
        tail = oy;
    }

    if !jitter {
        return Some(tail);
    }
    let w = g.add_node("motion.wiggle");
    g.set_param(w, "channel", CH_X);
    g.set_param(w, "amplitude", JITTER);
    g.set_param(w, "frequency", JITTER_HZ);
    wire(g, tail, 0, w, 0)?;
    Some(w)
}

/// Monta a cadeia de uma fileira sobre a geometria `geom` e devolve o nó terminal.
fn build_lane(g: &mut Graph, kind: Kind, geom: NodeId) -> Option<NodeId> {
    match kind {
        Kind::Size { smooth, jitter } => {
            let path = build_path(g, geom, Path::Swing, jitter)?;
            let Some(smooth) = smooth else {
                // O CONTROLE: o mesmo percurso, sem ninguém a medi-lo. É o que a
                // cadeia `value.attribute(Speed)` desenhava antes desta wave.
                return Some(path);
            };
            let vel = g.add_node("motion.velocity");
            g.set_param(vel, "smooth", smooth);
            let attr = g.add_node("value.attribute");
            g.set_param(
                attr,
                "mode",
                mode_of(ph2d_node_value_attribute::MODE_LENGTH),
            );
            g.set_text_param(attr, ph2d_node_value_attribute::ATTR_KEY, "vel");
            let drive = g.add_node("motion.drive");
            g.set_param(drive, "channel", CH_SIZE);
            g.set_param(drive, "scale", SPEED_TO_SIZE);
            g.set_param(drive, "mode", MODE_ADD);

            wire(g, path, 0, vel, 0)?;
            // ⚠️ O `pre` self-loop é escrito à MÃO: o editor o plumba ao SOLTAR um
            // nó, e um documento montado por `add_node` não o ganha. Sem ele o nó
            // nunca tem um ontem, toda velocidade é zero, e a fileira fica
            // **idêntica ao controle** — a feature quebrada e a feature ausente
            // desenham a mesma coisa.
            self_loop(g, vel, 1)?;
            wire(g, vel, 0, attr, 0)?;
            wire(g, vel, 0, drive, 0)?;
            wire(g, attr, 0, drive, 1)?;
            Some(drive)
        }
        Kind::Aim { align } => {
            let path = build_path(g, geom, Path::Circle, false)?;
            if !align {
                return Some(path);
            }
            let vel = g.add_node("motion.velocity");
            let attr = g.add_node("value.attribute");
            g.set_param(attr, "mode", mode_of(ph2d_node_value_attribute::MODE_ANGLE));
            g.set_text_param(attr, ph2d_node_value_attribute::ATTR_KEY, "vel");
            let drive = g.add_node("motion.drive");
            g.set_param(drive, "channel", CH_ROTATION);
            // **Set**, não Add: a direção JÁ É o ângulo que a peça tem de ter, e o
            // `rot` do stream fala graus — a mesma unidade que o `MODE_ANGLE`
            // devolve, de propósito (o doc dele diz porquê: radianos errariam por
            // 57× exactamente nesta costura).
            g.set_param(drive, "mode", MODE_SET);

            wire(g, path, 0, vel, 0)?;
            self_loop(g, vel, 1)?;
            wire(g, vel, 0, attr, 0)?;
            wire(g, vel, 0, drive, 0)?;
            wire(g, attr, 0, drive, 1)?;
            Some(drive)
        }
    }
}

/// O `mode` do `value.attribute` é um `i32` (as reduções crescem para BAIXO, as
/// lanes para cima) e o param que o grafo guarda é `f32`.
#[expect(clippy::cast_precision_loss, reason = "os modos sao -1..=5")]
fn mode_of(m: i32) -> f32 {
    m as f32
}

/// Uma aresta viva.
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
#[path = "motion_state_conferencia_demos_velocity_tests.rs"]
mod tests;
