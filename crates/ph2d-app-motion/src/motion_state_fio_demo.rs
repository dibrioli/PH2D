//! ⭐⭐⭐ **UM NÚMERO QUE MANDA EM TUDO** (`PH2D_GPU_COOK_DEMO=116`) — a cena que mostra o que a
//! W1a do [ciclo 6](../../docs/Motion%20Nodes/110_ciclo_6_valor_e_pulso.md) fez.
//!
//! ## O que ela encena
//!
//! Um pano no TECTO da grelha (`LADO_MAX_DE_GRELHA²`, hoje **32 761 quadrados**, `Shape` →
//! `Duplicator`) e **um fio**: um `value.lfo` a comandar o tamanho de todos eles. É a cadeia mais simples que existe com um nó de
//! valor — e era exactamente ela que, até à W1a, ia para a CPU **por causa do fio**. Medido a
//! 07/10 pela porta do produto (doc 110 §14.2 (5)): `0,051 ms` por quadro na placa, `0,134` com o
//! fio a derrubá-la (`2,6×`) — invisível a 60 fps; o que a cena ensina é a ROTA, no terminal.
//! Com os quadrados (15/10, release, `sonda_o_relogio_do_ciclo_6`, mínimo de 7 rodadas): `0,185`
//! na placa (rota HÍBRIDA: só a forma na CPU) contra `0,067` sem forma, e `0,476` com o fio a
//! derrubá-la (`2,5×`); o quadro inteiro do app fica nos `16,7 ms` (vsync) nos dois.
//!
//! ⚠️ **A cena não tem A/B de quadrantes, e é de propósito:** o que mudou não é uma aparência, é
//! **onde a cena corre**. Os dois lados da comparação não cabem no mesmo ecrã — eles são a MESMA
//! cena com o interruptor `PH2D_MOTION_DRIVEN_GPU` num sítio e noutro. *Uma cena que comparasse
//! duas metades estaria a ensinar que isto é uma questão de desenho.*
//!
//! ⚠️ **E é por isso que ela é GRANDE.** A `=111`..`=115` têm dezenas de peças porque ensinam uma
//! LEI; esta ensina um CUSTO, e o custo cresce com as peças. ⛔ O `50×` que aqui esteve era de
//! 4,19 M partículas (doc 98), que o tecto por nó já não deixa num nó só.
//!
//! ## Porquê o `scale` e não o `move`
//!
//! O tamanho é a propriedade que se lê **sem referência**: uma peça maior vê-se sozinha, enquanto
//! uma peça deslocada precisa de saber de onde partiu. Com o pano inteiro a respirar junto, o fio
//! está à vista em qualquer quadro parado.

use crate::motion_demo_legend::Caption;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// ⛔⛔⛔ **O lado do pano é DERIVADO do tecto** (ordens do dono de 2026-09-22 e seguintes: hoje
/// `181`, `32 761` peças): a grelha clampa o PRODUTO, logo um lado escrito à mão deixaria a cena a
/// anunciar uma população que ela não produz — e a legenda, o anúncio e o tutorial estiveram a
/// dizer `102 400` até 07/10 (doc 110 §14.2 (5)). *Uma cena que anuncia o que não produz ensina o
/// contrário do que acontece* (§5.0): os três lêem daqui.
const LADO: f32 = ph2d_nodegraph::node::LADO_MAX_DE_GRELHA as f32;
/// O vão entre peças. ⚠️ É ele que decide se o pano se LÊ: a `0,012` as `181` colunas cabiam em
/// `2,2` unidades e cada peça media menos de um pixel na câmara de abertura; a `0,03` o pano tem
/// `5,4` unidades e ainda cabe inteiro nela (`10` de altura).
const VAO: f32 = 0.03;
/// O `Size` do quadrado carimbado — o SEMI-lado (o `Rectangle` vive em raio `1`), `0,3` do vão:
/// no pico da onda (`× 1,6`) o lado é `0,96` do vão e as peças não se tocam.
pub(super) const MEIO_PECA: f32 = VAO * 0.3;
/// A cor das peças, saturada para o pano se ver contra o fundo escuro.
const COR: [f32; 3] = [0.95, 0.55, 0.2];
/// O período do oscilador, em segundos — lento o suficiente para a respiração se ver num quadro
/// parado, e não tão lento que o dono tenha de esperar para saber se algo mexe.
const PERIODO: f32 = 2.0;
/// Quanto o tamanho varia. `1` seria a peça a desaparecer no vale da onda.
const AMPLITUDE: f32 = 0.6;

/// A legenda que a cena pousa no canvas.
pub(super) fn captions() -> Vec<Caption> {
    // Debaixo do pano, derivada da meia-altura dele.
    vec![Caption::new(
        [0.0, -(LADO - 1.0) * VAO * 0.5 - 0.6],
        format!("{} pieces, ONE wire: the Value LFO sizes them all", pecas()),
    )]
}

/// Quantas peças o pano tem, com o separador de milhares que o dono lê (`32 761`).
pub(super) fn pecas() -> String {
    let n = ph2d_nodegraph::node::LADO_MAX_DE_GRELHA * ph2d_nodegraph::node::LADO_MAX_DE_GRELHA;
    match n {
        0..1000 => n.to_string(),
        _ => format!("{} {:03}", n / 1000, n % 1000),
    }
}

/// O que carimba as peças.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Carimbo {
    /// O quadrado da cena.
    Quadrado,
    /// Outro `kind` da `source.shape` (o caso do dono: `5` = estrela).
    Kind(f32),
    /// NENHUM — a corrente só de pontos (os marcadores) que a cena desenhou até 15/10; o controlo
    /// das sondas de custo.
    Nenhum,
}

/// Monta a cena. Devolve o sink.
///
/// ⚠️ A forma vem da env aqui e é ARGUMENTO em [`build_com`]: um ramo de ambiente dentro da lei
/// torna a cena inalcançável de um teste — e foi um teste que precisou dela primeiro.
/// `PH2D_FIO_FORMA=<kind>` troca o quadrado por outra forma; um valor negativo tira o carimbo.
pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    let carimbo = match std::env::var("PH2D_FIO_FORMA")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
    {
        None => Carimbo::Quadrado,
        Some(k) if k < 0.0 => Carimbo::Nenhum,
        Some(k) => Carimbo::Kind(k),
    };
    build_com(doc, reg, carimbo)
}

/// `grid → duplicator(shape) → scale ← lfo → output`, com o carimbo escolhido por quem chama —
/// ver [`build`].
///
/// ⭐ A grade dá os LUGARES, o `Duplicator` carimba um quadrado em cada um (o molde da `=118`), e o
/// fio comanda o tamanho do que foi carimbado. ⚠️ O carimbo vem ANTES da escala: o `motion.scale`
/// multiplica a coluna `size` que o carimbo escreveu (o semi-lado da forma).
pub(super) fn build_com(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
    carimbo: Carimbo,
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

    let pecas = if carimbo == Carimbo::Nenhum {
        grelha
    } else {
        let forma = super::jogo_demo::forma(
            g,
            &super::jogo_demo::no_em(0.0),
            ph2d_node_motion_shape::ShapeKind::Rectangle,
            MEIO_PECA,
            COR,
            -160.0,
            200.0,
        )?;
        if let Carimbo::Kind(k) = carimbo {
            g.set_param(forma, ph2d_node_motion_shape::param::KIND, k);
        }
        let dup = g.add_node("motion.duplicator");
        g.set_pos(dup, Pos { x: -160.0, y: 0.0 });
        for (de, porta) in [(forma, 0u16), (grelha, 1)] {
            g.connect(Edge {
                from: (de, 0),
                to: (dup, porta),
                delayed: false,
            })
            .ok()?;
        }
        dup
    };
    for (de, para) in [(pecas, escala), (escala, saida)] {
        g.connect(Edge {
            from: (de, 0),
            to: (para, 0),
            delayed: false,
        })
        .ok()?;
    }
    // ⭐ **O FIO** — o assunto da cena. É esta linha, e só esta, que até à W1a mandava as peças
    // todas para a CPU.
    g.drive_param(escala, "amount", (lfo, 0)).ok()?;

    for (no, x, y) in [
        (grelha, -360.0, 0.0),
        (lfo, -160.0, 360.0),
        (escala, 40.0, 0.0),
        (saida, 240.0, 0.0),
    ] {
        g.set_pos(no, Pos { x, y });
    }
    g.validate(reg).ok()?;
    Some(vec![saida])
}

#[cfg(test)]
#[path = "motion_state_fio_demo_tests.rs"]
mod tests;
