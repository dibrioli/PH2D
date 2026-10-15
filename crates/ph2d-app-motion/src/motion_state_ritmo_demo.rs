//! A cena `=132` — **O RITMO** (ciclo 17, [doc 126](../../../docs/Motion%20Nodes/126_ciclo_17_o_ritmo.md)).
//!
//! Duas galáxias de estrelas (a SIMULAÇÃO com campos: núcleo `motion.falloff` → `force.vortex` em
//! velocidade-alvo → `force.attractor`, a lei de órbita da cena das estrelas,
//! [`super::carimbo_demo::Galaxia`]), IGUAIS menos um botão: o `Order` do `Time` e do `Stagger`.
//! Uma onda acorda as estrelas a cada [`CICLO`] s — cada uma encolhe e volta a crescer quando a onda
//! a alcança:
//! - à ESQUERDA, `Order = Index`: a onda corre FILA A FILA (a ordem de sempre);
//! - à DIREITA, `Order = From Point`: a onda parte do CENTRO e corre em anéis — e como a distância é
//!   lida da posição SIMULADA, os anéis continuam redondos com a galáxia a girar.
//!
//! O `Time` dá o ATRASO (`delay`, moldado pela curva `Sine Out`); o `Stagger` dá a cada estrela uma
//! rotação pela MESMA ordem (`Back Out`, a ultrapassagem do GSAP).

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Estrelas por lado de cada galáxia.
pub(crate) const LADO: f32 = 16.0;
/// O vão da grelha de partida (metros).
pub(crate) const VAO: f32 = 0.32;
/// O tamanho da estrela desenhada.
const ESTRELA: f32 = 0.13;
/// Onde cada galáxia fica no mundo (`± COL_X`).
pub(crate) const COL_X: f32 = 4.5;
/// O atraso TOTAL da onda: a última estrela espera isto (segundos).
pub(crate) const ONDA: f32 = 2.4;
/// A onda repete-se a cada `CICLO` segundos (o `value.wrap` em `Repeat`).
pub(crate) const CICLO: f32 = 4.0;
/// Cada estrela cresce de `0` a `1` em `SUBIDA` segundos depois de a onda a alcançar.
pub(crate) const SUBIDA: f32 = 0.8;
/// O alcance do `From Point`: a onda chega à borda da galáxia.
pub(crate) const ALCANCE: f32 = 3.4;
/// A rotação que o `Stagger` distribui (graus). ⚠️ METADE da simetria da estrela (`72°`): girar
/// `72°` uma estrela de cinco pontas devolve a MESMA estrela, e a onda do `Stagger` ficava
/// invisível (foto de 09/10).
const GIRO: f32 = 36.0;
/// As duas ordens — a ÚNICA diferença entre as colunas.
pub(crate) const ORDENS: [f32; 2] = [
    ph2d_ease::order::ORDER_INDEX as f32,
    ph2d_ease::order::ORDER_FROM_POINT as f32,
];

/// A galáxia deste campo: o núcleo cobre o canto (`~3,4 m`), e o ímã sai da lei de órbita
/// (`s_a = s_v²/núcleo`, ver [`super::carimbo_demo::Galaxia`]): uma volta a cada `~31 s`.
const NUCLEO: f32 = 4.0;
pub(crate) const VORTEX: f32 = 0.8;
const RESISTENCIA: f32 = 5.0;
const IMA: f32 = 0.158;
const ALCANCE_FORCA: f32 = 200.0;

fn liga(g: &mut Graph, de: NodeId, para: NodeId, porta: u16, pre: bool) -> Option<()> {
    g.connect(Edge {
        from: (de, 0),
        to: (para, porta),
        delayed: pre,
    })
    .ok()
}

fn indice_da_estrela() -> Option<f32> {
    let i = ph2d_node_motion_shape::ALL_KINDS
        .iter()
        .position(|k| *k == ph2d_node_motion_shape::ShapeKind::Star)?;
    #[expect(
        clippy::cast_precision_loss,
        reason = "um indice de enum, sempre pequeno"
    )]
    Some(i as f32)
}

/// ⭐ **Uma galáxia de `LADO²` pontos a girar** — a grelha de partida e a SIMULAÇÃO com campos
/// (núcleo → redemoinho em velocidade-alvo → ímã → `Integrate`), com os cartões na fileira `y0`.
/// Partilhada pelas cenas `=132` (o ritmo) e `=133` (a cor). Devolve o `Integrate`.
pub(super) fn galaxia(g: &mut Graph, y0: f32) -> Option<NodeId> {
    galaxia_e_redemoinho(g, y0).map(|(galaxia, _)| galaxia)
}

/// A [`galaxia`] e o redemoinho dela (o `force.vortex`) — a cena `=134` (ciclo 19) dirige-lhe a
/// força por um gatilho. Devolve o `Integrate` e o redemoinho.
pub(super) fn galaxia_e_redemoinho(g: &mut Graph, y0: f32) -> Option<(NodeId, NodeId)> {
    let no = |g: &mut Graph, tipo: &str, x: f32, dy: f32| {
        let n = g.add_node(tipo.to_string());
        g.set_pos(n, Pos { x, y: y0 + dy });
        n
    };
    let grade = no(g, "motion.grid", -660.0, 0.0);
    g.set_param(grade, "rows", LADO);
    g.set_param(grade, "cols", LADO);
    g.set_param(grade, "gap_x", VAO);
    g.set_param(grade, "gap_y", VAO);
    let galaxia = no(g, "motion.integrate", -440.0, 0.0);
    let nucleo = no(g, "motion.falloff", -880.0, -110.0);
    g.set_param(nucleo, "shape", 0.0);
    g.set_param(nucleo, "curve", 0.0);
    g.set_param(nucleo, "radius", NUCLEO);
    g.set_param(nucleo, "invert", 1.0);
    let vortex = no(g, "force.vortex", -660.0, -110.0);
    g.set_param(vortex, "strength", VORTEX);
    g.set_param(vortex, "radius", ALCANCE_FORCA);
    g.set_param(vortex, ph2d_node_force_vortex::MODE, 1.0);
    g.set_param(vortex, ph2d_node_force_vortex::AIR_RESIST, RESISTENCIA);
    let ima = no(g, "force.attractor", -440.0, -110.0);
    g.set_param(ima, "strength", IMA);
    g.set_param(ima, "radius", ALCANCE_FORCA);
    liga(g, grade, galaxia, 0, false)?;
    liga(g, galaxia, nucleo, 0, true)?;
    liga(g, nucleo, vortex, 0, false)?;
    liga(g, vortex, ima, 0, false)?;
    liga(g, ima, galaxia, 1, false)?;
    Some((galaxia, vortex))
}

/// Os nós que os gates leem de uma coluna (o produto só usa a `saida`).
#[cfg_attr(not(test), expect(dead_code, reason = "lidos pelos gates da cena"))]
pub(crate) struct Coluna {
    pub saida: NodeId,
    pub tempo: NodeId,
    pub stagger: NodeId,
    pub galaxia: NodeId,
}

/// Uma galáxia com a sua onda. `k = 0` à esquerda, `1` à direita.
fn coluna(g: &mut Graph, k: usize) -> Option<Coluna> {
    #[expect(clippy::cast_precision_loss, reason = "duas colunas")]
    let y0 = 330.0 * k as f32;
    let no = |g: &mut Graph, tipo: &str, x: f32, dy: f32| {
        let n = g.add_node(tipo.to_string());
        g.set_pos(n, Pos { x, y: y0 + dy });
        n
    };
    // ── AS POSIÇÕES, e a SIMULAÇÃO COM CAMPOS que as faz girar.
    let galaxia = galaxia(g, y0)?;

    // ── A ONDA NO TEMPO: o atraso de cada estrela pela ORDEM, em laço, e o crescimento.
    let ordem = ORDENS[k];
    let tempo = no(g, "value.time", -220.0, -110.0);
    for (p, v) in [
        ("delay", ONDA),
        ("ease_curve", ph2d_ease::SINE as f32),
        ("ease_dir", 1.0),
        ("order", ordem),
        ("reach", ALCANCE),
    ] {
        g.set_param(tempo, p, v);
    }
    liga(g, galaxia, tempo, 0, false)?;
    let laco = no(g, "value.wrap", 0.0, -110.0);
    g.set_param(laco, "mode", 1.0);
    g.set_param(laco, "lo", 0.0);
    g.set_param(laco, "hi", CICLO);
    liga(g, tempo, laco, 0, false)?;
    let cresce = no(g, "value.map_range", 220.0, -110.0);
    for (p, v) in [
        ("in_lo", 0.0),
        ("in_hi", SUBIDA),
        ("out_lo", 0.0),
        ("out_hi", 1.0),
        ("clamp", 1.0),
    ] {
        g.set_param(cresce, p, v);
    }
    liga(g, laco, cresce, 0, false)?;
    let tamanho = no(g, "motion.drive", -220.0, 0.0);
    g.set_param(tamanho, "channel", 3.0);
    g.set_param(tamanho, "mode", 1.0);
    liga(g, galaxia, tamanho, 0, false)?;
    liga(g, cresce, tamanho, 1, false)?;

    // ── A ONDA NO VALOR: a rotação de cada estrela pela MESMA ordem.
    let stagger = no(g, "motion.stagger", 0.0, 0.0);
    for (p, v) in [
        ("channel", 2.0),
        ("min", 0.0),
        ("max", GIRO),
        ("ease_curve", ph2d_ease::BACK as f32),
        ("ease_dir", 1.0),
        ("order", ordem),
        ("reach", ALCANCE),
    ] {
        g.set_param(stagger, p, v);
    }
    liga(g, tamanho, stagger, 0, false)?;

    // ── A FORMA: uma estrela, vestida em cada ponto.
    let forma = no(g, "source.shape", 0.0, 160.0);
    use ph2d_node_motion_shape::param as p;
    let cor = if k == 0 {
        [0.95, 0.62, 0.25]
    } else {
        [0.35, 0.75, 0.98]
    };
    for (chave, valor) in [
        (p::KIND, indice_da_estrela()?),
        (p::SIZE, ESTRELA),
        (p::FILL, 1.0),
        (p::FILL_R, cor[0]),
        (p::FILL_G, cor[1]),
        (p::FILL_B, cor[2]),
        (p::FILL_A, 1.0),
    ] {
        g.set_param(forma, chave, valor);
    }
    let pecas = no(g, "motion.duplicator", 220.0, 100.0);
    // ⚠️ A escala do PONTO entra inteira — sem isto a forma manda no tamanho e a onda do `Time`
    // não se vê (foto de 09/10: as 800 estrelas do mesmo tamanho).
    g.set_param(pecas, "point_scale", 1.0);
    liga(g, forma, pecas, 0, false)?;
    liga(g, stagger, pecas, 1, false)?;
    let lugar = no(g, "motion.transform", 440.0, 100.0);
    #[expect(clippy::cast_precision_loss, reason = "duas colunas")]
    g.set_param(lugar, "offset_x", (k as f32 * 2.0 - 1.0) * COL_X);
    liga(g, pecas, lugar, 0, false)?;
    let saida = no(g, "motion.output", 660.0, 100.0);
    liga(g, lugar, saida, 0, false)?;
    Some(Coluna {
        saida,
        tempo,
        stagger,
        galaxia,
    })
}

/// O documento e as duas colunas.
pub(crate) fn monta(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<[Coluna; 2]> {
    let g = &mut doc.graph;
    let a = coluna(g, 0)?;
    let b = coluna(g, 1)?;
    g.validate(reg).ok()?;
    Some([a, b])
}

pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    monta(doc, reg).map(|[a, b]| vec![a.saida, b.saida])
}

/// As duas fichas, uma sobre cada galáxia.
pub(super) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    let ficha = |x: f32, t: &str| crate::motion_demo_legend::Caption::new([x, -4.0], t.to_string());
    vec![
        ficha(-COL_X, "Order = Index: row by row"),
        ficha(COL_X, "Order = From Point: from the center out"),
    ]
}

/// **O roteiro que o dono segue** (`CLAUDE.md` §0.8).
pub(super) fn announce() {
    eprintln!(
        "\n[o ritmo] DUAS GALAXIAS de estrelas a girar devagar: e' uma SIMULACAO com campos de forca\n\
         (um redemoinho e um iman). A cada 4 segundos passa uma ONDA: cada estrela encolhe e volta a\n\
         crescer quando a onda chega a ela, e gira um pouco. As duas sao IGUAIS, menos um botao.\n\
         \n\
         (1) A da ESQUERDA (laranja): a onda corre FILA A FILA, de um canto ao outro.\n\
         (2) A da DIREITA (azul): a onda parte do CENTRO e corre em ANEIS para fora.\n\
         (3) No grafo ha' duas filas de cartoes. Na fila de CIMA (a da laranja), clique no cartao\n    \
         `Time` e mude `Order` para `From Point`. A laranja passa a ondular em aneis como a azul.\n\
         (4) No mesmo cartao, mude `Ease` para `Steps`: a onda passa a andar aos SALTOS (anel a\n    \
         anel). Mude para `Elastic`: as estrelas da borda acordam quase juntas, num tremor.\n\
         \n\
         DEU ERRADO se: a azul ondular fila a fila; se mudar o `Order` da laranja nao mudar nada;\n\
         ou se as galaxias pararem de girar.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_ritmo_demo_tests.rs"]
mod tests;
