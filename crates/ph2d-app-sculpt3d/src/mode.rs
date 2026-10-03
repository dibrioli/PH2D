//! **O BARRO NA TELA — entrar e sair** (ADR-0150) e a porta única de nascer uma escultura.
//!
//! Filho (`#[path]`) de [`super`] para alcançar o papel da forma, que é privado; o corte é *o que
//! a mão faz com o barro* (os irmãos `input`/`keys`) contra *quem é dono da tela* (aqui).
//!
//! O barro na tela decide **duas** coisas de uma vez — o passe de cor desenha, e o ponteiro é da
//! cena ([`super::donation::FormRole::draws_clay`]). Quem o põe e tira é o MODO do objecto
//! ([`crate::sculpt_mode`], spec/06 F3: Sculpt/Paint = barro, Object = fora); o pill SCULPT, que
//! fazia o mesmo, saiu na mesma fase (dois caminhos para o mesmo módulo divergem — spec/06 §5).
//!
//! ⚠️ **A SAÍDA reusa a ordem que o ciclo já declara** (`role.next()` a partir do barro = a LUZ),
//! em vez de inventar um destino: sair para `Off` apagaria a doação de quem estava na luz.

use crate::Sculpt3dScene;
use crate::donation::FormRole;

/// ⭐ **Uma cena nova com `first` como primeira peça** — a porta única de nascer uma escultura,
/// do pill (a esfera) e do menu Add de objectos (a peça escolhida).
pub(crate) fn new_scene(
    device: &wgpu::Device,
    size: (u32, u32),
    first: crate::Primitive,
) -> Sculpt3dScene {
    let aspect = size.0 as f32 / size.1.max(1) as f32;
    let mesh = first.mesh();
    Sculpt3dScene::new(device, mesh, aspect)
}

impl Sculpt3dScene {
    /// Entra no barro, ou sai dele. Devolve o rótulo do papel novo (o mesmo vocabulário que o `D`
    /// imprime — um nome, uma fonte).
    /// ⚠️ Pública porque **quem toma o canvas liberta quem o tinha**: abrir o MODEL tira o barro da
    /// tela por esta porta (e o modo, que deixa de o ter, volta a Object). A alternativa era o
    /// modelador escrever a própria saída — uma segunda resposta a *"como se sai do barro"*.
    pub fn toggle_clay(&mut self) -> &'static str {
        self.role = if self.role.draws_clay() {
            // Ver o cabeçalho: a saída é a próxima posição do ciclo, não um destino escolhido aqui.
            self.role.next()
        } else {
            FormRole::Clay
        };
        self.role.label()
    }

    /// **O barro está na tela?** — a pergunta que o pill MOSTRA.
    ///
    /// ⚠️ Ela é a mesma do [`Self::shows_clay`], e existe separada só porque aquela é
    /// `pub(super)` do módulo do gesto. Um segundo campo aqui seria o bool que passa a discordar do
    /// `D`.
    pub fn clay_on_screen(&self) -> bool {
        self.role.draws_clay()
    }

    /// **O barro ACABOU de entrar, ou de sair?** — `Some(entrou)` na borda, `None` entre elas.
    ///
    /// ⚠️ **É uma TESTEMUNHA, não uma enumeração de quem move o papel.** Hoje quem o move são o
    /// pill e a tecla `D`; amanhã pode ser um terceiro. Uma lista de chamadores que avisam o painel
    /// nasce incompleta no dia em que ela cresce — comparar com o valor anterior não.
    ///
    /// ⚠️ **E ela CONSOME a borda** (o nome diz `take`): um segundo leitor receberia `None` e
    /// concluiria que nada mudou. O leitor é o `panel_bridge`, e é um só.
    pub(crate) fn take_clay_edge(&mut self) -> Option<bool> {
        let now = self.clay_on_screen();
        (now != std::mem::replace(&mut self.clay_was_on, now)).then_some(now)
    }
}

#[cfg(test)]
#[path = "mode_tests.rs"]
mod tests;
