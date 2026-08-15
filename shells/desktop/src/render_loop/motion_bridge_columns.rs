//! **QUE COLUNAS A STREAM AQUI CARREGA?** — a porta única da pergunta, com os DOIS
//! lugares de onde a resposta pode vir.
//!
//! O grafo coza na **GPU por default**, e nesse quadro o memo do `Cook` está VAZIO
//! (`motion_bridge::cook_gpu` devolve `Handled` e o laço que preenche o memo é
//! pulado); num quadro de CPU o memo é a coisa real. Quem quer saber as colunas tem
//! de perguntar aos dois, **e a ordem importa** — o memo primeiro, porque é a stream
//! COMPLETA; a tomada de GPU é uma sub-amostra de 48 linhas por nó ENCENADO, e
//! MEMBRESIA de coluna é o que aqui se pergunta (48 linhas descobrem os mesmos
//! nomes que quatro milhões).
//!
//! ⚠️ **Isto nasceu dentro do painel de params** (o *column picker* do
//! `value.attribute` foi quem precisou primeiro) e mudou-se para cá quando ganhou um
//! **segundo** consumidor que não é do painel — o diagnóstico do nome que não
//! resolve. Uma segunda cópia da escada memo-ou-tomada é exactamente a forma que
//! diverge no dia em que uma terceira fonte aparecer, e as duas respostas ficariam
//! certas por metade.
//!
//! ## `None` não é lista vazia
//!
//! A porta devolve `Option`, e a distinção é **load-bearing para o consumidor novo**:
//! *não sabemos* (nó ainda não cozido; porta > 0 num quadro de device, que a tomada
//! não publica) e *sabemos, e não há colunas* levam a decisões opostas — a primeira
//! manda calar, a segunda é uma acusação legítima. Para o picker as duas dão a mesma
//! lista vazia de chips, e é por isso que ele viveu sem a distinção.

use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::NodeId;

/// Uma coluna da stream viva, como os dois leitores precisam de a ver.
///
/// O `scalar` está aqui em vez de a porta filtrar porque os consumidores filtram
/// DIFERENTE: o picker oferece só escalares (é o que um `value.attribute` no modo
/// Scalar consome), e o diagnóstico do nome tem de ver TODAS — `vel` é `Vec2` e
/// resolve perfeitamente.
pub(super) struct LiveColumn {
    pub(super) name: String,
    pub(super) scalar: bool,
}

/// As colunas da stream na saída `(node, port)`, **owned** para sobreviverem a
/// qualquer das duas fontes; `None` quando não sabemos (ver o cabeçalho).
pub(super) fn at(motion: &MotionState, node: NodeId, port: u16) -> Option<Vec<LiveColumn>> {
    // Quadro de CPU: o memo tem a coisa real, e é uma consulta, nunca um 2º cook.
    if let Some(stream) = motion
        .pump
        .cook
        .peek(node)
        .and_then(|o| o.get(port as usize))
        .map(ph2d_nodegraph::value::CookValue::as_stream)
    {
        return Some(describe(stream));
    }
    // Quadro de device: a MESMA tomada que os readouts do painel de grafo leem.
    match motion.gpu_tap.as_ref().and_then(|t| t.get(&node)) {
        Some(stream) if port == 0 => Some(describe(stream)),
        _ => None,
    }
}

/// Só os NOMES, para quem não se importa com o tipo.
pub(super) fn names_at(motion: &MotionState, node: NodeId, port: u16) -> Option<Vec<String>> {
    Some(
        at(motion, node, port)?
            .into_iter()
            .map(|c| c.name)
            .collect(),
    )
}

fn describe(stream: &Stream) -> Vec<LiveColumn> {
    stream
        .columns()
        .map(|(n, c)| LiveColumn {
            name: n.to_string(),
            scalar: matches!(c, Column::Scalar(_)),
        })
        .collect()
}
