//! Os pedidos da fileira do gatilho ao Input Map (criar a acção · criar e esperar a tecla). A lei
//! mora ao lado do mapa: [`ph2d_editor_core::screens::hero::chrome::aplica_pedidos_do_mapa`].

use ph2d_editor_core::screens::hero::chrome::{PedidoDoMapa, aplica_pedidos_do_mapa};

impl crate::App {
    /// Aplica os pedidos que a fileira do gatilho fez neste quadro.
    pub(super) fn fase_criar_accao_do_mapa(&mut self, pedidos: Vec<PedidoDoMapa>) {
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            aplica_pedidos_do_mapa(hero, pedidos);
        }
    }
}
