//! **Os ids do painel de TOKENS** (plano UI/UX W6, degrau 1) — a tabela de cor do design system,
//! autorável pelo artista.
//!
//! # Porque um painel de MUNDO, e não uma seção do Inspector
//!
//! A tabela de tokens não é propriedade de nada que se selecione: ela é a cara do app inteiro. Uma
//! seção do Inspector precisaria de uma seleção para existir, e a pergunta *"de que cor é a
//! superfície deste editor?"* não tem sujeito. É a mesma categoria do painel de física (ADR-0131
//! D8), e o abridor é a tecla **`T`** pelo mesmo motivo que o `W` existe: um painel de mundo não é
//! tool-gated, então sem abridor próprio é feature que ninguém alcança.
//!
//! ⚠️ **A `T` era um scaffold de debug** (`"Toast key (T) pressed"`), órfão como o `KeyB` que a
//! timeline aposentou no W4.T5 — varrido antes de ser tomado, e nada no repo dependia dele.
//!
//! # Uma linha por token, e os ids são DERIVADOS
//!
//! A lista é `ColorToken::ALL` — tabela de compile-time, ~80 folhas —, então não há teto a
//! escolher: o id de cada linha sai do ÍNDICE, exactamente como o `vector_token_option_id` do
//! popover de binding. Uma segunda lista escrita à mão aqui nasceria desatualizada no primeiro
//! token acrescentado, e o modo de falha seria uma cor que o artista não consegue editar.

use ph2d_a11y::NodeId;

use super::painter::fnv_node_id_runtime;
use crate::ids::hash_node_id;

/// O retângulo externo do painel (z-order + barreira de hit + roteamento da roda).
pub const TOKENS_PANEL: NodeId = hash_node_id("tokens.panel");
/// O botão de fechar (X).
pub const TOKENS_CLOSE: NodeId = hash_node_id("tokens.close");
/// **Reset All** — devolve o MODO VIGENTE inteiro à fábrica.
///
/// ⚠️ Só o modo vigente, e não os quatro: o artista vê um modo de cada vez, e um botão que
/// apagasse trabalho que ele não está a olhar é a forma mais barata de perder uma re-vestida.
pub const TOKENS_RESET_ALL: NodeId = hash_node_id("tokens.reset_all");

/// A swatch da linha `row` — **alvo de PICKER**, como a swatch de Fill do vetor.
///
/// ⚠️ Registá-la como botão faria o clique acender o widget e **nunca abrir o picker** — a cor
/// ficaria ineditável com todos os gates verdes (a cicatriz que a lista de peças da W5b já pagou).
#[must_use]
pub fn tokens_swatch_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("tokens.swatch.{row}"))
}

/// O **Reset** da linha `row` — devolve UM token à fábrica.
#[must_use]
pub fn tokens_reset_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("tokens.reset.{row}"))
}

/// O **elo** da linha `row` — o gesto de fazer um token SEGUIR outro (plano UI/UX W4b).
///
/// ⚠️ **Um id por linha, e o mesmo botão faz as duas pontas do gesto**: clicar arma esta linha,
/// clicar noutra fecha o elo *desta → aquela*. Dois ids (um "armar", um "escolher") seriam dois
/// controlos para um gesto de dois toques, e a segunda metade nasceria sem sítio na linha alvo.
///
/// É o idioma de conta-gotas que o repo já usa para apontar coisa (o re-pick de corpo de joint, o
/// `vec_path_pick` do vetor) — aqui o alvo é outra ROW do mesmo painel, então o "clique no alvo"
/// cai no botão que a row já tem.
#[must_use]
pub fn tokens_link_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("tokens.link.{row}"))
}
