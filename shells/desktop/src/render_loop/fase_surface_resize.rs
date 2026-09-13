//! **Fase do quadro: O RESIZE COALESCIDO** — o modo de apresentação NÃO-bloqueante enquanto o arrasto da
//! janela corre (e o configurado de volta uns quadros depois de ele assentar), e o re-fit exacto — layout,
//! cada RT do pipeline e os rebinds — uma vez por quadro com o último tamanho (OBRA 2 da
//! `line/render-loop`, 2026-09-12).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_surface_resize(&mut self) {
        // O `gfx` e o `host` re-derivados; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            surface,
            layout,
            game_rt,
            motion_fx,
            tonemap,
            compositor,
            vello_pass,
            world_rt,
            compositor_reads_world,
            ..
        } = FrameGfx::of(gfx);
        let Some(host) = self.host.as_ref() else {
            return;
        };

        // ── Coalesced resize + FLUID-DRAG present mode (Enio 2026-07-05, take 2) ──
        // The full re-fit (layout + RT reallocs + rebinds) runs once per frame with the latest size —
        // everything stays exact-size, no stretching (a first "two-speed" attempt stretched between
        // re-fits and read as "truncado, sem fluidez"). The REAL live-drag jank lever is the PRESENT
        // MODE: under VSync (`Fifo`) every `surface.configure` discards the swapchain images and the
        // next acquire BLOCKS up to a full refresh — a drag reconfigures every frame, so the app ran at
        // a fraction of the refresh rate. While resize events stream we switch to the backend's best
        // NON-BLOCKING mode (Immediate, else Mailbox) and restore the configured mode a few quiet
        // frames after the drag settles.
        let resize_streaming = self.pending_resize.is_some();
        if resize_streaming {
            if self.resize_saved_present_mode.is_none() {
                let cur = surface.present_mode();
                let fast = surface.best_nonblocking_mode();
                if fast != cur {
                    self.resize_saved_present_mode = Some(cur);
                    surface.set_present_mode(fast);
                }
            }
            self.resize_settle_frames = RESIZE_SETTLE_FRAMES;
        } else if self.resize_settle_frames > 0 {
            self.resize_settle_frames -= 1;
            if self.resize_settle_frames == 0
                && let Some(mode) = self.resize_saved_present_mode.take()
            {
                surface.set_present_mode(mode);
            }
        }
        // Apply the coalesced resize once per frame.
        if let Some(size) = self.pending_resize.take() {
            surface.resize(size);
            // Layout + every offscreen RT in the pipeline must follow
            // surface size. M14.5: game_rt, tonemap output, vello
            // intermediate — all three; then the compositor's bind
            // group must be rebuilt against the new texture views.
            // Size every offscreen RT to the surface's CLAMPED size, not the
            // raw winit size. `surface.resize` clamps each dim to ≥1, and
            // `game_rt.ensure_size` REJECTS a 0 dim (keeps its old size). On a
            // transient 0-dimension frame (minimize/restore) the raw size
            // would diverge: game_rt stays old while the W3 §8 clip/mask
            // stencil sizes to `surface.size()` → a render pass pairing the
            // game_rt color attachment with a differently-sized stencil =
            // wgpu validation panic. Using the clamped size keeps color +
            // stencil extents equal every frame (audit MEDIUM fix).
            let clamped = surface.size();
            *layout = EditorLayout::new(clamped.width as f32, clamped.height as f32);
            let dim = (clamped.width, clamped.height);
            game_rt.ensure_size(surface.gpu(), dim);
            // doc 67: the Motion glow RT + blur chain track the surface too.
            motion_fx.ensure_size(surface.gpu(), dim);
            tonemap.ensure_size(surface.gpu(), dim);
            tonemap.rebind_game_view(
                surface.gpu(),
                game_rt
                    .texture()
                    .create_view(&wgpu::TextureViewDescriptor::default()),
            );
            vello_pass.ensure_size(surface.gpu(), dim);
            // ADR-0154 Fase 2 — o acumulador acompanha a superfície.
            //
            // ⚠️ E o `rebind` logo abaixo repõe o compositor a ler o TONEMAP; a bandeira tem de
            // acompanhar, senão o presente acha que já está no modo mundo e não re-liga. *Um
            // cache de «em que modo estou» que o resize não limpa é um quadro preto.*
            world_rt.ensure_size(surface.gpu(), dim);
            *compositor_reads_world = false;
            compositor.rebind(
                surface.gpu(),
                tonemap
                    .output_texture()
                    .create_view(&wgpu::TextureViewDescriptor::default()),
                vello_pass
                    .intermediate_texture()
                    .create_view(&wgpu::TextureViewDescriptor::default()),
            );
            self.handler.on_resize(clamped, host.scale_factor());
            self.title_dirty = true;
        }
    }
}
