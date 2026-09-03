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

/// ⭐⭐⭐ **QUE COLUNAS ENTRARAM NESTE NÓ E NÃO SAÍRAM** — a resposta que o cartão do Mini
/// Cavalry dá com os chips *lê/escreve* e o nosso não dava em lado nenhum
/// ([doc 99 §10d](../../../../docs/Motion%20Nodes/99_estudo_do_mini_cavalry_2026-09-02.md)).
///
/// ⛔⛔⛔ **É a pergunta que custou os TRÊS reports de 2026-09-01** — o `motion.duplicator` a
/// deitar fora `id`/`vel`/`age`/`life` (a simulação morria), o `size` que «parava de
/// funcionar», e a varredura que teve de ser feita por sonda porque **nenhuma superfície do
/// app respondia**.
///
/// ⭐⭐ **E é DERIVADA do cozimento, não declarada.** Ele escreve `reads_attrs`/`writes_attrs`
/// à mão em cada nó — uma segunda lista que pode divergir do que o nó faz, e que só cobriria
/// os **67 de 134** tipos que declaram bindings de device. Esta lê as correntes REAIS do memo
/// (ou da tomada do device) e responde por **todos os 134**, sobre o documento que o artista
/// tem à frente.
///
/// ⚠️ **Descritiva, nunca acusatória:** o `motion.integrate` consome `accel` de propósito e o
/// `motion.duplicator` deitava fora sem querer — as duas leem-se igual aqui, e é o artista que
/// sabe qual queria. *Uma superfície que ACUSA precisa de saber a intenção; uma que DESCREVE
/// não.*
///
/// ⚠️ **`None` é o caso comum e não aloca nada** — só um nó que de facto perde colunas paga
/// uma `String`. A comparação é feita sobre nomes EMPRESTADOS das duas correntes.
/// ⛔⛔⛔ **A SUPERFÍCIE QUE ISTO ALIMENTAVA FOI MEDIDA E REVERTIDA** — ver o doc 99 §10d. A
/// nota no cartão disparava em **quase todo nó** e quase sempre sobre comportamento
/// CORRECTO (`sim.step → drops accel`, `pulse.counter → drops beat_cycle · beat_primed`),
/// mesmo depois de duas cercas medidas (só a entrada PRINCIPAL · só quando entrada e saída
/// são da mesma espécie). *A distinção que falta é a INTENÇÃO, e essa não é derivável.*
///
/// ⚠️ **Fica como INSTRUMENTO** (`#[cfg(test)]`), que é o padrão desta casa para o que perde
/// o chamador de produção: é por aqui que se investiga o próximo report desta classe — foi
/// exactamente esta pergunta que os três de 2026-09-01 obrigaram a responder por sonda.
#[cfg(test)]
pub(crate) fn dropped_at(motion: &MotionState, node: NodeId) -> Option<String> {
    let saida = stream_at(motion, node, 0)?;
    // ⚠️ **CRU de propósito, e as duas cercas que aqui estiveram foram TIRADAS.** Elas — só a
    // entrada principal, só quando a espécie é a mesma — nasceram para salvar a nota do
    // cartão, e a nota foi medida e revertida (doc 99 §10d). *Um instrumento de diagnóstico
    // quer COMPLETUDE; o filtro era uma preocupação de produto, e o produto saiu.*
    //
    // ⛔ E elas eram código por testar: as duas mutações que as apagavam **sobreviveram** ao
    // gate, porque a fixtura dele não as exercitava.
    let mut perdidas: Vec<String> = Vec::new();
    for e in motion.doc.graph.edges() {
        if e.to.0 != node {
            continue;
        }
        let Some(entrada) = stream_at(motion, e.from.0, e.from.1) else {
            continue;
        };
        for (nome, _) in entrada.columns() {
            if saida.get(nome).is_none() && !perdidas.iter().any(|p| p == nome) {
                perdidas.push(nome.clone());
            }
        }
    }
    (!perdidas.is_empty()).then(|| perdidas.join(" · "))
}

/// A corrente viva de `(node, port)`, **emprestada** — o irmão do [`at`] para quem só quer
/// comparar e não guardar. Existe para o [`dropped_at`] não pagar uma `Vec<String>` por nó por
/// quadro só para descobrir que não há nada a dizer.
#[cfg(test)]
fn stream_at(
    motion: &MotionState,
    node: NodeId,
    port: u16,
) -> Option<&ph2d_nodegraph::attr::Stream> {
    if let Some(s) = motion
        .pump
        .cook
        .peek(node)
        .and_then(|o| o.get(port as usize))
        .map(ph2d_nodegraph::value::CookValue::as_stream)
    {
        return Some(s);
    }
    match motion.gpu_tap.as_ref().and_then(|t| t.get(&node)) {
        Some(s) if port == 0 => Some(s),
        _ => None,
    }
}
