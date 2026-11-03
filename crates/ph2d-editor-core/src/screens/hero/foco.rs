//! **Soltar o foco do teclado** — o par que a shell chama quando um gesto não passa pelo despachante
//! de ponteiro (um consumidor de canvas, a queda num campo). Irmão do `hero.rs` pelo tecto de LOC.

use super::*;

impl HeroScreen {
    /// **Largue o teclado que um campo do chrome estava a segurar** — para quem
    /// TOMA um gesto de canvas e devolve antes do despachante de ponteiro.
    ///
    /// ⚠️ **Existe porque um `return` cedo herda as obrigações da porta saltada**:
    /// o `dispatch_down` compromete o buffer e solta o foco em todo clique que não
    /// cai num widget, e um consumidor de canvas que devolve antes dele deixava o
    /// foco preso num chip numérico **para o resto da sessão** — matando `Delete`,
    /// `Ctrl+Z` e todo atalho do módulo que tomou o gesto. Mecanismo:
    /// [`crate::interaction::blur_focus`].
    pub fn blur_focus<'frame>(&mut self, arena: &'frame Bump) -> &'frame [WidgetEvent] {
        crate::interaction::blur_focus(&mut self.store, arena)
    }

    /// [`Self::blur_focus`] com os eventos aplicados já — para quem não tem arena (a queda num campo,
    /// GEF 21: um campo que segura o teclado não mostra o valor que a queda lhe escreveu).
    pub fn solta_o_teclado(&mut self) {
        let arena = Bump::new();
        for e in self.blur_focus(&arena).to_vec() {
            self.apply_event(e);
        }
    }
}
