//! ⭐ **As caixas gravadas na convenção antiga** (`PROJECT_SCHEMA` `209 → 210`, macros do mundo v2):
//! até ao `TimelineDoc` v24 o canal de uma caixa (`View::Toggle`) valia `0` = ligada. A curva migra no
//! próprio documento (`doc_v24.rs`); um componente que GUARDA números no canal de uma caixa (as faixas
//! do Follow Value das constraints) regista aqui a sua conversão, e quem lê bytes antigos — o projecto
//! e a macro — chama-a pelo registo, sem conhecer o tipo.

use std::sync::Arc;

use super::{ComponentRegistry, ComponentTypeId, stable_type_id};
use crate::scene::WorldSnapshot;

/// Os bytes de um componente com as caixas na convenção de agora; `None` = nada a mudar (ou
/// ilegível).
pub type DesviraFn = fn(&[u8]) -> Option<Vec<u8>>;

impl ComponentRegistry {
    /// Regista a conversão de `canonical_name` (já registado).
    ///
    /// # Panics
    /// O tipo não está registado (erro de quem chama).
    pub fn register_caixas_v209(&mut self, canonical_name: &'static str, f: DesviraFn) {
        assert!(
            self.get_by_name(canonical_name).is_some(),
            "{canonical_name}: registe o componente antes da conversão das caixas"
        );
        self.caixas_v209.insert(stable_type_id(canonical_name), f);
    }

    /// Os bytes do componente `canonical_name` com as caixas na convenção de agora.
    #[must_use]
    pub fn desvira_caixas(&self, canonical_name: &str, bytes: &[u8]) -> Option<Vec<u8>> {
        self.desvira_por_id(stable_type_id(canonical_name), bytes)
    }

    fn desvira_por_id(&self, id: ComponentTypeId, bytes: &[u8]) -> Option<Vec<u8>> {
        self.caixas_v209.get(&id).and_then(|f| f(bytes))
    }

    /// Cada componente de `world` com conversão registada; devolve quantos mudaram.
    pub fn desvira_caixas_do_mundo(&self, world: &mut WorldSnapshot) -> usize {
        let mut n = 0;
        for row in &mut world.entities {
            for slot in 0..row.components.len() {
                let c = &row.components[slot];
                if let Some(novo) = self.desvira_por_id(c.type_id, &c.data) {
                    Arc::make_mut(row).components[slot].data = novo;
                    n += 1;
                }
            }
        }
        n
    }
}
