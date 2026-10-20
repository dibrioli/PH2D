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

/// **As edições de etapa à espera da conta delas** — «a flutuar» por cima da pilha: o baseline e cada passo
/// registado na espera são fotografados SEM elas (o `Macro` de antes), e quando a conta acaba a edição e o
/// que ela escreveu entram num passo só, no topo. ⛔ Recusado: suprimir o desfazer na espera (W8) — o que o
/// artista fazia fundia-se com a edição; e emendar os passos já registados com a imagem que chega (o
/// Painter tem a pilha dele, e um passo dele por baixo de traços do artista não se desfaz fora de ordem).
#[derive(Debug, Default)]
pub struct EdicaoQueEspera {
    /// O `Macro` de antes da edição, por objecto (`StableId`).
    antes: BTreeMap<u64, Option<Vec<u8>>>,
    /// Os passos na pilha por cima da edição que espera.
    por_cima: usize,
}

impl EdicaoQueEspera {
    /// **Por quadro, antes do passo**: `esperam` = os objectos cuja edição ainda espera; `base` = o
    /// baseline do desfazer. `true` = uma espera acabou e o passo regista-se já (mesmo sem entrada: senão
    /// a edição fundia-se com a acção seguinte).
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

    /// **Antes de um Ctrl+Z** (`redo == false`) ou Ctrl+Y: `true` = o Ctrl+Z é da edição que espera (nada
    /// se registou por cima dela) — a shell regista-a já como passo próprio e desfá-lo; a espera acaba.
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

    /// **Depois de restaurar** `restaurado` (o passo que saiu da pilha), sendo `atual` o estado de antes
    /// do Ctrl+Z: o estado a aplicar (com as edições que esperam, como estão) e o baseline novo (sem elas).
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
