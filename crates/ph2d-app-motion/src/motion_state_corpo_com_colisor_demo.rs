//! ⭐⭐⭐ **O CORPO MOLE COM COLISOR** (cena `=129`) — o Bug #15 fechado e a decisão do dono de
//! 08/10 (doc 124 §6): *o botão `Collide` do Shape segura também as bolinhas de um corpo mole*.
//!
//! Duas gelatinas iguais, feitas de PEÇAS (`Grid → Duplicator(Shape) → Soft Body`, a porta `shape`),
//! penduradas de um mastro que varre, com um CAMPO DE FORÇA (`force.attractor`) dentro de cada uma
//! a dobrá-las sobre si mesmas. A ÚNICA diferença é o `Collide` do Shape: desligado na da ESQUERDA
//! (as peças atravessam-se), ligado na da DIREITA (as peças encostam). O gesto do dono é ligar o
//! da esquerda e vê-la segurar.
//!
//! A lei: o corpo LÊ o colisor que as peças declaram e separa-as dentro de cada passo (o que o
//! `sim.step` faz numa zona); os pares que o repouso sobrepõe são do corpo, não do contacto; e o
//! laço sub-tica. (O «não incha» tem o gate dele no `soft_body_radius`.)

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O lado da malha, o passo e o raio do colisor — o raio MAIOR que meio passo: os vizinhos de
/// malha sobrepõem-se no repouso (é o caso da lei).
pub(crate) const LADO: f32 = 7.0;
pub(crate) const PASSO: f32 = 0.30;
/// ⚠️ **Metade do passo: as peças TOCAM-SE no repouso.** Com `0,28` (maior que meio passo) as
/// vizinhas sobrepunham-se por desenho e a FOTO não distinguia um corpo que segura de um que se
/// atravessa (as duas gelatinas pareciam um monte de discos). A lei do repouso sobreposto tem o
/// gate dela no `soft_body_radius` e no `a_body_of_pieces…`.
pub(crate) const RAIO: f32 = 0.15;
/// Onde cada coluna vive, em mundo.
pub(crate) const COL_X: f32 = 3.2;
const MASTRO_PERIODO: f32 = 1.8;
/// O `Collider Shape` do cartão Shape em CÍRCULO (o índice `1` de `Box · Circle`, formato de
/// arquivo, append-only) — o colisor do mesmo tamanho do disco desenhado. O default é `Box`.
const COLISOR_CIRCULO: f32 = 1.0;
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
    /// Os sub-passos do corpo: um campo forte aperta o corpo inteiro, e segurar o diâmetro contra
    /// essa pressão pede TEMPO (medido: as varreduras do contacto não mudam nada).
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

/// Uma coluna: as peças (Grid → Duplicator com o Shape), o corpo feito DELAS, o campo e o mastro.
/// A ÚNICA diferença entre as duas colunas é o `Collide` do Shape. Devolve `(saída, corpo, shape)`.
fn coluna(g: &mut Graph, k: usize, collide: bool, r: Receita) -> Option<(NodeId, NodeId, NodeId)> {
    #[expect(clippy::cast_precision_loss, reason = "duas colunas")]
    let y = 380.0 * k as f32;
    let no = |g: &mut Graph, tipo: &str, x: f32, dy: f32| {
        let n = g.add_node(tipo.to_string());
        g.set_pos(n, Pos { x, y: y + dy });
        n
    };
    // As PEÇAS: uma grelha de pontos vestida com o Shape — o corpo mole é feito delas.
    let grelha = no(g, "motion.grid", -440.0, 0.0);
    g.set_param(grelha, "rows", LADO);
    g.set_param(grelha, "cols", LADO);
    g.set_param(grelha, "gap_x", PASSO);
    g.set_param(grelha, "gap_y", PASSO);
    let forma = no(g, "source.shape", -440.0, 160.0);
    use ph2d_node_motion_shape::param as p;
    let cor = if collide {
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
        (p::COLLIDE, f32::from(u8::from(collide))),
        (p::COLLIDER_SHAPE, COLISOR_CIRCULO),
        // O contorno do colisor desenhado por cima embaralhava a foto da verde.
        (p::SHOW_COLLIDER, 0.0),
    ] {
        g.set_param(forma, chave, valor);
    }
    let pecas = no(g, "motion.duplicator", -220.0, 80.0);
    liga(g, forma, pecas, 0, false)?;
    liga(g, grelha, pecas, 1, false)?;
    // O CORPO, feito das peças (a porta `shape`), pendurado de um mastro que varre.
    let mastro = no(g, "value.lfo", -220.0, -120.0);
    g.set_param(mastro, "period", MASTRO_PERIODO);
    g.set_param(mastro, "amplitude", r.curso);
    let corpo = no(g, "motion.soft_body", 0.0, 0.0);
    for (p, v) in [
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
    liga(g, pecas, corpo, 3, false)?;
    // O laço de estado: corpo --pre--> campo → corpo.state.
    let ima = no(g, "force.attractor", 220.0, 0.0);
    g.set_param(ima, "target_y", r.ima_y);
    g.set_param(ima, "strength", r.ima);
    g.set_param(ima, "radius", IMA_RAIO);
    liga(g, corpo, ima, 0, true)?;
    liga(g, ima, corpo, 2, false)?;
    // O desenho: o corpo já traz a aparência das peças.
    let lugar = no(g, "motion.transform", 440.0, 0.0);
    #[expect(clippy::cast_precision_loss, reason = "duas colunas")]
    g.set_param(lugar, "offset_x", (k as f32 - 0.5) * 2.0 * COL_X);
    g.set_param(lugar, "offset_y", 0.6);
    liga(g, corpo, lugar, 0, false)?;
    let saida = no(g, "motion.output", 660.0, 0.0);
    liga(g, lugar, saida, 0, false)?;
    Some((saida, corpo, forma))
}

/// O documento e os nós que os gates medem: `(saídas, corpos, shapes)`.
pub(crate) fn monta(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<(Vec<NodeId>, [NodeId; 2], [NodeId; 2])> {
    monta_com(doc, reg, RECEITA)
}

/// [`monta`] com outra receita — a sonda das receitas.
pub(crate) fn monta_com(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
    r: Receita,
) -> Option<(Vec<NodeId>, [NodeId; 2], [NodeId; 2])> {
    let g = &mut doc.graph;
    let (s0, b0, f0) = coluna(g, 0, false, r)?;
    let (s1, b1, f1) = coluna(g, 1, true, r)?;
    g.validate(reg).ok()?;
    Some((vec![s0, s1], [b0, b1], [f0, f1]))
}

pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    monta(doc, reg).map(|(s, _, _)| s)
}

/// **O roteiro que o dono segue** (`CLAUDE.md` §0.8).
pub(super) fn announce() {
    eprintln!(
        "\n[corpo mole com colisor] DUAS GELATINAS de bolinhas, penduradas e a balancar: e' uma\n\
         SIMULACAO de corpo mole. Dentro de cada uma ha' um IMA (um campo de forca) que a puxa\n\
         para o meio e a DOBRA sobre si mesma. As duas sao IGUAIS, menos um botao.\n\
         \n\
         (1) A da ESQUERDA (laranja): as bolinhas ENTRAM umas nas outras quando ela se dobra.\n\
         (2) A da DIREITA (verde): as bolinhas ENCOSTAM e nao se atravessam.\n\
         (3) No grafo (em baixo) ha' duas filas de cartoes. Na fila de CIMA (a da laranja), clique\n    \
         no cartao `Shape` e ligue a linha `Collide`. A laranja passa a segurar as bolinhas\n    \
         como a verde.\n\
         (4) Desligue de novo: as bolinhas voltam a entrar umas nas outras.\n\
         \n\
         DEU ERRADO se: a verde tambem virar um amontoado de bolinhas umas por cima das outras; se\n\
         ligar o `Collide` da laranja nao mudar nada; ou se alguma das duas parar de mexer.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_corpo_com_colisor_demo_tests.rs"]
mod tests;
