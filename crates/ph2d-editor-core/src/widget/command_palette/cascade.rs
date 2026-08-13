//! **O horário da CASCATA de entrada** — irmão do `command_palette.rs` pelo teto de 500 LOC dos
//! primitivos, e o corte é por responsabilidade: o pai diz o que um cartão DESENHA, o `layout` onde
//! ele CAI, e este QUANDO ele chega.
//!
//! A lei em si (o alvo escalonado, a subida, o carácter) mora no substrato
//! [`crate::motion`]; aqui fica só a identidade dos tracks.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// A semente da CASCATA — a base de que os ids por-cartão descendem.
const CMD_PALETTE_CASCADE: NodeId = hash_node_id("command_palette.cascade");

/// O id de motion do cartão `index`. ⚠️ **Só de MOTION** — ele nunca entra no `hit_index`, porque a
/// cascata não é um alvo: quem responde ao dedo é o pill lá dentro.
///
/// ⚠️ **A identidade é o ÍNDICE, e não o título da categoria**, porque é isso que a cascata
/// descreve: *o terceiro a chegar*. Se o modelo reordenar, o terceiro lugar continua a ser o
/// terceiro — e a mola carrega a diferença, que é precisamente o que ela sabe fazer.
///
/// ⚠️ **CONTINUA a mesma FNV em vez de somar ao id base.** `NodeId(base + i)` cunha inteiros
/// arbitrários vizinhos de um hash e a defesa contra colisão deste repo (`node_id_collisions`) não
/// os enxerga; misturar o índice com o MESMO primo é hashear uma string distinta, logo herda a
/// mesma propriedade dos outros 17 ids do arquivo. Sem tecto, sem pool: um cartão a mais é um
/// número a mais, não uma constante a rever.
#[must_use]
pub fn cascade_id(index: usize) -> NodeId {
    const FNV_PRIME_64: u64 = 0x0000_0100_0000_01b3;
    let mut h = CMD_PALETTE_CASCADE.0;
    for b in (index as u64).to_le_bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(FNV_PRIME_64);
    }
    NodeId(if h == 0 { 1 } else { h })
}
