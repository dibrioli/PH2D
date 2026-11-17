//! A cena `=143` — **o herói manda** (GEF 08 Motion Input, onda 3; o grafo de `PH2D_GAME_SMOKE=28`).
//!
//! - **`vida`** → a barra de 24 peças da `=134`, com um `Send signal` no fio: `low_health` sai UMA vez
//!   quando a vida cai abaixo de `0.25`.
//! - **`rastro`** (o par Velocity do herói, portas `x`/`y`) → dois `Wind` (um por eixo) sobre um rastro
//!   SIMULADO de estrelas que um emissor solta: o rastro aponta para onde o herói corre.

use super::jogo_demo::{carimbo, entrada, liga, no_em};
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Graph, NodeId};

pub(crate) const SINAL: &str = "low_health";
pub(crate) const LIMIAR: f32 = 0.25;
pub(crate) const RASTRO: &str = "rastro";
/// A aceleração do vento por m/s do herói, até `RAPIDEZ_MAX`.
pub(crate) const VENTO: f32 = 1.5;
pub(crate) const RAPIDEZ_MAX: f32 = 6.0;
/// Onde o rastro nasce no mundo (à direita, acima do chão da cena 28).
pub(crate) const RASTRO_AT: [f32; 2] = [4.5, -0.4];
/// As portas `x` e `y` do `value.input` (MI-5, no fim do manifesto).
pub(crate) const PORTAS_XY: [u16; 2] = [7, 8];

#[cfg_attr(not(test), expect(dead_code, reason = "lidos pelos gates da cena"))]
pub(crate) struct Cena {
    pub barra: NodeId,
    pub vida: NodeId,
    pub sinal: NodeId,
    pub remap: NodeId,
    pub rastro: NodeId,
    pub ventos: [NodeId; 2],
    pub saida_rastro: NodeId,
}

fn trilha(g: &mut Graph) -> Option<(NodeId, NodeId, [NodeId; 2])> {
    let no = no_em(660.0);
    let emissor = no(g, "motion.emitter", -880.0, 0.0);
    for (chave, v) in [
        ("rate", 40.0),
        ("life", 1.2),
        ("max", 48.0),
        ("size", 0.14),
        ("size_random", 0.3),
        ("speed", 0.25),
        ("speed_random", 0.5),
        ("spread", 360.0),
        ("seed", 7.0),
    ] {
        g.set_param(emissor, chave, v);
    }
    let rastro = entrada(
        g,
        &no,
        RASTRO,
        ph2d_node_value_input::KIND_NUMBER,
        -1100.0,
        -220.0,
    );
    let integra = no(g, "motion.integrate", -220.0, 0.0);
    liga(g, (emissor, 0), integra, 0, false)?;
    let mut de = (integra, 0);
    let mut pre = true;
    let mut ventos = [integra; 2];
    for (i, angulo) in [0.0, 90.0].into_iter().enumerate() {
        let x = -660.0 + 220.0 * i as f32;
        let vento = no(g, "force.wind", x, -110.0);
        g.set_param(vento, "angle", angulo);
        g.set_param(vento, "strength", 0.0);
        g.set_param(vento, "gust", 0.0);
        let escala = no(g, "value.map_range", x, -220.0);
        let (lo, hi) = (-RAPIDEZ_MAX, RAPIDEZ_MAX);
        for (p, v) in [
            ("in_lo", lo),
            ("in_hi", hi),
            ("out_lo", lo * VENTO),
            ("out_hi", hi * VENTO),
            ("clamp", 1.0),
        ] {
            g.set_param(escala, p, v);
        }
        liga(g, (rastro, PORTAS_XY[i]), escala, 0, false)?;
        g.drive_param(vento, "strength", (escala, 0)).ok()?;
        liga(g, de, vento, 0, pre)?;
        (de, pre, ventos[i]) = ((vento, 0), false, vento);
    }
    liga(g, de, integra, 1, false)?;
    let estrelas = carimbo(g, [1.0, 0.75, 0.25], 0.0, 760.0)?;
    liga(g, (integra, 0), estrelas, 1, false)?;
    let lugar = no(g, "motion.transform", 220.0, 100.0);
    g.set_param(lugar, "offset_x", RASTRO_AT[0]);
    g.set_param(lugar, "offset_y", RASTRO_AT[1]);
    liga(g, (estrelas, 0), lugar, 0, false)?;
    let saida = no(g, "motion.output", 440.0, 100.0);
    liga(g, (lugar, 0), saida, 0, false)?;
    Some((saida, rastro, ventos))
}

pub(crate) fn monta(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Cena> {
    let g = &mut doc.graph;
    let (barra, vida, remap) = super::jogo_demo::barra(g)?;
    let sinal = no_em(0.0)(g, ph2d_node_send_signal::MANIFEST.name, 0.0, -330.0);
    g.set_text_param(sinal, ph2d_node_send_signal::NAME_KEY, SINAL);
    let falls = ph2d_node_send_signal::WHEN_FALLS as f32;
    g.set_param(sinal, ph2d_node_send_signal::WHEN, falls);
    g.set_param(sinal, ph2d_node_send_signal::LEVEL, LIMIAR);
    liga(g, (vida, 0), sinal, 0, false)?;
    g.drive_param(remap, ph2d_nodegraph::time::TIME_DRIVE_PARAM, (sinal, 0))
        .ok()?;
    let (saida_rastro, rastro, ventos) = trilha(g)?;
    g.validate(reg).ok()?;
    Some(Cena {
        barra,
        vida,
        sinal,
        remap,
        rastro,
        ventos,
        saida_rastro,
    })
}

pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    monta(doc, reg).map(|c| vec![c.barra, c.saida_rastro])
}

pub(super) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    let ficha =
        |x: f32, y: f32, t: &str| crate::motion_demo_legend::Caption::new([x, y], t.to_string());
    let y = super::jogo_demo::BARRA_Y - 0.6;
    vec![
        ficha(
            -0.65,
            y,
            "vida: Hero's health; under 0.25 it sends low_health",
        ),
        ficha(
            RASTRO_AT[0],
            RASTRO_AT[1] - 1.6,
            "rastro: Hero's velocity (x, y) blows the stars (Wind)",
        ),
    ]
}

/// **O roteiro que o dono segue** (`CLAUDE.md` §0.8).
pub(super) fn announce() {
    eprintln!(
        "\n[o heroi manda] DOIS quadros, mandados pelo Hero (PH2D_GAME_SMOKE=28).\n\
         (1) Em CIMA: a barra verde e a vida do Hero (`vida`). Abaixo de 0.25 o no `Send signal`\n    \
         manda `low_health` uma vez.\n\
         (2) A DIREITA: estrelas que um emissor solta (uma SIMULACAO com dois ventos). Em Play,\n    \
         ande com o Hero: as estrelas voam para o lado para onde ele corre (`rastro`).\n\
         DEU ERRADO se: a barra nao descer com a vida, ou as estrelas nao seguirem a corrida.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_heroi_demo_tests.rs"]
mod tests;
