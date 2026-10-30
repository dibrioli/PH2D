//! ⭐ **As faixas do Follow Value sobre caixas, da convenção antiga para a de agora**
//! (`PROJECT_SCHEMA` `209 → 210`): a lei é [`ph2d_constraints::Valor::desvira_caixas`]; o que é caixa
//! diz [`ph2d_ecs::canal_componente::e_caixa`], o mesmo predicado da migração da curva (`doc_v24.rs`).
//! Gate: `tests/it/valor_caixa_v209.rs` do `ph2d-app-constraints`, sobre bytes gravados pelo código
//! de antes.

use ph2d_constraints::Kind;
use ph2d_ecs::FieldKey;
use ph2d_ecs::canal_componente::e_caixa;

use crate::Constraints;

/// Vira as faixas de cada Follow Value de `c` que lê ou escreve uma caixa; devolve quantos mudaram.
pub fn desvira_caixas(c: &mut Constraints) -> usize {
    let caixa = |k: [u16; 4]| e_caixa(FieldKey::new(k[0], k[1], k[2], k[3]));
    let mut n = 0;
    for it in &mut c.items {
        if let Kind::Valor(v) = &mut it.kind {
            n += usize::from(v.desvira_caixas(caixa));
        }
    }
    n
}

/// A porta do registo ([`ph2d_ecs::scene::ComponentRegistry::register_caixas_v209`]).
pub(crate) fn desvira_blob(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut c: Constraints = postcard::from_bytes(bytes).ok()?;
    (desvira_caixas(&mut c) > 0).then(|| postcard::to_allocvec(&c).ok())?
}
