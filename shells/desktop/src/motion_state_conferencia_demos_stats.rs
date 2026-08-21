//! **AS ESTATÍSTICAS** (`PH2D_GPU_COOK_DEMO=43`) — a cena do **grupo C** da
//! conferência (doc 89, folha 15): os quatro agregados novos do `value.reduce`,
//! as duas portas que os escopam, e os pesos da janela do `value.smooth`.
//!
//! ## A forma é diferente da dos grupos A e B, e o motivo é o que se mede
//!
//! Nas duas cenas anteriores cada fileira era um perfil e a comparação era de
//! FORMA, que sobrevive a um deslocamento vertical. **Aqui a comparação é de
//! ALTURA** — *onde a estatística cai em relação ao campo* —, e uma altura não
//! sobrevive a uma linha de base própria. Então uma **BANDA** é uma faixa
//! vertical onde várias cadeias desenham no MESMO `offset_y`: o campo, e por
//! cima dele as retas que as reduções difundem.
//!
//! ⚠️ É a lição do gate de lock-step do grupo B posta no LAYOUT em vez de no
//! oráculo: lá o gate comparava Y CRU de duas fileiras e media o `ROW_GAP`; aqui
//! as coisas que precisam de ser comparadas partilham a base por construção.
//!
//! ## As oito leituras
//!
//! **Bandas 1-4 — o `value.reduce`.** A fonte é um campo deliberadamente
//! **ENVIESADO** (`noise → Square → Square`, ou seja `x⁴`): quase tudo perto do
//! chão e uma cauda alta. Um campo simétrico teria média e mediana no mesmo
//! lugar e a banda 1 não diria nada.
//!
//! 1. **NÍVEIS** — o campo + `Mean` + `Median`. ⚠️ As duas retas **não coincidem**,
//!    e é essa a razão de existir da mediana: a média cede à cauda, o rank não.
//! 2. **MÁSCARA** — o campo + `Mean` + `Mean` com a porta `mask` ligada (só os
//!    elementos acima de um degrau). A segunda reta SOBE: a máscara escolhe quem
//!    é CONTADO. ⚠️ E ela não escolhe quem é RESPONDIDO — a reta é desenhada por
//!    todas as peças, inclusive as que ficaram de fora da conta.
//! 3. **GRUPO** — o campo + `Mean` + `Mean` com a porta `group` ligada (quatro
//!    bins por índice). A reta vira uma **ESCADA de quatro degraus**: é a redução
//!    SEGMENTADA, e é o item que nenhuma composição de nós alcançava.
//! 4. **MAGNITUDES** — o campo + `Range` + `Std Dev`. As duas são grandezas e não
//!    níveis: o `Range` mede o vão inteiro (uma reta alta) e o desvio mede a
//!    dispersão típica (bem mais baixa, porque a cauda é rara).
//!
//! **Bandas 5-8 — o `value.smooth`.** A fonte é um **DEGRAU** (banda 5), porque é
//! sobre uma descontinuidade que a forma de um núcleo se vê: as três seguintes
//! são o mesmo degrau filtrado com o mesmo raio e pesos diferentes.
//!
//! 6. **Box** — a rampa é RETA e **arranca de repente**: as duas quinas onde a
//!    janela começa e acaba de cruzar o degrau são a assinatura dele.
//! 7. **Triangle** — arranca **sete vezes mais devagar** e curva.
//! 8. **Smooth** — arranca **dezoito vezes mais devagar** que o Box: um **S**,
//!    que sai do platô sem que se veja onde.
//!
//! ⚠️ **A régua aqui é o ARRANQUE, e a primeira que escolhi estava errada.** Eu
//! ia medir CURVATURA e afirmar `Box > Triangle > Smooth`; medido, a ordem é
//! `Box 0,100 · Smooth 0,040 · Triangle 0,027` — um S **tem** de curvar mais no
//! meio justamente para ser chato nas pontas. O que ordena os três de facto é o
//! primeiro degrau da rampa contra o maior (`1,00 · 0,14 · 0,055`), que é
//! também o que o olho lê: *quão de repente a rampa começa*.
//!
//! ## O que a cena NÃO prova
//!
//! Ela não mostra as três recusas do device (`Variance`/`StdDev`/`Median` e as
//! duas portas caem para a CPU) — isso é uma propriedade do PLANO, e quem a
//! prova é `the_plan_claims_the_foldable_modes_and_recedes_from_the_other_three`
//! em `ph2d-gpu-cook`, que roda sem adapter nenhum. A cena responde à pergunta
//! que só o olho responde: *o número que a estatística diz é o número que se vê?*

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Quantas peças por fileira — a resolução do gráfico.
pub(crate) const COLS: f32 = 48.0;
/// Quantas BANDAS a cena empilha.
pub(crate) const BANDS: usize = 8;
/// A distância vertical entre bandas. Generosa de propósito: a banda 1 tem de
/// caber o campo E duas retas sem que elas encostem no vizinho.
const BAND_GAP: f32 = 1.55;
/// Quanto o valor levanta a peça. Um só para TODAS as cadeias — duas escalas
/// diferentes fariam a altura de uma reta deixar de ser comparável com o campo,
/// que é a única coisa que esta cena mede.
const VALUE_SCALE: f32 = 1.30;
/// Em quantos bins a banda 3 parte a fileira.
const GROUP_BINS: f32 = 4.0;
/// O raio das três janelas das bandas 6-8.
const SMOOTH_RADIUS: f32 = 6.0;

/// O índice do `mode` `Floor` no `value.quantize`.
const QUANTIZE_FLOOR: f32 = 1.0;
/// O `mode` `Index` do `value.instance_field` (`0 … N−1`) e o `Ramp` (`0 … 1`).
const FIELD_INDEX: f32 = 0.0;
const FIELD_RAMP: f32 = 1.0;

/// Onde o degrau das bandas 5-8 cai, na rampa `0…1`.
const STEP_AT: f32 = 0.5;
/// Onde o ruído da fonte é cortado. Alto de propósito: é ele que faz da cauda uma
/// MINORIA, e é a minoria que separa a média da mediana.
const SOURCE_CUT: f32 = 0.15;
/// A largura desse corte — o campo tem PLATÔS e rampas em vez de dois níveis
/// soltos, senão ele não lê como campo.
const SOURCE_SOFTNESS: f32 = 0.35;
/// Onde o degrau da MÁSCARA da banda 2 corta o campo. Acima dele fica a cauda —
/// as peças que a máscara CONTA.
const MASK_AT: f32 = 0.5;

/// Qual das duas portas opcionais do `value.reduce` esta cadeia liga.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Port {
    /// Nenhuma — o nó como sempre foi.
    None,
    /// `mask`: quem é CONTADO.
    Mask,
    /// `group`: um agregado por bin.
    Group,
}

/// O que uma cadeia desenha.
#[derive(Clone, Copy)]
enum Kind {
    /// O campo FONTE enviesado — a referência sobre a qual as retas caem.
    Field,
    /// Uma redução sobre esse campo.
    Reduce { mode: f32, port: Port },
    /// O DEGRAU — a fonte das três janelas.
    Step,
    /// Uma janela sobre o degrau.
    Smooth { weight: f32 },
}

/// Uma cadeia. Várias partilham `band`, e é isso que faz delas um gráfico só.
struct Lane {
    band: usize,
    kind: Kind,
    /// O tamanho da peça. O campo é desenhado com peças MENORES que as retas —
    /// é o que separa *o dado* de *a estatística sobre ele* sem um segundo canal
    /// de cor.
    dot: f32,
}

const FIELD_DOT: f32 = 0.16;
const LINE_DOT: f32 = 0.26;

static LANES: &[Lane] = &[
    // 1 — NÍVEIS: o campo, a média, a mediana.
    Lane {
        band: 0,
        kind: Kind::Field,
        dot: FIELD_DOT,
    },
    Lane {
        band: 0,
        kind: Kind::Reduce {
            mode: 1.0, // Mean
            port: Port::None,
        },
        dot: LINE_DOT,
    },
    Lane {
        band: 0,
        kind: Kind::Reduce {
            mode: 7.0, // Median
            port: Port::None,
        },
        dot: LINE_DOT,
    },
    // 2 — MÁSCARA.
    Lane {
        band: 1,
        kind: Kind::Field,
        dot: FIELD_DOT,
    },
    Lane {
        band: 1,
        kind: Kind::Reduce {
            mode: 1.0,
            port: Port::None,
        },
        dot: LINE_DOT,
    },
    Lane {
        band: 1,
        kind: Kind::Reduce {
            mode: 1.0,
            port: Port::Mask,
        },
        dot: LINE_DOT,
    },
    // 3 — GRUPO.
    Lane {
        band: 2,
        kind: Kind::Field,
        dot: FIELD_DOT,
    },
    Lane {
        band: 2,
        kind: Kind::Reduce {
            mode: 1.0,
            port: Port::None,
        },
        dot: LINE_DOT,
    },
    Lane {
        band: 2,
        kind: Kind::Reduce {
            mode: 1.0,
            port: Port::Group,
        },
        dot: LINE_DOT,
    },
    // 4 — MAGNITUDES.
    Lane {
        band: 3,
        kind: Kind::Field,
        dot: FIELD_DOT,
    },
    Lane {
        band: 3,
        kind: Kind::Reduce {
            mode: 4.0, // Range
            port: Port::None,
        },
        dot: LINE_DOT,
    },
    Lane {
        band: 3,
        kind: Kind::Reduce {
            mode: 6.0, // Std Dev
            port: Port::None,
        },
        dot: LINE_DOT,
    },
    // 5-8 — o degrau e as três janelas.
    Lane {
        band: 4,
        kind: Kind::Step,
        dot: LINE_DOT,
    },
    Lane {
        band: 5,
        kind: Kind::Smooth { weight: 0.0 },
        dot: LINE_DOT,
    },
    Lane {
        band: 6,
        kind: Kind::Smooth { weight: 1.0 },
        dot: LINE_DOT,
    },
    Lane {
        band: 7,
        kind: Kind::Smooth { weight: 2.0 },
        dot: LINE_DOT,
    },
];

/// O que a cena anuncia — uma linha por BANDA, na ordem em que estão na tela.
pub(crate) const BAND_LABELS: [&str; BANDS] = [
    "1 NIVEIS    campo + Mean + Median -- as duas retas NAO coincidem",
    "2 MASCARA   campo + Mean + Mean(mask: so' a cauda) -- a 2a reta SOBE",
    "3 GRUPO     campo + Mean + Mean(group: 4 bins) -- a reta vira ESCADA",
    "4 MAGNITUDE campo + Range + Std Dev -- o vao inteiro contra a dispersao",
    "5 DEGRAU    a fonte das tres de baixo",
    "6 smooth Box      -- rampa RETA, com duas QUINAS",
    "7 smooth Triangle -- a rampa curva",
    "8 smooth Smooth   -- um S: sem quina nenhuma",
];

/// `grid → scale → [<cadeia de valor> → drive(Y)] → transform → output`, uma vez
/// por lane. Devolve os sinks.
pub(crate) fn build_stats_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::new();

    for (k, lane) in LANES.iter().enumerate() {
        let row = 100.0 + k as f32 * 210.0;
        // A banda do topo é a PRIMEIRA da tabela — ler o gráfico de cima para
        // baixo tem de dar a mesma ordem que ler a tabela no código.
        let y = (BANDS as f32 - 1.0) * 0.5 * BAND_GAP - lane.band as f32 * BAND_GAP;

        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", 1.0);
        g.set_param(grid, "cols", COLS);
        g.set_param(grid, "gap_x", 0.22);
        g.set_param(grid, "gap_y", 0.22);

        let dot = g.add_node("motion.scale");
        g.set_param(dot, "amount", lane.dot);

        let value = build_value(g, lane.kind, dot)?;

        let drive = g.add_node("motion.drive");
        g.set_param(drive, "channel", 1.0); // Y
        g.set_param(drive, "mode", 0.0); // Add
        g.set_param(drive, "scale", VALUE_SCALE);

        let place = g.add_node("motion.transform");
        g.set_param(place, "offset_y", y);
        let out = g.add_node("motion.output");

        for (i, n) in [grid, dot, drive, place, out].into_iter().enumerate() {
            g.set_pos(
                n,
                Pos {
                    x: 80.0 + i as f32 * 190.0,
                    y: row,
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

/// O campo FONTE das bandas 1-4: um ruído cortado ALTO por um degrau macio —
/// platôs no chão, uma minoria no topo, e rampas entre eles.
///
/// ⚠️ **O viés é a fixture, não decoração.** Sobre um campo simétrico a média e a
/// mediana caem no mesmo lugar, e a banda 1 desenharia duas retas coincidentes —
/// verde por vácuo, no sentido visual.
///
/// ⚠️ **E a primeira versão desta fonte era `noise → Square → Square`, que a
/// sonda REPROVOU:** `x⁴` de um ruído que percorre `±0,5` chega a `0,0625`, então
/// o campo inteiro colapsava para um vão de **0,082** — enviesado, sim, e
/// invisível. O viés que serve aqui não é o de uma potência, é o de uma
/// MINORIA: com o corte alto a mediana cai no platô do chão e a média sobe para
/// a fracção que está no topo, e a distância entre as duas é essa fracção.
fn skewed_field(g: &mut Graph, geom: NodeId) -> Option<NodeId> {
    let vn = g.add_node("value.noise");
    g.set_param(vn, "frequency", 0.17);
    g.set_param(vn, "speed", 0.0);
    g.set_param(vn, "octaves", 3.0);
    g.set_param(vn, "roughness", 0.6);
    g.set_param(vn, "amplitude", 1.0);
    g.set_param(vn, "seed", 5.0);
    wire(g, geom, 0, vn, 0)?;
    let cut = g.add_node("value.step");
    g.set_param(cut, "threshold", SOURCE_CUT);
    g.set_param(cut, "width", SOURCE_SOFTNESS);
    g.set_param(cut, "mode", 1.0); // Smooth
    wire(g, vn, 0, cut, 0)?;
    Some(cut)
}

/// O DEGRAU das bandas 5-8: a rampa ordinal cortada no meio.
fn step_field(g: &mut Graph, geom: NodeId) -> Option<NodeId> {
    let idx = g.add_node("value.instance_field");
    g.set_param(idx, "mode", FIELD_RAMP);
    wire(g, geom, 0, idx, 0)?;
    let st = g.add_node("value.step");
    g.set_param(st, "threshold", STEP_AT);
    g.set_param(st, "width", 0.0);
    g.set_param(st, "mode", 0.0); // Hard
    wire(g, idx, 0, st, 0)?;
    Some(st)
}

/// Monta a cadeia de valor de uma lane e devolve o nó terminal dela.
fn build_value(g: &mut Graph, kind: Kind, geom: NodeId) -> Option<NodeId> {
    Some(match kind {
        Kind::Field => skewed_field(g, geom)?,
        Kind::Step => step_field(g, geom)?,
        Kind::Smooth { weight } => {
            let src = step_field(g, geom)?;
            let vs = g.add_node("value.smooth");
            g.set_param(vs, "radius", SMOOTH_RADIUS);
            g.set_param(vs, "weight", weight);
            wire(g, src, 0, vs, 0)?;
            vs
        }
        Kind::Reduce { mode, port } => {
            let src = skewed_field(g, geom)?;
            let vr = g.add_node("value.reduce");
            g.set_param(vr, "mode", mode);
            wire(g, src, 0, vr, 0)?;
            match port {
                Port::None => {}
                Port::Mask => {
                    // A máscara é o PRÓPRIO campo acima de um degrau: "conte só a
                    // cauda". Ela vem de uma segunda cópia da fonte porque o
                    // grafo é uma DAG e o mesmo nó serviria — mas duas cópias
                    // deixam a lane inteira legível de cima a baixo.
                    let s2 = skewed_field(g, geom)?;
                    let st = g.add_node("value.step");
                    g.set_param(st, "threshold", MASK_AT);
                    g.set_param(st, "width", 0.0);
                    g.set_param(st, "mode", 0.0); // Hard
                    wire(g, s2, 0, st, 0)?;
                    wire(g, st, 0, vr, 1)?;
                }
                Port::Group => {
                    // Quatro bins por índice: `0 … N−1` quantizado por `N/4`
                    // devolve quatro ids distintos, que é tudo o que um group id
                    // precisa de ser.
                    let idx = g.add_node("value.instance_field");
                    g.set_param(idx, "mode", FIELD_INDEX);
                    wire(g, geom, 0, idx, 0)?;
                    let q = g.add_node("value.quantize");
                    g.set_param(q, "step", COLS / GROUP_BINS);
                    g.set_param(q, "mode", QUANTIZE_FLOOR);
                    wire(g, idx, 0, q, 0)?;
                    wire(g, q, 0, vr, 2)?;
                }
            }
            vr
        }
    })
}

/// Em quantos bins a banda 3 parte a fileira — para o anúncio dizer o número que
/// o artista tem de contar na escada, em vez de um literal repetido na mensagem.
pub(crate) const fn group_bins() -> f32 {
    GROUP_BINS
}

/// Uma aresta. Função LIVRE e não closure: uma closure que captura `g` o empresta
/// até ao fim do escopo.
fn wire(g: &mut Graph, a: NodeId, ap: u16, b: NodeId, bp: u16) -> Option<()> {
    g.connect(Edge {
        from: (a, ap),
        to: (b, bp),
        delayed: false,
    })
    .ok()
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_stats_tests.rs"]
mod tests;
