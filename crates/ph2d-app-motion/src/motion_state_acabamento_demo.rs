//! A cena `=135` — **O ACABAMENTO** (ciclo 20, [doc 129](../../../docs/Motion%20Nodes/129_ciclo_20_o_acabamento.md)).
//!
//! Uma galáxia de discos coloridos (a SIMULAÇÃO com campos da `=132`, [`super::ritmo_demo::galaxia`],
//! pintada pelo degradê OKLCH da `=133`) que passa por um `Filter` (a sombra projectada da CAMADA)
//! e por um `Lens` antes da saída. A lente nasce com a VINHETA ligada (os cantos escurecem) e o
//! resto neutro: o artista liga, secção a secção, a aberração, o grade, a LUT e o grão, e troca o
//! filtro (borrão, brilho, contorno…), e vê a cena ganhar a cara de um plano de cinema.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

const ESTRELA: f32 = 0.15;
const AZUL: [f32; 4] = [0.05, 0.2, 1.0, 1.0];
const LARANJA: [f32; 4] = [1.0, 0.55, 0.05, 1.0];
/// O `Color Space` OKLCH Shorter do `Tint` (o da `=133`).
const OKLCH: f32 = 2.0;

fn liga(g: &mut Graph, de: NodeId, para: NodeId, porta: u16) -> Option<()> {
    g.connect(Edge {
        from: (de, 0),
        to: (para, porta),
        delayed: false,
    })
    .ok()
}

/// Os nós que os gates leem.
#[cfg_attr(not(test), expect(dead_code, reason = "lidos pelos gates da cena"))]
pub(crate) struct Cena {
    pub saida: NodeId,
    pub lente: NodeId,
    pub tint: NodeId,
    pub filtro: NodeId,
}

pub(crate) fn monta(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Cena> {
    let g = &mut doc.graph;
    let no = |g: &mut Graph, tipo: &str, x: f32, y: f32| {
        let n = g.add_node(tipo.to_string());
        g.set_pos(n, Pos { x, y });
        n
    };
    let galaxia = super::ritmo_demo::galaxia(g, 0.0)?;
    let forma = no(g, "source.shape", -220.0, 120.0);
    use ph2d_node_motion_shape::param as p;
    let circulo = ph2d_node_motion_shape::ALL_KINDS
        .iter()
        .position(|k| *k == ph2d_node_motion_shape::ShapeKind::Circle)?;
    #[expect(clippy::cast_precision_loss, reason = "um indice de enum")]
    let circulo = circulo as f32;
    for (chave, valor) in [
        (p::KIND, circulo),
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
    let tint = no(g, "motion.tint", 220.0, 60.0);
    g.set_param(tint, "mode", 1.0);
    g.set_param(tint, "space", OKLCH);
    for (chave, v) in ["r", "g", "b", "a"].into_iter().zip(AZUL) {
        g.set_param(tint, chave, v);
    }
    for (chave, v) in ["r2", "g2", "b2", "a2"].into_iter().zip(LARANJA) {
        g.set_param(tint, chave, v);
    }
    liga(g, pecas, tint, 0)?;
    // ⭐ UM FILTRO DA CAMADA (W3): a sombra projectada, com os defaults do nó.
    let filtro = no(g, ph2d_node_fx_filter::TYPE_NAME, 440.0, 60.0);
    g.set_param(
        filtro,
        ph2d_node_fx_filter::param::KIND,
        f32::from(ph2d_ecs::FxOp::DROP_SHADOW),
    );
    liga(g, tint, filtro, 0)?;
    // ⭐ A LENTE, com os defaults de quem a larga: só a vinheta.
    let lente = no(g, ph2d_node_fx_lens::TYPE_NAME, 660.0, 60.0);
    liga(g, filtro, lente, 0)?;
    let saida = no(g, "motion.output", 880.0, 60.0);
    liga(g, lente, saida, 0)?;
    g.validate(reg).ok()?;
    Some(Cena {
        saida,
        lente,
        tint,
        filtro,
    })
}

pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    monta(doc, reg).map(|c| vec![c.saida])
}

/// A ficha, por baixo da galáxia.
pub(super) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    vec![crate::motion_demo_legend::Caption::new(
        [0.0, -4.6],
        "Lens: a camera ve a cena inteira".to_string(),
    )]
}

/// **O roteiro que o dono segue** (`CLAUDE.md` §0.8).
pub(super) fn announce() {
    eprintln!(
        "\n[o acabamento] UMA GALAXIA de discos coloridos (azul para laranja) a girar devagar —\n\
         uma SIMULACAO com campos de forca. Antes da saida ha um no `Filter` (um filtro da\n\
         CAMADA do Motion) e um no `Lens`: a camera.\n\
         \n\
         (0) Cada disco tem uma SOMBRA escura para baixo e para a direita: e' o `Filter`, em\n    \
         `Drop Shadow`. No cartao `Filter`, `Offset X`/`Offset Y` movem a sombra; `Radius`\n    \
         amacia-a. Troque `Filter` para `Blur`: os discos borram; `Glow` (com `Color` clara):\n    \
         um brilho a volta; `Outline`: um contorno. A camada filtrada fica POR CIMA do mundo.\n\
         (1) Os CANTOS da cena estao mais escuros que o centro: e' a vinheta, que o `Lens`\n    \
         liga sozinho. No cartao `Lens`, seccao `Vignette`, arraste `Vignette` para 0: os\n    \
         cantos voltam ao normal. Para -0,5: os cantos CLAREIAM.\n\
         (2) Abra `Chromatic Aberration` e ponha `Aberration` em 0,03: perto das BORDAS cada\n    \
         disco ganha uma franja vermelha de um lado e azul do outro; no CENTRO nada muda.\n\
         (3) Abra `Color Grade`: `Temperature` 0,5 esquenta tudo (puxa para o laranja);\n    \
         `Saturation` -1 deixa a cena cinzenta; `Exposure` 1 dobra a luz.\n\
         (4) Ainda em `Color Grade`, `LUT File`: escolha `docs/Motion Nodes/tutoriais/luts/\n    \
         quente.cube` — sombras puxam para o azul-petroleo e as luzes para o laranja, e\n    \
         aparece o `LUT Mix` para dosear. Escolha `lut_1d.cube`: o cartao diz em baixo do\n    \
         titulo porque a recusou, e a cena fica como estava.\n\
         (5) Abra `Film Grain` e ponha `Grain` em 0,4: um chuvisco fino que mexe a cada\n    \
         quadro. Pare o tempo: o grao PARA (o mesmo quadro tem sempre o mesmo grao).\n\
         \n\
         DEU ERRADO se: os discos abrirem sem sombra; se os cantos nao escurecerem ao abrir a\n\
         cena; se o efeito sair por cima\n\
         dos paineis ou do grafo; se a franja aparecer no centro; ou se o grao continuar a\n\
         mexer com o tempo parado.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_acabamento_demo_tests.rs"]
mod tests;
