//! **O NOME QUE NÃO RESOLVE** — a segunda espécie de erro-que-não-produz-erro do
//! ADR-0155, e a primeira que a análise ESTRUTURAL não consegue responder.
//!
//! O `value.attribute` lê uma coluna cujo NOME é um text param. Um nome que a
//! stream de entrada não tem cai no miss ORDINÁRIO da escada (`_ => vec![0.0; n]`)
//! e devolve **zeros no comprimento certo**: o grafo coza, o device coza, o cook é
//! válido, e a cena fica parada — indistinguível de um campo que por acaso vale
//! zero. É exactamente a classe que o [ADR-0155] existe para pegar, e a folha 15 da
//! conferência a nomeia (linha 122, *"hoje um nome errado lê zeros em silêncio"*),
//! prescrevendo **este** canal em vez de um param.
//!
//! ## Por que isto NÃO é um [`Deficit`](crate::Deficit)
//!
//! Toda regra do [`diagnose`](crate::diagnose) é **pura e estrutural**: grafo +
//! registry, sem cook. Esta **não pode ser** — e a tentativa de a fazer estrutural
//! foi MEDIDA e descartada antes de uma linha ser escrita:
//!
//! > A união dos nomes que os `ColumnBinding` do repo DECLARAM escrever tem 20
//! > entradas; a união dos que o CPU de facto escreve (`.with("…")`, e nem isso é a
//! > única rota) tem **29**. Faltam entre outros `texture_id`, `geometry_id`,
//! > `uv_rect` e `nrm` — que são precisamente as **CONVENÇÕES de stream** que o
//! > CLAUDE.md §5 documenta.
//!
//! Uma regra que perguntasse *"algum nó declara escrever este nome?"* poria um badge
//! ⚠ em `texture_id`, que resolve perfeitamente. **É o falso positivo que o próprio
//! ADR-0155 já pagou uma vez** (o `MissingSource("P")` no Boids, cujo doc afirmava
//! *"zero false positives"*), e um badge que mente sobre um nome certo é pior que
//! silêncio: ele ensina o artista a ignorar o badge.
//!
//! ## A resposta vem do STREAM COZIDO, e por isso é exacta
//!
//! O chamador entrega `columns_at`, que devolve os nomes de coluna que a stream em
//! `(nó, porta)` de facto carrega — o **mesmo objecto** que o `eval` do nó lê. Se o
//! nome não está lá, a escada TOMA o ramo dos zeros; não há inferência no meio.
//! **Zero falsos positivos por construção**, e o preço é que a pergunta só pode ser
//! feita depois de um cook (o shell a faz por quadro, onde o badge é pintado).
//!
//! ⚠️ **`None` NÃO é lista vazia.** Um nó ainda não cozido, um quadro de GPU cuja
//! tomada não publicou aquela porta — nada disso é *"a stream não tem colunas"*, é
//! **não sabemos**, e a diferença entre os dois é a diferença entre calar e acusar.
//!
//! ## A regra é DERIVADA, nunca uma lista de tipos
//!
//! Quem a dispara é [`StreamOp::Project`] — a declaração *"este nó projecta uma
//! coluna nomeada por um text param"* que o próprio nó regista no registry, e que o
//! sequenciador de GPU já lê. Um nó novo com essa forma nasce coberto; uma tabela de
//! nomes de tipo nasceria desactualizada no dia seguinte (o molde do ADR-0155:
//! *derivar os papéis da própria declaração do nó, nunca de uma tabela paralela*).
//!
//! [ADR-0155]: ../../../docs/architecture/decisions/0155-motion-graph-setup-is-diagnosed-and-healed-not-refused.md

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::gpu::KernelResolver;
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::NodeTypeId;
use ph2d_nodegraph::stream_op_meta::StreamOp;

/// Um nó que lê uma coluna por NOME, e o nome não está na stream que entra nele.
///
/// Carrega a `String` porque o nome é conteúdo AUTORADO (vive no canal de text
/// param do `Graph`), o que é também a razão de este tipo não caber no
/// [`Deficit`](crate::Deficit) — aquele enum é `Copy` sobre `&'static str`, e
/// alargá-lo para caber um nome de runtime custaria o `Copy` a todos os
/// consumidores existentes por causa de uma regra que nem sequer é da mesma
/// espécie.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnresolvedRead {
    /// O nó que lê.
    pub node: NodeId,
    /// O nome que ele pediu, como o artista o escreveu.
    pub column: String,
}

/// Todo nó que projecta uma coluna por nome e cujo nome a stream de entrada não
/// carrega — a lista que o editor transforma em badge ⚠.
///
/// `columns_at(nó, porta)` devolve os nomes de coluna da stream naquela saída, ou
/// `None` quando o chamador **não sabe** (ver o cabeçalho: `None` e lista vazia são
/// respostas diferentes).
///
/// Quatro estados são deliberadamente SILENCIOSOS, e cada um por um motivo próprio:
///
/// * **nome ausente ou em branco** — o nó ainda não foi configurado. Um nó recém
///   solto é *inacabado*, não *errado*; é o mesmo critério que o `reaches_output`
///   aplica ao badge de produtor inerte.
/// * **sem aresta de entrada** — não há stream nenhuma para ter (ou não ter) a
///   coluna. Esse é o `MissingSource`, e reportar os dois nomearia a mesma causa
///   duas vezes.
/// * **`columns_at` devolve `None`** — desconhecido.
/// * **o nome resolve mas a PISTA não existe** (pedir `Z` de um `Vec2`) — esse é o
///   miss ordinário da escada, **sobre um nome certo**; o `mode` é um param f32 e
///   quem o julga é o painel, não esta regra.
#[must_use]
pub fn unresolved_reads(
    graph: &Graph,
    reg: &NodeRegistry,
    columns_at: &dyn Fn(NodeId, u16) -> Option<Vec<String>>,
) -> Vec<UnresolvedRead> {
    let mut out = Vec::new();
    for inst in graph.nodes() {
        let ty = NodeTypeId::of(&inst.type_name);
        let Some(key) = projected_text_param(reg, ty) else {
            continue;
        };
        let Some(name) = authored_name(graph, inst.id, key) else {
            continue;
        };
        // A stream que ENTRA: a aresta não-atrasada na porta 0. Uma aresta `delayed`
        // é o estado do quadro anterior de uma fonte com estado, não a entrada.
        let Some((sn, sp)) = graph
            .edges()
            .iter()
            .find(|e| e.to == (inst.id, 0) && !e.delayed)
            .map(|e| e.from)
        else {
            continue;
        };
        let Some(cols) = columns_at(sn, sp) else {
            continue; // desconhecido — calar, nunca acusar
        };
        if !cols.iter().any(|c| c == name) {
            out.push(UnresolvedRead {
                node: inst.id,
                column: name.to_string(),
            });
        }
    }
    out
}

/// O text param que o tipo `ty` usa para nomear a coluna que projecta, se ele
/// projectar alguma. **A porta única da derivação** — quem responde é o
/// [`StreamOp`] registado pelo próprio nó.
fn projected_text_param(reg: &NodeRegistry, ty: NodeTypeId) -> Option<&'static str> {
    match reg.stream_op(ty) {
        Some(StreamOp::Project { text_param, .. }) => Some(text_param),
        _ => None,
    }
}

/// O nome que o artista escreveu naquele param, já aparado — `None` para ausente ou
/// em branco (ver a lista de silêncios em [`unresolved_reads`]).
fn authored_name<'a>(graph: &'a Graph, node: NodeId, key: &str) -> Option<&'a str> {
    let raw = graph.node_text_params().get(&node)?.get(key)?.trim();
    (!raw.is_empty()).then_some(raw)
}

#[cfg(test)]
#[path = "reads_tests.rs"]
mod tests;
