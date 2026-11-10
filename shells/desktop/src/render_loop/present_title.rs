//! **O título da janela e o pedido do quadro seguinte** — o fim do [`super`] (`present`), depois do `acquire` e das
//! quatro passagens. Filho por `#[path]` (OBRA 3 da `line/render-bodies`): o `run_present_phase` chama-o no sítio
//! exacto do bloco, e ele re-deriva o `gfx` e a janela, que o início do quadro já verificou.

use crate::AppGfx;
use ph2d_host::PlatformHost;

impl crate::App {
    /// Refresca o título quando o estado mudou.
    pub(super) fn present_title_and_redraw(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let Some(host) = self.host.as_ref() else {
            return;
        };
        let AppGfx {
            surface, toasts, ..
        } = gfx;
        // A placa sem memória avisa no TÍTULO, o único texto que ela não desenha; na volta, um resize
        // ao mesmo tamanho refaz os alvos da janela que nasceram sob recusa (F35 §2).
        if let Some(sem) = ph2d_gpu::memoria::sem_memoria_mudou() {
            toasts.push(ph2d_editor_core::aviso_da_placa::toast(sem));
            self.title_dirty = true;
            self.pending_resize = (!sem).then(|| surface.size()).or(self.pending_resize);
        }
        // Window title carries editor state. Refresh only when state
        // actually changes — winit set_title triggers a platform call.
        // Com a janela do jogo aberta o host é ela, e o título dela é o do jogo (`janela_do_jogo`).
        if self.title_dirty && self.undo.janela_do_jogo.is_none() {
            // O título responde «que projeto é este?» e mais nada: as medidas de obra que moravam
            // aqui (sprites, atlas, tema, ferramenta) saíram com a barra de menus (dono, 12/11;
            // `docs/BarraDeMenus/01_plano.md`). fps e contagens vivem na barra de baixo.
            let title = format!(
                "{}{} — PH2D",
                ph2d_editor_core::aviso_da_placa::prefixo_do_titulo(
                    ph2d_gpu::memoria::sem_memoria()
                ),
                crate::project_io::title_name(self.recovery.project_path.as_deref()),
            );
            host.window().set_title(&title);
            self.title_dirty = false;
        }
        // O quadro seguinte pede-o o `about_to_wait`, pela lei do quadro a pedido (B23.2).
    }
}
