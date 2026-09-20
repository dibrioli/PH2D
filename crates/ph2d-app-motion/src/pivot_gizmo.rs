//! ⭐⭐⭐ **O GIZMO DO PIVÔ DA FORMA** — ordem do dono (2026-09-19): *«permita visualizar o ponto
//! do pivot ao arrastar os parâmetros de pivot»*.
//!
//! ## O que se vê, e porque ele acende só no arrasto
//!
//! Enquanto a mão arrasta o `Pivot X` ou o `Pivot Y` no cartão de uma forma, um **ALVO** (anel com
//! cruz) aparece sobre cada peça que essa forma carimbou, no ponto em que ela gira. Largar apaga.
//!
//! ⛔ **Sempre ligado seria chrome permanente sobre toda cópia de toda forma seleccionada**, e a
//! pergunta que ele responde — *«para onde é que eu estou a mandar este ponto?»* — só existe
//! enquanto o número está a mudar. *O gizmo do colisor acende por um BOTÃO porque o colisor é uma
//! declaração que dura; um pivô é uma coisa que se aponta.*
//!
//! ## ⭐⭐ O achado: o pivô de uma peça É a posição dela
//!
//! A pose de uma instância é `P + basis·(anchor + q·size)` — o ponto local `q = (0,0)` aterra
//! **exactamente** em `P`, em qualquer ângulo e em qualquer tamanho. E o pivô é, por definição, o
//! ponto local que o `motion_shape_gen` leva à origem da caixa de corte.
//!
//! ⇒ **o alvo desenha-se no `P` de cada linha do sink**, e não numa conta nova. É isso que o torna
//! honesto: se ele e a forma discordassem, um dos dois estaria a mentir — e o que se vê a arrastar
//! é a FORMA a deslizar por baixo de um alvo que fica **parado**, que é precisamente a lei.
//!
//! ⚠️ **É por isso que não há aqui uma segunda cópia da aritmética do pivô.** A tentação era
//! calcular «onde o pivô caiu» a partir do param; isso daria um gizmo que continua certo quando o
//! produto deixa de estar.

use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::NodeId;

/// **Quantos alvos se desenham, no máximo** — os primeiros do sink.
///
/// ⚠️ O recurso é o mesmo do contorno do colisor (codificar caminhos no quadro), e o número é o
/// dele: *dois gizmos que desenham uma silhueta por peça no mesmo quadro partilham o orçamento,
/// e dois tectos diferentes para o mesmo recurso são duas respostas à mesma pergunta.*
pub const MAX_ALVOS: usize = super::collider_gizmo::MAX_CONTORNOS;

/// **O retrato deste quadro.**
#[derive(Clone, Debug, PartialEq)]
pub struct PivotGizmoView {
    /// A forma cujo pivô está a ser arrastado.
    pub node: NodeId,
    /// As posições de MUNDO em que o pivô de cada peça caiu — que são os `P` do sink.
    pub pontos: Vec<[f32; 2]>,
}

/// **A forma cujo PIVÔ está a ser arrastado agora**, se houver.
///
/// ⚠️ A pergunta atravessa a fronteira painel→shell pelo canal publicado
/// (`ph2d_panel_motion_graph::current_graph_param_scrub`), que é o mesmo da selecção: ninguém
/// chama ninguém (ADR-0075).
#[must_use]
pub fn forma_com_pivot_em_arrasto(motion: &MotionState) -> Option<NodeId> {
    let (node, param) = ph2d_panel_motion_graph::current_graph_param_scrub()?;
    // ⚠️ **Os dois nomes, e não um prefixo `pivot_`:** um `starts_with` acenderia o gizmo no dia
    // em que outro nó ganhasse um `pivot_angle`, e a resposta seria acender sobre a forma errada.
    if param != ph2d_node_motion_shape::param::PIVOT_X
        && param != ph2d_node_motion_shape::param::PIVOT_Y
    {
        return None;
    }
    let nid = NodeId(node);
    motion
        .doc
        .graph
        .node(nid)
        .is_some_and(|n| n.type_name == ph2d_node_motion_shape::MANIFEST.name)
        .then_some(nid)
}

/// **As TOMADAS que este gizmo precisa** — a saída da própria forma, de onde sai o `geometry_id`
/// que identifica as peças dela.
///
/// ⚠️ **O sink não entra aqui de propósito:** o `ponto_gizmo::taps_for` já pede TODOS os sinks, e
/// pedir o mesmo nó duas vezes só alargaria a união. *Uma tomada é um pedido, não uma posse.*
#[must_use]
pub fn taps_for(motion: &MotionState) -> Vec<NodeId> {
    forma_com_pivot_em_arrasto(motion).into_iter().collect()
}

fn tap(motion: &MotionState, node: NodeId) -> Option<&Stream> {
    motion
        .pump
        .tap_streams()
        .iter()
        .find(|(n, _)| *n == node)
        .map(|(_, s)| s)
}

fn geometria(s: &Stream) -> Option<f32> {
    match s.get("geometry_id") {
        Some(Column::Scalar(v)) => v.first().copied(),
        _ => None,
    }
}

/// **Resolve o retrato a partir do estado** — a porta única, para o laço de render e o gate lerem
/// a MESMA resposta.
///
/// `None` quando: a tool não é a Motion · nenhum pivô está a ser arrastado · as tomadas ainda não
/// trouxeram os streams · nenhuma peça do sink veio desta forma.
#[must_use]
pub fn resolve(motion: &MotionState, tool_is_motion: bool) -> Option<PivotGizmoView> {
    if !tool_is_motion {
        return None;
    }
    let node = forma_com_pivot_em_arrasto(motion)?;
    let saida = tap(motion, node)?;
    let sink = super::collider_gizmo::sink_of(&motion.doc.graph, node)?;
    let pontos = pontos_de(saida, tap(motion, sink)?);
    (!pontos.is_empty()).then_some(PivotGizmoView { node, pontos })
}

/// **As posições das peças que vieram DESTA forma** — a lei, pura, para o gate a medir sem um
/// cozimento vivo.
///
/// ⚠️⚠️ **Um sink SEM coluna `geometry_id` devolve VAZIO, e isso é a resposta certa:** uma
/// corrente sem geometria não carimbou forma nenhuma, logo nenhuma peça dela é desta. *A cura
/// preguiçosa — «sem coluna, aceita tudo» — poria o alvo sobre as posições de uma corrente que
/// esta forma nunca tocou, que é exactamente a leitura errada que o gizmo existe para não dar.*
#[must_use]
pub fn pontos_de(saida: &Stream, sink: &Stream) -> Vec<[f32; 2]> {
    let Some(alvo) = geometria(saida) else {
        return Vec::new();
    };
    let Some(Column::Vec2(p)) = sink.get("P") else {
        return Vec::new();
    };
    // ⚠️ A coluna `geometry_id` do SINK diz de que forma veio cada linha — é ela que separa as
    // peças desta das de uma forma vizinha que caia no mesmo sink.
    let Some(Column::Scalar(quais)) = sink.get("geometry_id") else {
        return Vec::new();
    };
    (0..sink.count().min(p.len()).min(quais.len()))
        .filter(|&i| quais[i] == alvo)
        .map(|i| p[i])
        .take(MAX_ALVOS)
        .collect()
}

static VIEW: std::sync::Mutex<Option<PivotGizmoView>> = std::sync::Mutex::new(None);

/// Publica (ou limpa) o retrato deste quadro. ⚠️ Publicar de novo SUBSTITUI: largar o knob apaga
/// os alvos em vez de os deixar a pairar.
pub fn publish(v: Option<PivotGizmoView>) {
    if let Ok(mut slot) = VIEW.lock() {
        *slot = v;
    }
}

/// O retrato deste quadro, se houver.
#[must_use]
pub fn view() -> Option<PivotGizmoView> {
    VIEW.lock().ok().and_then(|s| s.clone())
}

#[cfg(test)]
#[path = "pivot_gizmo_tests.rs"]
mod tests;
