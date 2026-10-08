//! ⭐⭐⭐ **O CORPO MOLE COM COLISOR** (cena `=129`) — o Bug #15 fechado (doc 124 §2), na tela.
//!
//! Duas gelatinas iguais, penduradas de um mastro que varre, desenhadas como DISCOS do tamanho do
//! colisor, e um CAMPO DE FORÇA (`force.attractor`) dentro de cada uma a puxá-las sobre si
//! mesmas. A da ESQUERDA não tem colisor: os discos atravessam-se. A da DIREITA tem o
//! `motion.collide` no laço de estado: os discos encostam e não se atravessam. (O «não incha» —
//! o que o ciclo 14 lia como *«o que um particle radius faz»* era o defeito — tem o gate dele no
//! `soft_body_radius`: aqui a laranja COLAPSA, e comparar tamanhos com ela não diria nada.)
//!
//! A lei que a cena mostra: os pares que o REPOUSO já sobrepõe (os vizinhos de malha) são do
//! corpo, não do colisor; o contacto entra na velocidade; e o laço sub-tica (`substeps = 8`).

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O lado da malha, o passo e o raio do colisor — o raio MAIOR que meio passo: os vizinhos de
/// malha sobrepõem-se no repouso (é o caso da lei).
pub(crate) const LADO: f32 = 7.0;
pub(crate) const PASSO: f32 = 0.30;
pub(crate) const RAIO: f32 = 0.28;
/// Onde cada coluna vive, em mundo.
pub(crate) const COL_X: f32 = 3.2;
const MASTRO_PERIODO: f32 = 1.8;
/// O alcance do campo, no referencial da simulação (a âncora é a origem).
const IMA_RAIO: f32 = 3.0;

/// Os números que decidem se a gelatina se DOBRA — escolhidos pela sonda `sonda_as_receitas`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Receita {
    pub rigidez: f32,
    pub ima: f32,
    /// A altura do íman no referencial da simulação (a âncora é a origem).
    pub ima_y: f32,
    pub curso: f32,
    /// As varreduras do colisor e os sub-passos do corpo: um campo forte aperta o corpo inteiro,
    /// e segurar o diâmetro contra essa pressão pede mais iterações (o limite clássico do PBD).
    pub iteracoes: f32,
    pub sub_passos: f32,
    /// Regiões sobrepostas do casamento de forma: com UMA moldura o corpo só translada e roda —
    /// não DOBRA (o cabeçalho do `Params::clusters` do nó).
    pub clusters: f32,
}

/// Medido (`sonda_as_receitas`, pela bomba, três varreduras): um corpo mole (`0,03`) com `3`
/// regiões (com UMA moldura o casamento de forma não dobra — só balança rígido, e a foto mostrava
/// as duas gelatinas iguais) e um íman forte (`400`) a meia altura — sem colisor o pior par solto
/// fica em `0,14` a MEDIANA da 2.ª metade (os discos encavalitam-se o tempo todo), com ele nunca
/// desce de `0,53`. ⚠️ Os SUB-PASSOS decidem, as iterações do colisor não (`8`/`32`/`64` dão o
/// mesmo): a `4` sub-passos o mesmo íman vence o colisor.
pub(crate) const RECEITA: Receita = Receita {
    rigidez: 0.03,
    ima: 400.0,
    ima_y: 0.5,
    curso: 2.0,
    iteracoes: 8.0,
    sub_passos: 8.0,
    clusters: 3.0,
};

fn liga(g: &mut Graph, de: NodeId, para: NodeId, porta: u16, pre: bool) -> Option<()> {
    g.connect(Edge {
        from: (de, 0),
        to: (para, porta),
        delayed: pre,
    })
    .ok()
}

fn indice_do_circulo() -> Option<f32> {
    let i = ph2d_node_motion_shape::ALL_KINDS
        .iter()
        .position(|k| *k == ph2d_node_motion_shape::ShapeKind::Circle)?;
    #[expect(
        clippy::cast_precision_loss,
        reason = "um indice de enum, sempre pequeno"
    )]
    Some(i as f32)
}

/// Uma coluna: o corpo, o campo, o colisor (se `com_colisor`) e o desenho em discos.
/// Devolve `(saída, corpo, colisor)`.
fn coluna(
    g: &mut Graph,
    k: usize,
    com_colisor: bool,
    r: Receita,
) -> Option<(NodeId, NodeId, Option<NodeId>)> {
    #[expect(clippy::cast_precision_loss, reason = "duas colunas")]
    let y = 380.0 * k as f32;
    let no = |g: &mut Graph, tipo: &str, x: f32| {
        let n = g.add_node(tipo.to_string());
        g.set_pos(n, Pos { x, y });
        n
    };
    let mastro = no(g, "value.lfo", -220.0);
    g.set_param(mastro, "period", MASTRO_PERIODO);
    g.set_param(mastro, "amplitude", r.curso);
    let corpo = no(g, "motion.soft_body", 0.0);
    for (p, v) in [
        ("rows", LADO),
        ("cols", LADO),
        ("spacing", PASSO),
        ("gravity", 9.0),
        ("stiffness", r.rigidez),
        ("damping", 0.03),
        ("pin", 1.0),
        ("substeps", r.sub_passos),
        ("clusters", r.clusters),
    ] {
        g.set_param(corpo, p, v);
    }
    liga(g, mastro, corpo, 0, false)?;
    // O laço de estado: corpo --pre--> campo [→ colisor] → corpo.state.
    let ima = no(g, "force.attractor", 220.0);
    g.set_param(ima, "target_y", r.ima_y);
    g.set_param(ima, "strength", r.ima);
    g.set_param(ima, "radius", IMA_RAIO);
    liga(g, corpo, ima, 0, true)?;
    let colisor = if com_colisor {
        let c = no(g, "motion.collide", 440.0);
        g.set_param(c, "radius", RAIO);
        g.set_param(c, "iterations", r.iteracoes);
        liga(g, ima, c, 0, false)?;
        liga(g, c, corpo, 2, false)?;
        Some(c)
    } else {
        liga(g, ima, corpo, 2, false)?;
        None
    };
    // O desenho: discos do raio do colisor, em cada partícula.
    let lugar = no(g, "motion.transform", 660.0);
    #[expect(clippy::cast_precision_loss, reason = "duas colunas")]
    g.set_param(lugar, "offset_x", (k as f32 - 0.5) * 2.0 * COL_X);
    g.set_param(lugar, "offset_y", 0.6);
    liga(g, corpo, lugar, 0, false)?;
    let forma = no(g, "source.shape", 660.0);
    g.set_pos(
        forma,
        Pos {
            x: 660.0,
            y: y + 160.0,
        },
    );
    use ph2d_node_motion_shape::param as p;
    let cor = if com_colisor {
        [0.30, 0.80, 0.45]
    } else {
        [0.95, 0.45, 0.30]
    };
    for (chave, valor) in [
        (p::KIND, indice_do_circulo()?),
        (p::SIZE, RAIO),
        (p::FILL, 1.0),
        (p::FILL_R, cor[0]),
        (p::FILL_G, cor[1]),
        (p::FILL_B, cor[2]),
        (p::FILL_A, 0.75),
        (p::STROKE_WIDTH, 0.03),
        (p::STROKE_R, 0.08),
        (p::STROKE_G, 0.08),
        (p::STROKE_B, 0.10),
        (p::STROKE_A, 1.0),
    ] {
        g.set_param(forma, chave, valor);
    }
    let dup = no(g, "motion.duplicator", 880.0);
    liga(g, forma, dup, 0, false)?;
    liga(g, lugar, dup, 1, false)?;
    let saida = no(g, "motion.output", 1100.0);
    liga(g, dup, saida, 0, false)?;
    Some((saida, corpo, colisor))
}

/// O documento e os nós que os gates medem: `(saídas, corpos, colisor)`.
pub(crate) fn monta(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<(Vec<NodeId>, [NodeId; 2], NodeId)> {
    monta_com(doc, reg, RECEITA)
}

/// [`monta`] com outra receita — a sonda das receitas.
pub(crate) fn monta_com(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
    r: Receita,
) -> Option<(Vec<NodeId>, [NodeId; 2], NodeId)> {
    let g = &mut doc.graph;
    let (s0, b0, _) = coluna(g, 0, false, r)?;
    let (s1, b1, c1) = coluna(g, 1, true, r)?;
    g.validate(reg).ok()?;
    Some((vec![s0, s1], [b0, b1], c1?))
}

pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    monta(doc, reg).map(|(s, _, _)| s)
}

/// **O roteiro que o dono segue** (`CLAUDE.md` §0.8).
pub(super) fn announce() {
    eprintln!(
        "\n[corpo mole com colisor] DUAS GELATINAS de bolinhas, penduradas e a balançar: e' uma\n\
         SIMULACAO de corpo mole. Dentro de cada uma ha' um IMA (um campo de forca) que a puxa\n\
         para o meio e a DOBRA sobre si mesma.\n\
         \n\
         (1) A da ESQUERDA (laranja) nao tem colisor: quando o ima a dobra, as bolinhas ENTRAM\n    \
         umas nas outras e a gelatina fica um borrao.\n\
         (2) A da DIREITA (verde) tem o colisor: as bolinhas ENCOSTAM e nao se atravessam — da' para\n    \
         contar cada uma, mesmo dobrada.\n\
         \n\
         DEU ERRADO se: a verde tambem virar borrao (bolinhas umas por cima das outras), ou se\n\
         alguma das duas parar de mexer.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_corpo_com_colisor_demo_tests.rs"]
mod tests;
