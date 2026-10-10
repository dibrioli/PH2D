//! ⭐⭐⭐ **A CORDA COM COLISOR** (cenas `=130` e `=131`) — o Bug #16 fechado (doc 125 §1) e o
//! `Collide` depois do corpo (doc 125 §2).
//!
//! `=130`: duas cordas iguais, chicoteadas por um mastro, cada uma vestida por um Shape ligado à
//! porta `shape` dela (`Shape → Verlet Rope`). A ÚNICA diferença é o `Collide` do Shape: desligado
//! na da ESQUERDA (as contas atravessam-se quando ela se dobra), ligado na da DIREITA (as contas
//! encostam). O gesto do dono é ligar o da esquerda.
//!
//! `=131`: a mesma corda vestida DEPOIS (`Verlet Rope → Duplicator ← Shape`), com o `Collide`
//! ligado — o botão não segura nada, e o cartão do Shape diz porquê e o que fazer.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Os números da corda — escolhidos pela sonda `sonda_as_receitas_da_corda` (doc 125 §1.4).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Receita {
    pub contas: f32,
    pub comprimento: f32,
    /// O raio do disco desenhado E do colisor (o `Collider Shape` em círculo).
    pub raio: f32,
    pub periodo: f32,
    pub curso: f32,
    /// O mastro sacode a corda DE LADO (a âncora `y`, perpendicular à corda que nasce ao longo de
    /// `x`) — ao longo dela a corda só encolhe e estica em fila.
    pub de_lado: bool,
    pub gravidade: f32,
}

/// ⚠️ **SEM gravidade e sacudida DE LADO, e é a medição que o pede** (doc 125 §1.4): com gravidade
/// a corda estica-se para baixo e o chicote só a cruza por instantes; sacudida ao LONGO dela, a
/// corda só encolhe em fila e a foto não o mostra. De lado, no ar, a corda nua dobra-se e passa `38 %`
/// dos quadros da 2.ª metade com contas atravessadas — com o `Collide`, `0 %` (o pior par a `0,2305`
/// contra o diâmetro `0,24`). As contas tocam-se no repouso (`2r ≈ 6/23`): um colar.
pub(crate) const RECEITA: Receita = Receita {
    contas: 24.0,
    comprimento: 6.0,
    raio: 0.12,
    periodo: 1.3,
    curso: 3.0,
    de_lado: true,
    gravidade: 0.0,
};

/// Onde o mastro de cada corda vive, em mundo: a corda nasce para a DIREITA dele (`comprimento`),
/// logo as duas colunas não são simétricas — `MASTRO_X[k]`.
const MASTRO_X: [f32; 2] = [-8.5, 1.5];
const MASTRO_Y: f32 = -1.0;

fn liga(g: &mut Graph, de: NodeId, para: NodeId, porta: u16, pre: bool) -> Option<()> {
    g.connect(Edge {
        from: (de, 0),
        to: (para, porta),
        delayed: pre,
    })
    .ok()
}

/// O Shape que veste a corda: um disco do raio da receita, com o colisor em círculo.
fn forma(g: &mut Graph, n: NodeId, r: Receita, collide: bool) -> Option<()> {
    use ph2d_node_motion_shape::param as p;
    let circulo = ph2d_node_motion_shape::ALL_KINDS
        .iter()
        .position(|k| *k == ph2d_node_motion_shape::ShapeKind::Circle)?;
    #[expect(clippy::cast_precision_loss, reason = "um indice de enum")]
    let circulo = circulo as f32;
    let cor = if collide {
        [0.30, 0.80, 0.45]
    } else {
        [0.95, 0.45, 0.30]
    };
    for (chave, valor) in [
        (p::KIND, circulo),
        (p::SIZE, r.raio),
        (p::FILL, 1.0),
        (p::FILL_R, cor[0]),
        (p::FILL_G, cor[1]),
        (p::FILL_B, cor[2]),
        (p::FILL_A, 0.85),
        (p::STROKE_WIDTH, 0.02),
        (p::STROKE_R, 0.08),
        (p::STROKE_G, 0.08),
        (p::STROKE_B, 0.10),
        (p::STROKE_A, 1.0),
        (p::COLLIDE, f32::from(u8::from(collide))),
        // O `Collider Shape` em CÍRCULO (`Box · Circle`, append-only): o colisor do tamanho do disco.
        (p::COLLIDER_SHAPE, 1.0),
        (p::SHOW_COLLIDER, 0.0),
    ] {
        g.set_param(n, chave, valor);
    }
    Some(())
}

/// Uma linha: o mastro, a corda e o Shape. `depois = false` liga o Shape à porta `shape` da corda
/// (`=130`); `true` veste-a depois, num Duplicator (`=131`). Devolve `(saída, corda, shape)`.
fn linha(
    g: &mut Graph,
    k: usize,
    collide: bool,
    depois: bool,
    r: Receita,
) -> Option<(NodeId, NodeId, NodeId)> {
    #[expect(clippy::cast_precision_loss, reason = "duas linhas")]
    let y = 340.0 * k as f32;
    let no = |g: &mut Graph, tipo: &str, x: f32, dy: f32| {
        let n = g.add_node(tipo.to_string());
        g.set_pos(n, Pos { x, y: y + dy });
        n
    };
    let mastro = no(g, "value.lfo", -440.0, -100.0);
    g.set_param(mastro, "period", r.periodo);
    g.set_param(mastro, "amplitude", r.curso);
    let corda = no(g, "motion.verlet_rope", -200.0, 0.0);
    for (p, v) in [
        ("count", r.contas),
        ("length", r.comprimento),
        ("gravity", r.gravidade),
        ("iterations", 16.0),
    ] {
        g.set_param(corda, p, v);
    }
    liga(g, mastro, corda, u16::from(r.de_lado), false)?;
    liga(g, corda, corda, 2, true)?;
    let shape = no(g, "source.shape", -440.0, 120.0);
    forma(g, shape, r, collide)?;
    let vestida = if depois {
        let dup = no(g, "motion.duplicator", 20.0, 80.0);
        liga(g, shape, dup, 0, false)?;
        liga(g, corda, dup, 1, false)?;
        dup
    } else {
        liga(g, shape, corda, 3, false)?;
        corda
    };
    let lugar = no(g, "motion.transform", 240.0, 0.0);
    g.set_param(lugar, "offset_x", MASTRO_X[k]);
    g.set_param(lugar, "offset_y", MASTRO_Y);
    liga(g, vestida, lugar, 0, false)?;
    let saida = no(g, "motion.output", 460.0, 0.0);
    liga(g, lugar, saida, 0, false)?;
    Some((saida, corda, shape))
}

/// `=130`: `(saídas, cordas, shapes)` — a da esquerda com o `Collide` desligado, a da direita ligado.
pub(crate) fn monta(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
    r: Receita,
) -> Option<(Vec<NodeId>, [NodeId; 2], [NodeId; 2])> {
    let g = &mut doc.graph;
    let (s0, c0, f0) = linha(g, 0, false, false, r)?;
    let (s1, c1, f1) = linha(g, 1, true, false, r)?;
    g.validate(reg).ok()?;
    Some((vec![s0, s1], [c0, c1], [f0, f1]))
}

/// `=131`: `(saída, corda, shape)` — a corda vestida DEPOIS, com o `Collide` ligado.
pub(crate) fn monta_depois(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<(NodeId, NodeId, NodeId)> {
    let g = &mut doc.graph;
    let t = linha(g, 0, true, true, RECEITA)?;
    g.validate(reg).ok()?;
    Some(t)
}

pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    monta(doc, reg, RECEITA).map(|(s, _, _)| s)
}

pub(super) fn build_depois(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    monta_depois(doc, reg).map(|(s, _, _)| vec![s])
}

/// As fichas no canvas, por cima de cada corda.
pub(super) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    let ficha = |x: f32, t: &str| {
        crate::motion_demo_legend::Caption::new([x, MASTRO_Y + 4.0], t.to_string())
    };
    vec![
        ficha(MASTRO_X[0], "SEM Collide: atravessam-se"),
        ficha(MASTRO_X[1], "COM Collide: encostam"),
    ]
}

/// A ficha da `=131`.
pub(super) fn captions_depois() -> Vec<crate::motion_demo_legend::Caption> {
    vec![crate::motion_demo_legend::Caption::new(
        [MASTRO_X[0], MASTRO_Y + 4.0],
        "Collide DEPOIS da corda".to_string(),
    )]
}

/// **O roteiro que o dono segue** (`CLAUDE.md` §0.8).
pub(super) fn announce() {
    eprintln!(
        "\n[corda com colisor] DUAS CORDAS de contas, presas pela ponta da esquerda a um mastro que as\n\
         sacode para cima e para baixo: e' uma SIMULACAO de corda, SEM gravidade (para as voltas\n\
         ficarem no ar). Cada uma e' vestida por um cartao `Shape` (o disco). As duas sao IGUAIS,\n\
         menos um botao.\n\
         \n\
         (1) A da ESQUERDA (laranja): quando ela se dobra sobre si mesma, as contas ENTRAM umas\n    \
         nas outras.\n\
         (2) A da DIREITA (verde): as contas ENCOSTAM e a corda nunca se atravessa.\n\
         (3) No grafo ha' duas filas de cartoes. Na fila de CIMA (a da laranja), clique no cartao\n    \
         `Shape` e ligue a linha `Collide`. A laranja passa a segurar como a verde.\n\
         \n\
         DEU ERRADO se: a verde tambem se atravessar; se ligar o `Collide` da laranja nao mudar\n\
         nada; ou se a verde ficar esticada (muito mais comprida que a laranja).\n"
    );
}

/// **O roteiro da `=131`** (`CLAUDE.md` §0.8).
pub(super) fn announce_depois() {
    eprintln!(
        "\n[colisor depois da corda] UMA CORDA de contas a ser sacudida para cima e para baixo. O\n\
         cartao `Shape` esta'\n\
         ligado DEPOIS da corda (num `Duplicator`), com o `Collide` ligado — e as contas\n\
         atravessam-se na mesma.\n\
         \n\
         (1) O cartao `Shape` mostra um AVISO. Clique nele: a mensagem diz que ali o `Collide` so'\n    \
         arruma o desenho e manda ligar o Shape a entrada `shape` da corda.\n\
         (2) Arraste um fio da saida do `Shape` para a entrada `shape` do cartao `Verlet Rope`.\n    \
         A corda passa a segurar as contas, e o aviso desaparece.\n\
         \n\
         DEU ERRADO se: nao houver aviso no cartao `Shape`; ou se, depois do fio, as contas\n\
         continuarem a atravessar-se.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_corda_com_colisor_demo_tests.rs"]
pub(crate) mod corda_com_colisor_demo_tests;

#[cfg(test)]
#[path = "motion_colisor_botoes_tests.rs"]
mod botoes;
