//! **A edição de uma etapa que espera** (W14, `docs/Macros/12_W14_a_agua_depois_do_parar.md`): editar uma
//! etapa pede uma conta que demora — a conferência da água gravada, ou a imagem do fim nova e a água do
//! editor parada (W8 2b, até 120 s). A edição e o que essa conta escreve são UM passo do desfazer; o que o
//! artista faz entretanto são os passos DELE.
//!
//! A shell chama isto em três sítios fixos do desfazer global: por quadro ([`EdicaoQueEspera::quadro`]),
//! no que fotografa ([`EdicaoQueEspera::a_comparar`]) e em volta de um Ctrl+Z/Ctrl+Y
//! ([`EdicaoQueEspera::antes_de_desfazer`], [`EdicaoQueEspera::restaurado`]).

use std::collections::BTreeMap;

/// O estado do projeto que o desfazer global fotografa, visto daqui: o `Macro` de cada objecto.
pub trait EstadoComMacros: Clone {
    /// Os bytes do componente `Macro` do objecto `id` (`StableId`), se ele o tem.
    fn macro_de(&self, id: u64) -> Option<Vec<u8>>;
    /// Este estado com o `Macro` de `id` posto (`None` = tirado); sem o objecto, nada.
    fn com_macro(&mut self, id: u64, m: Option<Vec<u8>>);
}

/// As edições de etapa à espera da conta delas.
#[derive(Debug, Default)]
pub struct EdicaoQueEspera {
    /// O `Macro` de antes da edição, por objecto (`StableId`).
    antes: BTreeMap<u64, Option<Vec<u8>>>,
}

impl EdicaoQueEspera {
    /// **Por quadro, antes do passo**: `esperam` = os objectos cuja edição ainda espera; `base` = o
    /// baseline do desfazer. `true` = uma espera acabou e o passo regista-se já (mesmo sem entrada).
    pub fn quadro<S: EstadoComMacros>(&mut self, esperam: &[u64], base: Option<&S>) -> bool {
        self.antes.retain(|id, _| esperam.contains(id));
        if let Some(b) = base {
            for &id in esperam {
                self.antes.entry(id).or_insert_with(|| b.macro_de(id));
            }
        }
        false
    }

    /// Há uma edição à espera.
    #[must_use]
    pub fn espera(&self) -> bool {
        !self.antes.is_empty()
    }

    /// **O que o passo global compara com o baseline** (`base`): `atual`, salvo as edições que esperam.
    #[must_use]
    pub fn a_comparar<S: EstadoComMacros>(&self, atual: S, base: &S) -> S {
        if self.espera() { base.clone() } else { atual }
    }

    /// Registou-se um passo.
    pub fn registou(&mut self) {}

    /// **Antes de um Ctrl+Z** (`redo == false`) ou Ctrl+Y: `true` = o Ctrl+Z é da edição que espera (nada
    /// se registou por cima dela) — a shell regista-a já como passo próprio e desfá-lo.
    pub fn antes_de_desfazer(&mut self, redo: bool) -> bool {
        let _ = redo;
        false
    }

    /// **Depois de restaurar** `restaurado` (o passo que saiu da pilha), sendo `atual` o estado de antes
    /// do Ctrl+Z: o estado a aplicar e o baseline novo.
    #[must_use]
    pub fn restaurado<S: EstadoComMacros>(&self, restaurado: &S, atual: &S) -> (S, S) {
        let _ = atual;
        (restaurado.clone(), restaurado.clone())
    }
}

#[cfg(test)]
#[path = "espera_tests.rs"]
mod tests;
