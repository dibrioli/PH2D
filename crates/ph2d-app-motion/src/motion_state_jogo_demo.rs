//! A cena `=134` — **O JOGO MANDA** (ciclo 19, [doc 128](../../../docs/Motion%20Nodes/128_ciclo_19_o_jogo_manda.md)).
//!
//! Quatro entradas publicadas (`value.input`) mandam em três quadros — e o número de cada CARTÃO é
//! o «jogo» do artista enquanto não há jogo:
//! - **`vida`** (Number) → em CIMA, uma barra de 24 peças que enche: a cascata da barra é cozida
//!   debaixo de um `Time Remap` em `Driven` — o tempo dela é a vida (o Blend 1D do Rive);
//! - **`estado`** (Toggle) → `Follow` (Linear, `0,6 s`) → o `Blend` de um `Mixer`: à ESQUERDA, uma
//!   grelha que se cruza num anel; **`time`** (Color) pinta-a pelos canais do `Tint`;
//! - **`boom`** (Trigger) → `ADSR` → a galáxia SIMULADA à DIREITA (os campos da `=132`): o envelope
//!   acelera o REDEMOINHO (a força do `Vortex`, um campo da simulação) e incha as estrelas — e o
//!   redemoinho em velocidade-alvo devolve a calma em 2 s. ⛔ Um empurrão radial foi MEDIDO e
//!   recusado (doc 128 §5.3): a galáxia não volta, e três disparos tiram-na do ecrã.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// As 24 peças da barra.
pub(crate) const PECAS_DA_BARRA: f32 = 24.0;
/// A cascata da barra: a última peça começa a crescer aos `ONDA` segundos e cada uma leva `SOBE`.
const ONDA: f32 = 2.0;
const SOBE: f32 = 0.2;
/// O lado de uma peça da barra (cheia).
pub(crate) const PECA: f32 = 0.12;
/// O `duration` do `Driven`: a vida `1` mostra o fim da cascata.
pub(crate) const DURACAO: f32 = ONDA + SOBE;
/// Onde cada quadro fica no mundo.
pub(crate) const BARRA_Y: f32 = 2.6;
pub(crate) const COL_X: f32 = super::ritmo_demo::COL_X;
const QUADROS_Y: f32 = -0.6;
/// O crossfade: `0,6 s` de um estado ao outro (o `xfade_time`).
pub(crate) const CRUZA: f32 = 0.6;
/// Quantas vezes o redemoinho acelera no pico do envelope (doc 128 §5.3: medido).
pub(crate) const EXPLOSAO: f32 = 6.0;
/// Quantas vezes as estrelas incham no pico (o clarão).
pub(crate) const CLARAO: f32 = 2.5;
/// Os valores de fábrica dos cartões.
pub(crate) const VIDA: f32 = 0.6;
pub(crate) const COR_DO_TIME: [f32; 4] = [1.0, 0.45, 0.1, 1.0];

fn liga(g: &mut Graph, de: (NodeId, u16), para: NodeId, porta: u16, pre: bool) -> Option<()> {
    g.connect(Edge {
        from: de,
        to: (para, porta),
        delayed: pre,
    })
    .ok()
}

fn indice(k: ph2d_node_motion_shape::ShapeKind) -> Option<f32> {
    let i = ph2d_node_motion_shape::ALL_KINDS
        .iter()
        .position(|x| *x == k)?;
    #[expect(clippy::cast_precision_loss, reason = "um indice de enum")]
    Some(i as f32)
}

/// Uma forma de cor `rgb`.
fn forma(
    g: &mut Graph,
    no: &impl Fn(&mut Graph, &str, f32, f32) -> NodeId,
    k: ph2d_node_motion_shape::ShapeKind,
    tamanho: f32,
    rgb: [f32; 3],
    x: f32,
    dy: f32,
) -> Option<NodeId> {
    use ph2d_node_motion_shape::param as p;
    let f = no(g, "source.shape", x, dy);
    for (chave, valor) in [
        (p::KIND, indice(k)?),
        (p::SIZE, tamanho),
        (p::FILL, 1.0),
        (p::FILL_R, rgb[0]),
        (p::FILL_G, rgb[1]),
        (p::FILL_B, rgb[2]),
        (p::FILL_A, 1.0),
    ] {
        g.set_param(f, chave, valor);
    }
    Some(f)
}

fn entrada(
    g: &mut Graph,
    no: &impl Fn(&mut Graph, &str, f32, f32) -> NodeId,
    nome: &str,
    kind: i32,
    x: f32,
    dy: f32,
) -> NodeId {
    let n = no(g, "value.input", x, dy);
    g.set_text_param(n, ph2d_node_value_input::NAME_KEY, nome);
    #[expect(clippy::cast_precision_loss, reason = "um indice de enum")]
    g.set_param(n, ph2d_node_value_input::KIND, kind as f32);
    n
}

/// Os nós que os gates e o tutorial leem.
#[cfg_attr(not(test), expect(dead_code, reason = "lidos pelos gates da cena"))]
pub(crate) struct Cena {
    pub barra: NodeId,
    pub cruza: NodeId,
    pub galaxia: NodeId,
    pub vida: NodeId,
    pub estado: NodeId,
    pub time: NodeId,
    pub boom: NodeId,
    pub remap: NodeId,
    pub seguir: NodeId,
    pub mixer: NodeId,
    pub redemoinho: NodeId,
}

fn no_em(y0: f32) -> impl Fn(&mut Graph, &str, f32, f32) -> NodeId {
    move |g: &mut Graph, tipo: &str, x: f32, dy: f32| {
        let n = g.add_node(tipo.to_string());
        g.set_pos(n, Pos { x, y: y0 + dy });
        n
    }
}

/// ── A BARRA: 24 peças que crescem em cascata — o tempo dela é a `vida`.
fn barra(g: &mut Graph) -> Option<(NodeId, NodeId, NodeId)> {
    let no = no_em(0.0);
    let grade = no(g, "motion.grid", -660.0, 0.0);
    g.set_param(grade, "rows", 1.0);
    g.set_param(grade, "cols", PECAS_DA_BARRA);
    g.set_param(grade, "gap_x", 0.3);
    let tempo = no(g, "value.time", -440.0, -110.0);
    g.set_param(tempo, "delay", ONDA);
    g.set_param(tempo, "order", ph2d_ease::order::ORDER_INDEX as f32);
    liga(g, (grade, 0), tempo, 0, false)?;
    let cresce = no(g, "value.map_range", -220.0, -110.0);
    for (p, v) in [
        ("in_lo", 0.0),
        ("in_hi", SOBE),
        ("out_lo", 0.0),
        ("out_hi", 1.0),
        ("clamp", 1.0),
    ] {
        g.set_param(cresce, p, v);
    }
    liga(g, (tempo, 0), cresce, 0, false)?;
    let tamanho = no(g, "motion.drive", -220.0, 0.0);
    g.set_param(tamanho, "channel", 3.0);
    g.set_param(tamanho, "mode", 1.0);
    liga(g, (grade, 0), tamanho, 0, false)?;
    liga(g, (cresce, 0), tamanho, 1, false)?;
    let f = forma(
        g,
        &no,
        ph2d_node_motion_shape::ShapeKind::Rectangle,
        PECA,
        [0.3, 0.9, 0.45],
        0.0,
        120.0,
    )?;
    // Peças ALTAS e separadas (a foto de 10/10: quadradas de 0,26 com vão de 0,3 eram uma faixa).
    g.set_param(f, ph2d_node_motion_shape::param::ASPECT, 2.2);
    let pecas = no(g, "motion.duplicator", 0.0, 0.0);
    g.set_param(pecas, "point_scale", 1.0);
    liga(g, (f, 0), pecas, 0, false)?;
    liga(g, (tamanho, 0), pecas, 1, false)?;
    let remap = no(g, "motion.time_remap", 220.0, 0.0);
    g.set_param(
        remap,
        "mode",
        f32::from(ph2d_nodegraph::time::TimeMode::Driven.index()),
    );
    g.set_param(remap, "duration", DURACAO);
    g.set_param(remap, ph2d_nodegraph::time::TIME_DRIVE_PARAM, VIDA);
    liga(g, (pecas, 0), remap, 0, false)?;
    let vida = entrada(
        g,
        &no,
        "vida",
        ph2d_node_value_input::KIND_NUMBER,
        0.0,
        -220.0,
    );
    g.set_param(vida, ph2d_node_value_input::VALUE_PARAM, VIDA);
    g.drive_param(remap, ph2d_nodegraph::time::TIME_DRIVE_PARAM, (vida, 0))
        .ok()?;
    let lugar = no(g, "motion.transform", 440.0, 0.0);
    g.set_param(lugar, "offset_y", BARRA_Y);
    liga(g, (remap, 0), lugar, 0, false)?;
    let saida = no(g, "motion.output", 660.0, 0.0);
    liga(g, (lugar, 0), saida, 0, false)?;
    Some((saida, vida, remap))
}

/// ── O CRUZAMENTO: uma grelha e um anel, e o `estado` decide qual (pelo `Follow`).
fn cruzamento(g: &mut Graph) -> Option<[NodeId; 5]> {
    let no = no_em(330.0);
    let grade = no(g, "motion.grid", -660.0, 0.0);
    g.set_param(grade, "rows", 6.0);
    g.set_param(grade, "cols", 6.0);
    g.set_param(grade, "gap_x", 0.5);
    g.set_param(grade, "gap_y", 0.5);
    let anel = no(g, "motion.distribute_radial", -660.0, 110.0);
    g.set_param(anel, "count", 36.0);
    g.set_param(anel, "rings", 1.0);
    g.set_param(anel, "radius", 1.6);
    let estado = entrada(
        g,
        &no,
        "estado",
        ph2d_node_value_input::KIND_TOGGLE,
        -660.0,
        -110.0,
    );
    let seguir = no(g, "value.follow", -440.0, -110.0);
    g.set_param(seguir, ph2d_node_value_follow::DURATION, CRUZA);
    liga(g, (estado, 0), seguir, 0, false)?;
    liga(g, (seguir, 0), seguir, 1, true)?;
    let mixer = no(g, "motion.mixer", -440.0, 0.0);
    g.set_param(mixer, "mode", 2.0);
    liga(g, (grade, 0), mixer, 0, false)?;
    liga(g, (anel, 0), mixer, 1, false)?;
    liga(g, (seguir, 0), mixer, 4, false)?;
    let f = forma(
        g,
        &no,
        ph2d_node_motion_shape::ShapeKind::Circle,
        0.2,
        [1.0, 1.0, 1.0],
        -220.0,
        120.0,
    )?;
    let pecas = no(g, "motion.duplicator", -220.0, 0.0);
    liga(g, (f, 0), pecas, 0, false)?;
    liga(g, (mixer, 0), pecas, 1, false)?;
    let time = entrada(
        g,
        &no,
        "time",
        ph2d_node_value_input::KIND_COLOR,
        0.0,
        -110.0,
    );
    for (k, c) in ph2d_node_value_input::COLOR.into_iter().enumerate() {
        g.set_param(time, c, COR_DO_TIME[k]);
    }
    let tint = no(g, "motion.tint", 0.0, 0.0);
    for (k, c) in ["r", "g", "b"].into_iter().enumerate() {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "as portas r g b do value.input"
        )]
        g.drive_param(tint, c, (time, 1 + k as u16)).ok()?;
    }
    liga(g, (pecas, 0), tint, 0, false)?;
    let lugar = no(g, "motion.transform", 220.0, 0.0);
    g.set_param(lugar, "offset_x", -COL_X);
    g.set_param(lugar, "offset_y", QUADROS_Y);
    liga(g, (tint, 0), lugar, 0, false)?;
    let saida = no(g, "motion.output", 440.0, 0.0);
    liga(g, (lugar, 0), saida, 0, false)?;
    Some([saida, estado, time, seguir, mixer])
}

/// ── O BOOM: a galáxia simulada e o redemoinho dela, que o `boom` acelera por um envelope.
fn explosao(g: &mut Graph) -> Option<[NodeId; 4]> {
    let y0 = 660.0;
    let no = no_em(y0);
    let (galaxia, redemoinho) = super::ritmo_demo::galaxia_e_redemoinho(g, y0)?;
    let boom = entrada(
        g,
        &no,
        "boom",
        ph2d_node_value_input::KIND_TRIGGER,
        -880.0,
        -330.0,
    );
    let envelope = no(g, "pulse.adsr", -660.0, -330.0);
    for (p, v) in [
        ("attack", 0.03),
        ("decay", 0.15),
        ("sustain", 0.5),
        ("hold", 0.1),
        ("release", 0.5),
    ] {
        g.set_param(envelope, p, v);
    }
    liga(g, (boom, 5), envelope, 0, false)?;
    liga(g, (envelope, 0), envelope, 1, true)?;
    let forca = no(g, "value.map_range", -440.0, -330.0);
    let base = super::ritmo_demo::VORTEX;
    for (p, v) in [
        ("in_lo", 0.0),
        ("in_hi", 1.0),
        ("out_lo", base),
        ("out_hi", base * EXPLOSAO),
        ("clamp", 1.0),
    ] {
        g.set_param(forca, p, v);
    }
    liga(g, (envelope, 0), forca, 0, false)?;
    g.drive_param(redemoinho, "strength", (forca, 0)).ok()?;
    let f = forma(
        g,
        &no,
        ph2d_node_motion_shape::ShapeKind::Star,
        0.13,
        [0.35, 0.75, 0.98],
        0.0,
        160.0,
    )?;
    // O CLARÃO: o mesmo envelope incha as estrelas (`Size × 1 → CLARAO`).
    let incha = no(g, "value.map_range", -440.0, -440.0);
    for (p, v) in [
        ("in_lo", 0.0),
        ("in_hi", 1.0),
        ("out_lo", 1.0),
        ("out_hi", CLARAO),
        ("clamp", 1.0),
    ] {
        g.set_param(incha, p, v);
    }
    liga(g, (envelope, 0), incha, 0, false)?;
    let clarao = no(g, "motion.drive", 0.0, 0.0);
    g.set_param(clarao, "channel", 3.0);
    g.set_param(clarao, "mode", 2.0);
    liga(g, (galaxia, 0), clarao, 0, false)?;
    liga(g, (incha, 0), clarao, 1, false)?;
    let pecas = no(g, "motion.duplicator", 220.0, 100.0);
    g.set_param(pecas, "point_scale", 1.0);
    liga(g, (f, 0), pecas, 0, false)?;
    liga(g, (clarao, 0), pecas, 1, false)?;
    let lugar = no(g, "motion.transform", 440.0, 100.0);
    g.set_param(lugar, "offset_x", COL_X);
    g.set_param(lugar, "offset_y", QUADROS_Y);
    liga(g, (pecas, 0), lugar, 0, false)?;
    let saida = no(g, "motion.output", 660.0, 100.0);
    liga(g, (lugar, 0), saida, 0, false)?;
    Some([saida, boom, galaxia, redemoinho])
}

/// O documento e os três quadros.
pub(crate) fn monta(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Cena> {
    let g = &mut doc.graph;
    let (barra, vida, remap) = barra(g)?;
    let [cruza, estado, time, seguir, mixer] = cruzamento(g)?;
    let [galaxia, boom, sim, redemoinho] = explosao(g)?;
    let _ = sim;
    g.validate(reg).ok()?;
    Some(Cena {
        barra,
        cruza,
        galaxia,
        vida,
        estado,
        time,
        boom,
        remap,
        seguir,
        mixer,
        redemoinho,
    })
}

pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    monta(doc, reg).map(|c| vec![c.barra, c.cruza, c.galaxia])
}

/// As três fichas.
pub(super) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    let ficha = |x: f32, y: f32, t: &str| crate::motion_demo_legend::Caption {
        text: t.to_string(),
        world: [x, y],
    };
    vec![
        ficha(
            0.0,
            BARRA_Y - 0.6,
            "vida: o tempo da barra e' a vida (Time Remap = Driven)",
        ),
        ficha(
            -COL_X,
            -4.0,
            "estado: a grelha cruza no anel (Follow -> Mixer)",
        ),
        ficha(
            COL_X,
            -4.0,
            "boom: cada disparo sacode a galaxia (ADSR -> Vortex)",
        ),
    ]
}

/// **O roteiro que o dono segue** (`CLAUDE.md` §0.8).
pub(super) fn announce() {
    eprintln!(
        "\n[o jogo manda] TRES QUADROS, mandados por QUATRO entradas que o jogo publica. Sem jogo,\n\
         manda o numero de cada CARTAO `Input` no grafo.\n\
         \n\
         (1) Em CIMA: uma barra verde de 24 pecas, cheia ate 60%. No cartao `Input` chamado\n    \
         `vida`, arraste `Value`: a barra enche e esvazia com o numero.\n\
         (2) A ESQUERDA: uma grelha de discos laranja. No cartao `estado`, ligue `On`: em meio\n    \
         segundo a grelha vira um anel; desligue e ela volta. No cartao `time`, mude a cor.\n\
         (3) A DIREITA: uma galaxia de estrelas azuis a girar devagar (uma SIMULACAO com campos).\n    \
         No cartao `boom`, clique `Fire`: as estrelas incham e o redemoinho acelera; em dois\n    \
         segundos tudo volta a calma.\n\
         (4) Toque a cena, mexa nos cartoes, PARE e volte a fita: o que aconteceu repete-se\n    \
         igual, no mesmo instante.\n\
         \n\
         DEU ERRADO se: a barra nao seguir o `Value`; a grelha saltar de uma vez para o anel;\n\
         o `Fire` nao mexer na galaxia; ou se, ao voltar a fita, a cena fizer outra coisa.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_jogo_demo_tests.rs"]
mod tests;

/// A sonda do relógio do grupo (doc 128 §5.4).
#[cfg(test)]
#[path = "motion_state_jogo_relogio.rs"]
mod relogio;
