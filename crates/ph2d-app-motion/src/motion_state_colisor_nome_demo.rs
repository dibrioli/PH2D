//! A cena `=137` — **O COLISOR PELO NOME, NA PLACA** ([doc 131 §3](../../../docs/Motion%20Nodes/131_zerar_o_ultimo_bloco.md), D.14).
//!
//! Duas galáxias SIMULADAS (os campos da `=132`) de discos. Em cada uma, um `Drive` no canal
//! `Custom` escreve o raio de cada disco numa coluna PELO NOME — `|P| · RAIO`, maior na borda. À
//! ESQUERDA o nome é `collider`: o passe do `Output` lê-o e os discos da borda empurram os vizinhos.
//! À DIREITA o nome é `colisor` (o controlo): ninguém o lê e os discos sobrepõem-se. As duas cozem
//! na PLACA (`StreamOp::WriteNamed`); antes a da esquerda caía na CPU (`RECUSA_COLISOR`).

use super::jogo_demo::{liga, no_em};
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// O raio do colisor por unidade de distância ao centro (unidade da geometria × `size`).
pub(crate) const RAIO: f32 = 0.8;
/// Os dois nomes — a ÚNICA diferença entre as colunas.
pub(crate) const NOMES: [&str; 2] = ["collider", "colisor"];
const COL_X: f32 = super::ritmo_demo::COL_X;
pub(crate) const DISCO: f32 = 0.2;
const VARREDURAS: f32 = 16.0;

/// Uma galáxia de discos com o raio escrito pelo nome `NOMES[k]`. Devolve a saída e o drive.
fn coluna(g: &mut ph2d_nodegraph::graph::Graph, k: usize) -> Option<(NodeId, NodeId)> {
    #[expect(clippy::cast_precision_loss, reason = "duas colunas")]
    let y0 = 330.0 * k as f32;
    let no = no_em(y0);
    let galaxia = super::ritmo_demo::galaxia(g, y0)?;
    let cor = if k == 0 {
        [0.98, 0.55, 0.25]
    } else {
        [0.35, 0.65, 0.98]
    };
    let disco = super::jogo_demo::forma(
        g,
        &no,
        ph2d_node_motion_shape::ShapeKind::Circle,
        DISCO,
        cor,
        -220.0,
        160.0,
    )?;
    let pecas = no(g, "motion.duplicator", -220.0, 0.0);
    g.set_param(pecas, "point_scale", 1.0);
    liga(g, (disco, 0), pecas, 0, false)?;
    liga(g, (galaxia, 0), pecas, 1, false)?;
    // A distância de cada disco ao centro da galáxia (`|P|`, o modo comprimento).
    let distancia = no(g, "value.attribute", 0.0, -110.0);
    g.set_text_param(distancia, "attr", "P");
    g.set_param(distancia, "mode", 1.0);
    liga(g, (pecas, 0), distancia, 0, false)?;
    let raio = no(g, "motion.drive", 220.0, 0.0);
    g.set_param(raio, "channel", 9.0);
    g.set_param(raio, "mode", 1.0);
    g.set_param(raio, "scale", RAIO);
    g.set_text_param(raio, "column", NOMES[k]);
    liga(g, (pecas, 0), raio, 0, false)?;
    liga(g, (distancia, 0), raio, 1, false)?;
    let lugar = no(g, "motion.transform", 440.0, 0.0);
    #[expect(clippy::cast_precision_loss, reason = "duas colunas")]
    g.set_param(lugar, "offset_x", (k as f32 * 2.0 - 1.0) * COL_X);
    liga(g, (raio, 0), lugar, 0, false)?;
    let saida = no(g, "motion.output", 660.0, 0.0);
    g.set_param(
        saida,
        ph2d_eval_motion::SINK_COLLIDE_ITERATIONS_PARAM,
        VARREDURAS,
    );
    liga(g, (lugar, 0), saida, 0, false)?;
    Some((saida, raio))
}

/// O documento: `(saídas, drives)`.
pub(crate) fn monta(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<([NodeId; 2], [NodeId; 2])> {
    let g = &mut doc.graph;
    let (s0, d0) = coluna(g, 0)?;
    let (s1, d1) = coluna(g, 1)?;
    g.validate(reg).ok()?;
    Some(([s0, s1], [d0, d1]))
}

pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    monta(doc, reg).map(|(s, _)| s.to_vec())
}

/// As fichas do canvas.
pub(super) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    let ficha = |x: f32, t: &str| crate::motion_demo_legend::Caption::new([x, -3.2], t.to_string());
    vec![
        ficha(-COL_X, "Custom = collider: os discos da borda empurram-se"),
        ficha(
            COL_X,
            "Custom = colisor (controlo): ninguem le, sobrepoem-se",
        ),
    ]
}

/// **O roteiro que o dono segue** (`CLAUDE.md` §0.8).
pub(super) fn announce() {
    eprintln!(
        "\n[colisor pelo nome] DUAS galaxias de discos a girar (uma SIMULACAO com campos). Em cada\n\
         uma, um cartao `Drive` (canal `Custom`) da' a cada disco um raio de colisao: pequeno no\n\
         meio, grande na borda. A UNICA diferenca e' o NOME escrito no campo `Column` do Drive.\n\
         \n\
         (1) De PLAY. A da ESQUERDA (laranja, `collider`): na borda os discos ENCOSTAM e\n    \
         empurram-se uns aos outros; no meio ficam soltos.\n\
         (2) A da DIREITA (azul, `colisor` - o nome errado): os discos da borda passam uns por\n    \
         cima dos outros. E' o controlo.\n\
         (3) No cartao `Drive` da DIREITA, troque `colisor` por `collider`: ela passa a fazer\n    \
         igual a' da esquerda.\n\
         (4) Aberta com `PH2D_MOTION_ROUTE_LOG=1`, o terminal diz UMA vez\n    \
         `[motion-route] device: HIBRIDO` - as duas correm na PLACA.\n\
         \n\
         DEU ERRADO se: as duas ficarem iguais antes do passo (3); o passo (3) nao mudar nada;\n\
         ou o terminal disser `CPU: uma peca declara colisor pelo nome`.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_colisor_nome_demo_tests.rs"]
mod tests;
