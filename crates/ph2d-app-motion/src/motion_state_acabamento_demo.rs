//! A cena `=135` — **O ACABAMENTO** (ciclo 20, [doc 129](../../../docs/Motion%20Nodes/129_ciclo_20_o_acabamento.md)).
//!
//! Uma galáxia de discos coloridos (a SIMULAÇÃO com campos da `=132`, [`super::ritmo_demo::galaxia`],
//! pintada pelo degradê OKLCH da `=133`) que passa por DOIS `Filter` e por um `Lens` antes da saída:
//! o primeiro é o relevo (`Bevel`) em CADA disco, o segundo a sombra projectada da galáxia INTEIRA
//! (doc 129 §3.4 — as duas opções do `Apply To`). A lente nasce com a VINHETA ligada e o resto
//! neutro: o artista troca cada filtro de opção, liga secção a secção a aberração, o grade, a LUT e
//! o grão, e vê a cena ganhar a cara de um plano de cinema.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O RAIO do disco: o diâmetro `0,4` passa o vão da grelha (`0,32` entre centros) ⇒ os vizinhos SE
/// TOCAM, e o passo (1) mostra a diferença entre `Each Shape` e `Whole Layer` (a `0,15` sobrava um
/// vão de `0,02` e a foto dos dois saía igual, W4).
const ESTRELA: f32 = 0.2;
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
    /// O relevo, em CADA disco (`Each Shape`).
    pub forma: NodeId,
    /// A sombra projectada da galáxia INTEIRA (`Whole Layer`).
    pub camada: NodeId,
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
    // ⭐ OS DOIS FILTROS (W3b): o relevo em cada disco, a sombra da galáxia inteira — o tipo pelo
    // MESMO gesto do cartão (os defaults do tipo, `switch_kind`).
    let forma = no(g, ph2d_node_fx_filter::TYPE_NAME, 440.0, 60.0);
    ph2d_node_fx_filter::switch_kind(g, forma, ph2d_ecs::FxOp::BEVEL);
    liga(g, tint, forma, 0)?;
    let camada = no(g, ph2d_node_fx_filter::TYPE_NAME, 660.0, 60.0);
    ph2d_node_fx_filter::switch_kind(g, camada, ph2d_ecs::FxOp::DROP_SHADOW);
    g.set_param(camada, ph2d_node_fx_filter::param::SCOPE, 1.0);
    liga(g, forma, camada, 0)?;
    // ⭐ A LENTE, com os defaults de quem a larga: só a vinheta.
    let lente = no(g, ph2d_node_fx_lens::TYPE_NAME, 880.0, 60.0);
    liga(g, camada, lente, 0)?;
    let saida = no(g, "motion.output", 1100.0, 60.0);
    liga(g, lente, saida, 0)?;
    g.validate(reg).ok()?;
    Some(Cena {
        saida,
        lente,
        tint,
        forma,
        camada,
    })
}

pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    let cena = monta(doc, reg)?;
    if let Some(n) = std::env::var("PH2D_ACABAMENTO_PASSO").ok() {
        if !passo(&mut doc.graph, &cena, n.trim()) {
            eprintln!(
                "[o acabamento] PH2D_ACABAMENTO_PASSO={n}: passo desconhecido, cena de sempre"
            );
        }
    }
    Some(vec![cena.saida])
}

/// O caminho do `.cube` que o passo (7) nomeia, relativo à raiz do repo.
pub(crate) const LUT_QUENTE: &str = "docs/Motion Nodes/tutoriais/luts/quente.cube";

/// Os passos que [`passo`] reconhece, pela ordem do roteiro (`0` = a cena como abre).
pub(crate) const PASSOS: [&str; 13] = [
    "0", "1", "2", "3", "4", "5", "5b", "6", "7a", "7b", "7c", "7d", "8",
];

/// **Um passo do roteiro já feito** — `PH2D_ACABAMENTO_PASSO=<passo>`, a porta das FOTOS: os
/// eventos sintéticos não chegam à tela virtual, logo cada foto é a cena com o param do passo
/// escrito pelo MESMO gesto do cartão (o `switch_kind` no `Effect`). `false` = passo
/// desconhecido, e a cena fica a de sempre ao bit.
pub(crate) fn passo(g: &mut Graph, c: &Cena, n: &str) -> bool {
    use ph2d_node_fx_filter::param::SCOPE;
    use ph2d_node_fx_lens::param as l;
    match n {
        "0" => {}
        "1" => g.set_param(c.forma, SCOPE, 1.0),
        "2" => ph2d_node_fx_filter::switch_kind(g, c.forma, ph2d_ecs::FxOp::INNER_SHADOW),
        "3" => g.set_param(c.camada, SCOPE, 0.0),
        "4" => {
            g.set_param(c.forma, SCOPE, 1.0);
            g.set_param(c.camada, SCOPE, 0.0);
        }
        "5" => g.set_param(c.lente, l::VIN, 0.0),
        "5b" => g.set_param(c.lente, l::VIN, -0.5),
        "6" => g.set_param(c.lente, l::AB, 0.03),
        "7a" => g.set_param(c.lente, l::TEMPERATURE, 0.5),
        "7b" => g.set_param(c.lente, l::SATURATION, -1.0),
        "7c" => g.set_param(c.lente, l::EXPOSURE, 1.0),
        "7d" => g.set_text_param(c.lente, ph2d_node_fx_lens::LUT, LUT_QUENTE),
        "8" => g.set_param(c.lente, l::GRAIN, 0.4),
        // Fora do roteiro, uma MEDIÇÃO (doc 129 §7): a saída em `Add` manda as formas ao Vello.
        "mistura" => g.set_param(c.saida, ph2d_eval_motion::SINK_BLEND_PARAM, 1.0),
        _ => return false,
    }
    true
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
         uma SIMULACAO com campos de forca. Antes da saida ha DOIS nos `Filter` (filtros da\n\
         CAMADA do Motion) e um no `Lens`: a camera.\n\
         \n\
         (0) Cada disco tem RELEVO: a borda de cima-esquerda clara, a de baixo-direita escura. E' o\n    \
         1.o `Filter` (`Effect` = `Bevel`), com `Apply To` = `Each Shape`: cada disco por si — onde\n    \
         dois se tocam, cada um guarda a sua borda inteira. A galaxia gira: o relevo fica FIRME.\n\
         (1) No 1.o `Filter`, troque `Apply To` para `Whole Layer`: os discos que se tocam viram UMA\n    \
         peca e a borda some onde eles se encostam. Volte para `Each Shape`.\n\
         (2) Ainda no 1.o, troque `Effect` para `Inner Shadow`: cada disco ganha uma sombra POR\n    \
         DENTRO, igual nos do meio e nos da beira da galaxia.\n\
         (3) O 2.o `Filter` e' a SOMBRA da galaxia inteira (`Drop Shadow`, `Whole Layer`), para baixo\n    \
         e para a direita. Troque o `Apply To` dele para `Each Shape`: cada disco projecta a sua, e a\n    \
         sombra de um cai em cima do vizinho. `Radius` amacia; `Offset X`/`Offset Y` movem-na.\n\
         (4) Ponha o 1.o em `Whole Layer` e o 2.o em `Each Shape`: o cartao do 2.o diz, em baixo do\n    \
         titulo, que ele ja' corre na camada inteira (os discos foram juntos antes dele).\n\
         (5) Os CANTOS da cena estao mais escuros que o centro: e' a vinheta, que o `Lens`\n    \
         liga sozinho. No cartao `Lens`, seccao `Vignette`, arraste `Vignette` para 0: os\n    \
         cantos voltam ao normal. Para -0,5: os cantos CLAREIAM.\n\
         (6) Abra `Chromatic Aberration` e ponha `Aberration` em 0,03: perto das BORDAS cada\n    \
         disco ganha uma franja vermelha de um lado e azul do outro; no CENTRO nada muda.\n\
         (7) Abra `Color Grade`: `Temperature` 0,5 esquenta tudo; `Saturation` -1 deixa a cena\n    \
         cinzenta; `Exposure` 1 dobra a luz. `LUT File`: `docs/Motion Nodes/tutoriais/luts/\n    \
         quente.cube` puxa as sombras para o azul-petroleo e as luzes para o laranja.\n\
         (8) Abra `Film Grain` e ponha `Grain` em 0,4: um chuvisco fino que mexe a cada\n    \
         quadro. Pare o tempo: o grao PARA.\n\
         \n\
         DEU ERRADO se: o relevo tremer com a galaxia a girar; se em `Each Shape` os discos do meio\n\
         tiverem sombra interna diferente dos da beira; se em `Whole Layer` a borda continuar entre\n\
         discos que se tocam; se o efeito sair por cima dos paineis ou do grafo; se a franja\n\
         aparecer no centro; ou se o grao continuar a mexer com o tempo parado.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_acabamento_demo_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "motion_state_acabamento_tutorial_tests.rs"]
mod tutorial_tests;
