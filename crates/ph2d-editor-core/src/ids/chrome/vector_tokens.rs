//! **Os ids do BINDING DE TOKEN** (plano UI/UX §4/W4) — irmão do [`super::vector`], que está a 16
//! linhas do teto de 700 LOC.
//!
//! O corte é por ASSUNTO: aqui mora *que propriedade desta forma segue um token, e qual*.

use ph2d_a11y::NodeId;

use super::super::hash_node_id;
use super::painter::fnv_node_id_runtime;

/// **O chip de token do PREENCHIMENTO** — abre a lista, e mostra o token vigente (ou `—`).
///
/// Fica ao lado da swatch de Fill, e não numa seção à parte, porque é isso que responde à pior
/// pergunta que esta feature pode gerar: *"por que a cor que eu escolhi não aparece?"*. Um valor
/// que não obedece ao que se digita e não diz por quê é a pior UI possível.
pub const VECTOR_TOKEN_FILL: NodeId = hash_node_id("vector.token.fill");

/// **O chip de token do TRAÇO** — idem, ao lado da cor do traço.
///
/// ⚠️ Só é pintado quando a seleção TEM traço: o token de traço colore o traço que existe e não
/// inventa largura (ver `VecPath::painted`), então oferecê-lo sem traço seria um controle que o
/// artista escolhe e que não muda um pixel.
pub const VECTOR_TOKEN_STROKE: NodeId = hash_node_id("vector.token.stroke");

/// A opção `i` no popover de tokens da propriedade `prop`.
///
/// ⚠️ **Derivado do ÍNDICE na lista `ColorToken::ALL`, e o índice é de RUNTIME** — ele nunca toca
/// o documento (lá a identidade é a CHAVE do token). Um id derivado do índice num arquivo teria o
/// mesmo defeito que a chave existe para evitar: reordenar a lista mudaria o significado.
///
/// `i == 0` é a linha **Unbind** — soltar a propriedade; as demais são `ColorToken::ALL[i - 1]`.
#[must_use]
pub fn vector_token_option_id(prop: u16, i: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.token.opt.{prop}.{i}"))
}
