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
            atlas_is_real,
            script,
            theme,
            zen,
            toasts,
            tools,
            ..
        } = gfx;
        // Window title carries editor state. Refresh only when state
        // actually changes — winit set_title triggers a platform call.
        if self.title_dirty {
            let tool_label = tools.active().map(|t| t.label()).unwrap_or("none");
            let title = format!(
                "PH2D — {} | sprites={SPRITE_COUNT} | atlas={} ({} assets) \
                 | script={} | theme={:?} | zen={} | toasts={} | tool={}",
                // **O NOME DO FICHEIRO**, e não a lista de milestones que morava aqui: a barra de
                // título é o único sítio do app que responde *«que projeto é este?»*, e a resposta
                // dela era «M5+M6+M7+M11+M12 demo» — verdade sobre o binário, e sobre nada que o
                // artista tenha aberto.
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

        // Continuous redraw (paired with `ControlFlow::Poll` in main.rs):
        // the frame is rebuilt every loop iteration regardless of input,
        // so any per-frame cost shows as ~100% idle CPU and, if a frame
        // gets heavy, mouse-move stutter (worst over the Hierarchy panel,
        // which has the most per-frame text).
        //
        // IF MOUSE STUTTER RETURNS, look here first:
        //  1. Per-frame text shaping — mitigated by the shaped-layout
        //     cache in `ph2d-text/src/system.rs` (`layout_cache`). A new
        //     uncached text path, or text that changes every frame and
        //     thrashes the cache, re-introduces the cost. Profile with a
        //     `PH2D_PROF`-style timer around `paint_hero_screen`.
        //  2. The continuous redraw + present saturation — the
        //     user-facing fix is Config → Display → Immediate (a
        //     non-blocking present mode, see `ph2d-gpu/src/surface.rs`
        //     `set_present_mode`), which stops `acquire_frame` stalling.
        //     Default is VSync (`Fifo`) for smooth motion. The deeper
        //     idle-CPU win (event-driven `ControlFlow::Wait`) stays
        //     deferred and only pays off once the scene is static (the
        //     M5 demo bouncing-motion sim animates every frame, so the
        //     loop is continuous regardless).
        host.request_redraw();
    }
}
