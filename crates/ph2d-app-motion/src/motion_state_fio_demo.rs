//! ⭐⭐⭐ **UM NÚMERO QUE MANDA EM TUDO** (`PH2D_GPU_COOK_DEMO=116`) — a cena que mostra o que a
//! W1a do [ciclo 6](../../docs/Motion%20Nodes/110_ciclo_6_valor_e_pulso.md) fez.
//!
//! ## O que ela encena
//!
//! Um pano de **102 400 peças** e **um fio**: um `value.lfo` a comandar o tamanho de todas elas.
//! É a cadeia mais simples que existe com um nó de valor — e era exactamente ela que, até esta
//! wave, trocava o caminho de `3,85 ms` pelo de `195,9 ms` **por causa do fio**.
//!
//! ⚠️ **A cena não tem A/B de quadrantes, e é de propósito:** o que mudou não é uma aparência, é
//! **onde a cena corre**. Os dois lados da comparação não cabem no mesmo ecrã — eles são a MESMA
//! cena com o interruptor `PH2D_MOTION_DRIVEN_GPU` num sítio e noutro. *Uma cena que comparasse
//! duas metades estaria a ensinar que isto é uma questão de desenho.*
//!
//! ⚠️ **E é por isso que ela é GRANDE.** A `=111`..`=115` têm dezenas de peças porque ensinam uma
//! LEI; esta ensina um CUSTO, e um custo de `50×` sobre dez peças não se vê. O número de peças é o
//! sujeito, não o cenário.
//!
//! ## Porquê o `scale` e não o `move`
//!
//! O tamanho é a propriedade que se lê **sem referência**: uma peça maior vê-se sozinha, enquanto
//! uma peça deslocada precisa de saber de onde partiu. Com 102 400 delas a respirar juntas, o fio
//! está à vista em qualquer quadro parado.

use crate::motion_demo_legend::Caption;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// ⛔⛔⛔ **O lado do pano DESCEU de `320` para `128` em 2026-09-22, por ordem do dono** (*«vamos
/// efetivar o limite de 16 384»*): a grelha clampa o PRODUTO, logo um `320` escrito continuaria a
/// entregar `16 384` e a cena diria `102 400` em toda a prosa dela. ⚠️ *Uma cena que anuncia uma
/// população que ela não produz ensina o contrário do que acontece* (§5.0) — por isso ele é hoje
/// **derivado** do tecto.
///
/// ⚠️ **E a cena perdeu o SUJEITO que a justificava**, como a `=126` da mesma ordem: as `102 400`
/// peças eram o ponto (*«um custo de `50×` sobre dez peças não se vê»*), e a `16 384` a margem
/// encolhe. *Ela não ficou errada — ficou a mostrar menos do que existia para mostrar.*
///
/// ⚠️ O lado de `320 × 320 = 102 400` era a mesma contagem das tabelas do doc 98, para que
/// o que se vê aqui e o que lá está medido sejam o MESMO número de objectos.
const LADO: f32 = ph2d_nodegraph::node::LADO_MAX_DE_GRELHA as f32;
/// O vão entre peças. ⚠️ Apertado de propósito: a `320` de lado, um vão folgado põe o pano fora do
/// alcance do zoom e a cena ensinaria *«não aparece nada»*.
const VAO: f32 = 0.012;
/// O período do oscilador, em segundos — lento o suficiente para a respiração se ver num quadro
/// parado, e não tão lento que o dono tenha de esperar para saber se algo mexe.
const PERIODO: f32 = 2.0;
/// Quanto o tamanho varia. `1` seria a peça a desaparecer no vale da onda.
const AMPLITUDE: f32 = 0.6;

/// A legenda que a cena pousa no canvas.
pub(super) fn captions() -> Vec<Caption> {
    vec![Caption::new(
        [0.0, -2.6],
        "102 400 pecas, UM fio: o Value LFO manda no tamanho de todas",
    )]
}

/// Monta a cena. Devolve o sink.
///
/// ⚠️ A forma vem da env aqui e é ARGUMENTO em [`build_com`]: um ramo de ambiente dentro da lei
/// torna a cena inalcançável de um teste — e foi um teste que precisou dela primeiro.
pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    let forma = std::env::var("PH2D_FIO_FORMA")
        .ok()
        .and_then(|v| v.parse::<f32>().ok());
    build_com(doc, reg, forma)
}

/// A cena, com a FORMA escolhida por quem chama — ver [`build`].
pub(super) fn build_com(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
    forma_kind: Option<f32>,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};

    let g = &mut doc.graph;
    let grelha = g.add_node("motion.grid");
    g.set_param(grelha, "rows", LADO);
    g.set_param(grelha, "cols", LADO);
    g.set_param(grelha, "gap_x", VAO);
    g.set_param(grelha, "gap_y", VAO);

    let escala = g.add_node("motion.scale");
    let saida = g.add_node("motion.output");
    let lfo = g.add_node("value.lfo");
    g.set_param(lfo, "period", PERIODO);
    g.set_param(lfo, "amplitude", AMPLITUDE);
    // O centro da respiração: sem ele a onda desceria abaixo de zero e metade do ciclo seria uma
    // peça de tamanho negativo.
    g.set_param(lfo, "offset", 1.0);

    for (de, para) in [(grelha, escala), (escala, saida)] {
        g.connect(Edge {
            from: (de, 0),
            to: (para, 0),
            delayed: false,
        })
        .ok()?;
    }
    // ⭐ **O FIO** — o assunto da cena. É esta linha, e só esta, que até à W1a mandava as 102 400
    // peças para a CPU.
    g.drive_param(escala, "amount", (lfo, 0)).ok()?;

    // ⭐⭐ **O CASO DO DONO, reproduzível** (report de 2026-09-14: *«usando shape (exemplo: star)
    // fps cai para 27»*). Com `PH2D_FIO_FORMA=<índice do kind>` as peças passam a ser uma FORMA
    // carimbada em vez do quad de omissão.
    //
    // ⚠️ **É um knob de DIAGNÓSTICO e não um modo da cena**: a `=116` ensina onde a cena corre, e
    // uma forma muda a resposta por DOIS motivos que nada têm a ver com o fio (o `source.shape`
    // declara-se fonte vectorial viva e recusa o grafo antes de planear; o `motion.duplicator` é
    // CPU-only). *Sem uma porta assim, o report do dono não é reproduzível por ninguém.*
    if let Some(kind) = forma_kind {
        let forma = g.add_node("source.shape");
        g.set_param(forma, "kind", kind);
        let carimbo = g.add_node("motion.duplicator");
        g.connect(Edge {
            from: (forma, 0),
            to: (carimbo, 0),
            delayed: false,
        })
        .ok()?;
        g.connect(Edge {
            from: (escala, 0),
            to: (carimbo, 1),
            delayed: false,
        })
        .ok()?;
        // O carimbo entra ENTRE a escala e a saída: o fio continua a comandar o tamanho.
        g.disconnect(saida, 0);
        g.connect(Edge {
            from: (carimbo, 0),
            to: (saida, 0),
            delayed: false,
        })
        .ok()?;
        g.set_pos(forma, Pos { x: -60.0, y: 200.0 });
        g.set_pos(carimbo, Pos { x: 40.0, y: 0.0 });
    }

    for (no, x, y) in [
        (grelha, -260.0, 0.0),
        (lfo, -260.0, 160.0),
        (escala, -60.0, 0.0),
        (saida, 140.0, 0.0),
    ] {
        g.set_pos(no, Pos { x, y });
    }
    g.validate(reg).ok()?;
    Some(vec![saida])
}

#[cfg(test)]
#[path = "motion_state_fio_demo_tests.rs"]
mod tests;
