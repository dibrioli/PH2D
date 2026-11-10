//! As portas das cenas de smoke (`PH2D_GAME_SMOKE`, plano §8): põem a aba num estado sem a mão do dono — a vista,
//! a sub-vista, a acção aberta, o ponteiro sobre uma tecla, a busca por tecla e a escuta. O estado é o MESMO que os
//! gestos escrevem; a escuta passa pela porta da família (`take_listen_request`), nunca pelo `WidgetStore`.

use crate::doc::{InputMapDoc, View};
use crate::view_keyboard::state::{KbHit, Sub};
use ph2d_input::{ActionId, Key};

impl InputMapDoc {
    /// A vista (List · Keyboard · Try it).
    pub fn smoke_view(&mut self, view: View) {
        self.set_view(view);
    }

    /// O desenho de baixo da vista B: o teclado ou o comando.
    pub fn smoke_sub(&mut self, sub: Sub) {
        self.kb.sub = sub;
    }

    /// A acção aberta da vista B.
    pub fn smoke_open(&mut self, action: Option<ActionId>) {
        self.kb.open = action;
    }

    /// O ponteiro sobre uma tecla do desenho (a frase do pé da vista B).
    pub fn smoke_hover_key(&mut self, key: Option<Key>) {
        self.kb.hover = key.map(KbHit::Key);
    }

    /// O filtro POR TECLA da busca da vista A (o chip dentro do campo).
    pub fn smoke_search_key(&mut self, key: Option<Key>) {
        self.list.search_key = key;
    }

    /// Arma a escuta partilhada nesta acção (a família pede-a no próximo `frame`).
    pub fn smoke_listen(&mut self, action: ActionId) {
        self.list.listen_request = Some(Some(action));
    }
}
