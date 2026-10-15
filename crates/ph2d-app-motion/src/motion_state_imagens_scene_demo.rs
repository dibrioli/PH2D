//! A cena `=142` — **O `Scene` COM IMAGENS E FORMAS, NA PLACA, NA ORDEM DAS LINHAS**
//! ([doc 132 §4.1](../../../docs/Motion%20Nodes/132_zerar_as_seis_pendencias.md), item 3).
//!
//! UMA galáxia SIMULADA (os campos da `=132`: um redemoinho e um íman) sobre o FUNDO da `=140`. Cada
//! ponto leva um quadrado de IMAGEM (o ladrilho do átlas, laranja, opaco) e uma ESTRELA (forma), e o
//! `combine` põe as IMAGENS na porta `0` e as ESTRELAS na `1`: as linhas das imagens vêm ANTES. A saída
//! mistura com `Blend With` = `Scene` em `Screen`: as cópias juntam-se em `Normal` (a linha de cima
//! TAPA) e só o grupo clareia o fundo. Antes do item 3 o `Scene` com imagens no dispositivo voltava à
//! CPU; hoje um desenho ordenado percorre as duas médias pela ordem das linhas.

use super::jogo_demo::{forma, liga, no_em};
use ph2d_eval_motion::BlendWith;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Graph, NodeId};

/// O tamanho de uma estrela e o lado de um quadrado de imagem — a estrela cabe DENTRO do quadrado
/// dela (com a ordem trocada sumiria por baixo), e os quadrados sobrepõem-se aos vizinhos (o passo
/// da galáxia é `0,32`). Fotografado: a `0,34` as estrelas cobriam a galáxia e as imagens não se viam.
pub(crate) const ESTRELA: f32 = 0.14;
pub(crate) const IMAGEM: f32 = 0.42;
/// A mistura do grupo com o fundo (`Screen`).
pub(crate) const MISTURA: u8 = 4;

/// Os nós que os gates leem (o produto só usa as saídas).
#[cfg_attr(not(test), expect(dead_code, reason = "lidos pelos gates da cena"))]
pub(crate) struct Cena {
    pub fundo: NodeId,
    pub saida: NodeId,
    pub junta: NodeId,
    pub imagens: NodeId,
    pub estrelas: NodeId,
}

/// A galáxia com imagens (porta 0) e estrelas (porta 1) numa saída `Scene`.
fn galaxia(g: &mut Graph) -> Option<(NodeId, NodeId, NodeId, NodeId)> {
    let no = no_em(0.0);
    let galaxia = super::ritmo_demo::galaxia(g, 0.0)?;
    let lado = no(g, "motion.scale", -220.0, -150.0);
    g.set_param(lado, "amount", IMAGEM);
    let imagens = no(g, "motion.tint", 0.0, -150.0);
    for (c, v) in [("r", 0.98), ("g", 0.55), ("b", 0.22), ("a", 1.0)] {
        g.set_param(imagens, c, v);
    }
    liga(g, (galaxia, 0), lado, 0, false)?;
    liga(g, (lado, 0), imagens, 0, false)?;
    let estrela = forma(
        g,
        &no,
        ph2d_node_motion_shape::ShapeKind::Star,
        ESTRELA,
        [0.35, 0.6, 0.98],
        -220.0,
        160.0,
    )?;
    let estrelas = no(g, "motion.duplicator", 0.0, 160.0);
    liga(g, (estrela, 0), estrelas, 0, false)?;
    liga(g, (galaxia, 0), estrelas, 1, false)?;
    // ⭐ A ORDEM: as imagens na porta 0, as estrelas na 1 — as estrelas desenham-se por cima.
    let junta = no(g, "motion.combine", 220.0, 0.0);
    liga(g, (imagens, 0), junta, 0, false)?;
    liga(g, (estrelas, 0), junta, 1, false)?;
    let saida = no(g, "motion.output", 440.0, 0.0);
    g.set_param(
        saida,
        ph2d_eval_motion::SINK_BLEND_PARAM,
        f32::from(MISTURA),
    );
    let scene = BlendWith::ALL.iter().position(|c| *c == BlendWith::Scene)?;
    #[expect(clippy::cast_precision_loss, reason = "um índice pequeno")]
    g.set_param(saida, ph2d_eval_motion::SINK_BLEND_WITH_PARAM, scene as f32);
    liga(g, (junta, 0), saida, 0, false)?;
    Some((saida, junta, imagens, estrelas))
}

/// O documento: o fundo e a galáxia.
pub(crate) fn monta(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Cena> {
    let g = &mut doc.graph;
    let fundo = super::imagens_grupo_demo::fundo(g)?;
    let (saida, junta, imagens, estrelas) = galaxia(g)?;
    g.validate(reg).ok()?;
    Some(Cena {
        fundo,
        saida,
        junta,
        imagens,
        estrelas,
    })
}

pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    monta(doc, reg).map(|c| vec![c.fundo, c.saida])
}

/// As fichas do canvas.
pub(super) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    vec![crate::motion_demo_legend::Caption::new(
        [0.0, -3.2],
        "Scene: a estrela (forma) tapa o quadrado laranja (imagem); so' o grupo clareia o fundo",
    )]
}

/// **O roteiro que o dono segue** (`CLAUDE.md` §0.8).
pub(super) fn announce() {
    eprintln!(
        "\n[scene com imagens] UMA galaxia a girar (uma SIMULACAO com campos: um redemoinho e um\n\
         ima) sobre um FUNDO escuro. Em cada ponto ha' um QUADRADO laranja (uma imagem) e, por\n\
         cima dele, uma ESTRELA azul (uma forma). Os dois juntam-se numa saida com\n\
         `Blend With` = `Scene`: entre eles vale `Normal` (o de cima TAPA), e so' o grupo inteiro\n\
         se mistura com o fundo, em `Screen`.\n\
         \n\
         (1) De PLAY. Cada estrela azul aparece INTEIRA por cima do seu quadrado laranja - e por\n    \
         cima dos quadrados vizinhos que a tocam. Os quadrados tapam-se uns aos outros, sem clarear.\n\
         (2) Aproxime o grafo (roda do rato) ate' ler o cartao `Output` e mude `Blend With` para\n    \
         `Copies`: agora cada copia mistura com as outras em `Screen` - onde estrela e quadrado se\n    \
         tocam a cor CLAREIA, e a estrela deixa de tapar. Volte a `Scene`.\n\
         (3) Aberta com `PH2D_MOTION_ROUTE_LOG=1`, o terminal diz `[motion-route] device: ...`\n    \
         e `[formas] ... pela PLACA (do dispositivo)` - imagens e estrelas desenham na PLACA.\n\
         \n\
         DEU ERRADO se: os quadrados laranja taparem as estrelas (a ordem trocada); faltarem os\n\
         quadrados ou as estrelas; ou o terminal disser `[motion-route] CPU: ...`.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_imagens_scene_demo_tests.rs"]
mod tests;
