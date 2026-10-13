//! A cena `=140` — **IMAGENS E FORMAS NUM GRUPO QUE MISTURA, NA PLACA** ([doc 132 §4](../../../docs/Motion%20Nodes/132_zerar_as_seis_pendencias.md), P4).
//!
//! Duas galáxias SIMULADAS (os campos da `=132`: um redemoinho e um íman) sobre um FUNDO escuro. Em
//! cada uma, cada ponto leva uma ESTRELA (forma) e um quadrado de IMAGEM (o ladrilho do átlas, tintado
//! de laranja), e as duas médias juntam-se numa saída que mistura EM GRUPO (`Blend With` = `Copies`):
//! à ESQUERDA `Screen`, à DIREITA `Add`. Onde imagens e estrelas se sobrepõem a cor CLAREIA — e com a
//! mesma regra nas duas médias. Até ao P4 a imagem num grupo devolvia a cena à CPU (e desenhava-se um
//! a dois quadros pelas sprites, em luz linear); hoje as duas médias cozem e desenham na PLACA.

use super::jogo_demo::{forma, liga, no_em};
use ph2d_eval_motion::BlendWith;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Graph, NodeId};

const COL_X: f32 = super::ritmo_demo::COL_X;
/// O lado de uma estrela e o de um quadrado de imagem (o passo da galáxia é `0,32`: os quadrados
/// SOBREPÕEM-SE, e é na sobreposição que a mistura se vê).
pub(crate) const ESTRELA: f32 = 0.3;
pub(crate) const IMAGEM: f32 = 0.42;
/// Os tags de mistura das duas colunas (`Screen`, `Add`).
pub(crate) const MISTURAS: [u8; 2] = [4, 1];

/// Uma galáxia com estrelas e imagens, numa saída em grupo `MISTURAS[k]`. Devolve a saída.
fn coluna(g: &mut Graph, k: usize) -> Option<NodeId> {
    #[expect(clippy::cast_precision_loss, reason = "duas colunas")]
    let y0 = 330.0 * k as f32;
    let no = no_em(y0);
    let galaxia = super::ritmo_demo::galaxia(g, y0)?;
    let estrela = forma(
        g,
        &no,
        ph2d_node_motion_shape::ShapeKind::Star,
        ESTRELA,
        [0.35, 0.6, 0.98],
        -220.0,
        160.0,
    )?;
    let estrelas = no(g, "motion.duplicator", -220.0, 0.0);
    liga(g, (estrela, 0), estrelas, 0, false)?;
    liga(g, (galaxia, 0), estrelas, 1, false)?;
    // As IMAGENS: os MESMOS pontos, com tamanho e tinta (o `combine` enche de zeros a coluna que só
    // uma entrada tem — sem as duas a imagem chegava invisível).
    let lado = no(g, "motion.scale", -220.0, -150.0);
    g.set_param(lado, "amount", IMAGEM);
    let tinta = no(g, "motion.tint", 0.0, -150.0);
    for (c, v) in [("r", 0.98), ("g", 0.55), ("b", 0.22), ("a", 0.6)] {
        g.set_param(tinta, c, v);
    }
    liga(g, (galaxia, 0), lado, 0, false)?;
    liga(g, (lado, 0), tinta, 0, false)?;
    let junta = no(g, "motion.combine", 220.0, 0.0);
    liga(g, (estrelas, 0), junta, 0, false)?;
    liga(g, (tinta, 0), junta, 1, false)?;
    let lugar = no(g, "motion.transform", 440.0, 0.0);
    #[expect(clippy::cast_precision_loss, reason = "duas colunas")]
    g.set_param(lugar, "offset_x", (k as f32 * 2.0 - 1.0) * COL_X);
    liga(g, (junta, 0), lugar, 0, false)?;
    let saida = no(g, "motion.output", 660.0, 0.0);
    g.set_param(
        saida,
        ph2d_eval_motion::SINK_BLEND_PARAM,
        f32::from(MISTURAS[k]),
    );
    let copias = BlendWith::ALL
        .iter()
        .position(|c| *c == BlendWith::Copies)?;
    #[expect(clippy::cast_precision_loss, reason = "um índice pequeno")]
    g.set_param(
        saida,
        ph2d_eval_motion::SINK_BLEND_WITH_PARAM,
        copias as f32,
    );
    liga(g, (lugar, 0), saida, 0, false)?;
    Some(saida)
}

/// O FUNDO: um quadrado escuro grande atrás das duas galáxias (uma saída em `Normal`, a primeira).
fn fundo(g: &mut Graph) -> Option<NodeId> {
    let no = no_em(-330.0);
    let ponto = no(g, "motion.grid", -440.0, 0.0);
    g.set_param(ponto, "rows", 1.0);
    g.set_param(ponto, "cols", 1.0);
    let quadrado = forma(
        g,
        &no,
        ph2d_node_motion_shape::ShapeKind::Square,
        14.0,
        [0.07, 0.09, 0.16],
        -440.0,
        160.0,
    )?;
    let pecas = no(g, "motion.duplicator", -220.0, 0.0);
    liga(g, (quadrado, 0), pecas, 0, false)?;
    liga(g, (ponto, 0), pecas, 1, false)?;
    let saida = no(g, "motion.output", 0.0, 0.0);
    liga(g, (pecas, 0), saida, 0, false)?;
    Some(saida)
}

/// O documento: as três saídas — o fundo, a galáxia `Screen` e a `Add`.
pub(crate) fn monta(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<[NodeId; 3]> {
    let g = &mut doc.graph;
    let f = fundo(g)?;
    let e = coluna(g, 0)?;
    let d = coluna(g, 1)?;
    g.validate(reg).ok()?;
    Some([f, e, d])
}

pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    monta(doc, reg).map(|s| s.to_vec())
}

/// As fichas do canvas.
pub(super) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    let ficha = |x: f32, t: &str| crate::motion_demo_legend::Caption::new([x, -3.2], t.to_string());
    vec![
        ficha(
            -COL_X,
            "Screen em grupo: imagens e estrelas clareiam onde se tocam",
        ),
        ficha(COL_X, "Add em grupo: clareia mais, ate ao branco"),
    ]
}

/// **O roteiro que o dono segue** (`CLAUDE.md` §0.8).
pub(super) fn announce() {
    eprintln!(
        "\n[imagens no grupo] DUAS galaxias a girar (uma SIMULACAO com campos: um redemoinho e um\n\
         ima) sobre um FUNDO escuro. Em cada ponto ha' uma ESTRELA azul (uma forma) e um QUADRADO\n\
         laranja (uma imagem). Os dois juntam-se numa saida que MISTURA EM GRUPO.\n\
         \n\
         (1) De PLAY. A da ESQUERDA (`Screen`): onde os quadrados laranja se sobrepoem, e onde\n    \
         cobrem uma estrela, a cor CLAREIA - e clareia igual nas estrelas e nos quadrados.\n\
         (2) A da DIREITA (`Add`): clareia mais; onde muitos se juntam chega ao branco.\n\
         (3) Aproxime o grafo (roda do rato) ate' ler o cartao `Output` da galaxia de CIMA e mude\n    \
         `Blend` para `Normal`: os quadrados passam a TAPAR-SE uns aos outros, sem clarear.\n    \
         Volte a `Screen`.\n\
         (4) Aberta com `PH2D_MOTION_ROUTE_LOG=1`, o terminal diz `[motion-route] device: ...`\n    \
         e `[formas] ... pela PLACA (do dispositivo)` - imagens e formas cozem na PLACA.\n\
         \n\
         DEU ERRADO se: os quadrados laranja nao clarearem onde se tocam (como no passo 3);\n\
         piscarem por um instante mais escuros ao abrir; ou o terminal disser\n\
         `CPU: uma linha de IMAGEM num sink que mistura em grupo`.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_imagens_grupo_demo_tests.rs"]
mod tests;
