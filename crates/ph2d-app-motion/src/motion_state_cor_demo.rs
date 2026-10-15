//! A cena `=133` — **A COR QUE O OLHO VÊ** (ciclo 18, [doc 127](../../../docs/Motion%20Nodes/127_ciclo_18_a_cor_que_o_olho_ve.md)).
//!
//! Duas galáxias de estrelas (a SIMULAÇÃO com campos da `=132`, [`super::ritmo_demo::galaxia`]),
//! pintadas por um `Tint` em degradê de AZUL para AMARELO pela ordem das estrelas, IGUAIS menos um
//! botão: o `Color Space` do `Tint`.
//! - à ESQUERDA, `Linear RGB`: o meio do degradê passa pelo CINZA (a croma cai a zero — medido);
//! - à DIREITA, `OKLCH Shorter`: o meio fica VERDE-AZULADO vivo (croma `0,126` contra `0,007`),
//!   porque a matiz roda pelo círculo.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Onde cada galáxia fica no mundo (`± COL_X`) — a mesma folga da `=132`.
pub(crate) const COL_X: f32 = super::ritmo_demo::COL_X;
/// As duas cores do degradê (RGB linear).
pub(crate) const AZUL: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
pub(crate) const AMARELO: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
/// Os dois espaços — a ÚNICA diferença entre as colunas.
pub(crate) const ESPACOS: [f32; 2] = [0.0, 2.0];
const ESTRELA: f32 = 0.15;

fn liga(g: &mut Graph, de: NodeId, para: NodeId, porta: u16) -> Option<()> {
    g.connect(Edge {
        from: (de, 0),
        to: (para, porta),
        delayed: false,
    })
    .ok()
}

fn indice_do_circulo() -> Option<f32> {
    let i = ph2d_node_motion_shape::ALL_KINDS
        .iter()
        .position(|k| *k == ph2d_node_motion_shape::ShapeKind::Circle)?;
    #[expect(clippy::cast_precision_loss, reason = "um indice de enum")]
    Some(i as f32)
}

/// Os nós que os gates leem de uma coluna.
#[cfg_attr(not(test), expect(dead_code, reason = "lidos pelos gates da cena"))]
pub(crate) struct Coluna {
    pub saida: NodeId,
    pub tint: NodeId,
}

fn coluna(g: &mut Graph, k: usize) -> Option<Coluna> {
    #[expect(clippy::cast_precision_loss, reason = "duas colunas")]
    let y0 = 330.0 * k as f32;
    let no = |g: &mut Graph, tipo: &str, x: f32, dy: f32| {
        let n = g.add_node(tipo.to_string());
        g.set_pos(n, Pos { x, y: y0 + dy });
        n
    };
    let galaxia = super::ritmo_demo::galaxia(g, y0)?;
    // ── A FORMA: um disco BRANCO (o `Tint` multiplica, então o branco mostra a cor dele).
    let forma = no(g, "source.shape", -220.0, 120.0);
    use ph2d_node_motion_shape::param as p;
    for (chave, valor) in [
        (p::KIND, indice_do_circulo()?),
        (p::SIZE, ESTRELA),
        (p::FILL, 1.0),
        (p::FILL_R, 1.0),
        (p::FILL_G, 1.0),
        (p::FILL_B, 1.0),
        (p::FILL_A, 1.0),
    ] {
        g.set_param(forma, chave, valor);
    }
    let pecas = no(g, "motion.duplicator", 0.0, 60.0);
    liga(g, forma, pecas, 0)?;
    liga(g, galaxia, pecas, 1)?;
    // ── A COR: o degradê pela ordem das peças, no espaço da coluna.
    let tint = no(g, "motion.tint", 220.0, 60.0);
    g.set_param(tint, "mode", 1.0);
    g.set_param(tint, "space", ESPACOS[k]);
    for (chave, v) in ["r", "g", "b", "a"].into_iter().zip(AZUL) {
        g.set_param(tint, chave, v);
    }
    for (chave, v) in ["r2", "g2", "b2", "a2"].into_iter().zip(AMARELO) {
        g.set_param(tint, chave, v);
    }
    liga(g, pecas, tint, 0)?;
    let lugar = no(g, "motion.transform", 440.0, 60.0);
    #[expect(clippy::cast_precision_loss, reason = "duas colunas")]
    g.set_param(lugar, "offset_x", (k as f32 * 2.0 - 1.0) * COL_X);
    liga(g, tint, lugar, 0)?;
    let saida = no(g, "motion.output", 660.0, 60.0);
    liga(g, lugar, saida, 0)?;
    Some(Coluna { saida, tint })
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

/// As duas fichas, por baixo de cada galáxia.
pub(super) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    let ficha = |x: f32, t: &str| crate::motion_demo_legend::Caption::new([x, -4.0], t.to_string());
    vec![
        ficha(-COL_X, "Color Space = Linear RGB: the middle turns gray"),
        ficha(COL_X, "Color Space = OKLCH: the color stays vivid"),
    ]
}

/// **O roteiro que o dono segue** (`CLAUDE.md` §0.8).
pub(super) fn announce() {
    eprintln!(
        "\n[a cor que o olho ve] DUAS GALAXIAS de discos a girar devagar (uma SIMULACAO com\n\
         campos de forca). Cada uma e' pintada por um degrade de AZUL para AMARELO, disco a disco.\n\
         As duas sao IGUAIS, menos um botao.\n\
         \n\
         (1) A da ESQUERDA: no MEIO o degrade passa por um CINZA apagado.\n\
         (2) A da DIREITA: no meio a cor fica VERDE-AZULADA e viva.\n\
         (3) No grafo, na fila de CIMA (a da esquerda), clique no cartao `Tint` e mude\n    \
         `Color Space` para `OKLab`: o cinza vira um azul-esverdeado suave.\n\
         Depois `OKLCH Shorter`: fica como a da direita. `OKLCH Longer`: o degrade da a volta\n    \
         pelo outro lado do circulo (roxo, vermelho e laranja).\n\
         \n\
         DEU ERRADO se: a da direita tambem tiver cinza no meio; se mudar o `Color Space` nao\n\
         mudar nada; ou se aparecerem pontos pretos ou brancos estourados.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_cor_demo_tests.rs"]
mod tests;
