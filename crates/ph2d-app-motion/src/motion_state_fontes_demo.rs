//! ⭐⭐⭐ **DE ONDE VÊM AS COISAS** (`PH2D_GPU_COOK_DEMO=119`) — a cena do **ciclo 8**
//! ([doc 113](../../docs/Motion%20Nodes/113_ciclo_8_fontes_e_dados.md) §8).
//!
//! ## O que ela ensina, e por que são TRÊS fileiras
//!
//! Os sete ciclos anteriores fizeram coisas **às** peças — puseram-nas em grelha, moveram-nas,
//! dobraram-nas, escolheram quem é afectado, entregaram-nas a uma lei, deram-lhes uma cara. Este
//! responde à pergunta que vem ANTES de todas: **de onde vem a peça**. Há três respostas
//! diferentes, e é isso que parte a cena:
//!
//! ```text
//!   CIMA    O QUE EU FIZ      Text «OLA»               |  Shape (uma estrela)
//!   MEIO    O QUE EU TENHO    Table: as linhas         |  + a COLUNA a dar a altura
//!   BAIXO   O QUE A CENA DÁ   Emitter: as partículas   |  Camera: o tamanho no ECRÃ
//! ```
//!
//! ⚠️ **A fileira do MEIO é o par do ciclo 7** — o mesmo ficheiro dos dois lados, e à direita **um
//! cartão a mais** (`value.attribute` + `motion.drive`): *um ficheiro não traz só LINHAS, traz
//! COLUNAS*. É o passo que transforma uma fila de quadrados num gráfico sem escrever um número.
//!
//! ⚠️⚠️ **A fileira de BAIXO é a única que pede a MÃO do dono:** as peças da direita mantêm o
//! tamanho no **ecrã** enquanto ele dá zoom, e **sem zoom os dois panos são iguais**. O anúncio
//! tem de o dizer no passo, senão a metade nova lê-se como inerte — a lei do controlo que não
//! pode passar pelo motivo errado.
//!
//! ⛔ **Duas fontes do grupo NÃO estão aqui, e as duas têm smoke PRÓPRIO:** o `source.lsystem`
//! (cena **`=108`**, com auditoria própria no doc 96) e o `source.object` — este porque uma cena
//! de ciclo constrói um **GRAFO**, e um objecto é uma coisa na **HIERARQUIA**
//! (`PH2D_MOTION_OBJ_SMOKE`). *Uma cena de ciclo mostra a lei; o catálogo tem as cenas da
//! conferência.*
//!
//! ⚠️⚠️ **Esta cena inteira coze na CPU, e é PROPRIEDADE, não defeito:** um `source.shape` ou um
//! `source.text` no documento recusa o dispositivo para o grafo TODO
//! (`gpu::graph_has_live_vector_source` — a placa não tem rota para `geometry_id`). Com seis panos
//! pequenos não se vê; é a §7 que o mede, e é por isso que a medição do grupo não sai desta cena.

use crate::motion_demo_legend::Caption;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};

/// A que distância do centro cada metade vive — o mesmo `2,9` da `=118`.
const COL_X: f32 = 2.9;
/// A distância vertical entre fileiras. ⚠️ A câmara abre com **10** unidades de altura, e três
/// fileiras mais as fichas têm de caber nela **sem zoom** — senão o primeiro passo do smoke é
/// procurar a cena.
const ROW_GAP: f32 = 3.2;
/// A que altura acima do centro do pano pousa a ficha.
const FICHA_Y: f32 = 1.5;

/// A palavra que a fonte de texto escreve. ⚠️ **Três letras**, e curta de propósito: o passo do
/// tutorial manda TROCÁ-LA, e uma frase faria as peças ficarem minúsculas ao caber no pano.
const PALAVRA: &str = "OLA";
/// O tamanho do texto, em unidades de mundo.
const TEXTO_TAM: f32 = 1.6;

/// O tamanho da estrela.
const FORMA_TAM: f32 = 1.3;

/// O passo entre as linhas da tabela — as doze cabem no pano.
const TABELA_VAO: f32 = 0.42;
/// O tamanho da peça de uma linha da tabela.
const TABELA_PECA: f32 = 0.3;
/// A coluna do ficheiro que dá a ALTURA no pano da direita.
const COLUNA: &str = "vendas";
/// Quanto do mundo vale uma unidade da coluna (`7..35` ⇒ `0,28..1,4`).
const COLUNA_ESCALA: f32 = 0.04;

/// Partículas por segundo e a vida de cada uma — a nuvem em regime é o produto das duas.
const EMISSOR_RITMO: f32 = 60.0;
const EMISSOR_VIDA: f32 = 2.0;
const EMISSOR_PECA: f32 = 0.16;

/// O lado da grelha do pano da câmara.
const CAM_LADO: f32 = 3.0;
const CAM_VAO: f32 = 0.9;
/// Quantos PIXELS de ecrã cada peça do pano da câmara mede, esteja o zoom onde estiver.
/// ⚠️ **É a alavanca do passo do zoom** e não um teto: `size = pixels ÷ zoom`.
const CAM_PIXELS: f32 = 40.0;

/// A escada do `motion.drive`: o CANAL (`X · Y · Rotation · Size …`).
const CANAL_Y: f32 = 1.0;
const CANAL_SIZE: f32 = 3.0;
/// A escada do MODO (`Add · Set · Multiply · Subtract · Divide …`).
const MODO_ADD: f32 = 0.0;
const MODO_DIVIDE: f32 = 4.0;

fn no(doc: &mut MotionDoc, tipo: &str, x: f32, y: f32) -> NodeId {
    let n = doc.graph.add_node(tipo);
    doc.graph.set_pos(n, Pos { x, y });
    n
}

fn liga(doc: &mut MotionDoc, de: NodeId, para: (NodeId, u16)) -> Option<()> {
    doc.graph
        .connect(Edge {
            from: (de, 0),
            to: para,
            delayed: false,
        })
        .ok()
}

/// `escala → move(centro) → output`, o final comum de todos os seis panos. ⚠️ **Ele é o mesmo em
/// toda a cena de propósito:** o que muda de pano para pano é só a FONTE, que é a lição.
fn pousa(
    doc: &mut MotionDoc,
    de: NodeId,
    tamanho: f32,
    centro: [f32; 2],
    y: f32,
) -> Option<NodeId> {
    let s = no(doc, "motion.scale", 120.0, y);
    doc.graph.set_param(s, "amount", tamanho);
    let mv = no(doc, "motion.move", 300.0, y);
    doc.graph.set_param(mv, "dx", centro[0]);
    doc.graph.set_param(mv, "dy", centro[1]);
    let o = no(doc, "motion.output", 480.0, y);
    liga(doc, de, (s, 0))?;
    liga(doc, s, (mv, 0))?;
    liga(doc, mv, (o, 0))?;
    Some(o)
}

fn fileira_y(k: usize) -> f32 {
    match k {
        0 => ROW_GAP,
        1 => 0.0,
        _ => -ROW_GAP,
    }
}

fn grafo_y(k: usize, lado: usize) -> f32 {
    #[expect(clippy::cast_precision_loss, reason = "seis cadeias")]
    let i = (k * 2 + lado) as f32;
    -600.0 + i * 240.0
}

pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    let mut sinks = Vec::new();

    // ── CIMA: O QUE EU FIZ — um texto que escrevi · uma forma que escolhi ────────────────
    {
        let y = grafo_y(0, 0);
        let t = no(doc, "source.text", -400.0, y);
        doc.graph
            .set_text_param(t, ph2d_node_source_text::TEXT_KEY, PALAVRA);
        doc.graph
            .set_param(t, ph2d_node_source_text::param::SIZE, TEXTO_TAM);
        doc.graph.set_label(t, "Text: a palavra");
        sinks.push(pousa(doc, t, 1.0, [-COL_X, fileira_y(0)], y)?);
    }
    {
        let y = grafo_y(0, 1);
        let f = no(doc, "source.shape", -400.0, y);
        // A ESCADA lida do tipo, nunca um literal — quem acrescentar uma forma no meio da lista
        // moveria o índice, e a cena passaria a mostrar outra coisa em silêncio.
        doc.graph.set_param(
            f,
            ph2d_node_motion_shape::param::KIND,
            ph2d_node_motion_shape::ShapeKind::Star as u8 as f32,
        );
        doc.graph
            .set_param(f, ph2d_node_motion_shape::param::SIZE, FORMA_TAM);
        doc.graph.set_label(f, "Shape: a forma");
        sinks.push(pousa(doc, f, 1.0, [COL_X, fileira_y(0)], y)?);
    }

    // ── MEIO: O QUE EU TENHO — as linhas · as linhas MAIS uma coluna ────────────────────
    let ficheiro = super::conferencia_mods::conferencia_demos_table::write_fixture();
    let ficheiro = ficheiro.to_string_lossy().into_owned();
    for lado in 0..2 {
        let y = grafo_y(1, lado);
        let x = if lado == 0 { -COL_X } else { COL_X };
        let t = no(doc, "source.table", -400.0, y);
        doc.graph
            .set_text_param(t, ph2d_node_source_table::FILE_KEY, &ficheiro);
        doc.graph
            .set_param(t, ph2d_node_source_table::param::SPACING, TABELA_VAO);
        doc.graph.set_label(
            t,
            if lado == 0 {
                "Table: as linhas"
            } else {
                "Table: o grafico"
            },
        );
        let ultimo = if lado == 0 {
            t
        } else {
            // ⭐ O cartão a mais: a COLUNA pelo nome, e o `Drive` a pô-la na altura.
            let a = no(doc, "value.attribute", -220.0, y);
            doc.graph
                .set_text_param(a, ph2d_node_value_attribute::ATTR_KEY, COLUNA);
            let d = no(doc, "motion.drive", -40.0, y);
            doc.graph.set_param(d, "channel", CANAL_Y);
            doc.graph.set_param(d, "mode", MODO_ADD);
            doc.graph.set_param(d, "scale", COLUNA_ESCALA);
            doc.graph.set_label(d, "Drive: a coluna «vendas»");
            liga(doc, t, (a, 0))?;
            liga(doc, t, (d, 0))?;
            liga(doc, a, (d, 1))?;
            d
        };
        sinks.push(pousa(doc, ultimo, TABELA_PECA, [x, fileira_y(1)], y)?);
    }

    // ── BAIXO: O QUE A CENA DÁ — as partículas · a VISTA ────────────────────────────────
    {
        let y = grafo_y(2, 0);
        let e = no(doc, "motion.emitter", -400.0, y);
        doc.graph.set_param(e, "rate", EMISSOR_RITMO);
        doc.graph.set_param(e, "life", EMISSOR_VIDA);
        doc.graph.set_param(e, "speed", 0.9);
        doc.graph.set_param(e, "spread", 0.6);
        doc.graph.set_param(e, "angle", 90.0);
        doc.graph.set_label(e, "Emitter: de onde nascem");
        sinks.push(pousa(
            doc,
            e,
            EMISSOR_PECA,
            [-COL_X, fileira_y(2) - 1.0],
            y,
        )?);
    }
    {
        let y = grafo_y(2, 1);
        let g = no(doc, "motion.grid", -580.0, y);
        doc.graph.set_param(g, "rows", CAM_LADO);
        doc.graph.set_param(g, "cols", CAM_LADO);
        doc.graph.set_param(g, "gap_x", CAM_VAO);
        doc.graph.set_param(g, "gap_y", CAM_VAO);
        let cam = no(doc, "source.camera", -580.0, y - 120.0);
        doc.graph.set_label(cam, "Camera: a vista");
        let a = no(doc, "value.attribute", -400.0, y - 120.0);
        doc.graph.set_text_param(
            a,
            ph2d_node_value_attribute::ATTR_KEY,
            ph2d_node_source_camera::ZOOM_COLUMN,
        );
        let d = no(doc, "motion.drive", -220.0, y);
        doc.graph.set_param(d, "channel", CANAL_SIZE);
        doc.graph.set_param(d, "mode", MODO_DIVIDE);
        doc.graph.set_param(d, "scale", 1.0 / CAM_PIXELS);
        doc.graph.set_label(d, "Drive: o tamanho no ecra");
        liga(doc, g, (d, 0))?;
        liga(doc, cam, (a, 0))?;
        liga(doc, a, (d, 1))?;
        // ⚠️ **`1.0` na escala:** quem escreve o tamanho aqui é o `Drive` (`px ÷ zoom`), e um
        // `motion.scale` com outro número multiplicaria por cima do que a vista decidiu.
        sinks.push(pousa(doc, d, 1.0, [COL_X, fileira_y(2)], y)?);
    }

    doc.graph.validate(reg).ok()?;
    Some(sinks)
}

/// As fichas que a cena pousa no canvas — uma por pano, mais o título de cada fileira.
pub(super) fn captions() -> Vec<Caption> {
    let ficha = |x: f32, k: usize, t: &str| Caption {
        text: t.to_string(),
        world: [x, fileira_y(k) + FICHA_Y],
    };
    vec![
        ficha(-COL_X, 0, "O QUE EU FIZ: uma palavra"),
        ficha(COL_X, 0, "...ou uma forma"),
        ficha(-COL_X, 1, "O QUE EU TENHO: as linhas do ficheiro"),
        ficha(COL_X, 1, "...e uma COLUNA delas"),
        ficha(-COL_X, 2, "O QUE A CENA DA: as particulas"),
        ficha(COL_X, 2, "...e a VISTA (de' zoom!)"),
    ]
}

#[cfg(test)]
#[path = "motion_state_fontes_demo_tests.rs"]
mod tests;
#[cfg(all(test, feature = "panel-motion-graph"))]
#[path = "motion_state_fontes_tutorial_tests.rs"]
mod tutorial_tests;
