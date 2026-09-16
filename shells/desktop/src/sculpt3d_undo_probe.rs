//! **O EXECUTOR DA SONDA DO UNDO DA ESCULTURA** — o roteiro, a leitura e o que ela mediu vivem em
//! [`ph2d_app_sculpt3d::sonda_undo`]; aqui só o que exige o teclado e o ponteiro reais da `App`.

use ph2d_app_sculpt3d::sonda_undo::{self, Passo};
use winit::event::ElementState::{Pressed, Released};

impl crate::App {
    /// Roda no prólogo do quadro, ao lado das outras sondas. No-op sem a env.
    pub fn sculpt3d_undo_probe(&mut self) {
        let Some(janela) = self.gfx.as_ref().map(|g| g.surface.size()) else {
            return;
        };
        let Some(f) = sonda_undo::quadro() else {
            return;
        };
        match sonda_undo::passo(f, (janela.width, janela.height), |id| {
            self.smoke_find_widget(id)
        }) {
            Passo::Nada => {}
            Passo::Tecla(k, texto) => {
                let k = winit::keyboard::PhysicalKey::Code(k);
                self.key_input(k, Pressed, false, texto.map(Into::into));
                self.key_input(k, Released, false, None);
            }
            Passo::Enter => self.smoke_key_enter(),
            Passo::CtrlZ { refazer, desce } => self.smoke_key_z(refazer, desce),
            Passo::Desce(x, y) => self.smoke_pointer_down(x, y),
            Passo::Move(x, y) => self.smoke_pointer_move(x, y),
            Passo::Solta => self.smoke_pointer_up(),
        }
        let gfx = self.gfx.as_ref();
        if let Some(linha) = sonda_undo::leitura(f, gfx.and_then(|g| g.sculpt3d.as_ref())) {
            let mao = gfx.and_then(|g| g.tools.active()).map(|t| t.id());
            eprintln!(
                "{linha} foco={} teclas={:?} mao={mao:?} held={:?}",
                self.text_entry_focused(),
                self.sculpt3d_keys_dead_reason(),
                self.held_button,
            );
        }
    }
}
