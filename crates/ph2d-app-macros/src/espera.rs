//! **A edição de uma etapa que espera** (`docs/Macros/12_W14_a_agua_depois_do_parar.md`): a conta que
//! demora (até 120 s) e a edição são UM passo do desfazer; o que o artista faz entretanto é DELE. A shell
//! chama-o por quadro ([`EdicaoQueEspera::quadro`]), no que fotografa ([`EdicaoQueEspera::a_comparar`])
//! e no Ctrl+Z/Y ([`EdicaoQueEspera::antes_de_desfazer`], [`EdicaoQueEspera::restaurado`]).

use std::collections::BTreeMap;

/// Os objectos (`StableId`) cuja edição espera a conta: os `bits` da composição mais as conferências
/// desta família, por ordem, sem repetir.
#[must_use]
pub fn os_que_esperam(world: &ph2d_ecs::World, bits: impl IntoIterator<Item = u64>) -> Vec<u64> {
    let conferir = crate::com(|m| m.a_conferir_em());
    let mut ids: Vec<u64> = bits
        .into_iter()
        .chain(conferir)
        .filter_map(|b| {
            world
                .get::<ph2d_ecs::StableId>(ph2d_ecs::Entity::from_bits(b))
                .map(|s| s.0)
        })
        .collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}

/// O estado do projeto que o desfazer global fotografa, visto daqui: o `Macro` de cada objecto.
pub trait EstadoComMacros: Clone {
    /// Os bytes do componente `Macro` do objecto `id` (`StableId`), se ele o tem.
    fn macro_de(&self, id: u64) -> Option<Vec<u8>>;
    /// Este estado com o `Macro` de `id` posto (`None` = tirado); sem o objecto, nada.
    fn com_macro(&mut self, id: u64, m: Option<Vec<u8>>);
}

/// **As edições à espera** «a flutuar» sobre a pilha: tudo se fotografa SEM elas, e ao acabar a conta
/// entram num passo só, no topo. ⛔ Recusado: suprimir o desfazer na espera, e emendar passos já feitos.
#[derive(Debug, Default)]
pub struct EdicaoQueEspera {
    /// O `Macro` de antes da edição, por objecto (`StableId`).
    antes: BTreeMap<u64, Option<Vec<u8>>>,
    /// Os passos na pilha por cima da edição que espera.
    por_cima: usize,
}

impl EdicaoQueEspera {
    /// Por quadro, antes do passo: `true` = uma espera acabou e o passo regista-se já (mesmo sem
    /// entrada: senão fundia-se com a acção seguinte).
    pub fn quadro<S: EstadoComMacros>(&mut self, esperam: &[u64], base: Option<&S>) -> bool {
        let n = self.antes.len();
        self.antes.retain(|id, _| esperam.contains(id));
        let acabou = self.antes.len() < n;
        if self.antes.is_empty() {
            self.por_cima = 0;
        }
        if let Some(b) = base {
            for &id in esperam {
                self.antes.entry(id).or_insert_with(|| b.macro_de(id));
            }
        }
        acabou
    }

    /// Há uma edição à espera.
    #[must_use]
    pub fn espera(&self) -> bool {
        !self.antes.is_empty()
    }

    /// `s` com o `Macro` de antes das edições que esperam.
    fn sem<S: EstadoComMacros>(&self, mut s: S) -> S {
        for (&id, m) in &self.antes {
            s.com_macro(id, m.clone());
        }
        s
    }

    /// **O que o passo global compara com o baseline**: `atual`, salvo as edições que esperam.
    #[must_use]
    pub fn a_comparar<S: EstadoComMacros>(&self, atual: S) -> S {
        self.sem(atual)
    }

    /// Registou-se um passo.
    pub fn registou(&mut self) {
        if self.espera() {
            self.por_cima += 1;
        }
    }

    /// Antes de um Ctrl+Z/Y: `true` = o Ctrl+Z é da edição que espera (a shell regista-a e desfá-la).
    pub fn antes_de_desfazer(&mut self, redo: bool) -> bool {
        if !self.espera() {
            return false;
        }
        if redo {
            self.por_cima += 1;
            return false;
        }
        if self.por_cima == 0 {
            self.antes.clear();
            return true;
        }
        self.por_cima -= 1;
        false
    }

    /// Depois de restaurar: o estado a aplicar (com as edições que esperam) e o baseline novo (sem elas).
    #[must_use]
    pub fn restaurado<S: EstadoComMacros>(&self, restaurado: &S, atual: &S) -> (S, S) {
        let mut aplica = restaurado.clone();
        for &id in self.antes.keys() {
            aplica.com_macro(id, atual.macro_de(id));
        }
        let base = self.sem(aplica.clone());
        (aplica, base)
    }
}

#[cfg(test)]
#[path = "espera_tests.rs"]
mod tests;
