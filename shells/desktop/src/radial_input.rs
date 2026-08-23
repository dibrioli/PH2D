//! **O PIE MENU, do lado da shell** (estudo de UI viva, E4) — as três pontas que a shell possui:
//! a tecla que ABRE, o ponteiro que ACENDE, e a soltura que ESCOLHE.
//!
//! ⚠️ **O modelo e a execução vivem os dois na `editor-core`** (`screens::hero::radial` +
//! `global_palette::route_global_pick`), e é isso que torna um item morto estruturalmente
//! impossível: o radial não conhece comando nenhum — ele é uma **vista** da lista que a paleta já
//! oferece, e quem executa é quem já executava.
//!
//! ⚠️ **A soltura de uma tecla é um evento que este app nunca tinha usado.** Todas as outras teclas
//! agem no `Down`; esta é a única cuja SOLTURA significa alguma coisa, e é a soltura que faz o
//! gesto ser um gesto só — chamar, apontar e escolher sem largar nada pelo caminho.

use ph2d_editor::screens::hero::radial;

impl crate::App {
    /// **O ponteiro mexeu-se com o menu aberto** — acende o sector da direcção.
    ///
    /// ⚠️ No-op sem menu aberto, e é por isso que ela pode ser chamada de todo movimento sem uma
    /// guarda no chamador: o estado é a guarda.
    pub(crate) fn radial_point(&mut self) {
        let p = [self.last_pointer.0, self.last_pointer.1];
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.store.radial_point(p);
        }
    }

    /// **SOLTAR ESCOLHE** — fecha o menu e executa o sector aceso, se houve um.
    ///
    /// ⚠️ **A zona morta devolve `None` e é o CANCELAR**: soltar sem sair do centro é o mesmo
    /// movimento de nunca ter chamado o menu.
    ///
    /// ⚠️ **O sector de transbordo é reconhecido AQUI, antes de tudo**, e num sítio só: ele não é
    /// um comando (não há nada a executar), é o pedido da **outra vista** da mesma lista.
    pub(crate) fn radial_commit(&mut self) {
        let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return;
        };
        let Some(item) = hero.store.close_radial() else {
            return; // zona morta, ou menu fechado: nada a fazer
        };
        if item.id == radial::RADIAL_MORE {
            crate::global_palette_input::open_global_palette(hero);
            return;
        }
        ph2d_editor::screens::hero::global_palette::route_global_pick(hero, item.id);
    }
}

#[cfg(test)]
#[path = "radial_input_tests.rs"]
mod tests;
