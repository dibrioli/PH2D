//! **O título da janela e o pedido do quadro seguinte** — o fim do [`super`] (`present`), depois do `acquire` e das
//! quatro passagens. Filho por `#[path]` (OBRA 3 da `line/render-bodies`): o `run_present_phase` chama-o no sítio
//! exacto do bloco, e ele re-deriva o `gfx` e a janela, que o início do quadro já verificou.

use crate::{AppGfx, SPRITE_COUNT};
use ph2d_host::PlatformHost;

impl crate::App {
    /// Refresca o título quando o estado mudou, e pede o redesenho contínuo.
    pub(super) fn present_title_and_redraw(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let Some(host) = self.host.as_ref() else {
            return;
        };
        let AppGfx {
            asset_db,
            surface,
            atlas_is_real,
            script,
            theme,
            zen,
            toasts,
            tools,
            ..
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
            let tool_label = tools.active().map(|t| t.label()).unwrap_or("none");
            let title = format!(
                "{}PH2D — {} | sprites={SPRITE_COUNT} | atlas={} ({} assets) \
                 | script={} | theme={:?} | zen={} | toasts={} | tool={}",
                // **O NOME DO FICHEIRO**, e não a lista de milestones que morava aqui: a barra de
                // título é o único sítio do app que responde *«que projeto é este?»*, e a resposta
                // dela era «M5+M6+M7+M11+M12 demo» — verdade sobre o binário, e sobre nada que o
                // artista tenha aberto.
                ph2d_editor_core::aviso_da_placa::prefixo_do_titulo(
                    ph2d_gpu::memoria::sem_memoria()
                ),
                crate::project_io::title_name(self.project_path.as_deref()),
                if *atlas_is_real { "PNG" } else { "dummy" },
                asset_db.len_assets(),
                if script.is_some() { "ok" } else { "off" },
                theme,
                if zen.is_active() { "on" } else { "off" },
                toasts.len(),
                tool_label,
            );
            host.window().set_title(&title);
            self.title_dirty = false;
        }

        // Continuous redraw (paired with `ControlFlow::Poll` in main.rs). If mouse stutter returns:
        // (1) per-frame text shaping (`ph2d-text` `layout_cache`); (2) present saturation —
        // Config → Display → Immediate (`ph2d-gpu/src/surface.rs` `set_present_mode`).
        host.request_redraw();
    }
}
